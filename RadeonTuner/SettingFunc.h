#pragma once
#include "pch.h"
#include "WindowsService.h"

namespace winrt::RadeonTuner::implementation
{
	void MainPage::button_Website_Project_Click(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			ShellExecuteW(0, 0, L"https://projects.arnoldvink.com", 0, 0, 0);
		}
		catch (...) {}
	}

	void MainPage::button_Website_Donation_Click(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			ShellExecuteW(0, 0, L"https://donation.arnoldvink.com", 0, 0, 0);
		}
		catch (...) {}
	}

	void MainPage::button_Fps_Overlayer_Click(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			ShellExecuteW(0, 0, L"https://github.com/dumbie/FpsOverlayer", 0, 0, 0);
		}
		catch (...) {}
	}

	void MainPage::LaunchDriverCleanup()
	{
		try
		{
			//Get path to AMD Cleanup Utility
			std::wstring pathProgramFiles = PathGetFolderKnown(FOLDERID_ProgramFiles);
			std::wstring pathCleanUtility = PathMerge(pathProgramFiles, L"AMD\\CIM\\Bin64\\AMDCleanupUtility.exe");

			//Check if AMD Cleanup Utility exists
			if (FileExists(pathCleanUtility))
			{
				//Launch AMD Cleanup Utility
				AVProcesses::Launch_ApplicationDesktop(pathCleanUtility, L"", L"", false);
			}
			else
			{
				ShowNotification(L"未找到 AMD Cleanup Utility");
				AVDebugWriteLine("No soup for you! AMD Cleanup Utility has not been found.");
			}
		}
		catch (...) {}
	}

	void MainPage::ToggleRadeonServices(bool disableServices)
	{
		try
		{
			//Note: Disabling services when you still use RadeonSoftware can cause heavy stuttering, for example when AFMF is enabled because it keeps trying to enable the setting over and over but fails.
			//Note: Disabling services will break graphic features like AFMF and Radeon Boost.
			//Fix check if event service is disabled and set graphics settings effected support to false.

			//Ignore Chipset services
			//AMD 3D V-Cache Performance Optimizer Service
			//AMD Application Compatibility Database Service
			//AMD Provisioning Packages Service

			//Toggle Radeon services
			int configCount = 0;
			if (WindowsServiceDisableEnable(L"AMD External Events Utility", disableServices)) { configCount++; }
			if (WindowsServiceDisableEnable(L"AMD Crash Defender Service", disableServices)) { configCount++; }

			//Start when enabling, stop when disabling
			int controlCount = 0;
			if (WindowsServiceStartStop(L"AMD External Events Utility", !disableServices)) { controlCount++; }
			if (WindowsServiceStartStop(L"AMD Crash Defender Service", !disableServices)) { controlCount++; }

			//Only report full success when both configuration and start/stop steps succeeded
			if (configCount == 2 && controlCount == 2)
			{
				if (disableServices)
				{
					AVDebugWriteLine(L"Disabled Radeon services.");
					ShowNotification(L"已禁用 Radeon 服务");
				}
				else
				{
					AVDebugWriteLine(L"Enabled Radeon services.");
					ShowNotification(L"已启用 Radeon 服务");
				}
			}
			else
			{
				AVDebugWriteLine(L"Radeon service operation incomplete. Config: " << configCount << L"/2, start/stop: " << controlCount << L"/2");
				ShowNotification(L"Radeon 服务操作未完全成功（配置 " + number_to_wstring(configCount) + L"/2，启停 " + number_to_wstring(controlCount) + L"/2）");
			}
		}
		catch (...) {}
	}

	winrt::IAsyncAction MainPage::button_Update_Check_Click(IInspectable const& sender, RoutedEventArgs const& e)
	{
		auto strongThis = get_strong();
		try
		{
			AVDebugWriteLine("Checking for application update.");

			UpdateCheckResult updateCheckResult = UpdateCheck(AppVariables::hInstance, "dumbie", "RadeonTuner");
			if (updateCheckResult.UpdateFound)
			{
				//Set update version
				vUpdateVersion = updateCheckResult.UpdateVersion;

				//Show messagebox
				int messageResult = co_await ShowMessageBox(L"发现新版本", L"是否将应用程序更新到 " + vUpdateVersion + L"？", { L"是", L"否" });

				//Check messagebox result
				if (messageResult == 0)
				{
					//Launch updater and exit only when the official updater actually started
					if (UpdateRestart())
					{
						AppVariables::App.Exit(true);
						co_return;
					}
					co_await ShowMessageBox(L"无法启动更新", L"未能启动官方更新器 Updater.exe。请确认程序目录中的 Updater.exe 完整后重试。", { L"确定" });
				}
			}
			else
			{
				co_await ShowMessageBox(L"检查更新", L"未发现新的应用程序更新。", { L"确定" });
			}
		}
		catch (...) {}
	}

	winrt::IAsyncAction MainPage::button_Update_Launch_Click(IInspectable const& sender, RoutedEventArgs const& e)
	{
		auto strongThis = get_strong();
		try
		{
			//Show messagebox
			int messageResult = co_await ShowMessageBox(L"有可用更新", L"是否将应用程序更新到最新版本？", { L"是", L"否" });

			//Check messagebox result
			if (messageResult == 0)
			{
				//Launch updater and exit only when the official updater actually started
				if (UpdateRestart())
				{
					AppVariables::App.Exit(true);
						co_return;
				}
				co_await ShowMessageBox(L"无法启动更新", L"未能启动官方更新器 Updater.exe。请确认程序目录中的 Updater.exe 完整后重试。", { L"确定" });
			}
		}
		catch (...) {}
	}
}
