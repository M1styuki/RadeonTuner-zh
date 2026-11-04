#pragma once
#include "pch.h"
#include "MainPage.h"
#include "MainVariables.h"

namespace winrt::RadeonTuner::implementation
{
	void MainPage::AdlxValuesImportGraphics()
	{
		try
		{
			//Show file dialog
			std::wstring importPath = filepicker_open(NULL, L"导入显卡设置……", { { L"设置文件", L"*.radg" } });

			//Check file path
			if (importPath.empty())
			{
				ShowNotification(L"未导入显卡设置，未指定路径");
				AVDebugWriteLine(L"Graphics not imported, no path set");
				return;
			}

			AVDebugWriteLine("Importing graphics settings: " << importPath.c_str());

			//Load settings from file
			GraphicsSettings graphicsSettings = GraphicsSettings_Profile_LoadFromFile(importPath).value();

			//Set settings values
			GraphicsSettings_Convert_ToUI_Profile(graphicsSettings, AdlSettingGet::Current);

			//Set result
			ShowNotification(L"显卡设置已导入");
			AVDebugWriteLine(L"Graphics settings imported");
		}
		catch (...)
		{
			//Set result
			ShowNotification(L"未导入显卡设置（异常）");
			AVDebugWriteLine(L"Graphics not imported, exception");
		}
	}
}