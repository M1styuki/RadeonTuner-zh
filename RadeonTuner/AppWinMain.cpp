#pragma once
#include "pch.h"
#include "AppVariables.h"
#include "SettingCheck.h"
#include "SingleInstanceRecall.h"
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

int CALLBACK wWinMain(_In_ HINSTANCE hInstance, _In_opt_ HINSTANCE hPrevInstance, _In_ PWSTR pCmdLine, _In_ int nCmdShow)
{
#ifdef RADEONTUNER_UI_VALIDATION
	return RadeonTunerUiValidation::UiValidationMain(hInstance, pCmdLine);
#else
	try
	{
		//Convert command line to string
		std::wstring commandLineString = pCmdLine;

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

		//Check file attributes
		CheckFileAttributes();

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
		std::wstring currentExecutablePath = PathGetAppExecutable();
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

		//Replace updater executable
		UpdateCleanup();

		//Check settings
		SettingCheck();

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

		//Create application window
		AVDebugWriteLine("Creating application window.");
		AppVariables::App.CreateWindowXaml(hInstance, setFirstLaunchBool || setStartWindowVisibleBool, setTopMostBool);

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