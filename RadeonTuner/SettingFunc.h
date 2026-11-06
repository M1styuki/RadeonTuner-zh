#pragma once
#include "pch.h"

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

	winrt::IAsyncAction MainPage::button_Update_Check_Click(IInspectable const& sender, RoutedEventArgs const& e)
	{
		auto strongThis = get_strong();
		try
		{
			AVDebugWriteLine("Checking for application update.");

			UpdateCheckResult updateCheckResult = UpdateCheck(AppVariables::hInstance, "dumbie", "RadeonTuner");
			if (updateCheckResult.UpdateFound)
			{
				//Get online version
				std::wstring onlineVersion = updateCheckResult.UpdateVersion;

				//Show messagebox
				int messageResult = co_await ShowMessageBox(L"发现新版本", L"是否将应用程序更新到 " + onlineVersion + L"？", { L"是", L"否" });

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
