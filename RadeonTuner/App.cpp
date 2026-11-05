#pragma once
#include "pch.h"
#include "resource.h"
#include "App.h"
#include "NativeTheme.h"
#if __has_include("App.g.cpp")
#include "App.g.cpp"
#endif
#include "AppVariables.h"
#include "MainPage.h"
#include "RadeonTunerBackend.h"
#include "RadeonTunerUiSession.h"
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
	const UINT_PTR TIMER_RELEASE_UI = 0x8101;

	//Application tray callback
	void AppTrayCallback(LPARAM lParam)
	{
		try
		{
			switch (lParam)
			{
			case WM_LBUTTONDBLCLK:
			{
				//Recreate the released UI before showing the stable native window
				AppVariables::App.EnsureUI(true);
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
				//Recreate the released UI before showing the stable native window
				AppVariables::App.EnsureUI(true);
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
			case WM_SHOWWINDOW:
			{
				//Portable launcher, single-instance recall and tray all end up here.
				//Create the released XAML island before Windows actually shows the window.
				if (wParam)
				{
					AppVariables::App.EnsureUI(false);
				}
				return DefWindowProcW(hWnd, messageCode, wParam, lParam);
			}
			case WM_TIMER:
			{
				//Deferred UI release after pending UI work and safety timers finished
				AppVariables::App.HandleUiReleaseTimer((UINT_PTR)wParam);
				return 0;
			}
			case WM_SIZE:
			{
				//Resize xaml window only while the island exists
				if (_hWnd_XamlWindow != NULL)
				{
					RECT rectClient;
					GetClientRect(hWnd, &rectClient);
					MoveWindow(_hWnd_XamlWindow, 0, 0, rectClient.right, rectClient.bottom, true);
				}
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
				//Hide now, release the whole UI after pending interactions finish
				HideToTray();
			}
			else
			{
				AVDebugWriteLine("Exiting application.");

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

	void App::HideToTray()
	{
		try
		{
			AVDebugWriteLine("Closing application to tray.");

			//Stop notification-only UI timers immediately. Resolution safety timers
			//stay alive while hidden so an unconfirmed mode change still reverts.
			if (auto inspectable = CurrentPageForInternal())
			{
				if (auto* page = winrt::get_self<MainPage>(inspectable))
				{
					page->StopNotificationTimer();
				}
			}

			ShowWindow(_hWnd_MainWindow, SW_HIDE);

			_uiReleasePending = true;
			ScheduleUiRelease();
		}
		catch (...) {}
	}

	winrt::Windows::Foundation::IInspectable App::CurrentPageForInternal()
	{
		try
		{
			std::lock_guard<std::mutex> lock(_uiPageMutex);
			return _uiPage.get();
		}
		catch (...)
		{
			return nullptr;
		}
	}

	void App::ScheduleUiRelease()
	{
		try
		{
			if (_hWnd_MainWindow != NULL)
			{
				KillTimer(_hWnd_MainWindow, TIMER_RELEASE_UI);
				SetTimer(_hWnd_MainWindow, TIMER_RELEASE_UI, 120, NULL);
			}
			else
			{
				ReleaseUI();
			}
		}
		catch (...) {}
	}

	void App::HandleUiReleaseTimer(UINT_PTR timerId)
	{
		if (timerId != TIMER_RELEASE_UI)
		{
			return;
		}
		try
		{
			KillTimer(_hWnd_MainWindow, TIMER_RELEASE_UI);
			if (!_uiReleasePending)
			{
				return;
			}

			//Keep the page while page initialization or a resolution confirmation
			//is still finishing, then release it for real.
			if (_uiWorkCount > 0)
			{
				SetTimer(_hWnd_MainWindow, TIMER_RELEASE_UI, 200, NULL);
				return;
			}
			if (auto inspectable = CurrentPageForInternal())
			{
				if (auto* page = winrt::get_self<MainPage>(inspectable))
				{
					if (page->HasActiveResolutionTimer())
					{
						SetTimer(_hWnd_MainWindow, TIMER_RELEASE_UI, 500, NULL);
						return;
					}
				}
			}
			ReleaseUI();
		}
		catch (...) {}
	}

	void App::BeginUiWork()
	{
		_uiWorkCount++;
	}

	void App::EndUiWork()
	{
		if (_uiWorkCount > 0)
		{
			_uiWorkCount--;
		}
		if (_uiWorkCount == 0 && _uiReleasePending)
		{
			ScheduleUiRelease();
		}
	}

	void App::NotifyUiPageRegistered(MainPage const& page)
	{
		std::lock_guard<std::mutex> lock(_uiPageMutex);
		winrt::RadeonTuner::MainPage projected = page;
		winrt::Windows::Foundation::IInspectable inspectable = projected;
		_uiPage = winrt::weak_ref<winrt::Windows::Foundation::IInspectable>(inspectable);
		_uiSessionId++;
		_uiSessionPublished.store(_uiSessionId);
		_uiPageCreateCount++;
	}

	void App::NotifyUiPageReleased()
	{
		std::lock_guard<std::mutex> lock(_uiPageMutex);
		_uiPage = winrt::weak_ref<winrt::Windows::Foundation::IInspectable>{};
		_uiSessionId++;
		_uiSessionPublished.store(_uiSessionId);
	}

	void App::CreateXamlIsland()
	{
		//Attach a fresh DesktopWindowXamlSource to the stable native window.
		//WindowsXamlManager and the dispatcher stay alive for the thread lifetime.
		_desktopWindowXamlSource = DesktopWindowXamlSource();
		auto desktopWindowXamlSourceNative = _desktopWindowXamlSource.as<IDesktopWindowXamlSourceNative2>();

		HRESULT hResult = desktopWindowXamlSourceNative->AttachToWindow(_hWnd_MainWindow);
		if (FAILED(hResult))
		{
#ifdef RADEONTUNER_UI_VALIDATION
			RadeonTunerUiValidation::RecordInitFailure(hResult, L"AttachToWindow", L"AttachToWindow failed");
#endif
			DestroyXamlIsland();
			return;
		}

		hResult = desktopWindowXamlSourceNative->get_WindowHandle(&_hWnd_XamlWindow);
		if (FAILED(hResult) || _hWnd_XamlWindow == NULL)
		{
#ifdef RADEONTUNER_UI_VALIDATION
			RadeonTunerUiValidation::RecordInitFailure(FAILED(hResult) ? hResult : E_FAIL, L"get_WindowHandle", L"get_WindowHandle failed");
#endif
			DestroyXamlIsland();
			return;
		}

		RECT rectClient;
		GetClientRect(_hWnd_MainWindow, &rectClient);
		MoveWindow(_hWnd_XamlWindow, 0, 0, rectClient.right, rectClient.bottom, true);

		auto page = winrt::make_self<MainPage>();
		NotifyUiPageRegistered(*page);
		try
		{
			_desktopWindowXamlSource.Content(page.as<FrameworkElement>());
			ShowWindow(_hWnd_XamlWindow, SW_SHOWNORMAL);
		}
		catch (...)
		{
			NotifyUiPageReleased();
			DestroyXamlIsland();
			throw;
		}
	}

	void App::DestroyXamlIsland()
	{
		try
		{
			if (_desktopWindowXamlSource != NULL)
			{
				try { _desktopWindowXamlSource.Content(nullptr); } catch (...) {}
				try { _desktopWindowXamlSource.Close(); } catch (...) {}
				_desktopWindowXamlSource = NULL;
			}
			//The DesktopWindowXamlSource owns its child HWND; Close above releases it.
			_hWnd_XamlWindow = NULL;
		}
		catch (...) {}
	}

	void App::ReleaseUI()
	{
		if (_uiWorkCount > 0)
		{
			_uiReleasePending = true;
			return;
		}
		_uiReleasePending = false;
		KillTimer(_hWnd_MainWindow, TIMER_RELEASE_UI);

		//Snapshot plain values and stop storyboards/timers before the page and its
		//control tree are released. No XAML object is kept in the snapshot.
		if (auto inspectable = CurrentPageForInternal())
		{
			if (auto* page = winrt::get_self<MainPage>(inspectable))
			{
				RadeonTunerUiSession::Capture(*page);
				RadeonTunerUiSession::PrepareForRelease(*page);
			}
		}

		//Invalidate the session first so queued UI callbacks resolve no page.
		NotifyUiPageReleased();
		DestroyXamlIsland();
		AVDebugWriteLine("Released application UI for tray.");
	}

	void App::EnsureUI(bool showWindow)
	{
		try
		{
			if (_uiEnsureInProgress)
			{
				return;
			}
			_uiEnsureInProgress = true;

			//Reopening before the deferred release cancels it and keeps the existing page.
			if (_uiReleasePending)
			{
				KillTimer(_hWnd_MainWindow, TIMER_RELEASE_UI);
				_uiReleasePending = false;
			}

			if (_desktopWindowXamlSource == NULL)
			{
				if (_windowsXamlManager == NULL)
				{
					winrt::init_apartment(apartment_type::single_threaded);
					_windowsXamlManager = WindowsXamlManager::InitializeForCurrentThread();
				}
				if (_dispatcher == NULL)
				{
					if (CoreWindow coreWindow = CoreWindow::GetForCurrentThread())
					{
						_dispatcher = coreWindow.Dispatcher();
					}
				}
				CreateXamlIsland();
			}

			if (showWindow && _hWnd_MainWindow != NULL && _desktopWindowXamlSource != NULL)
			{
				ShowWindow(_hWnd_MainWindow, SW_SHOWNORMAL);
			}
			_uiEnsureInProgress = false;
		}
		catch (...)
		{
			_uiEnsureInProgress = false;
			try
			{
				NotifyUiPageReleased();
				DestroyXamlIsland();
			}
			catch (...) {}
			AVDebugWriteLine("Failed ensuring application UI (Exception).");
		}
	}

	void App::EnsureBackendStarted()
	{
		if (_backendStarted.exchange(true))
		{
			return;
		}
		_backendStartCount++;
#ifdef RADEONTUNER_UI_VALIDATION
		//The isolated validation build never starts ADL/ADLX loops.
		AVDebugWriteLine("Backend start requested (validation build, no hardware loop).");
#else
		_backendMetricsThread = std::thread(RadeonTunerBackend::MetricsLoop);
		_backendMetricsThread.detach();
		_backendProfileThread = std::thread(RadeonTunerBackend::ProfileLoop);
		_backendProfileThread.detach();
		AVDebugWriteLine("Started backend loops.");
#endif
	}

	bool App::IsCurrentUiPage(MainPage const& page)
	{
		try
		{
			std::lock_guard<std::mutex> lock(_uiPageMutex);
			auto current = _uiPage.get();
			if (!current)
			{
				return false;
			}
			winrt::RadeonTuner::MainPage projected = page;
			winrt::Windows::Foundation::IInspectable inspectable = projected;
			return winrt::get_abi(current) == winrt::get_abi(inspectable);
		}
		catch (...)
		{
			return false;
		}
	}

	uint64_t App::CurrentUiSessionId()
	{
		return _uiSessionPublished.load();
	}

	void App::RunOnUi(std::function<void(MainPage&)> const& action, uint64_t expectedSession)
	{
		try
		{
			if (!action)
			{
				return;
			}
			CoreDispatcher dispatcher = _dispatcher;
			if (dispatcher == NULL)
			{
				return;
			}

			winrt::weak_ref<winrt::Windows::Foundation::IInspectable> weakPage;
			uint64_t session = 0;
			{
				std::lock_guard<std::mutex> lock(_uiPageMutex);
				weakPage = _uiPage;
				session = expectedSession != 0 ? expectedSession : _uiSessionPublished.load();
			}
			if (!weakPage)
			{
				return;
			}

			dispatcher.RunAsync(CoreDispatcherPriority::Normal, [this, weakPage, session, action]()
				{
					try
					{
						if (session != _uiSessionPublished.load())
						{
							return;
						}
						auto inspectable = weakPage.get();
						if (!inspectable)
						{
							return;
						}
						auto* page = winrt::get_self<MainPage>(inspectable);
						if (!page)
						{
							return;
						}
						action(*page);
					}
					catch (...) {}
				});
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

	uint64_t App::UiSessionIdForValidation()
	{
		return _uiSessionPublished.load();
	}

	int App::UiPageCreateCountForValidation()
	{
		return _uiPageCreateCount;
	}

	int App::BackendStartCountForValidation()
	{
		return _backendStartCount;
	}

	bool App::UiReleasedForValidation()
	{
		return _desktopWindowXamlSource == NULL;
	}

	bool App::UiPageAliveForValidation()
	{
		return CurrentPageForInternal() != nullptr;
	}

	winrt::Windows::Foundation::IInspectable App::CurrentPageForValidation()
	{
		return CurrentPageForInternal();
	}

	void App::ShowFromTrayForValidation()
	{
		EnsureUI(true);
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
			if (_hWnd_MainWindow == NULL)
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
			//Initialize for current thread once; the manager must live until the thread exits
			winrt::init_apartment(apartment_type::single_threaded);
			_windowsXamlManager = WindowsXamlManager::InitializeForCurrentThread();
			if (CoreWindow coreWindow = CoreWindow::GetForCurrentThread())
			{
				_dispatcher = coreWindow.Dispatcher();
			}

			//Set window strings
#ifdef RADEONTUNER_UI_VALIDATION
			const LPCWSTR szWindowTitle = RadeonTunerUiValidation::PreviewMode ? L"RadeonTuner 界面预览（示例数据）" : L"RadeonTuner";
#else
			const LPCWSTR szWindowTitle = L"RadeonTuner";
#endif
			const LPCWSTR szWindowClass = L"RadeonTunerMainWindow";

			//Load application icon
			std::wstring iconPath = L"Assets\\AppIconColor.ico";
			std::optional<bool> setThemeIconMono = AppVariables::Settings.Load<bool>("ThemeIconMono");
			if (setThemeIconMono.has_value() && setThemeIconMono.value())
			{
				iconPath = L"Assets\\AppIconMono.ico";
			}
			HICON iconHandle = (HICON)LoadImageW(nullptr, iconPath.c_str(), IMAGE_ICON, GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON), LR_LOADFROMFILE);

			//Register window class once
			WNDCLASSEX windowClass = { };
			windowClass.cbSize = sizeof(WNDCLASSEX);
			windowClass.lpfnWndProc = WindowProc;
			windowClass.hInstance = hInstance;
			windowClass.lpszClassName = szWindowClass;
			windowClass.hbrBackground = (HBRUSH)(COLOR_WINDOWFRAME);
			windowClass.hIcon = iconHandle;
			if (RegisterClassExW(&windowClass) == NULL)
			{
				DWORD lastError = GetLastError();
				if (lastError != ERROR_CLASS_ALREADY_EXISTS)
				{
#ifdef RADEONTUNER_UI_VALIDATION
					RadeonTunerUiValidation::RecordInitFailure(HRESULT_FROM_WIN32(lastError), L"RegisterClassExW", L"RegisterClassExW failed");
					return;
#else
					MessageBoxW(NULL, L"注册窗口类失败", L"错误", NULL);
					return;
#endif
				}
			}

			//Create the stable native window. It is shown only after the XAML island exists.
			int windowWidth = 1340;
			int windowHeight = 960;
			long dwStyle = WS_OVERLAPPEDWINDOW;

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

			//Create the first UI island; later tray/EXE recalls call EnsureUI again
			EnsureUI(winVisible);
			if (_desktopWindowXamlSource == NULL)
			{
				return;
			}
			if (!winVisible)
			{
				ShowWindow(_hWnd_MainWindow, SW_HIDE);
			}
			if (winOnTop)
			{
				SetTopMost(true);
			}

			//Create application tray icon unless the setting hides it
			TrayIconCreate(hInstance);

			//Window message loop; the window and message queue outlive every UI page
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
