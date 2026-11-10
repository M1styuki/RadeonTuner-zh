#pragma once
#include "pch.h"
#include "MainPage.h"
#include "MainVariables.h"

namespace winrt::RadeonTuner::implementation
{
	inline std::wstring GammaTargetIdentifier(int adapterIndex)
	{
		try
		{
			//Full ADL device instance + PCI location + logical adapter index.
			//Do not trim to VEN/DEV/SUBSYS: that would merge two physical GPUs or
			//two logical adapters of the same model. Fail closed when ADL returns
			//no stable instance identity.
			auto gpuInfo = MainPage::AdlGetGpuByAdapterIndex(adapterIndex);
			if (!gpuInfo.has_value()) { return std::wstring(); }
			std::wstring pnpString = char_to_wstring(gpuInfo.value().strPNPString);
			std::wstring udid = char_to_wstring(gpuInfo.value().strUDID);
			if (pnpString.empty() && udid.empty()) { return std::wstring(); }
			return pnpString + L"|UDID=" + udid + L"|PCI=" + std::to_wstring(gpuInfo.value().iBusNumber) + L":" + std::to_wstring(gpuInfo.value().iDeviceNumber) + L":" + std::to_wstring(gpuInfo.value().iFunctionNumber) + L"|ADAPTER=" + std::to_wstring(adapterIndex);
		}
		catch (...) { return std::wstring(); }
	}

	inline bool GammaTargetNeedsRestore(std::wstring const& targetIdentifier)
	{
		std::lock_guard<std::mutex> lock(gammaAppliedTargetsMutex);
		return !targetIdentifier.empty() && gammaAppliedTargets.find(targetIdentifier) != gammaAppliedTargets.end();
	}

	inline void GammaTargetMarkWritten(std::wstring const& targetIdentifier)
	{
		if (!targetIdentifier.empty()) { std::lock_guard<std::mutex> lock(gammaAppliedTargetsMutex); gammaAppliedTargets.insert(targetIdentifier); }
	}

	inline void GammaTargetMarkRestored(std::wstring const& targetIdentifier)
	{
		if (!targetIdentifier.empty()) { std::lock_guard<std::mutex> lock(gammaAppliedTargetsMutex); gammaAppliedTargets.erase(targetIdentifier); }
	}

	DWORD MainPage::AdlGammaRampClamp(float clampValue)
	{
		if (clampValue <= 0.0F) { return 0; }
		if (clampValue >= 65535.0F) { return 65535; }
		return (DWORD)clampValue;
	}

	bool MainPage::AdlGammaRampMatch(const AdlGammaRamp& gammaRampBegin, const AdlGammaRamp& gammaRampEnd)
	{
		return std::equal(std::begin(gammaRampBegin.rgb), std::end(gammaRampBegin.rgb), std::begin(gammaRampEnd.rgb));
	}

	AdlGammaRamp MainPage::AdlGammaRampBuild(float baseGamma, float redGamma, float greenGamma, float blueGamma)
	{
		AdlGammaRamp gammaRamp{};
		try
		{
			DWORD red[256];
			DWORD green[256];
			DWORD blue[256];
			for (UINT i = 0; i < 255; i++)
			{
				//Convert to float value
				float rampValue = i / 255.0F;

				//Apply gamma adjustment
				rampValue = std::pow(rampValue, 1.0F / baseGamma);

				//Convert to ramp range
				float rampRange = rampValue * 65535.0F;

				//Apply color adjustment
				red[i] = AdlGammaRampClamp(rampRange * redGamma);
				green[i] = AdlGammaRampClamp(rampRange * greenGamma);
				blue[i] = AdlGammaRampClamp(rampRange * blueGamma);

				//Pack color channels
				int colorIndex = 3 + i * 2;
				gammaRamp.rgb[colorIndex++] = (DWORD(green[i]) << 16) | DWORD(red[i]);
				gammaRamp.rgb[colorIndex++] = DWORD(blue[i]);
			}
		}
		catch (...) {}
		//Return result
		return gammaRamp;
	}

	//void MainPage::AdlGammaRampConvertToFloat(AdlGammaRamp gammaRamp, float& redGamma, float& greenGamma, float& blueGamma)
	//{
	//	try
	//	{
	//		//Get color channels
	//		int colorIndex = 3 + 128 * 2;
	//		DWORD redGreen = gammaRamp.rgb[colorIndex];
	//		DWORD blue = gammaRamp.rgb[colorIndex + 1];

	//		//Unpack color channels
	//		redGamma = (redGreen & 65535) / 32768.0F;
	//		greenGamma = (redGreen >> 16) / 32768.0F;
	//		blueGamma = (blue & 65535) / 32768.0F;

	//		//Floor color channels
	//		redGamma = std::floor(redGamma * 100.0F) / 100.0F;
	//		greenGamma = std::floor(greenGamma * 100.0F) / 100.0F;
	//		blueGamma = std::floor(blueGamma * 100.0F) / 100.0F;
	//	}
	//	catch (...) {}
	//}

	bool MainPage::AdlGammaApply(int gpuAdapterIndex, float baseGamma, float redGamma, float greenGamma, float blueGamma, bool restoreDefault)
	{
		try
		{
			//Serializes both ADL Gamma_Set calls with their ownership update.
			//Lock order is always gammaWriteMutex -> gammaAppliedTargetsMutex.
			std::unique_lock<std::mutex> writeLock(gammaWriteMutex);

			std::wstring targetIdentifier = GammaTargetIdentifier(gpuAdapterIndex);
			if (targetIdentifier.empty())
			{
				//Do not write gamma that cannot be attributed to a unique adapter.
				AVDebugWriteLine(L"Gamma target identity is unavailable.");
				return false;
			}

			//Restore eligibility can change while waiting for the write lock.
			if (restoreDefault && !GammaTargetNeedsRestore(targetIdentifier))
			{
				return false;
			}

			//Fix Some applications and resolution changes seem to block or reset set gamma (ADL2 wraps around SetDeviceGammaRamp) find better alternative that always works.
			//Fix Applying other settings at the same time breaks applying gamma, wait before others are set before applying gamma.

			//Reset display settings
			//ChangeDisplaySettingsW(nullptr, CDS_RESET);

			//Set temporary gamma
			//Note: Gamma value sometimes gets stuck so setting temporary value first may help.
			float randomFloat = (rand() % 8 + 1) / 100.0F;
			AdlGammaRamp gammaRampTemporary = AdlGammaRampBuild(baseGamma - randomFloat, redGamma, greenGamma, blueGamma);
			int temporaryResult = _ADL2_Adapter_Gamma_Set(adl_Context, gpuAdapterIndex, gammaRampTemporary);
			bool wroteTemporary = temporaryResult == ADL_OK;

			//Set target gamma
			AdlGammaRamp gammaRampTarget = AdlGammaRampBuild(baseGamma, redGamma, greenGamma, blueGamma);
			int targetResult = _ADL2_Adapter_Gamma_Set(adl_Context, gpuAdapterIndex, gammaRampTarget);
			bool wroteTarget = targetResult == ADL_OK;

			//Only actual writes own gamma restoration; failed defaults must not clear it.
			if (restoreDefault)
			{
				if (wroteTarget) { GammaTargetMarkRestored(targetIdentifier); }
			}
			else if (wroteTemporary || wroteTarget)
			{
				GammaTargetMarkWritten(targetIdentifier);
			}

			//Return result
			//AVDebugWriteLine("Applied target gamma ramp.");
			return wroteTarget;
		}
		catch (...) {}
		return false;
	}
}