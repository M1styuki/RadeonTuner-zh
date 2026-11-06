#pragma once
#include "pch.h"
#include <atomic>
#include <mutex>
#include <thread>
#include "App.g.h"

namespace winrt::RadeonTuner::implementation
{
	struct MainPage;

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
		void CreateHostWindow(HINSTANCE hInstance, bool showUi, bool winOnTop);
		void LaunchUiWorker();
		void CompleteUiWorkerIfExited();
		void StopUiWorker();
		void NotifyWorkerClosing(DWORD processId);
		void NotifyHostSettingsChanged();
		void NotifyWorkerSettings(DWORD processId);
		winrt::fire_and_forget CloseUiWorkerAsync();
		void DispatcherInvoke(std::function<void()> const& action);
		void Exit(bool forceExit);
		void TrayIconCreate(HINSTANCE hInstance);
		void TrayIconDelete();
		HRESULT RefreshTrayIcon();
		HRESULT RefreshTrayIconForDpiChange();
		void RefreshNativeTheme();
		void ShowTrayMenu(POINT point, int autoCloseMilliseconds);

		//UI session lifecycle. EnsureUI and ReleaseUI run on the UI thread.
		void EnsureUI(bool showWindow);
		void ReleaseUI();
		void HandleUiReleaseTimer(UINT_PTR timerId);
		void HideToTray();
		void EnsureBackendStarted();
		void RunOnUi(std::function<void(MainPage&)> const& action, uint64_t expectedSession = 0);
		uint64_t CurrentUiSessionId();
		bool IsCurrentUiPage(MainPage const& page);
		void BeginUiWork();
		void EndUiWork();

#ifdef RADEONTUNER_UI_VALIDATION
		void UiValidationExit(int exitCode);
		bool TrayIconIsAddedForValidation();
		HICON TrayIconHandleForValidation();
		HRESULT TrayIconLastHresultForValidation();
		void ShowTrayMenuForValidation(int autoCloseMilliseconds);
		uint64_t UiSessionIdForValidation();
		int UiPageCreateCountForValidation();
		int BackendStartCountForValidation();
		bool UiReleasedForValidation();
		bool UiPageAliveForValidation();
		winrt::Windows::Foundation::IInspectable CurrentPageForValidation();
		void ShowFromTrayForValidation();
#endif

	private:
		winrt::Windows::Foundation::IInspectable CurrentPageForInternal();
		void CreateXamlIsland();
		void DestroyXamlIsland();
		void ScheduleUiRelease();
		void NotifyUiPageRegistered(MainPage const& page);
		void NotifyUiPageReleased();

		CoreDispatcher _dispatcher = NULL;
		WindowsXamlManager _windowsXamlManager = NULL;
		DesktopWindowXamlSource _desktopWindowXamlSource = NULL;
		HICON _trayIconHandle = NULL;
		bool _trayIconAdded = false;
		UINT _trayIconWidth = 0;
		UINT _trayIconHeight = 0;
		HRESULT _trayIconLastHresult = S_OK;
		bool _nativeThemeRefreshing = false;
		bool _nativeThemeApplied = false;
		bool _nativeThemeDark = false;
		bool _nativeThemeHighContrast = false;
		HWND _nativeThemeWindow = NULL;

		//The weak page is the only UI reference held by App. The page itself owns
		//all controls, animations and dispatcher timers.
		mutable std::mutex _uiPageMutex;
		winrt::weak_ref<winrt::Windows::Foundation::IInspectable> _uiPage;
		uint64_t _uiSessionId = 0;
		std::atomic<uint64_t> _uiSessionPublished{ 0 };
		int _uiPageCreateCount = 0;
		int _uiWorkCount = 0;
		bool _uiReleasePending = false;
		bool _uiEnsureInProgress = false;
		HANDLE _uiWorkerProcess = NULL;
		HANDLE _uiWorkerThread = NULL;
		HANDLE _uiWorkerInWrite = NULL;
		HANDLE _uiWorkerOutRead = NULL;
		HANDLE _uiWorkerJob = NULL;
		bool _uiWorkerRunning = false;
		bool _uiWorkerPending = false;
		bool _uiWorkerClosing = false;
		bool _exitAllPending = false;
		bool _workerExitStarted = false;
		bool _workerExitAll = false;
		bool _uiWorkerFirst = true;
		bool _hostSettingsInitialized = false;
		bool _hostHideTray = false;
		bool _hostThemeMono = false;
		std::string _uiWorkerStateJson;
		std::string _uiWorkerPayload;
		bool _uiWorkerPayloadReady = false;

		//Backend loops are started once, independently of the page lifetime.
		std::atomic<bool> _backendStarted{ false };
		int _backendStartCount = 0;
#ifdef RADEONTUNER_UI_VALIDATION
		bool _backendValidationCounted = false;
#else
		std::thread _backendMetricsThread;
		std::thread _backendProfileThread;
#endif
	};
}
