#pragma once
#include "pch.h"
#include "MainPage.h"
#include "SystemIntegrationIdentity.h"

namespace winrt::RadeonTuner::implementation
{
	void MainPage::toggleswitch_Shortcut_Startup_Toggled(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving_settings) { return; }

			//Create or delete startup shortcut with this build's independent identity
			StartupShortcutManage(RadeonTunerSystemIntegration::ApplicationName, L"", StartupShortcutType::Startup);
		}
		catch (...) {}
	}

	void MainPage::toggleswitch_Shortcut_StartMenu_Toggled(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving_settings) { return; }

			//Create or delete start menu shortcut with this build's independent identity
			StartupShortcutManage(RadeonTunerSystemIntegration::ApplicationName, L"", StartupShortcutType::StartMenu);
		}
		catch (...) {}
	}

	void MainPage::toggleswitch_Shortcut_ContextMenu_Toggled(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			//Note: you can also set context shortcut in HKEY_CURRENT_USER\Software\Classes\Directory\Background\shell which does not require admin but only works for current user.

			//Check if saving is disabled
			if (disable_saving_settings) { return; }

			//Create or delete only this build's independent context menu key
			std::wstring contextMenuKey = std::wstring(L"Directory\\background\\shell\\") + RadeonTunerSystemIntegration::ContextMenuKey;
			bool contextShortcut = RegistryCheck(HKEY_ENUM::CLASSES_ROOT, contextMenuKey);
			if (contextShortcut)
			{
				//Delete shortcut from registry
				RegistryDelete(HKEY_ENUM::CLASSES_ROOT, contextMenuKey);
			}
			else
			{
				//Set shortcut details
				std::wstring targetFilePath = PathGetAppExecutable();

				//Create shortcut in registry, quoting the executable path for the command
				RegistrySet(HKEY_ENUM::CLASSES_ROOT, contextMenuKey, L"", L"打开 RadeonTuner");
				RegistrySet(HKEY_ENUM::CLASSES_ROOT, contextMenuKey, L"Icon", targetFilePath);
				RegistrySet(HKEY_ENUM::CLASSES_ROOT, contextMenuKey + L"\\command", L"", RadeonTunerSystemIntegration::ContextMenuCommand(targetFilePath));
			}
		}
		catch (...) {}
	}

	void MainPage::toggleswitch_StartWindowVisible_Toggled(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving_settings) { return; }

			ToggleSwitch senderElement = sender.as<ToggleSwitch>();
			if (senderElement.IsOn())
			{
				AppVariables::Settings.Set("StartWindowVisible", true);
			}
			else
			{
				AppVariables::Settings.Set("StartWindowVisible", false);
			}
		}
		catch (...) {}
	}

	void MainPage::toggleswitch_Window_Top_Toggled(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving_settings) { return; }

			ToggleSwitch senderElement = sender.as<ToggleSwitch>();
			if (senderElement.IsOn())
			{
				AppVariables::Settings.Set("SetTopMost", true);
				AppVariables::App.SetTopMost(true);
				AVDebugWriteLine(L"Set window on top");
			}
			else
			{
				AppVariables::Settings.Set("SetTopMost", false);
				AppVariables::App.SetTopMost(false);
				AVDebugWriteLine(L"Set window not on top");
			}
		}
		catch (...) {}
	}

	void MainPage::toggleswitch_Close_Tray_Toggled(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving_settings) { return; }

			ToggleSwitch senderElement = sender.as<ToggleSwitch>();
			if (senderElement.IsOn())
			{
				AppVariables::Settings.Set("CloseTray", true);
			}
			else
			{
				AppVariables::Settings.Set("CloseTray", false);
			}
		}
		catch (...) {}
	}

	void MainPage::toggleswitch_Hide_Tray_Icon_Toggled(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving_settings) { return; }

			ToggleSwitch senderElement = sender.as<ToggleSwitch>();
			bool hideTrayIcon = senderElement.IsOn();
			AppVariables::Settings.Set("HideTrayIcon", hideTrayIcon);

			//Apply the tray visibility immediately
			HRESULT refreshResult = AppVariables::App.RefreshTrayIcon();
			if (FAILED(refreshResult))
			{
				AVDebugWriteLine(L"Failed refreshing tray icon after setting change: " << refreshResult);
			}
		}
		catch (...) {}
	}

	void MainPage::toggleswitch_ShowExperimental_Toggled(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving_settings) { return; }

			ToggleSwitch senderElement = sender.as<ToggleSwitch>();
			if (senderElement.IsOn())
			{
				AppVariables::Settings.Set("ShowExperimental", true);
			}
			else
			{
				AppVariables::Settings.Set("ShowExperimental", false);
			}

			//Show or hide experimental settings
			ShowExperimentalSettings(false);
		}
		catch (...) {}
	}

	winrt::IAsyncAction MainPage::toggleswitch_SkipSupportCheckGraphics_Toggled(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving_settings) { co_return; }

			ToggleSwitch senderElement = sender.as<ToggleSwitch>();
			if (senderElement.IsOn())
			{
				AppVariables::Settings.Set("SkipSupportCheckGraphics", true);
			}
			else
			{
				AppVariables::Settings.Set("SkipSupportCheckGraphics", false);
			}

			//Load graphics settings
			co_await AdlxValuesLoadSelectGraphicsApp(adl_Gpu_AdapterIndex, graphicsSettingsProfile.Application.value());
		}
		catch (...) {}
	}

	void MainPage::toggleswitch_StartCheckUpdate_Toggled(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving_settings) { return; }

			ToggleSwitch senderElement = sender.as<ToggleSwitch>();
			if (senderElement.IsOn())
			{
				AppVariables::Settings.Set("StartCheckUpdate", true);
			}
			else
			{
				AppVariables::Settings.Set("StartCheckUpdate", false);
			}
		}
		catch (...) {}
	}

	void MainPage::combobox_ThemeStyle_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving_settings) { return; }

			//Get setting value
			auto newSender = sender.as<ComboBox>();
			int newValue = newSender.SelectedIndex();

			//Update settings
			AppVariables::Settings.Set("ThemeStyle", newValue);

			//Adjust theme style
			ThemeAdjustStyle();
		}
		catch (...) {}
	}

	void MainPage::toggleswitch_ThemeIconMono_Toggled(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving_settings) { return; }

			ToggleSwitch senderElement = sender.as<ToggleSwitch>();
			if (senderElement.IsOn())
			{
				AppVariables::Settings.Set("ThemeIconMono", true);
			}
			else
			{
				AppVariables::Settings.Set("ThemeIconMono", false);
			}

			//Refresh the tray icon with the selected theme icon, while still respecting the hide setting
			AppVariables::App.RefreshTrayIcon();
		}
		catch (...) {}
	}

	void MainPage::toggleswitch_UpdateExcludeDrivers_Toggled(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving_settings) { return; }

			ToggleSwitch senderElement = sender.as<ToggleSwitch>();
			if (senderElement.IsOn())
			{
				RegistrySet(HKEY_ENUM::LOCAL_MACHINE, L"Software\\Policies\\Microsoft\\Windows\\WindowsUpdate", L"ExcludeWUDriversInQualityUpdate", 1);
			}
			else
			{
				RegistrySet(HKEY_ENUM::LOCAL_MACHINE, L"Software\\Policies\\Microsoft\\Windows\\WindowsUpdate", L"ExcludeWUDriversInQualityUpdate", 0);
			}
		}
		catch (...) {}
	}
}