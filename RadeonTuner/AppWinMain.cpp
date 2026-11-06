#pragma once
#include "pch.h"
#include <cstdlib>
#include <shellapi.h>
#include "AppVariables.h"
#include "SettingCheck.h"
#include "SingleInstanceRecall.h"
#include "RadeonTunerBackend.h"
#include "RadeonTunerIpc.h"
#ifdef RADEONTUNER_UI_VALIDATION
#if __has_include("..\tools\UiValidation\UiValidationEntry.h")
#include "..\tools\UiValidation\UiValidationEntry.h"
#else
#include "..\..\RadeonTuner\tools\UiValidation\UiValidationEntry.h"
#endif
#endif

void CheckFileAttributes()
{
	try
	{
		//Get path to local application data
		std::wstring pathLocalAppData = PathGetFolderKnown(FOLDERID_LocalAppData);

		//Check setting files
		FileAdjustAttributes(PathMerge(AppVariables::SaveDataPath, L"Settings\\Settings.json"), true, FILE_ATTRIBUTE_READONLY);

		//Check profile files
		FileAdjustAttributes(PathMerge(AppVariables::SaveDataPath, L"Profiles\\DisplayProfiles.json"), true, FILE_ATTRIBUTE_READONLY);
		FileAdjustAttributes(PathMerge(AppVariables::SaveDataPath, L"Profiles\\GraphicsProfiles.json"), true, FILE_ATTRIBUTE_READONLY);
		FileAdjustAttributes(PathMerge(AppVariables::SaveDataPath, L"Profiles\\MultimediaProfiles.json"), true, FILE_ATTRIBUTE_READONLY);
		FileAdjustAttributes(PathMerge(AppVariables::SaveDataPath, L"Profiles\\TuningProfiles.json"), true, FILE_ATTRIBUTE_READONLY);

		//Check user.blb file
		FileAdjustAttributes(PathMerge(pathLocalAppData, L"ATI\\ACE\\APL\\User.blb"), true, FILE_ATTRIBUTE_READONLY);
	}
	catch (...) {}
}

bool ParseUiWorkerArguments(std::wstring const& commandLine, HANDLE& inputPipe, HANDLE& outputPipe, bool& firstWorker)
{
	try
	{
		int argumentCount = 0;
		LPWSTR* arguments = CommandLineToArgvW(GetCommandLineW(), &argumentCount);
		if (arguments == NULL)
		{
			return false;
		}
		bool worker = false;
		for (int index = 1; index < argumentCount; index++)
		{
			if (_wcsicmp(arguments[index], L"-ui-worker") == 0)
			{
				worker = true;
				if (index + 2 < argumentCount)
				{
					inputPipe = (HANDLE)(UINT_PTR)_wcstoui64(arguments[index + 1], NULL, 10);
					outputPipe = (HANDLE)(uintptr_t)_wcstoui64(arguments[index + 2], NULL, 10);
				}
			}
			else if (_wcsicmp(arguments[index], L"-ui-first") == 0)
			{
				firstWorker = true;
			}
			else if (_wcsicmp(arguments[index], L"-ui-host") == 0 && index + 2 < argumentCount)
			{
				AppVariables::UiWorkerHostWindow = (HWND)(UINT_PTR)_wcstoui64(arguments[index + 1], NULL, 10);
				AppVariables::UiWorkerHostPid = (DWORD)_wcstoui64(arguments[index + 2], NULL, 10);
			}
		}
		LocalFree(arguments);
		return worker && inputPipe != NULL && outputPipe != NULL;
	}
	catch (...)
	{
		return false;
	}
}

int CALLBACK wWinMain(_In_ HINSTANCE hInstance, _In_opt_ HINSTANCE hPrevInstance, _In_ PWSTR pCmdLine, _In_ int nCmdShow)
{
#ifdef RADEONTUNER_UI_VALIDATION
	return RadeonTunerUiValidation::UiValidationMain(hInstance, pCmdLine);
#else
	try
	{
		//Convert command line to string
		std::wstring commandLineString = pCmdLine;
		HANDLE workerInput = NULL;
		HANDLE workerOutput = NULL;
		AppVariables::UiWorkerMode = ParseUiWorkerArguments(commandLineString, workerInput, workerOutput, AppVariables::UiWorkerFirst);
		AppVariables::NativeHostMode = !AppVariables::UiWorkerMode;
		if (AppVariables::UiWorkerMode)
		{
			AppVariables::UiWorkerInput = workerInput;
			AppVariables::UiWorkerOutput = workerOutput;
		}

		//Get application root path
		std::wstring applicationRootPath = PathGetAppRoot();

		//Set working directory to application root
		SetCurrentDirectoryW(applicationRootPath.c_str());

		//Check folder write permission
		if (!FolderWritePermission(L""))
		{
			AVDebugWriteLine("No write permission in working directory.");

			//Get path to local application data
			std::wstring pathLocalAppData = PathGetFolderKnown(FOLDERID_LocalAppData);
			std::wstring pathDataFolder = PathMerge(pathLocalAppData, L"RadeonTuner");

			//Set data path to local application data
			AppVariables::SaveDataPath = pathDataFolder;

			//Create RadeonTuner folder
			FolderCreate(AppVariables::SaveDataPath);
		}
		else
		{
			//Set data path to application root
			AppVariables::SaveDataPath = applicationRootPath;
		}

		//Check file attributes once in the host.
		if (!AppVariables::UiWorkerMode)
		{
			CheckFileAttributes();
		}

		//Enable debug logging
		if (commandLineString.find(L"-debug") != std::wstring::npos)
		{
			std::wstring debugLogPath = PathMerge(AppVariables::SaveDataPath, L"Debug.log");
			FileDelete(debugLogPath);
			AVDebugWriteLineLogFileEnabled = true;
			AVDebugWriteLineLogFilePath = debugLogPath;
		}

		AVDebugWriteLine("Welcome to RadeonTuner.");

		//Check if this executable is already running
		//Note: only match the full executable path, another RadeonTuner.exe from a different folder is not this application
		std::wstring currentExecutablePath = AppVariables::UiWorkerMode ? std::wstring() : PathGetAppExecutable();
		if (!currentExecutablePath.empty())
		{
			if (RadeonTunerSingleInstance::RecallExistingInstance(currentExecutablePath))
			{
				AVDebugWriteLine("Application is already running, exiting and showing window.");
				return 0;
			}
		}
		else
		{
			AVDebugWriteLine("Could not determine own executable path, skipping single instance match.");
		}

		//Set instance handle
		AppVariables::hInstance = hInstance;

		//Create default folders
		FolderCreate(PathMerge(AppVariables::SaveDataPath, L"Profiles"));
		FolderCreate(PathMerge(AppVariables::SaveDataPath, L"Settings"));

		//Initialize settings
		AppVariables::Settings = AVSettingsJson(PathMerge(AppVariables::SaveDataPath, L"Settings\\Settings.json"));

		//One-time host initialization; a worker must not repeat global defaults or updater cleanup.
		if (!AppVariables::UiWorkerMode)
		{
			UpdateCleanup();
			SettingCheck();
		}

		//Load window settings
		bool setFirstLaunchBool = false;
		std::optional<bool> firstLaunchSetting = AppVariables::Settings.Load<bool>("FirstLaunch");
		if (firstLaunchSetting.has_value())
		{
			setFirstLaunchBool = firstLaunchSetting.value();
		}

		bool setStartWindowVisibleBool = false;
		std::optional<bool> startWindowVisibleSetting = AppVariables::Settings.Load<bool>("StartWindowVisible");
		if (startWindowVisibleSetting.has_value())
		{
			setStartWindowVisibleBool = startWindowVisibleSetting.value();
		}

		bool setTopMostBool = false;
		std::optional<bool> setTopMostSetting = AppVariables::Settings.Load<bool>("SetTopMost");
		if (setTopMostSetting.has_value())
		{
			setTopMostBool = setTopMostSetting.value();
		}

		AVDebugWriteLine("Creating application window.");
		if (AppVariables::UiWorkerMode)
		{
			std::string state;
			if (AppVariables::UiWorkerInput != NULL)
			{
				RadeonTunerIpc::ReadFrame(AppVariables::UiWorkerInput, state);
				CloseHandle(AppVariables::UiWorkerInput);
				AppVariables::UiWorkerInput = NULL;
			}
			AppVariables::UiWorkerStateJson = state;
			AppVariables::App.CreateWindowXaml(hInstance, true, setTopMostBool);
		}
		else
		{
			//The host never initializes XAML. It owns the native tray window, the
			//one-time driver defaults/sync and the profile backend.
			RadeonTunerBackend::HeadlessOptions options{};
			options.InitializeAdl = true;
			options.LoadProfiles = true;
			options.SelectDefaults = true;
			options.WriteDefaults = true;
			options.SyncApplications = true;
			std::wstring initError = RadeonTunerBackend::InitializeHeadless(options);
			if (!initError.empty())
			{
				AVDebugWriteLine(L"Headless backend initialization failed: " << initError.c_str());
				//Open the original worker error UI, but do not start invalid host polling.
				AppVariables::App.CreateHostWindow(hInstance, true, false);
				return 0;
			}
			AppVariables::App.EnsureBackendStarted();
			AppVariables::App.CreateHostWindow(hInstance, setFirstLaunchBool || setStartWindowVisibleBool, setTopMostBool);
		}

		//Return result
		return 0;
	}
	catch (...)
	{
		//Return result
		AVDebugWriteLine("Failed creating application window (Exception)");
		return -1;
	}
#endif
}
