#pragma once
#include "pch.h"
#include <Windows.h>
#include <shellapi.h>
#include <cstdlib>
#include <filesystem>
#include <string>
#include <thread>
#include "AppVariables.h"
#include "SingleInstanceRecall.h"
#include "UiValidationCommon.h"
#include "UnicodePathTests.h"
#include "SystemIntegrationTests.h"

//Separate entry point for the compile-only UI validation build.
//It never initializes ADL/ADLX, the updater, autostart shortcuts, registry settings,
//process recall or the normal hardware loops, and keeps all data under the test OutDir.
namespace RadeonTunerUiValidation
{
	inline bool CommandLineHasArgument(std::wstring const& argument)
	{
		try
		{
			//Exact, case-insensitive token match, so an empty command line never matches a flag
			int argumentCount = 0;
			LPWSTR* arguments = CommandLineToArgvW(GetCommandLineW(), &argumentCount);
			if (arguments == NULL)
			{
				return false;
			}
			bool found = false;
			for (int index = 1; index < argumentCount; index++)
			{
				if (_wcsicmp(arguments[index], argument.c_str()) == 0)
				{
					found = true;
					break;
				}
			}
			LocalFree(arguments);
			return found;
		}
		catch (...)
		{
			return false;
		}
	}

	inline void WriteEnvironmentReport(bool preflightOnly)
	{
		nlohmann::json report = nlohmann::json::object();
		report["started_utc"] = UtcNowString();
		report["mode"] = AutoMode ? "auto" : "stay";
		report["preflight_only"] = preflightOnly;
		report["executable"] = WideToUtf8(ExecutablePath);
		report["test_data_path"] = WideToUtf8(TestDataPath);
		report["product_exe_launched"] = false;
		report["skipped"] = nlohmann::json::array({ "CheckFileAttributes", "Updater/UpdateCleanup", "real autostart/start menu shortcuts", "normal startup process recall path", "ADL/ADLX initialization", "registry reads/writes for settings", "hardware loops", "update network check", "save-enabled normal startup path" });
		report["used"] = nlohmann::json::array({ "task-private settings under test data", "real MainPage XAML and theme", "real page navigation", "real ShowMessageBox async path", "real tray icon add/remove and TaskbarCreated handler", "cross-process full-path single-instance recall through the shared matcher", "native Win32 tray menu via -ui-show-tray-menu-only", "taskbar hide/show", "window hide/show" });
		WriteUtf8File(EnvironmentJsonPath, report.dump(2));
	}

	inline bool ProbeWriteAccess()
	{
		std::wstring probePath = TestDataPath + L"\\.ui-validation-write-test";
		bool writeResult = WriteUtf8File(probePath, "ok");
		if (writeResult)
		{
			FileDelete(probePath);
		}
		return writeResult;
	}

	inline void WritePreflightReport(bool writeAccess, bool assetsPresent, bool resourcesPresent)
	{
		nlohmann::json report = nlohmann::json::object();
		report["checked_utc"] = UtcNowString();
		report["mode"] = AutoMode ? "auto" : "stay";
		report["executable"] = WideToUtf8(ExecutablePath);
		report["test_data_path"] = WideToUtf8(TestDataPath);
		report["test_data_writable"] = writeAccess;
		report["assets_present"] = assetsPresent;
		report["resources_pri_present"] = resourcesPresent;
		report["unicode_path_tests_passed"] = UnicodePathTestsPassed;
		report["system_integration_tests_passed"] = SystemIntegrationTestsPassed;
		report["skipped"] = nlohmann::json::array({ "CheckFileAttributes", "Updater/UpdateCleanup", "real autostart/start menu shortcuts", "normal startup process recall path", "ADL/ADLX initialization", "registry reads/writes for settings", "hardware loops", "update network check", "save-enabled normal startup path" });
		WriteUtf8File(PreflightJsonPath, report.dump(2));
	}

	inline void WriteTrayMenuReport()
	{
		nlohmann::json report = nlohmann::json::object();
		report["shown_utc"] = UtcNowString();
		report["native_win32_menu"] = true;
		report["owner_window"] = (uint64_t)(uintptr_t)AppVariables::App.GetWindowHandle();
		report["theme_style"] = AppVariables::Settings.Load<int>("ThemeStyle").value_or(0);
		report["note"] = "The real CreatePopupMenu/TrackPopupMenuEx tray menu was anchored inside the test window; no owner-draw, external input or desktop automation.";
		WriteUtf8File(TestDataPath + L"\\ui-validation-tray-menu.json", report.dump(2));
	}

	inline void WriteReturnedResult()
	{
		if (ScenarioCompleted)
		{
			return;
		}
		nlohmann::json report = nlohmann::json::object();
		report["status"] = "failed";
		report["mode"] = AutoMode ? "auto" : "stay";
		report["reason"] = "Message loop exited before the UI validation scenario completed.";
		report["unicode_path_tests_passed"] = UnicodePathTestsPassed;
		report["test_data_path"] = WideToUtf8(TestDataPath);
		report["scenario_started"] = ScenarioStarted;
		WriteUtf8File(ResultJsonPath, report.dump(2));
	}

	inline void StartWatchdog()
	{
		std::thread watchdog([]()
		{
			for (int second = 0; second < 240; second++)
			{
				Sleep(1000);
				if (ScenarioCompleted || ScenarioFailed)
				{
					return;
				}
			}
			WriteUtf8File(TestDataPath + L"\\ui-validation-timeout.txt", "The UI validation scenario did not complete within 240 seconds.");
			ExitProcess(124);
		});
		watchdog.detach();
	}

	inline int UiValidationMain(HINSTANCE hInstance, PWSTR pCmdLine)
	{
		//Cross-process recall helper: a second copy of this test build reuses the same matcher as the
		//normal startup path, finds the first test window by full executable path and shows it.
		if (CommandLineHasArgument(L"-ui-recall-check"))
		{
			bool recallResult = RadeonTunerSingleInstance::RecallExistingInstance(PathGetAppExecutable());
			return recallResult ? 0 : 3;
		}

		AutoMode = CommandLineHasArgument(L"-ui-auto") || !CommandLineHasArgument(L"-ui-stay");
		AnimationCheckMode = CommandLineHasArgument(L"-ui-animation-check");
		LifecycleCheckMode = CommandLineHasArgument(L"-ui-lifecycle-check");
		AnimationCheckStay = AnimationCheckMode && CommandLineHasArgument(L"-ui-check-stay");
		PreviewMode = CommandLineHasArgument(L"-ui-preview") || AnimationCheckMode || LifecycleCheckMode;
		if (PreviewMode && !LifecycleCheckMode)
		{
			//Preview is a visual-only stay mode; never run the automated stress scenario or watchdog
			AutoMode = false;
		}
		PreflightOnly = CommandLineHasArgument(L"-ui-preflight");
		ShowTrayMenuOnly = CommandLineHasArgument(L"-ui-show-tray-menu-only") || CommandLineHasArgument(L"-ui-menu");
		bool delayWindow = CommandLineHasArgument(L"-ui-window-delay");
		UNREFERENCED_PARAMETER(pCmdLine);

		ExecutablePath = PathGetExecutableFile();
		TestDataPath = PathMerge(PathGetAppRoot(), L"UiValidationData");
		EnvironmentJsonPath = TestDataPath + L"\\ui-validation-environment.json";
		PreflightJsonPath = TestDataPath + L"\\ui-validation-preflight.json";
		ResultJsonPath = TestDataPath + L"\\ui-validation-result.json";

		//Set working directory to the test build root
		SetCurrentDirectoryW(PathGetAppRoot().c_str());

		//Create a fresh private data folder for this run
		FolderDelete(TestDataPath + L"\\Settings", true);
		FolderDelete(TestDataPath + L"\\Profiles", true);
		FolderDelete(TestDataPath + L"\\Debug.log", false);
		FolderCreate(TestDataPath);
		FolderCreate(TestDataPath + L"\\Settings");
		FolderCreate(TestDataPath + L"\\Profiles");

		//Log only inside the test data folder
		DebugLogPath = TestDataPath + L"\\Debug.log";
		AVDebugWriteLineLogFilePath = DebugLogPath;
		AVDebugWriteLineLogFile = true;
		AVDebugWriteLineLogFileEnabled = true;
		WriteEnvironmentReport(PreflightOnly);

		if (PreflightOnly)
		{
			UnicodePathTestsPassed = UnicodePathTests::Run(TestDataPath);
			SystemIntegrationTestsPassed = SystemIntegrationTests::Run(TestDataPath);
			bool writeAccess = ProbeWriteAccess();
			bool assetsPresent = FileExists(PathMerge(PathGetAppRoot(), L"Assets\\AppIconColor.ico")) && FileExists(PathMerge(PathGetAppRoot(), L"Assets\\LauncherSteam.png"));
			bool resourcesPresent = FileExists(PathMerge(PathGetAppRoot(), L"Resources.pri")) && FileExists(PathMerge(PathGetAppRoot(), L"Microsoft.UI.Xaml.winmd"));
			WritePreflightReport(writeAccess, assetsPresent, resourcesPresent);
			AVDebugWriteLine("UI validation preflight completed.");
			std::exit((UnicodePathTestsPassed && SystemIntegrationTestsPassed && writeAccess && assetsPresent && resourcesPresent) ? 0 : 2);
		}

		//Initialize task-private settings only
		AppVariables::hInstance = hInstance;
		AppVariables::SaveDataPath = TestDataPath;
		AppVariables::Settings = AVSettingsJson(PathMerge(TestDataPath, L"Settings\\Settings.json"));
		AppVariables::Settings.Set("FirstLaunch", false);
		AppVariables::Settings.Set("MenuIndex", 0);
		AppVariables::Settings.Set("StartCheckUpdate", false);
		AppVariables::Settings.Set("StartWindowVisible", true);
		AppVariables::Settings.Set("SetTopMost", false);
		AppVariables::Settings.Set("CloseTray", false);
		AppVariables::Settings.Set("HideTrayIcon", false);
		AppVariables::Settings.Set("ShowExperimental", false);
		AppVariables::Settings.Set("SkipSupportCheckGraphics", false);
		AppVariables::Settings.Set("ThemeStyle", 0);
		AppVariables::Settings.Set("ThemeIconMono", false);

		//Run the Unicode path and pure system-integration checks before touching the UI
		if (!PreviewMode)
		{
			UnicodePathTestsPassed = UnicodePathTests::Run(TestDataPath);
			SystemIntegrationTestsPassed = SystemIntegrationTests::Run(TestDataPath);
			bool writeAccess = ProbeWriteAccess();
			bool assetsPresent = FileExists(PathMerge(PathGetAppRoot(), L"Assets\\AppIconColor.ico")) && FileExists(PathMerge(PathGetAppRoot(), L"Assets\\LauncherSteam.png"));
			bool resourcesPresent = FileExists(PathMerge(PathGetAppRoot(), L"Resources.pri")) && FileExists(PathMerge(PathGetAppRoot(), L"Microsoft.UI.Xaml.winmd"));
			WritePreflightReport(writeAccess, assetsPresent, resourcesPresent);
		}

		if (AutoMode)
		{
			StartWatchdog();
		}

		//Test-only switch: keep the process alive without a main window so the portable
		//launcher's bounded cold-start recall path can be exercised.
		if (delayWindow)
		{
			AVDebugWriteLine("Delaying isolated UI validation window creation for launcher recall test.");
			Sleep(4000);
		}
		AVDebugWriteLine("Starting isolated UI validation window.");
		AppVariables::App.CreateWindowXaml(hInstance, true, false);

		if (PreviewMode)
		{
			return 0;
		}

		//Only reached in stay mode after the user closes the window, or when startup failed
		WriteReturnedResult();
		return ScenarioCompleted ? 0 : 1;
	}
}
