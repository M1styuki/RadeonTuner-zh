#pragma once
#include "pch.h"
#include "MainPage.h"
#include "MainVariables.h"

namespace winrt::RadeonTuner::implementation
{
	void MainPage::AdlxValuesExportTuning()
	{
		try
		{
			//Show file dialog
			std::wstring exportPath = filepicker_save(NULL, L"导出调校与风扇设置……", { { L"设置文件", L"*.radt" } });

			//Check file path
			if (exportPath.empty())
			{
				ShowNotification(L"未导出调校与风扇设置，未指定路径");
				AVDebugWriteLine(L"Tuning and fans not exported, no path set");
				return;
			}

			//Save settings to file
			bool saveResult = TuningFanSettings_Profile_SaveToFile(tuningFanSettingsProfile, exportPath);

			//Set result
			if (saveResult)
			{
				ShowNotification(L"调校与风扇设置已导出");
				AVDebugWriteLine(L"Tuning and fans exported");
			}
			else
			{
				ShowNotification(L"调校与风扇设置导出失败");
				AVDebugWriteLine(L"Tuning and fans export failed");
			}
		}
		catch (...)
		{
			//Set result
			ShowNotification(L"未导出调校与风扇设置（异常）");
			AVDebugWriteLine(L"Tuning and fans not exported, exception");
		}
	}
}