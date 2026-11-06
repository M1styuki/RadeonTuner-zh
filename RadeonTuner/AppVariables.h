#pragma once
#include "App.h"

namespace AppVariables
{
	inline HINSTANCE hInstance = NULL;
	inline bool UiWorkerMode = false;
	inline bool NativeHostMode = false;
	inline bool UiWorkerFirst = false;
	inline HANDLE UiWorkerInput = NULL;
	inline HANDLE UiWorkerOutput = NULL;
	inline HWND UiWorkerHostWindow = NULL;
	inline DWORD UiWorkerHostPid = 0;
	inline std::string UiWorkerStateJson;
	inline BOOL LaunchLoopProfile = true;
	inline BOOL ApplicationExiting = false;
	inline std::wstring SaveDataPath = L"";
	inline AVSettingsJson Settings = AVSettingsJson();
	inline winrt::RadeonTuner::implementation::App App = winrt::RadeonTuner::implementation::App();
}