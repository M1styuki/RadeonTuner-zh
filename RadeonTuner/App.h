#pragma once
#include "pch.h"
#include "App.g.h"

namespace winrt::RadeonTuner::implementation
{
	//Classes
	class App
	{
	public:
		App();
		CoreDispatcher GetDispatcher();
		HWND GetWindowHandle();
		HWND GetXamlHandle();
		bool GetIsWindowVisible();
		void SetTopMost(bool topMost);
		void SetContent(FrameworkElement const& content);
		void CreateWindowXaml(HINSTANCE hInstance, bool winVisible, bool winOnTop);
		void DispatcherInvoke(std::function<void()> const& action);
		void Exit(bool forceExit);
		void TrayIconCreate(HINSTANCE hInstance);
		void TrayIconDelete();
		HRESULT RefreshTrayIcon();
		void RefreshNativeTheme();
		void ShowTrayMenu(POINT point, int autoCloseMilliseconds);
#ifdef RADEONTUNER_UI_VALIDATION
		void UiValidationExit(int exitCode);
		bool TrayIconIsAddedForValidation();
		HICON TrayIconHandleForValidation();
		HRESULT TrayIconLastHresultForValidation();
		void ShowTrayMenuForValidation(int autoCloseMilliseconds);
#endif

	private:
		CoreDispatcher _dispatcher = NULL;
		WindowsXamlManager _windowsXamlManager = NULL;
		DesktopWindowXamlSource _desktopWindowXamlSource = NULL;
		HICON _trayIconHandle = NULL;
		bool _trayIconAdded = false;
		HRESULT _trayIconLastHresult = S_OK;
		bool _nativeThemeRefreshing = false;
		bool _nativeThemeApplied = false;
		bool _nativeThemeDark = false;
		bool _nativeThemeHighContrast = false;
		HWND _nativeThemeWindow = NULL;
	};
}