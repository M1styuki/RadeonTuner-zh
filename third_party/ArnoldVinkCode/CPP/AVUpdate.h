#pragma once
#include <windows.h>
#include "AVDebug.h"
#include "AVFiles.h"
#include "AVHighResDelay.h"
#include "AVVersion.h"
#include "AVProcesses\ProcessClose.h"
#include "Api\ApiTokens.h"
#include "Api\GitHub\GitHub_Paths.h"
#include "Api\GitHub\GitHub_Version.h"

namespace ArnoldVinkCode
{
	//Update result class
	struct UpdateCheckResult
	{
		bool UpdateFound;
		std::wstring UpdateVersion;
	};

	//Clean application update files
	inline void UpdateCleanup()
	{
		try
		{
			AVDebugWriteLine(L"Cleaning application update.");

			//Close running application updater
			if (AVProcesses::Close_ProcessByName("Updater.exe", true))
			{
				AVHighResDelay(500);
			}

			//Move new updater executable file
			FileMove(L"Settings\\UpdaterReplace.exe", L"Updater.exe", true);
			FileMove(L"Updater\\UpdaterReplace.exe", L"Updater.exe", true);
		}
		catch (...) {}
	}

	//Launch updater and report whether it started. The caller decides when to exit, so a missing
	//or blocked official updater leaves the application window open instead of closing silently.
	inline bool UpdateRestart()
	{
		try
		{
			//Launch updater
			if (!FileExists(L"Updater.exe"))
			{
				AVDebugWriteLine("Official updater executable is missing.");
				return false;
			}
			if (!AVProcesses::Launch_ApplicationDesktop(L"Updater.exe", L"", L"-ProcessLaunch", true))
			{
				AVDebugWriteLine("Failed launching the official updater.");
				return false;
			}
			return true;
		}
		catch (...)
		{
			return false;
		}
	}

	//Check for available application update
	inline UpdateCheckResult UpdateCheck(HINSTANCE hInstance, std::string gitUsername, std::string gitRepoName)
	{
		UpdateCheckResult updateCheckResult = UpdateCheckResult{};
		try
		{
			AVDebugWriteLine("Checking for application update: " << gitUsername.c_str() << " / " << gitRepoName.c_str());

			//Get online version
			std::string onlineVersion = string_to_lower(GitHub::GetLatestVersion(gitUsername, gitRepoName));

			//Get current version
			std::string currentVersion = "v" + GetVersionFromResource(hInstance);

			//Check if version matches
			if (!onlineVersion.empty() && currentVersion != onlineVersion)
			{
				AVDebugWriteLine("Application update found: " << onlineVersion.c_str() << " / " << currentVersion.c_str());
				updateCheckResult.UpdateFound = true;
				updateCheckResult.UpdateVersion = string_to_wstring_utf8(onlineVersion);
			}
			else
			{
				AVDebugWriteLine("No application update found.");
			}
		}
		catch (...)
		{
			AVDebugWriteLine("Failed checking application update (Exception)");
		}
		//Return result
		return updateCheckResult;
	}
}