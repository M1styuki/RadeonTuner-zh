#pragma once
#include "pch.h"
#include "resource.h"
#include "MainPage.h"
#include "MainVariables.h"
#include "AppVariables.h"
#include "SettingEnumStrings.h"

namespace winrt::RadeonTuner::implementation
{
	std::wstring MainPage::AdlxInfoGpu()
	{
		try
		{
			//Fix missing Pixel and Texture Fillrate, Shaders Count, Bus Width, Fabrication Technology, Transistor Count

			//Set gpu information
			std::wstring gpu_info = L"";

			//Get gpu information
			std::optional<AdapterInfo> gpuInfo = AdlGetGpuByAdapterIndex(adl_Gpu_AdapterIndex);
			if (gpuInfo.has_value())
			{
				AdapterInfo adapterInfo = gpuInfo.value();

				//Device name
				gpu_info += L"设备名称：" + char_to_wstring(adapterInfo.strAdapterName);

				//Device identifier
				std::wstring device_id = char_to_wstring(adapterInfo.strPNPString);
				device_id = wstring_get_between(device_id, L"\\", L"\\");
				gpu_info += L"\n设备标识符：" + device_id;
			}

			////Device codename
			//Note: this can cause ntdll crash that cannot be catched on certain gpu's
			//try
			//{
			//	const int productNameSize = 32;
			//	char productNameBuffer[productNameSize]{};
			//	adl_Res0 = _ADL2_Adapter_ProductName_Get(adl_Context, adl_Gpu_AdapterIndex, productNameBuffer);
			//	if (adl_Res0 == ADL_OK)
			//	{
			//		gpu_info += L"\nDevice codename: " + char_to_wstring(productNameBuffer);
			//	}
			//}
			//catch (...) {}

			//Driver information
			try
			{
				ADLVersionsInfoX2 driverInfo{};
				adl_Res0 = _ADL2_Graphics_VersionsX3_Get(adl_Context, adl_Gpu_AdapterIndex, &driverInfo);
				if (adl_Res0 == ADL_OK)
				{
					gpu_info += L"\n驱动程序版本：" + char_to_wstring(driverInfo.strCrimsonVersion) + L" / " + char_to_wstring(driverInfo.strDriverVer);
				}
			}
			catch (...) {}

			//Bios information
			try
			{
				ADLBiosInfo biosInfo{};
				adl_Res0 = _ADL2_Adapter_VideoBiosInfo_Get(adl_Context, adl_Gpu_AdapterIndex, &biosInfo);
				if (adl_Res0 == ADL_OK)
				{
					gpu_info += L"\nBIOS 版本：" + char_to_wstring(biosInfo.strPartNumber) + L" / " + char_to_wstring(biosInfo.strVersion) + L" / " + char_to_wstring(biosInfo.strDate);
				}
			}
			catch (...) {}

			//Memory information
			try
			{
				ADLMemoryInfoX4 lpMemoryInfoX4{};
				adl_Res0 = _ADL2_Adapter_MemoryInfoX4_Get(adl_Context, adl_Gpu_AdapterIndex, &lpMemoryInfoX4);
				if (adl_Res0 == ADL_OK)
				{
					gpu_info += L"\n显存详情：" + number_to_wstring(lpMemoryInfoX4.iMemorySize / 1024 / 1024) + L"MB ";
					gpu_info += char_to_wstring(lpMemoryInfoX4.strMemoryType);

					if (lpMemoryInfoX4.iVramVendorRevId > 0)
					{
						gpu_info += L" (" + VramVendorNameFromId(lpMemoryInfoX4.iVramVendorRevId) + L")";
					}
					if (lpMemoryInfoX4.iMemoryBandwidth > 0)
					{
						gpu_info += L"\n显存带宽：" + number_to_wstring(lpMemoryInfoX4.iMemoryBandwidth) + L"MB/s";
					}
				}
			}
			catch (...) {}

			//Asic Information
			try
			{
				ADLGcnInfo gcnInfo{};
				adl_Res0 = _ADL2_GcnAsicInfo_Get(adl_Context, adl_Gpu_AdapterIndex, &gcnInfo);
				if (adl_Res0 == ADL_OK)
				{
					if (gcnInfo.CuCount > 0)
					{
						gpu_info += L"\n计算单元（WGP）：" + number_to_wstring(gcnInfo.CuCount);
					}
					if (gcnInfo.RopCount > 0)
					{
						gpu_info += L"\n渲染输出单元：" + number_to_wstring(gcnInfo.RopCount);
					}
					if (gcnInfo.TexCount > 0)
					{
						gpu_info += L"\n纹理映射单元：" + number_to_wstring(gcnInfo.TexCount);
					}
				}
			}
			catch (...) {}

			//PCI Bus information
			try
			{
				ADLChipSetInfo adlChipSetInfo{};
				adl_Res0 = _ADL2_Adapter_ChipSetInfo_Get(adl_Context, adl_Gpu_AdapterIndex, &adlChipSetInfo);
				if (adl_Res0 == ADL_OK)
				{
					if (adlChipSetInfo.iBusType > 0)
					{
						gpu_info += L"\n总线类型：" + ADLX_PCI_BUS_TYPE_STRING[adlChipSetInfo.iBusType + 1];
						if (adlChipSetInfo.iCurrentPCIELaneWidth > 0)
						{
							gpu_info += L" (x" + number_to_wstring(adlChipSetInfo.iCurrentPCIELaneWidth) + L" / x" + number_to_wstring(adlChipSetInfo.iMaxPCIELaneWidth) + L")";
						}
					}
				}
			}
			catch (...) {}

			//Resizable BAR information
			try
			{
				ADL_SAM_MODE samMode{};
				adl_Res0 = _ADL2_SmartAccessMemory_Mode_Get(adl_Context, adl_Gpu_AdapterIndex, &samMode);
				if (adl_Res0 == ADL_OK)
				{
					std::wstring samStatus = (samMode.ModeFlag > 1) ? L"已启用" : L"已禁用";
					gpu_info += L"\n可变基址寄存器（SAM）：" + samStatus;
				}
			}
			catch (...) {}

			//Hardware-accelerated GPU scheduling information
			try
			{
				std::optional<DWORD> hagsMode = RegistryGetDword(HKEY_ENUM::LOCAL_MACHINE, L"System\\CurrentControlSet\\Control\\GraphicsDrivers", L"HwSchMode");
				if (hagsMode.has_value())
				{
					std::wstring hagsStatus = (hagsMode.value() == 2) ? L"已启用" : L"已禁用";
					gpu_info += L"\nGPU 硬件加速调度（HAGS）：" + hagsStatus;
				}
			}
			catch (...) {}

			//Return information
			AVDebugWriteLine("Loaded gpu information.");
			return gpu_info;
		}
		catch (...) {}
		return L"无法加载 GPU 信息。";
	}

	std::wstring MainPage::AdlxInfoDisplay()
	{
		try
		{
			//Fix missing Current Resolution, FreeSync Range and HDR status, Refresh rate, Pixel clock

			//Set display information
			std::wstring display_info = L"";

			//Get connected displays
			std::vector<ADLDisplayInfo> displayList = AdlGetDisplayByAdapterIndex(adl_Gpu_AdapterIndex);
			for (ADLDisplayInfo& displayInfo : displayList)
			{
				//Display name
				display_info += L"显示器名称：" + char_to_wstring(displayInfo.strDisplayName);

				//Connector type
				display_info += L" (" + ADLX_DISPLAY_CONNECTOR_TYPE_STRING[displayInfo.iDisplayConnector - 1] + L")\n";
			}

			//Check connected displays
			if (displayList.empty())
			{
				display_info = L"所选 GPU 上没有已连接的显示器。";
			}

			//Trim string ending
			display_info = wstring_trim_right(display_info);

			//Return information
			AVDebugWriteLine("Loaded display information.");
			return display_info;
		}
		catch (...) {}
		return L"无法加载显示信息。";
	}

	std::wstring MainPage::AdlxInfoApplication()
	{
		try
		{
			//Get version
			std::wstring app_version = L"V" + string_to_wstring(GetVersionFromResource(AppVariables::hInstance));

			//Return version
			AVDebugWriteLine("Loaded application information.");
			return L"由 Arnold Vink 制作\n" + app_version;
		}
		catch (...) {}
		return L"无法加载应用程序信息。";
	}

	void MainPage::AdlxInfoLoad()
	{
		try
		{
			//Set information
			textblock_GpuInfo().Text(AdlxInfoGpu());
			textblock_DisplayInfo().Text(AdlxInfoDisplay());
			textblock_AppInfo().Text(AdlxInfoApplication());

			//Set result
			AVDebugWriteLine("ADLX loaded information.");
		}
		catch (...)
		{
			//Set result
			AVDebugWriteLine("ADLX failed loading information.");
		}
	}
}