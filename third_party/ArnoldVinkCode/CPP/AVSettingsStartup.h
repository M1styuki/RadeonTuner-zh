#pragma once
#include <windows.h>
#include <knownfolders.h>
#include "AVDebug.h"
#include "AVPaths.h"
#include "AVFiles.h"
#include "AVString.h"

namespace ArnoldVinkCode
{
	enum StartupShortcutType
	{
		StartMenu = 0,
		Startup = 1
	};

	//Check startup shortcut
	inline bool StartupShortcutCheck(std::wstring appName, StartupShortcutType targetType)
	{
		try
		{
			//Set shortcut type
			GUID shortcutType = FOLDERID_StartMenu;
			if (targetType == StartupShortcutType::Startup)
			{
				shortcutType = FOLDERID_Startup;
			}

			//Get startup path
			std::wstring startupFolderPath = PathGetFolderKnown(shortcutType);
			std::wstring startupFilePath = PathMerge(startupFolderPath, appName + L".url");

			//Return result
			return FileExists(startupFilePath);
		}
		catch (...)
		{
			//Return result
			AVDebugWriteLine(L"Failed checking startup shortcut.");
			return false;
		}
	}

	//Write an Internet Shortcut (.url) file using Unicode (UTF-16 LE with BOM).
	//The Windows profile APIs read this format and keep non-ANSI target paths intact.
	inline bool StartupShortcutWriteFile(std::wstring shortcutFilePath, std::wstring targetFilePath, DWORD* errorCode = NULL)
	{
		if (errorCode != NULL)
		{
			*errorCode = ERROR_SUCCESS;
		}
		HANDLE fileHandle = INVALID_HANDLE_VALUE;
		try
		{
			//Build shortcut content in the original .url format
			std::wstring shortcutText = L"[InternetShortcut]\r\n";
			shortcutText += L"URL=" + targetFilePath + L"\r\n";
			shortcutText += L"IconFile=" + targetFilePath + L"\r\n";
			shortcutText += L"IconIndex=0\r\n";

			//Create shortcut file
			fileHandle = CreateFileW(shortcutFilePath.c_str(), GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
			if (fileHandle == INVALID_HANDLE_VALUE)
			{
				if (errorCode != NULL)
				{
					*errorCode = GetLastError();
				}
				AVDebugWriteLine(L"Failed creating startup shortcut file: " + shortcutFilePath);
				return false;
			}

			//Write UTF-16 LE byte order mark
			const BYTE byteOrderMark[2] = { 0xFF, 0xFE };
			DWORD bytesWritten = 0;
			if (!WriteFile(fileHandle, byteOrderMark, sizeof(byteOrderMark), &bytesWritten, NULL) || bytesWritten != sizeof(byteOrderMark))
			{
				if (errorCode != NULL)
				{
					*errorCode = GetLastError();
				}
				CloseHandle(fileHandle);
				FileDelete(shortcutFilePath);
				AVDebugWriteLine(L"Failed writing startup shortcut header: " + shortcutFilePath);
				return false;
			}

			//Write Unicode shortcut content
			DWORD contentBytes = (DWORD)(shortcutText.size() * sizeof(wchar_t));
			if (!WriteFile(fileHandle, shortcutText.data(), contentBytes, &bytesWritten, NULL) || bytesWritten != contentBytes)
			{
				if (errorCode != NULL)
				{
					*errorCode = GetLastError();
				}
				CloseHandle(fileHandle);
				FileDelete(shortcutFilePath);
				AVDebugWriteLine(L"Failed writing startup shortcut content: " + shortcutFilePath);
				return false;
			}

			//Flush and close shortcut file
			if (!FlushFileBuffers(fileHandle))
			{
				if (errorCode != NULL)
				{
					*errorCode = GetLastError();
				}
				CloseHandle(fileHandle);
				FileDelete(shortcutFilePath);
				return false;
			}
			CloseHandle(fileHandle);
			return true;
		}
		catch (...)
		{
			if (fileHandle != INVALID_HANDLE_VALUE)
			{
				CloseHandle(fileHandle);
			}
			if (errorCode != NULL)
			{
				*errorCode = ERROR_GEN_FAILURE;
			}
			AVDebugWriteLine(L"Failed writing startup shortcut (Exception): " + shortcutFilePath);
			return false;
		}
	}

	//Read the target path from an Internet Shortcut file.
	inline std::wstring StartupShortcutReadTarget(std::wstring shortcutFilePath)
	{
		try
		{
			WCHAR targetPath[32768] = {};
			DWORD targetLength = GetPrivateProfileStringW(L"InternetShortcut", L"URL", L"", targetPath, 32768, shortcutFilePath.c_str());
			return std::wstring(targetPath, targetLength);
		}
		catch (...) {}
		return std::wstring();
	}

	//Create or remove a startup shortcut in the given folder.
	inline bool StartupShortcutManageAt(std::wstring startupFolderPath, std::wstring appName, std::wstring targetFilePath, DWORD* errorCode = NULL)
	{
		if (errorCode != NULL)
		{
			*errorCode = ERROR_SUCCESS;
		}
		try
		{
			//Get startup path
			std::wstring startupFilePath = PathMerge(startupFolderPath, appName + L".url");

			//Check if the shortcut already exists
			if (!FileExists(startupFilePath))
			{
				AVDebugWriteLine(L"Adding application to Windows startup: " + targetFilePath);
				return StartupShortcutWriteFile(startupFilePath, targetFilePath, errorCode);
			}
			else
			{
				AVDebugWriteLine(L"Removing application from Windows startup: " + targetFilePath);
				if (!FileDelete(startupFilePath))
				{
					if (errorCode != NULL)
					{
						*errorCode = GetLastError();
					}
					return false;
				}
				return true;
			}
		}
		catch (...)
		{
			if (errorCode != NULL)
			{
				*errorCode = ERROR_GEN_FAILURE;
			}
			AVDebugWriteLine(L"Failed creating startup shortcut (Exception).");
			return false;
		}
	}

	//Create or remove startup shortcut
	inline bool StartupShortcutManage(std::wstring appName, bool useLauncher, StartupShortcutType targetType)
	{
		try
		{
			//Set shortcut type
			GUID shortcutType = FOLDERID_StartMenu;
			if (targetType == StartupShortcutType::Startup)
			{
				shortcutType = FOLDERID_Startup;
			}

			//Get startup path
			std::wstring startupFolderPath = PathGetFolderKnown(shortcutType);

			//Set shortcut details
			std::wstring targetFilePath = PathGetExecutableFile();
			std::wstring targetExecutableFile = PathGetFileName(targetFilePath);

			//Check launcher executable
			if (useLauncher)
			{
				std::wstring executableLauncher = L"";
				if (FileExists(appName + L"-Launcher.exe"))
				{
					executableLauncher = appName + L"-Launcher.exe";
				}
				else if (FileExists(L"Launcher.exe"))
				{
					executableLauncher = L"Launcher.exe";
				}

				if (!executableLauncher.empty())
				{
					targetFilePath = wstring_replace(targetFilePath, targetExecutableFile, executableLauncher);
				}
			}

			//Create or remove shortcut and return actual result
			return StartupShortcutManageAt(startupFolderPath, appName, targetFilePath);
		}
		catch (...)
		{
			//Return result
			AVDebugWriteLine(L"Failed creating startup shortcut.");
			return false;
		}
	}
}
