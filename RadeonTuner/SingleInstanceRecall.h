#pragma once
#include "pch.h"
#include <filesystem>
#include <vector>

//Single instance recall shared by the normal startup path and the isolated validation build.
//The process enumeration returns AVProcess objects that own a process handle, so this code
//keeps the original list untouched and never copies an AVProcess that already opened a handle.
namespace RadeonTunerSingleInstance
{
	inline std::wstring GetExecutableName(std::wstring const& executablePath)
	{
		try
		{
			return std::filesystem::path(executablePath).filename().wstring();
		}
		catch (...)
		{
			return std::wstring();
		}
	}

	inline bool RecallExistingInstance(std::wstring const& currentExecutablePath)
	{
		try
		{
			if (currentExecutablePath.empty())
			{
				return false;
			}

			//Query the running process list by our own executable file name so renamed builds still match
			std::wstring executableName = GetExecutableName(currentExecutablePath);
			if (executableName.empty())
			{
				return false;
			}

			DWORD currentProcessId = GetCurrentProcessId();
			bool matchingInstanceFound = false;
			std::vector<AVProcess> processList = Get_ProcessByName(executableName, true);
			for (AVProcess& process : processList)
			{
				try
				{
					//Skip the current process
					if ((DWORD)process.Identifier() == currentProcessId)
					{
						continue;
					}

					//Only match another copy of this exact executable, not a copy in a different folder
					std::wstring processExecutablePath = process.ExePathW();
					if (processExecutablePath.empty())
					{
						continue;
					}
					if (CompareStringOrdinal(currentExecutablePath.c_str(), -1, processExecutablePath.c_str(), -1, TRUE) != CSTR_EQUAL)
					{
						continue;
					}

					//Remember only a bool, then show the matching window while the owned handle is still valid
					matchingInstanceFound = true;
					HWND windowHandle = process.WindowHandleMain(false);
					if (windowHandle != NULL)
					{
						//Only the native host is the recall control window; never show a UI worker.
						wchar_t className[64] = {};
						GetClassNameW(windowHandle, className, _countof(className));
						if (wcscmp(className, L"RadeonTunerMainWindow") == 0)
						{
							AllowSetForegroundWindow((DWORD)process.Identifier());
							PostMessageW(windowHandle, RegisterWindowMessageW(L"RadeonTuner.ShowSettings"), 0, 0);
						}
					}
				}
				catch (...) {}
			}
			return matchingInstanceFound;
		}
		catch (...)
		{
			return false;
		}
	}
}
