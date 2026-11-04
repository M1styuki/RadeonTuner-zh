#pragma once
#include <windows.h>
#include <string>
#include <vector>
#pragma comment(lib, "Version.lib")
#include "AVString.h"

namespace ArnoldVinkCode
{
	inline std::string GetVersionFromResource(HINSTANCE hInstance)
	{
		//Set default version string
		std::string stringVersion = "?.?.?.?";
		try
		{
			//Check empty hinstance
			if (hInstance == NULL)
			{
				hInstance = GetModuleHandleW(NULL);
			}

			//Find file version resource
			HRSRC hResourceFind = FindResource(hInstance, MAKEINTRESOURCE(VS_VERSION_INFO), RT_VERSION);
			if (hResourceFind)
			{
				HGLOBAL hResourceInfo = LoadResource(hInstance, hResourceFind);
				if (hResourceInfo)
				{
					LPVOID hResourceLock = LockResource(hResourceInfo);
					if (hResourceLock)
					{
						UINT resourceLength = 0;
						VS_FIXEDFILEINFO* resourceBuffer = NULL;
						if (VerQueryValueW(hResourceLock, L"\\", (LPVOID*)&resourceBuffer, &resourceLength))
						{
							if (resourceBuffer)
							{
								DWORD dwFileVersionMS = resourceBuffer->dwFileVersionMS;
								DWORD dwFileVersionLS = resourceBuffer->dwFileVersionLS;

								DWORD dwMajor = HIWORD(dwFileVersionMS);
								DWORD dwMinor = LOWORD(dwFileVersionMS);
								DWORD dwPatch = HIWORD(dwFileVersionLS);
								DWORD dwRevision = LOWORD(dwFileVersionLS);

								//Format version string
								stringVersion = number_to_string(dwMajor) + "." + number_to_string(dwMinor) + "." + number_to_string(dwPatch) + "." + number_to_string(dwRevision);
							}
						}
					}
				}
			}
		}
		catch (...) {}
		//Return version string
		return stringVersion;
	}

	inline std::wstring GetVersionFromFile(std::wstring filePath)
	{
		try
		{
			//Get version resource size
			DWORD versionHandle = 0;
			DWORD versionSize = GetFileVersionInfoSizeW(filePath.c_str(), &versionHandle);
			if (versionSize == 0)
			{
				return L"";
			}

			//Load version resource
			std::vector<BYTE> versionBuffer(versionSize);
			if (!GetFileVersionInfoW(filePath.c_str(), 0, versionSize, versionBuffer.data()))
			{
				return L"";
			}

			//Read fixed file info
			VS_FIXEDFILEINFO* fixedFileInfo = NULL;
			UINT fixedFileInfoSize = 0;
			if (!VerQueryValueW(versionBuffer.data(), L"\\", (LPVOID*)&fixedFileInfo, &fixedFileInfoSize) || fixedFileInfo == NULL)
			{
				return L"";
			}

			//Format version string
			return std::to_wstring(HIWORD(fixedFileInfo->dwFileVersionMS)) + L"." +
				std::to_wstring(LOWORD(fixedFileInfo->dwFileVersionMS)) + L"." +
				std::to_wstring(HIWORD(fixedFileInfo->dwFileVersionLS)) + L"." +
				std::to_wstring(LOWORD(fixedFileInfo->dwFileVersionLS));
		}
		catch (...) {}
		return L"";
	}
}