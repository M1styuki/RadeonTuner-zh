#pragma once
#include "pch.h"
#include "MainPage.h"
#include "AppVariables.h"
#include "MainVariables.h"

namespace RadeonTunerBackend
{
	winrt::fire_and_forget LoadTuningApplication(winrt::Windows::Foundation::IInspectable page, int gpuAdapterIndex, std::wstring application)
	{
		if (!page)
		{
			co_return;
		}
		auto* currentPage = winrt::get_self<winrt::RadeonTuner::implementation::MainPage>(page);
		if (!currentPage)
		{
			co_return;
		}
		if (!AppVariables::App.IsCurrentUiPage(*currentPage))
		{
			co_return;
		}
		co_await currentPage->AdlxValuesLoadSelectTuningApp(gpuAdapterIndex, application);
	}
}

namespace winrt::RadeonTuner::implementation
{
	void MainPage::AdlxLoopTuningApplyProfile(int adapterIndex, TuningFanSettings tuningFanSettingsRunning)
	{
		try
		{
			//Check if keep active is enabled
			bool keepActiveEnabled = false;
			if (tuningFanSettingsRunning.KeepActive.Current.has_value())
			{
				keepActiveEnabled = tuningFanSettingsRunning.KeepActive.Current.value();
			}

			//Check if apply is required
			if (tuningFanSettingsRunning.UsingProfile && !keepActiveEnabled)
			{
				//AVDebugWriteLine(L"Profile is currently in use: " << tuningFanSettingsRunning.Application.value());
				return;
			}

			//Get current and default settings
			TuningFanSettings tuningFanSettingsAdl = TuningFanSettings_Generate_FromADL(adapterIndex, L"", false).value();

			//Check if settings match
			if (!TuningFanSettings_Match(tuningFanSettingsRunning, tuningFanSettingsAdl))
			{
				AVDebugWriteLine("Tuning and fans settings do not match, applying settings.");

				//Apply tuning and fans settings
				bool applyResult = AdlTuningFanSettingsApply(adapterIndex, tuningFanSettingsRunning, AdlSettingGet::Current);

				//Check result
				if (applyResult)
				{
					//Update application using status
					TuningFanSettings_Profile_Set_Using(tuningFanSettingsRunning.DeviceId.value(), tuningFanSettingsRunning.Application.value());
				}

				if (!applyResult)
				{
					return;
				}

				//UI updates resolve the current weak page on the UI thread.
				std::wstring applicationDisplay = tuningFanSettingsRunning.Application.value();
				if (applicationDisplay == L"Global") { applicationDisplay = L"全局"; }
				std::wstring applicationProfile = tuningFanSettingsProfile.Application.value();
				int gpuAdapterIndex = adl_Gpu_AdapterIndex;
				AppVariables::App.RunOnUi([=](MainPage& page)
					{
						auto strongPage = page.KeepAliveForUi();
						page.ShowNotification(L"调校与风扇设置已应用：" + applicationDisplay);
						AVDebugWriteLine(L"Tuning and fans settings applied: " << applicationDisplay);
						RadeonTunerBackend::LoadTuningApplication(strongPage, gpuAdapterIndex, applicationProfile);
					});
			}
		}
		catch (...) {}
	}
}
