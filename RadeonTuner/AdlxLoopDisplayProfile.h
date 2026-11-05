#pragma once
#include "pch.h"
#include "MainPage.h"
#include "AppVariables.h"
#include "MainVariables.h"

namespace RadeonTunerBackend
{
	winrt::fire_and_forget LoadDisplayApplication(winrt::Windows::Foundation::IInspectable page, int adapterIndex, int displayIndex, std::wstring application)
	{
		if (!page)
		{
			co_return;
		}
		auto* currentPage = winrt::get_self<winrt::RadeonTuner::implementation::MainPage>(page.as<winrt::RadeonTuner::MainPage>());
		if (!currentPage)
		{
			co_return;
		}
		if (!AppVariables::App.IsCurrentUiPage(*currentPage))
		{
			co_return;
		}
		co_await currentPage->AdlxValuesLoadSelectDisplayApp(adapterIndex, displayIndex, application);
	}
}

namespace winrt::RadeonTuner::implementation
{
	void MainPage::AdlxLoopDisplayApplyProfile(int adapterIndex, int displayIndex, DisplaySettings displaySettingsRunning)
	{
		try
		{
			//Check if apply is required
			if (displaySettingsRunning.UsingProfile)
			{
				//AVDebugWriteLine(L"Profile is currently in use: " << displaySettingsRunning.Application.value());
				return;
			}

			//Get current and default settings
			DisplaySettings displaySettingsAdl = DisplaySettings_Generate_FromADL(adapterIndex, displayIndex, L"", false).value();

			//Apply display settings
			bool applyResult = AdlDisplaySettingsApply(adapterIndex, displayIndex, displaySettingsRunning, displaySettingsAdl, AdlSettingGet::Current, true);

			//Check result
			if (applyResult)
			{
				//Update application using status
				DisplaySettings_Profile_Set_Using(displaySettingsRunning.DeviceId.value(), displaySettingsRunning.Application.value());
			}

			if (!applyResult)
			{
				return;
			}

			//UI updates resolve the current weak page on the UI thread; the backend
			//never captures the page that started the loop.
			std::wstring applicationDisplay = displaySettingsRunning.Application.value();
			if (applicationDisplay == L"Global") { applicationDisplay = L"全局"; }
			std::wstring applicationProfile = displaySettingsProfile.Application.value();
			int displayAdapterIndex = adl_Display_AdapterIndex;
			int displayDisplayIndex = adl_Display_DisplayIndex;
			AppVariables::App.RunOnUi([=](MainPage& page)
				{
					auto strongPage = page.KeepAliveForUi();
					page.ShowNotification(L"显示设置已应用：" + applicationDisplay);
					AVDebugWriteLine(L"Display settings applied: " << applicationDisplay);
					RadeonTunerBackend::LoadDisplayApplication(strongPage, displayAdapterIndex, displayDisplayIndex, applicationProfile);
				});
		}
		catch (...) {}
	}
}
