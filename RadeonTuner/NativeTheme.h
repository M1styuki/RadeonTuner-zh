#pragma once
#include <Windows.h>
#include <winternl.h>
#include <string>

//Small, dependency-free native theme helper for the standard window title bar and the
//native Win32 popup menu. It never installs owner-draw handlers and never links directly
//to undocumented exports: every optional entry point is resolved from the system DLL at
//runtime so older Windows versions keep starting normally.
namespace NativeTheme
{
	inline bool IsHighContrastEnabled()
	{
		try
		{
			HIGHCONTRASTW highContrast = { };
			highContrast.cbSize = sizeof(highContrast);
			if (SystemParametersInfoW(SPI_GETHIGHCONTRAST, sizeof(highContrast), &highContrast, 0))
			{
				return (highContrast.dwFlags & HCF_HIGHCONTRASTON) != 0;
			}
		}
		catch (...) {}
		return false;
	}

	inline bool IsDarkMenuSupported()
	{
		try
		{
			//Dark native menus were introduced with Windows 10 version 1809 (build 17763).
			//The application minimum is 10.0.18362, this check only guards unusual downgraded systems.
			HMODULE ntdllHandle = LoadLibraryExW(L"ntdll.dll", NULL, LOAD_LIBRARY_SEARCH_SYSTEM32);
			if (ntdllHandle == NULL)
			{
				return false;
			}
			typedef LONG(WINAPI* RtlGetVersionFunction)(PRTL_OSVERSIONINFOW);
			RtlGetVersionFunction rtlGetVersion = (RtlGetVersionFunction)GetProcAddress(ntdllHandle, "RtlGetVersion");
			if (rtlGetVersion == NULL)
			{
				return false;
			}
			RTL_OSVERSIONINFOW versionInfo = { };
			versionInfo.dwOSVersionInfoSize = sizeof(versionInfo);
			if (rtlGetVersion(&versionInfo) != 0)
			{
				return false;
			}
			return versionInfo.dwMajorVersion >= 10 && versionInfo.dwBuildNumber >= 17763;
		}
		catch (...)
		{
			return false;
		}
	}

	inline void SetProcessMenuTheme(bool dark)
	{
		try
		{
			if (!IsDarkMenuSupported())
			{
				return;
			}

			static HMODULE uxthemeHandle = LoadLibraryExW(L"uxtheme.dll", NULL, LOAD_LIBRARY_SEARCH_SYSTEM32);
			if (uxthemeHandle == NULL)
			{
				return;
			}

			//Ordinals 135/136 are the uxtheme preferred app mode and menu theme flush entries.
			//Resolve them dynamically so a build without these entries simply keeps the system menu.
			typedef int(WINAPI* SetPreferredAppModeFunction)(int);
			typedef void(WINAPI* FlushMenuThemesFunction)();
			SetPreferredAppModeFunction setPreferredAppMode = (SetPreferredAppModeFunction)GetProcAddress(uxthemeHandle, MAKEINTRESOURCEA(135));
			FlushMenuThemesFunction flushMenuThemes = (FlushMenuThemesFunction)GetProcAddress(uxthemeHandle, MAKEINTRESOURCEA(136));
			if (setPreferredAppMode == NULL)
			{
				return;
			}

			//0: default/system, 2: force dark. High contrast always returns to the system default.
			bool highContrast = IsHighContrastEnabled();
			setPreferredAppMode((dark && !highContrast) ? 2 : 0);
			if (flushMenuThemes != NULL)
			{
				flushMenuThemes();
			}
		}
		catch (...) {}
	}

	inline bool ApplyWindowTitleBarTheme(HWND windowHandle, bool dark)
	{
		try
		{
			if (windowHandle == NULL)
			{
				return false;
			}

			static HMODULE dwmapiHandle = LoadLibraryExW(L"dwmapi.dll", NULL, LOAD_LIBRARY_SEARCH_SYSTEM32);
			if (dwmapiHandle == NULL)
			{
				return false;
			}
			typedef HRESULT(WINAPI* DwmSetWindowAttributeFunction)(HWND, DWORD, LPCVOID, DWORD);
			DwmSetWindowAttributeFunction dwmSetWindowAttribute = (DwmSetWindowAttributeFunction)GetProcAddress(dwmapiHandle, "DwmSetWindowAttribute");
			if (dwmSetWindowAttribute == NULL)
			{
				return false;
			}

			//Attribute 20 is current, 19 is used by Windows 10 1809/1903/1909.
			//A failed call is expected on unsupported systems and is ignored.
			//High contrast returns the title bar to the system-provided appearance.
			bool highContrast = IsHighContrastEnabled();
			BOOL darkMode = (dark && !highContrast) ? TRUE : FALSE;
			HRESULT result = dwmSetWindowAttribute(windowHandle, 20, &darkMode, sizeof(darkMode));
			if (FAILED(result))
			{
				result = dwmSetWindowAttribute(windowHandle, 19, &darkMode, sizeof(darkMode));
			}
			return SUCCEEDED(result) != FALSE;
		}
		catch (...)
		{
			return false;
		}
	}
}
