#pragma once
#include "pch.h"
#include "MainPage.h"
#include "AdlDefinitions.h"
#include "MainVariables.h"

namespace winrt::RadeonTuner::implementation
{
	void MainPage::TuningMetrics_Update(uint64_t expectedSession)
	{
		try
		{
			//Set metric variables
			int gpuActivityCore = -1;
			int gpuActivityMemory = -1;
			int gpuMemoryUsageMegabytes = -1;
			int gpuSpeedCore = -1;
			int gpuSpeedMemory = -1;
			int gpuWattage = -1;
			int gpuVoltage = -1;
			int gpuFanSpeedRpm = -1;
			int gpuFanSpeedPercentage = -1;
			int gpuTemperatureCore = -1;
			int gpuTemperatureMemory = -1;
			int gpuTemperatureHotspot = -1;
			int gpuTemperatureIntake = -1;

			//Get metric logging data using query
			ADLPMLogDataOutput adl_Metrics_Logging_Output;
			adl_Res0 = _ADL2_New_QueryPMLogData_Get(adl_Context, adl_Gpu_AdapterIndex, &adl_Metrics_Logging_Output);
			//Note: I tried using ADL2_Device_PMLog_Device_Create and ADL2_Adapter_PMLog_Start but kept returning 0 values on integrated GPU's

			//Get current memory load in megabits
			adl_Res0 = _ADL2_Adapter_DedicatedVRAMUsage_Get(adl_Context, adl_Gpu_AdapterIndex, &gpuMemoryUsageMegabytes);

			//Set metric values
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_INFO_ACTIVITY_GFX].supported)
			{
				gpuActivityCore = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_INFO_ACTIVITY_GFX].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_INFO_ACTIVITY_MEM].supported)
			{
				gpuActivityMemory = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_INFO_ACTIVITY_MEM].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_CLK_GFXCLK].supported)
			{
				gpuSpeedCore = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_CLK_GFXCLK].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_CLK_MEMCLK].supported)
			{
				gpuSpeedMemory = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_CLK_MEMCLK].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_GFX_POWER].supported)
			{
				//Integrated
				gpuWattage = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_GFX_POWER].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_ASIC_POWER].supported)
			{
				//Dedicated RDNA2
				gpuWattage = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_ASIC_POWER].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_BOARD_POWER].supported)
			{
				//Dedicated RDNA4
				gpuWattage = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_BOARD_POWER].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_GFX_VOLTAGE].supported)
			{
				gpuVoltage = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_GFX_VOLTAGE].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_FAN_RPM].supported)
			{
				gpuFanSpeedRpm = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_FAN_RPM].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_FAN_PERCENTAGE].supported)
			{
				gpuFanSpeedPercentage = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_FAN_PERCENTAGE].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_TEMPERATURE_GFX].supported)
			{
				//Integrated
				gpuTemperatureCore = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_TEMPERATURE_GFX].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_TEMPERATURE_EDGE].supported)
			{
				//Dedicated
				gpuTemperatureCore = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_TEMPERATURE_EDGE].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_TEMPERATURE_MEM].supported)
			{
				gpuTemperatureMemory = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_TEMPERATURE_MEM].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_TEMPERATURE_HOTSPOT].supported)
			{
				gpuTemperatureHotspot = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_TEMPERATURE_HOTSPOT].value;
			}
			if (adl_Metrics_Logging_Output.sensors[ADL_PMLOG_TEMPERATURE_INTAKE].supported)
			{
				gpuTemperatureIntake = adl_Metrics_Logging_Output.sensors[ADL_PMLOG_TEMPERATURE_INTAKE].value;
			}

			//Update current statistics
			std::function<void(MainPage&)> updateFunction = [=](MainPage& page)
				{
					if (gpuActivityCore >= 0 && gpuActivityCore < 65535)
					{
						page.textblock_Current_Gpu_Activity().Text(number_to_wstring(gpuActivityCore));
					}
					else
					{
						page.textblock_Current_Gpu_Activity().Text(L"—");
					}

					if (gpuSpeedCore >= 0 && gpuSpeedCore < 65535)
					{
						page.textblock_Current_Core_Speed().Text(number_to_wstring(gpuSpeedCore));
					}
					else
					{
						page.textblock_Current_Core_Speed().Text(L"—");
					}

					if (gpuActivityMemory >= 0 && gpuActivityMemory < 65535)
					{
						page.textblock_Current_Memory_Activity().Text(number_to_wstring(gpuActivityMemory));
					}
					else
					{
						page.textblock_Current_Memory_Activity().Text(L"—");
					}

					if (gpuSpeedMemory >= 0 && gpuSpeedMemory < 65535)
					{
						page.textblock_Current_Memory_Speed().Text(number_to_wstring(gpuSpeedMemory));
					}
					else
					{
						page.textblock_Current_Memory_Speed().Text(L"—");
					}

					if (gpuMemoryUsageMegabytes >= 0)
					{
						float gpuMemoryUsageGigabyte = gpuMemoryUsageMegabytes / 1024.0F;
						page.textblock_Current_Memory_Usage().Text(float_to_wstring(gpuMemoryUsageGigabyte, 1));
					}
					else
					{
						page.textblock_Current_Memory_Usage().Text(L"—");
					}

					if (gpuWattage >= 0 && gpuWattage < 65535)
					{
						page.textblock_Current_Power_Wattage().Text(number_to_wstring(gpuWattage));
					}
					else
					{
						page.textblock_Current_Power_Wattage().Text(L"—");
					}

					if (gpuVoltage >= 0 && gpuVoltage < 65535)
					{
						page.textblock_Current_Power_Voltage().Text(number_to_wstring(gpuVoltage));
					}
					else
					{
						page.textblock_Current_Power_Voltage().Text(L"—");
					}

					if (gpuFanSpeedPercentage >= 0 && gpuFanSpeedPercentage < 65535)
					{
						page.textblock_Current_Fan_Speed_Percentage().Text(number_to_wstring(gpuFanSpeedPercentage));
					}
					else
					{
						page.textblock_Current_Fan_Speed_Percentage().Text(L"—");
					}

					if (gpuFanSpeedRpm >= 0 && gpuFanSpeedRpm < 65535)
					{
						page.textblock_Current_Fan_Speed_RPM().Text(number_to_wstring(gpuFanSpeedRpm));
					}
					else
					{
						page.textblock_Current_Fan_Speed_RPM().Text(L"—");
					}

					if (gpuTemperatureCore >= 0 && gpuTemperatureCore < 65535)
					{
						page.textblock_Current_Temp_Core().Text(number_to_wstring(gpuTemperatureCore));
					}
					else
					{
						page.textblock_Current_Temp_Core().Text(L"—");
					}

					if (gpuTemperatureMemory >= 0 && gpuTemperatureMemory < 65535)
					{
						page.textblock_Current_Temp_Memory().Text(number_to_wstring((int)gpuTemperatureMemory));
					}
					else
					{
						page.textblock_Current_Temp_Memory().Text(L"—");
					}

					if (gpuTemperatureHotspot >= 0 && gpuTemperatureHotspot < 65535)
					{
						page.textblock_Current_Temp_Hotspot().Text(number_to_wstring((int)gpuTemperatureHotspot));
					}
					else
					{
						page.textblock_Current_Temp_Hotspot().Text(L"—");
					}

					if (gpuTemperatureIntake >= 0 && gpuTemperatureIntake < 65535)
					{
						page.textblock_Current_Temp_Intake().Text(number_to_wstring((int)gpuTemperatureIntake));
					}
					else
					{
						page.textblock_Current_Temp_Intake().Text(L"—");
					}
				};
			AppVariables::App.RunOnUi(updateFunction, expectedSession);
		}
		catch (...) {}
	}
}