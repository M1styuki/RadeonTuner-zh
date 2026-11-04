#pragma once
#include "pch.h"
#include "resource.h"
#include "App.h"
#include "NativeTheme.h"
#if __has_include("App.g.cpp")
#include "App.g.cpp"
#endif
#include "AppVariables.h"
#ifdef RADEONTUNER_UI_VALIDATION
#if __has_include("..\tools\UiValidation\UiValidationCommon.h")
#include "..\tools\UiValidation\UiValidationCommon.h"
#else
#include "..\..\RadeonTuner\tools\UiValidation\UiValidationCommon.h"
#endif
#endif

namespace winrt::RadeonTuner::implementation
{
	//Variables
	HWND _hWnd_MainWindow = NULL;
	HWND _hWnd_XamlWindow = NULL;

	//Constants
	const UINT WM_TASKBARCREATED = RegisterWindowMessageW(L"TaskbarCreated");
	const UINT WM_TRAYICON_CALL = 0x8001;
	const UINT TM_SETTINGS = 0x8002;
	const UINT TM_WEBSITE = 0x8003;
	const UINT TM_EXIT = 0x8004;

	//Application tray callback
	void AppTrayCallback(LPARAM lParam)
	{
		try
		{
			switch (lParam)
			{
			case WM_LBUTTONDBLCLK:
			{
				//Show window in normal state
				ShowWindow(_hWnd_MainWindow, SW_SHOWNORMAL);
				return;
			}
			case WM_RBUTTONUP:
			{
				//Get cursor position
				POINT pointCursor;
				GetCursorPos(&pointCursor);

				//Show native tray menu at the cursor
				AppVariables::App.ShowTrayMenu(pointCursor, 0);
				return;
			}
			}
		}
		catch (...) {}
	}

	//Application tray click
	void AppTrayClick(WPARAM wParam)
	{
		try
		{
			switch (wParam)
			{
			case TM_SETTINGS:
			{
				//Show window in normal state
				ShowWindow(_hWnd_MainWindow, SW_SHOWNORMAL);
				return;
			}
			case TM_WEBSITE:
			{
				//Open project website
				ShellExecuteW(0, 0, L"https://projects.arnoldvink.com", 0, 0, 0);
				return;
			}
			case TM_EXIT:
			{
				//Exit application
				AppVariables::App.Exit(true);
				return;
			}
			}
		}
		catch (...) {}
	}

	//Application tray icon create
	void App::TrayIconCreate(HINSTANCE hInstance)
	{
		//Remove the previous notification icon and destroy the icon handle owned by this class
		TrayIconDelete();
		_trayIconLastHresult = S_OK;
		try
		{
			//Respect the hide tray icon setting, missing setting means the icon stays visible
			std::optional<bool> hideTrayIcon = AppVariables::Settings.Load<bool>("HideTrayIcon");
			if (hideTrayIcon.value_or(false))
			{
				AVDebugWriteLine("Tray icon hidden by setting.");
				return;
			}

			//Load application icon
			std::wstring iconPath = L"Assets\\AppIconColor.ico";
			std::optional<bool> setThemeIconMono = AppVariables::Settings.Load<bool>("ThemeIconMono");
			if (setThemeIconMono.value_or(false))
			{
				iconPath = L"Assets\\AppIconMono.ico";
			}
			HICON iconHandle = (HICON)LoadImageW(nullptr, iconPath.c_str(), IMAGE_ICON, GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON), LR_LOADFROMFILE);
			if (iconHandle == NULL)
			{
				_trayIconLastHresult = HRESULT_FROM_WIN32(GetLastError());
				AVDebugWriteLine("Failed loading tray icon.");
				return;
			}

			//Keep ownership of the loaded icon so it can be destroyed on the next change
			_trayIconHandle = iconHandle;

			//Create tray data
			NOTIFYICONDATAW notifyIcon = { };
			notifyIcon.cbSize = sizeof(notifyIcon);
			notifyIcon.hWnd = _hWnd_MainWindow;
			notifyIcon.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
			notifyIcon.uCallbackMessage = WM_TRAYICON_CALL;
			notifyIcon.hIcon = _trayIconHandle;
			wcscpy_s(notifyIcon.szTip, _countof(notifyIcon.szTip), L"RadeonTuner");

			//Show tray icon
			if (!Shell_NotifyIconW(NIM_ADD, &notifyIcon))
			{
				_trayIconLastHresult = HRESULT_FROM_WIN32(GetLastError());
				AVDebugWriteLine("Failed adding tray icon.");
				TrayIconDelete();
				return;
			}
			_trayIconAdded = true;
		}
		catch (...)
		{
			_trayIconLastHresult = E_FAIL;
			TrayIconDelete();
		}
	}

	//Application tray icon delete
	void App::TrayIconDelete()
	{
		try
		{
			if (_hWnd_MainWindow != NULL)
			{
				//Create tray data
				NOTIFYICONDATAW notifyIcon = { };
				notifyIcon.cbSize = sizeof(notifyIcon);
				notifyIcon.hWnd = _hWnd_MainWindow;

				//Hide tray icon
				Shell_NotifyIconW(NIM_DELETE, &notifyIcon);
			}
		}
		catch (...) {}

		_trayIconAdded = false;

		//Destroy only the HICON loaded for the tray, never the window class icon
		if (_trayIconHandle != NULL)
		{
			DestroyIcon(_trayIconHandle);
			_trayIconHandle = NULL;
		}
	}

	HRESULT App::RefreshTrayIcon()
	{
		TrayIconCreate(AppVariables::hInstance);
		return _trayIconLastHresult;
	}

	void App::RefreshNativeTheme()
	{
		//DWM/theme calls can notify the same window again; drop nested refreshes instead of recursing
		if (_nativeThemeRefreshing)
		{
			return;
		}
		_nativeThemeRefreshing = true;
		try
		{
			//Theme style 0 and 1 are dark, style 2 is light
			int themeStyle = 0;
			std::optional<int> setThemeStyle = AppVariables::Settings.Load<int>("ThemeStyle");
			if (setThemeStyle.has_value())
			{
				themeStyle = setThemeStyle.value();
			}
			bool dark = themeStyle != 2;
			bool highContrast = NativeTheme::IsHighContrastEnabled();
			bool effectiveDark = dark && !highContrast;

			//Skip identical requests; frequent WM_SETTINGCHANGE broadcasts must not reload native theme state
			bool needsApply = !_nativeThemeApplied || _nativeThemeWindow != _hWnd_MainWindow || _nativeThemeDark != effectiveDark || _nativeThemeHighContrast != highContrast;
			if (needsApply)
			{
				//Never call SetWindowTheme here: the helper only updates DWM/uxtheme state and high contrast wins
				NativeTheme::ApplyWindowTitleBarTheme(_hWnd_MainWindow, dark);
				NativeTheme::SetProcessMenuTheme(dark);

				_nativeThemeWindow = _hWnd_MainWindow;
				_nativeThemeDark = effectiveDark;
				_nativeThemeHighContrast = highContrast;
				_nativeThemeApplied = true;
			}
		}
		catch (...) {}
		_nativeThemeRefreshing = false;
	}

	void App::ShowTrayMenu(POINT point, int autoCloseMilliseconds)
	{
		try
		{
			//Use the native Win32 menu only, no owner-draw and no XAML/HTML imitation
			RefreshNativeTheme();

			HMENU hMenu = CreatePopupMenu();
			if (hMenu == NULL)
			{
				return;
			}
			InsertMenuW(hMenu, 0, MF_BYPOSITION | MF_STRING, TM_SETTINGS, L"设置");
			InsertMenuW(hMenu, 1, MF_BYPOSITION | MF_SEPARATOR, NULL, NULL);
			InsertMenuW(hMenu, 2, MF_BYPOSITION | MF_STRING, TM_WEBSITE, L"项目网站");
			InsertMenuW(hMenu, 3, MF_BYPOSITION | MF_SEPARATOR, NULL, NULL);
			InsertMenuW(hMenu, 4, MF_BYPOSITION | MF_STRING, TM_EXIT, L"退出");

			//Optional auto close for the isolated validation build, the menu remains fully native
			if (autoCloseMilliseconds > 0)
			{
				HWND windowHandle = _hWnd_MainWindow;
				std::thread([windowHandle, autoCloseMilliseconds]()
					{
						Sleep(autoCloseMilliseconds);
						PostMessageW(windowHandle, WM_CANCELMODE, 0, 0);
					}).detach();
			}

			//Track native tray menu
			SetForegroundWindow(_hWnd_MainWindow);
			TrackPopupMenuEx(hMenu, TPM_RIGHTBUTTON | TPM_LEFTALIGN | TPM_BOTTOMALIGN, point.x, point.y, _hWnd_MainWindow, NULL);
			PostMessageW(_hWnd_MainWindow, WM_NULL, 0, 0);

			//Destroy tray menu
			DestroyMenu(hMenu);
		}
		catch (...) {}
	}

	//Callbacks
	LRESULT CALLBACK WindowProc(HWND hWnd, UINT messageCode, WPARAM wParam, LPARAM lParam)
	{
		try
		{
			if (messageCode == WM_TASKBARCREATED)
			{
				//Handle taskbar created message, the hide tray icon setting still applies
				AppVariables::App.RefreshTrayIcon();
				return 0;
			}

			switch (messageCode)
			{
			case WM_TRAYICON_CALL:
			{
				//Handle tray icon callback
				AppTrayCallback(lParam);
				return 0;
			}
			case WM_COMMAND:
			{
				//Handle tray icon click
				AppTrayClick(wParam);
				return 0;
			}
			case WM_CLOSE:
			{
				//Handle app close click
				AppVariables::App.Exit(false);
				return 0;
			}
			case WM_SIZE:
			{
				//Resize xaml window
				RECT rectClient;
				GetClientRect(hWnd, &rectClient);
				MoveWindow(_hWnd_XamlWindow, 0, 0, rectClient.right, rectClient.bottom, true);
				return 0;
			}
			case WM_SETTINGCHANGE:
			{
				//System settings changed, including high contrast; change detection suppresses unrelated repeats
				AppVariables::App.RefreshNativeTheme();
				return DefWindowProcW(hWnd, messageCode, wParam, lParam);
			}
			case WM_THEMECHANGED:
			{
				//System theme changed; refresh with change detection and the reentry guard above
				AppVariables::App.RefreshNativeTheme();
				return DefWindowProcW(hWnd, messageCode, wParam, lParam);
			}
			default:
			{
				return DefWindowProcW(hWnd, messageCode, wParam, lParam);
			}
			}
		}
		catch (...) {}
		return 0;
	}

	//Functions
	App::App() {}

	void App::Exit(bool forceExit)
	{
		try
		{
			std::optional<bool> CloseTray = AppVariables::Settings.Load<bool>("CloseTray");
			if (!forceExit && CloseTray.has_value() && CloseTray.value())
			{
				AVDebugWriteLine("Closing application to tray.");

				//Fix unload xaml resources to reduce memory load when in tray

				//Show window in hidden state
				ShowWindow(_hWnd_MainWindow, SW_HIDE);
			}
			else
			{
				AVDebugWriteLine("Exiting application.");

				//Fix stop and wait for loops

				//Delete tray icon
				TrayIconDelete();

				//Update variables
				AppVariables::ApplicationExiting = true;

				//Exit application
				exit(0);
			}
		}
		catch (...) {}
	}

#ifdef RADEONTUNER_UI_VALIDATION
	void App::UiValidationExit(int exitCode)
	{
		try
		{
			//Remove the transient tray icon before the test build exits
			TrayIconDelete();
			AppVariables::ApplicationExiting = true;
		}
		catch (...) {}
		exit(exitCode);
	}

	bool App::TrayIconIsAddedForValidation()
	{
		return _trayIconAdded;
	}

	HICON App::TrayIconHandleForValidation()
	{
		return _trayIconHandle;
	}

	HRESULT App::TrayIconLastHresultForValidation()
	{
		return _trayIconLastHresult;
	}

	void App::ShowTrayMenuForValidation(int autoCloseMilliseconds)
	{
		try
		{
			//Anchor the native menu inside this application's own window, no external desktop input
			RECT windowRect = { };
			if (!GetWindowRect(_hWnd_MainWindow, &windowRect))
			{
				return;
			}
			POINT pointMenu = { (windowRect.left + windowRect.right) / 2, windowRect.top + 240 };
			ShowTrayMenu(pointMenu, autoCloseMilliseconds);
		}
		catch (...) {}
	}
#endif

	void App::DispatcherInvoke(std::function<void()> const& action)
	{
		try
		{
			if (_dispatcher == NULL)
			{
				return;
			}

			_dispatcher.RunAsync(CoreDispatcherPriority::Normal, action);
		}
		catch (...) {}
	}

	CoreDispatcher App::GetDispatcher()
	{
		return _dispatcher;
	}

	HWND App::GetWindowHandle()
	{
		try
		{
			return _hWnd_MainWindow;
		}
		catch (...)
		{
			return NULL;
		}
	}

	HWND App::GetXamlHandle()
	{
		try
		{
			return _hWnd_XamlWindow;
		}
		catch (...)
		{
			return NULL;
		}
	}

	bool App::GetIsWindowVisible()
	{
		try
		{
			return IsWindowVisible(_hWnd_MainWindow);
		}
		catch (...)
		{
			return false;
		}
	}

	void App::SetContent(FrameworkElement const& content)
	{
		try
		{
			if (_desktopWindowXamlSource == NULL)
			{
				return;
			}

			//Set DesktopWindowXamlSource content
			_desktopWindowXamlSource.Content(content);
		}
		catch (...) {}
	}

	void App::SetTopMost(bool topMost)
	{
		try
		{
			if (_desktopWindowXamlSource == NULL)
			{
				return;
			}

			//Set window style
			if (topMost)
			{
				SetWindowPos(_hWnd_MainWindow, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
			}
			else
			{
				SetWindowPos(_hWnd_MainWindow, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
			}
		}
		catch (...) {}
	}

	void App::CreateWindowXaml(HINSTANCE hInstance, bool winVisible, bool winOnTop)
	{
		try
		{
			//Initialize for current thread
			winrt::init_apartment(apartment_type::single_threaded);
			_windowsXamlManager = WindowsXamlManager::InitializeForCurrentThread();
			if (CoreWindow coreWindow = CoreWindow::GetForCurrentThread())
			{
				_dispatcher = coreWindow.Dispatcher();
			}

			//Set window strings
			const LPCWSTR szWindowTitle = L"RadeonTuner";
			const LPCWSTR szWindowClass = L"RadeonTunerMainWindow";

			//Load application icon
			std::wstring iconPath = L"Assets\\AppIconColor.ico";
			std::optional<bool> setThemeIconMono = AppVariables::Settings.Load<bool>("ThemeIconMono");
			if (setThemeIconMono.has_value() && setThemeIconMono.value())
			{
				iconPath = L"Assets\\AppIconMono.ico";
			}
			HICON iconHandle = (HICON)LoadImageW(nullptr, iconPath.c_str(), IMAGE_ICON, GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON), LR_LOADFROMFILE);

			//Register window class
			WNDCLASSEX windowClass = { };
			windowClass.cbSize = sizeof(WNDCLASSEX);
			windowClass.lpfnWndProc = WindowProc;
			windowClass.hInstance = hInstance;
			windowClass.lpszClassName = szWindowClass;
			windowClass.hbrBackground = (HBRUSH)(COLOR_WINDOWFRAME);
			windowClass.hIcon = iconHandle;
			if (RegisterClassExW(&windowClass) == NULL)
			{
#ifdef RADEONTUNER_UI_VALIDATION
				RadeonTunerUiValidation::RecordInitFailure(HRESULT_FROM_WIN32(GetLastError()), L"RegisterClassExW", L"RegisterClassExW failed");
				return;
#else
				MessageBoxW(NULL, L"注册窗口类失败", L"错误", NULL);
				return;
#endif
			}

			//Create main window
			int windowWidth = 1340;
			int windowHeight = 960;
			long dwStyle = WS_VISIBLE | WS_OVERLAPPEDWINDOW;

			_hWnd_MainWindow = CreateWindowW(szWindowClass, szWindowTitle, dwStyle, CW_USEDEFAULT, CW_USEDEFAULT, windowWidth, windowHeight, NULL, NULL, hInstance, NULL);
			if (_hWnd_MainWindow == NULL)
			{
#ifdef RADEONTUNER_UI_VALIDATION
				RadeonTunerUiValidation::RecordInitFailure(HRESULT_FROM_WIN32(GetLastError()), L"CreateWindowW", L"CreateWindowW failed");
				return;
#else
				MessageBoxW(NULL, L"创建窗口失败", L"错误", NULL);
				return;
#endif
			}

			//Apply the native title bar and popup menu theme for the saved theme style
			RefreshNativeTheme();

			//Create desktop window xaml source
			_desktopWindowXamlSource = DesktopWindowXamlSource();

			//Get native desktop window xaml source
			auto desktopWindowXamlSourceNative = _desktopWindowXamlSource.as<IDesktopWindowXamlSourceNative2>();

			//Attach DesktopWindowXamlSource to main window
			HRESULT hResult = desktopWindowXamlSourceNative->AttachToWindow(_hWnd_MainWindow);
			if (FAILED(hResult))
			{
#ifdef RADEONTUNER_UI_VALIDATION
				RadeonTunerUiValidation::RecordInitFailure(hResult, L"AttachToWindow", L"AttachToWindow failed");
				return;
#else
				MessageBoxW(NULL, L"附加窗口失败", L"错误", NULL);
				return;
#endif
			}

			//Get DesktopWindowXamlSource window handle
			hResult = desktopWindowXamlSourceNative->get_WindowHandle(&_hWnd_XamlWindow);
			if (FAILED(hResult))
			{
#ifdef RADEONTUNER_UI_VALIDATION
				RadeonTunerUiValidation::RecordInitFailure(hResult, L"get_WindowHandle", L"get_WindowHandle failed");
				return;
#else
				MessageBoxW(NULL, L"获取 XAML 窗口句柄失败", L"错误", NULL);
				return;
#endif
			}

			//Set xaml window size and location
			RECT rectClient;
			GetClientRect(_hWnd_MainWindow, &rectClient);
			MoveWindow(_hWnd_XamlWindow, 0, 0, rectClient.right, rectClient.bottom, true);

			//Show windows in normal state
			ShowWindow(_hWnd_XamlWindow, SW_SHOWNORMAL);
			ShowWindow(_hWnd_MainWindow, SW_SHOWNORMAL);

			//Set main window on top based on setting
			if (winOnTop)
			{
				SetTopMost(winOnTop);
			}

			//Hide main window based on setting
			//Note: On Windows 10 hiding the xaml window directly breaks rendering
			if (!winVisible)
			{
				ShowWindow(_hWnd_MainWindow, SW_HIDE);
			}

			//Set DesktopWindowXamlSource content
			_desktopWindowXamlSource.Content(MainPage());

			//Create application tray icon unless the setting hides it
			TrayIconCreate(hInstance);

			//Window message loop
			MSG lpMsg;
			while (GetMessageW(&lpMsg, NULL, 0, 0))
			{
				try
				{
					TranslateMessage(&lpMsg);
					DispatchMessageW(&lpMsg);
				}
				catch (...) {}
			}
		}
#ifdef RADEONTUNER_UI_VALIDATION
		catch (winrt::hresult_error const& exception)
		{
			RadeonTunerUiValidation::RecordInitFailure(exception.code(), L"CreateWindowXaml", exception.message().c_str());
			AVDebugWriteLine(L"Failed creating application window (HRESULT): " << exception.message().c_str());
		}
#endif
		catch (...)
		{
			AVDebugWriteLine("Failed creating application window (Exception)");
		}
	}
}