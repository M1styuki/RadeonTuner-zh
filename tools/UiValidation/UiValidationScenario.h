#pragma once
#include <Windows.h>
#include <chrono>
#include <filesystem>
#include <string>
#include <vector>
#include "pch.h"
#include <winrt/Windows.UI.Xaml.Automation.h>
#include "MainPage.h"
#include "AppVariables.h"
#include "MainVariables.h"
#include "AppPickerIdl.h"
#include "SettingCheck.h"
#include "UiValidationCommon.h"
#include "UiBehaviorTests.h"

namespace RadeonTunerUiValidation
{
	using winrt::Windows::UI::Xaml::DependencyObject;
	using winrt::Windows::UI::Xaml::FrameworkElement;
	using winrt::Windows::UI::Xaml::Visibility;
	using winrt::Windows::UI::Xaml::Media::VisualTreeHelper;
	using winrt::RadeonTuner::implementation::MainPage;

	const int SampleWaitMilliseconds = 500;

	inline int CountVisualTree(DependencyObject const& root)
	{
		int count = 1;
		int childCount = VisualTreeHelper::GetChildrenCount(root);
		for (int index = 0; index < childCount; index++)
		{
			count += CountVisualTree(VisualTreeHelper::GetChild(root, index));
		}
		return count;
	}

	inline int MinInt(int left, int right)
	{
		return left < right ? left : right;
	}

	inline bool IsEffectivelyVisible(DependencyObject const& element, DependencyObject const& pageRoot)
	{
		DependencyObject current = element;
		while (current)
		{
			auto frameworkElement = current.try_as<FrameworkElement>();
			if (frameworkElement && frameworkElement.Visibility() == Visibility::Collapsed)
			{
				return false;
			}
			if (winrt::get_abi(current) == winrt::get_abi(pageRoot))
			{
				break;
			}
			current = VisualTreeHelper::GetParent(current);
		}
		return true;
	}

	inline std::wstring TooltipText(FrameworkElement const& element)
	{
		try
		{
			auto tooltip = winrt::Windows::UI::Xaml::Controls::ToolTipService::GetToolTip(element);
			auto text = tooltip.try_as<winrt::hstring>();
			return text ? std::wstring(text.value()) : L"";
		}
		catch (...)
		{
			return L"";
		}
	}

	inline int ValidateFlatPage(FrameworkElement const& pageRoot, std::wstring const& pageName)
	{
		int validatedRows = 0;
		auto headingStyle = winrt::Windows::UI::Xaml::Application::Current().Resources().Lookup(winrt::box_value(L"ApplicationSectionHeader")).as<winrt::Windows::UI::Xaml::Style>();
		std::function<void(DependencyObject const&)> inspect = [&](DependencyObject const& node)
		{
			auto textBlock = node.try_as<winrt::Windows::UI::Xaml::Controls::TextBlock>();
			if (textBlock && textBlock.Style() && winrt::get_abi(textBlock.Style()) == winrt::get_abi(headingStyle))
			{
				throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Group heading remains on page: " + pageName));
			}

			auto border = node.try_as<winrt::Windows::UI::Xaml::Controls::Border>();
			if (border && border.Background() && (border.CornerRadius().TopLeft > 0 || border.CornerRadius().TopRight > 0 || border.CornerRadius().BottomLeft > 0 || border.CornerRadius().BottomRight > 0))
			{
				//Borders inside native input controls are allowed and are not page cards
				bool insideNativeControl = false;
				DependencyObject ancestor = VisualTreeHelper::GetParent(border);
				while (ancestor && winrt::get_abi(ancestor) != winrt::get_abi(pageRoot))
				{
					if (ancestor.try_as<winrt::Windows::UI::Xaml::Controls::Control>())
					{
						insideNativeControl = true;
						break;
					}
					ancestor = VisualTreeHelper::GetParent(ancestor);
				}
				if (!insideNativeControl)
				{
					throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Card-like filled border remains on page: " + pageName));
				}
			}

			auto toggle = node.try_as<winrt::Windows::UI::Xaml::Controls::ToggleSwitch>();
			if (toggle && IsEffectivelyVisible(toggle, pageRoot))
			{
				std::wstring toggleName(winrt::Windows::UI::Xaml::Automation::AutomationProperties::GetName(toggle).c_str());
				if (toggleName.empty())
				{
					throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Toggle has no automation label on page: " + pageName));
				}
				if (!SomeText(toggle.OnContent()).empty() || !SomeText(toggle.OffContent()).empty())
				{
					throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Toggle still has On/OffContent text on page: " + pageName));
				}
				if (toggle.HorizontalAlignment() != winrt::Windows::UI::Xaml::HorizontalAlignment::Right)
				{
					throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Toggle is not right aligned on page: " + pageName));
				}

				auto parent = VisualTreeHelper::GetParent(toggle).try_as<winrt::Windows::UI::Xaml::Controls::Grid>();
				if (!parent)
				{
					throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Toggle row is not a Grid on page: " + pageName));
				}
				int toggleColumn = winrt::Windows::UI::Xaml::Controls::Grid::GetColumn(toggle);
				int labelColumn = 1000;
				int helpColumn = 1000;
				int warningColumn = 1000;
				int childCount = VisualTreeHelper::GetChildrenCount(parent);
				for (int index = 0; index < childCount; index++)
				{
					auto child = VisualTreeHelper::GetChild(parent, index).try_as<FrameworkElement>();
					if (!child) { continue; }
					int childColumn = winrt::Windows::UI::Xaml::Controls::Grid::GetColumn(child);
					if (childColumn >= toggleColumn) { continue; }
					auto textBlockChild = child.try_as<winrt::Windows::UI::Xaml::Controls::TextBlock>();
					if (textBlockChild && !std::wstring(textBlockChild.Text().c_str()).empty())
					{
						labelColumn = MinInt(labelColumn, childColumn);
					}
					auto imageChild = child.try_as<winrt::Windows::UI::Xaml::Controls::Image>();
					if (imageChild)
					{
						std::wstring tooltip = TooltipText(imageChild);
						if (!tooltip.empty())
						{
							if (tooltip[0] == L'*') { warningColumn = MinInt(warningColumn, childColumn); }
							else { helpColumn = MinInt(helpColumn, childColumn); }
						}
					}
				}
				if (labelColumn == 1000)
				{
					throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Toggle row has no leading label on page: " + pageName));
				}
				if (helpColumn != 1000 && helpColumn <= labelColumn)
				{
					throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Help icon does not follow the label on page: " + pageName));
				}
				int warningReference = helpColumn != 1000 ? helpColumn : labelColumn;
				if (warningColumn != 1000 && warningColumn <= warningReference)
				{
					throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Warning icon does not follow the help icon/label on page: " + pageName));
				}
				validatedRows++;
			}

			int childCount = VisualTreeHelper::GetChildrenCount(node);
			for (int index = 0; index < childCount; index++)
			{
				inspect(VisualTreeHelper::GetChild(node, index));
			}
		};
		inspect(pageRoot);
		return validatedRows;
	}

	inline void InitializePageControls(MainPage* page)
	{
		//Keep all save paths disabled for the whole validation run
		disable_saving = true;
		disable_saving_settings = true;
		disable_saving_customresolution = true;

		//Populate the pure UI enum lists once, this does not call ADL/ADLX or touch the GPU
		static bool valuesPrepared = false;
		if (!valuesPrepared)
		{
			page->AdlxValuesPrepare();
			valuesPrepared = true;
		}

		//Initialize only real controls and private settings, no shortcut/registry queries
		auto setToggle = [](auto const& toggle, char const* settingName)
		{
			std::optional<bool> settingValue = AppVariables::Settings.Load<bool>(settingName);
			if (settingValue.has_value())
			{
				toggle.IsOn(settingValue.value());
			}
		};
		setToggle(page->toggleswitch_Window_Top(), "SetTopMost");
		setToggle(page->toggleswitch_Close_Tray(), "CloseTray");
		setToggle(page->toggleswitch_Hide_Tray_Icon(), "HideTrayIcon");
		setToggle(page->toggleswitch_StartWindowVisible(), "StartWindowVisible");
		setToggle(page->toggleswitch_StartCheckUpdate(), "StartCheckUpdate");
		setToggle(page->toggleswitch_ShowExperimental(), "ShowExperimental");
		setToggle(page->toggleswitch_SkipSupportCheckGraphics(), "SkipSupportCheckGraphics");
		setToggle(page->toggleswitch_ThemeIconMono(), "ThemeIconMono");

		std::optional<int> themeStyle = AppVariables::Settings.Load<int>("ThemeStyle");
		if (themeStyle.has_value())
		{
			page->combobox_ThemeStyle().SelectedIndex(themeStyle.value());
		}

		page->ThemeAdjustStyle();
		page->ShowExperimentalSettings(true);
		page->SelectDefaultIndexes();
		page->UpdateLayout();
	}

	inline void ShowMessageBoxOverlay(MainPage* page, std::wstring const& question, std::wstring const& description, std::vector<std::wstring> const& answers)
	{
		page->textblock_Overlay_MessageBox_Text().Text(question);
		if (description.empty())
		{
			page->textblock_Overlay_MessageBox_SubText().Visibility(Visibility::Collapsed);
		}
		else
		{
			page->textblock_Overlay_MessageBox_SubText().Text(description);
			page->textblock_Overlay_MessageBox_SubText().Visibility(Visibility::Visible);
		}

		page->listview_Overlay_MessageBox().Items().Clear();
		for (std::wstring const& answer : answers)
		{
			page->listview_Overlay_MessageBox().Items().Append(winrt::box_value(winrt::hstring(answer)));
		}
		page->listview_Overlay_MessageBox().SelectedIndex(-1);
		page->grid_Overlay_MessageBox().Visibility(Visibility::Visible);
		page->listview_Overlay_MessageBox().Focus(winrt::Windows::UI::Xaml::FocusState::Keyboard);
	}

	inline void HideMessageBoxOverlay(MainPage* page)
	{
		page->listview_Overlay_MessageBox().ItemsSource(NULL);
		page->grid_Overlay_MessageBox().Visibility(Visibility::Collapsed);
	}

	inline winrt::Windows::Foundation::Collections::IObservableVector<winrt::RadeonTuner::AppPickerIdl> CreateSampleAppPickerList()
	{
		auto sampleList = winrt::single_threaded_observable_vector<winrt::RadeonTuner::AppPickerIdl>();
		struct SampleEntry
		{
			const wchar_t* AppName;
			const wchar_t* ExeName;
		};
		const SampleEntry sampleEntries[] =
		{
			{ L"\u6837\u672C\u5E94\u7528 A", L"sample-a.exe" },
			{ L"Caf\u00E9 Sample", L"cafe-sample.exe" },
			{ L"\u6D4B\u8BD5\u6E38\u620F-\u00E9", L"game-\u00E9.exe" },
			{ L"Sample 04", L"sample-04.exe" },
			{ L"Sample 05", L"sample-05.exe" }
		};
		for (SampleEntry const& sampleEntry : sampleEntries)
		{
			winrt::RadeonTuner::AppPickerIdl pickerItem;
			pickerItem.IconPath(L"/Assets/LauncherSteam.png");
			pickerItem.AppName(winrt::hstring(sampleEntry.AppName));
			pickerItem.ExeName(winrt::hstring(sampleEntry.ExeName));
			pickerItem.Detail(winrt::hstring(L"UiValidation sample"));
			sampleList.Append(pickerItem);
		}
		return sampleList;
	}

	inline void ShowAppPickerOverlay(MainPage* page, winrt::Windows::Foundation::Collections::IObservableVector<winrt::RadeonTuner::AppPickerIdl> const& items)
	{
		page->button_Overlay_AppPicker_Confirm().Visibility(Visibility::Visible);
		page->button_Overlay_AppPicker_Cancel().Visibility(Visibility::Visible);
		page->button_Overlay_AppPicker_Confirm().Content(winrt::box_value(L"Add selected items"));
		page->textblock_Overlay_AppPicker_Text().Text(L"Add applications");
		page->grid_Overlay_AppPicker().Visibility(Visibility::Visible);
		page->listview_Overlay_AppPicker().ItemsSource(items);
		appPickerCompleted = false;
		appPickerCancel = false;
	}

	inline void HideAppPickerOverlay(MainPage* page)
	{
		page->grid_Overlay_AppPicker().Visibility(Visibility::Collapsed);
		page->listview_Overlay_AppPicker().ItemsSource(NULL);
	}

	inline int CycleSimpleOverlay(FrameworkElement const& overlay, std::wstring const& name)
	{
		overlay.Visibility(Visibility::Visible);
		PumpMessagesFor(60);
		Require(overlay.Visibility() == Visibility::Visible, L"Overlay did not open: " + name);
		int treeCount = CountVisualTree(overlay);
		Require(treeCount > 1, L"Overlay control tree not realized: " + name);
		overlay.Visibility(Visibility::Collapsed);
		PumpMessagesFor(20);
		Require(overlay.Visibility() == Visibility::Collapsed, L"Overlay did not close: " + name);
		return treeCount;
	}

	inline void AppendCurrentMeasurement(std::string const& stage, int round, int pageIndex, std::wstring const& detail)
	{
		ProcessMetrics metrics = SampleProcessMetrics();
		bool windowVisible = IsWindowVisible(AppVariables::App.GetWindowHandle()) != FALSE;
		AppendMeasurement(stage, round, pageIndex, ElapsedSinceStartMilliseconds(), metrics, windowVisible, detail);
	}

	inline void SampleStage(std::string const& stage, int round, int pageIndex, std::wstring const& detail)
	{
		PumpMessagesFor(SampleWaitMilliseconds);
		AppendCurrentMeasurement(stage, round, pageIndex, detail);
	}

	inline std::wstring SettingsFilePath()
	{
		return AppVariables::SaveDataPath + L"\\Settings\\Settings.json";
	}

	inline std::optional<bool> LoadHideTrayIconFromDisk()
	{
		AVSettingsJson settings(SettingsFilePath());
		return settings.Load<bool>("HideTrayIcon");
	}

	inline void RequireTrayState(bool expectedAdded, std::wstring const& context)
	{
		bool added = AppVariables::App.TrayIconIsAddedForValidation();
		HICON iconHandle = AppVariables::App.TrayIconHandleForValidation();
		HRESULT lastResult = AppVariables::App.TrayIconLastHresultForValidation();
		bool handleMatches = expectedAdded ? iconHandle != NULL : iconHandle == NULL;
		if (added != expectedAdded || !handleMatches)
		{
			std::wstring detail = context;
			detail += L"; expected_added=" + std::to_wstring(expectedAdded ? 1 : 0);
			detail += L"; actual_added=" + std::to_wstring(added ? 1 : 0);
			detail += L"; icon_handle=" + std::to_wstring((uintptr_t)iconHandle);
			detail += L"; last_hresult=" + Utf8ToWide(HresultToString(lastResult));
			throw winrt::hresult_error(FAILED(lastResult) ? lastResult : E_FAIL, winrt::hstring(detail));
		}
	}

	inline void SimulateTaskbarCreated(HWND windowHandle)
	{
		UINT taskbarCreatedMessage = RegisterWindowMessageW(L"TaskbarCreated");
		Require(taskbarCreatedMessage != 0, L"Could not register the TaskbarCreated message.");
		Require(PostMessageW(windowHandle, taskbarCreatedMessage, 0, 0) != FALSE, L"Could not post TaskbarCreated to the main window.");
		PumpMessagesFor(200);
	}

	inline void ToggleHideTrayIcon(MainPage* page, bool hide)
	{
		//Run the real Toggled handler, but only against the private test settings file
		bool previousSavingSettings = disable_saving_settings;
		disable_saving_settings = false;
		page->toggleswitch_Hide_Tray_Icon().IsOn(hide);
		disable_saving_settings = previousSavingSettings;
		PumpMessagesFor(150);
	}

	inline void ToggleThemeIconMono(MainPage* page, bool mono)
	{
		//Run the real theme toggle so its tray refresh path is exercised as well
		bool previousSavingSettings = disable_saving_settings;
		disable_saving_settings = false;
		page->toggleswitch_ThemeIconMono().IsOn(mono);
		disable_saving_settings = previousSavingSettings;
		PumpMessagesFor(150);
	}

	inline void VerifyTraySettingsAndLifecycle(MainPage* page, HWND windowHandle)
	{
		//Missing setting defaults to false and SettingCheck writes that default to the private settings
		AppVariables::Settings.Remove("HideTrayIcon");
		std::optional<bool> missingSetting = LoadHideTrayIconFromDisk();
		Require(!missingSetting.has_value(), L"HideTrayIcon was unexpectedly present after removal.");
		SettingCheck();
		std::optional<bool> defaultSetting = LoadHideTrayIconFromDisk();
		Require(defaultSetting.has_value() && !defaultSetting.value(), L"HideTrayIcon default is not false.");
		TrayDefaultFalseVerified = true;

		//A fresh test start shows the tray icon
		page->toggleswitch_Hide_Tray_Icon().IsOn(false);
		RequireTrayState(true, L"Tray icon should be present before hiding.");

		//Save and apply the hiding setting through the real Toggled handler
		ToggleHideTrayIcon(page, true);
		std::optional<bool> savedSetting = LoadHideTrayIconFromDisk();
		Require(savedSetting.has_value() && savedSetting.value(), L"HideTrayIcon true was not persisted.");
		TrayPersistedVerified = true;
		RequireTrayState(false, L"Tray icon should be removed after hiding.");
		TrayHiddenVerified = true;
		Require(IsWindow(windowHandle), L"Main window was destroyed after hiding the tray icon.");

		//Closing to tray still only hides the window and leaves the process running
		bool previousCloseTray = AppVariables::Settings.Load<bool>("CloseTray").value_or(false);
		AppVariables::Settings.Set("CloseTray", true);
		AppVariables::App.Exit(false);
		PumpMessagesFor(150);
		Require(!IsWindowVisible(windowHandle), L"Close to tray did not hide the main window while the tray icon was hidden.");
		Require(IsWindow(windowHandle), L"Close to tray destroyed the main window while the tray icon was hidden.");
		ShowWindow(windowHandle, SW_SHOWNORMAL);
		PumpMessagesFor(150);
		Require(IsWindowVisible(windowHandle), L"Main window did not show again after close to tray.");
		AppVariables::Settings.Set("CloseTray", previousCloseTray);

		//Simulate loading the persisted value on the next start
		std::optional<bool> reloadedSetting = LoadHideTrayIconFromDisk();
		Require(reloadedSetting.has_value() && reloadedSetting.value(), L"HideTrayIcon true was not readable from a fresh settings instance.");
		page->toggleswitch_Hide_Tray_Icon().IsOn(false);
		page->toggleswitch_Hide_Tray_Icon().IsOn(reloadedSetting.value());
		Require(page->toggleswitch_Hide_Tray_Icon().IsOn(), L"HideTrayIcon reload did not restore the toggle state.");
		TrayReloadedVerified = true;
		RequireTrayState(false, L"Tray icon should stay hidden after settings reload.");

		//Explorer rebuilding the notification area must not bring the hidden icon back
		SimulateTaskbarCreated(windowHandle);
		RequireTrayState(false, L"TaskbarCreated brought back a hidden tray icon.");
		TrayHiddenAfterTaskbarRecreatedVerified = true;

		//Changing the theme icon while hidden must also respect the hide setting
		ToggleThemeIconMono(page, true);
		RequireTrayState(false, L"Theme icon change brought back a hidden tray icon.");
		TrayThemeChangeWhileHiddenVerified = true;
		ToggleThemeIconMono(page, false);
		RequireTrayState(false, L"Restoring the color theme icon brought back a hidden tray icon.");

		//Unhiding restores the icon and the setting is persisted
		ToggleHideTrayIcon(page, false);
		std::optional<bool> restoredSetting = LoadHideTrayIconFromDisk();
		Require(restoredSetting.has_value() && !restoredSetting.value(), L"HideTrayIcon false was not persisted.");
		RequireTrayState(true, L"Tray icon was not restored after unhiding.");
		TrayRestoredVerified = true;

		//TaskbarCreated also recreates a visible icon without violating the setting
		SimulateTaskbarCreated(windowHandle);
		RequireTrayState(true, L"Tray icon was not recreated after TaskbarCreated.");
		TrayVisibleAfterTaskbarRecreatedVerified = true;

		//Repeat the real toggle path and verify icon handles/user objects do not grow
		ProcessMetrics beforeToggles = SampleProcessMetrics();
		for (int round = 1; round <= 20; round++)
		{
			bool hide = (round % 2) == 1;
			ToggleHideTrayIcon(page, hide);
			RequireTrayState(!hide, L"Tray icon state mismatch during repeated toggle round " + std::to_wstring(round) + L".");
			TrayToggleRoundsCompleted++;
		}
		PumpMessagesFor(250);
		ProcessMetrics afterToggles = SampleProcessMetrics();
		TrayUserObjectDelta = (int)afterToggles.UserObjects - (int)beforeToggles.UserObjects;
		TrayHandleCountDelta = (int)afterToggles.HandleCount - (int)beforeToggles.HandleCount;

		std::wstring toggleDetail = L"rounds=" + std::to_wstring(TrayToggleRoundsCompleted);
		toggleDetail += L";user_object_delta=" + std::to_wstring(TrayUserObjectDelta);
		toggleDetail += L";handle_delta=" + std::to_wstring(TrayHandleCountDelta);
		toggleDetail += L";gdi_object_delta=" + std::to_wstring((int)afterToggles.GdiObjects - (int)beforeToggles.GdiObjects);
		AppendMeasurement("tray_toggle_stress", TrayToggleRoundsCompleted, -1, ElapsedSinceStartMilliseconds(), afterToggles, IsWindowVisible(windowHandle), toggleDetail);

		Require(afterToggles.UserObjects <= beforeToggles.UserObjects + 2, L"Repeated tray toggles leaked USER objects.");
		Require(afterToggles.GdiObjects <= beforeToggles.GdiObjects + 2, L"Repeated tray toggles leaked GDI objects.");
		Require(afterToggles.HandleCount <= beforeToggles.HandleCount + 8, L"Repeated tray toggles leaked process handles.");
		RequireTrayState(true, L"Tray icon should end visible after repeated toggles.");
	}

	inline void WriteResultJson(bool success, long failureHresult)
	{
		nlohmann::json report = nlohmann::json::object();
		report["finished_utc"] = UtcNowString();
		report["mode"] = AutoMode ? "auto" : "stay";
		report["duration_ms"] = ElapsedSinceStartMilliseconds();
		report["unicode_path_tests_passed"] = UnicodePathTestsPassed;
		report["system_integration_tests_passed"] = SystemIntegrationTestsPassed;
		report["pages_navigated"] = PagesNavigated;
		report["popup_rounds_completed"] = PopupRoundsCompleted;
		report["real_messagebox_rounds_completed"] = RealMessageBoxRoundsCompleted;
		report["cross_process_recall_verified"] = CrossProcessRecallVerified;
		report["cross_process_recall_child_exit_code"] = CrossProcessRecallChildExitCode;
		report["cross_process_recall_ms"] = CrossProcessRecallMilliseconds;
		report["enum_lists_prepared"] = EnumListsPrepared;
		report["theme_switch_verified"] = ThemeSwitchVerified;
		report["ui_compatibility_verified"] = UiCompatibilityVerified;
		report["flat_layout_verified"] = FlatLayoutVerified;
		report["flat_toggle_rows_verified"] = FlatToggleRowsVerified;
		report["window_handle_visible_after_show"] = IsWindowVisible(AppVariables::App.GetWindowHandle()) != FALSE;
		report["measurements_csv"] = WideToUtf8(MeasurementsCsvPath);
		report["test_data_path"] = WideToUtf8(TestDataPath);

		nlohmann::json tray = nlohmann::json::object();
		tray["default_false_verified"] = TrayDefaultFalseVerified;
		tray["saved_true_verified"] = TrayPersistedVerified;
		tray["reload_verified"] = TrayReloadedVerified;
		tray["hidden_verified"] = TrayHiddenVerified;
		tray["hidden_after_taskbar_recreated_verified"] = TrayHiddenAfterTaskbarRecreatedVerified;
		tray["restored_verified"] = TrayRestoredVerified;
		tray["visible_after_taskbar_recreated_verified"] = TrayVisibleAfterTaskbarRecreatedVerified;
		tray["theme_change_while_hidden_verified"] = TrayThemeChangeWhileHiddenVerified;
		tray["toggle_rounds"] = TrayToggleRoundsCompleted;
		tray["user_object_delta"] = TrayUserObjectDelta;
		tray["handle_count_delta"] = TrayHandleCountDelta;
		report["tray"] = tray;

		report["limitations"] = nlohmann::json::array({
			"ADL/ADLX and the normal hardware loops are not initialized in this isolated build",
			"GPU metrics/profile auto-apply behavior is not exercised",
			"a full process restart is not performed; persisted setting reload is simulated with a fresh AVSettingsJson instance",
			"the normal startup path is not launched; cross-process single-instance recall reuses the same matcher from the isolated entry point",
			"visual appearance is not validated by this automated build"
		});

		if (success)
		{
			report["status"] = (UnicodePathTestsPassed && SystemIntegrationTestsPassed) ? "success" : "completed_with_preflight_failures";
		}
		else
		{
			report["status"] = "failed";
			report["failure"] = nlohmann::json::object();
			report["failure"]["context"] = WideToUtf8(ScenarioFailureContext);
			report["failure"]["hresult"] = HresultToString(failureHresult);
			report["failure"]["message"] = WideToUtf8(ScenarioFailureMessage);
		}
		WriteUtf8File(ResultJsonPath, report.dump(2));
	}

	inline bool RunCrossProcessRecall(int& childExitCode, uint64_t& elapsedMilliseconds)
	{
		childExitCode = -1;
		elapsedMilliseconds = 0;

		//Start a second copy of this isolated test build. Its entry point calls the same
		//RadeonTunerSingleInstance matcher as the normal startup path and shows our window.
		std::wstring executablePath = PathGetAppExecutable();
		std::wstring commandLine = L"\"" + executablePath + L"\" -ui-recall-check";
		std::vector<wchar_t> commandLineBuffer(commandLine.begin(), commandLine.end());
		commandLineBuffer.push_back(L'\0');

		STARTUPINFOW startupInfo = { };
		startupInfo.cb = sizeof(startupInfo);
		PROCESS_INFORMATION processInformation = { };
		if (!CreateProcessW(executablePath.c_str(), commandLineBuffer.data(), NULL, NULL, FALSE, 0, NULL, PathGetAppRoot().c_str(), &startupInfo, &processInformation))
		{
			childExitCode = (int)GetLastError();
			return false;
		}

		ULONGLONG startTime = GetTickCount64();
		bool exited = false;
		while (GetTickCount64() - startTime < 15000)
		{
			//Keep pumping so the child's Show_ProcessByWindowHandle request is handled
			PumpMessagesFor(50);
			DWORD exitCode = STILL_ACTIVE;
			if (GetExitCodeProcess(processInformation.hProcess, &exitCode) && exitCode != STILL_ACTIVE)
			{
				childExitCode = (int)exitCode;
				exited = true;
				break;
			}
		}
		if (!exited)
		{
			TerminateProcess(processInformation.hProcess, 124);
			WaitForSingleObject(processInformation.hProcess, 1000);
			childExitCode = 124;
		}
		elapsedMilliseconds = GetTickCount64() - startTime;
		CloseHandle(processInformation.hThread);
		CloseHandle(processInformation.hProcess);
		return exited && childExitCode == 0;
	}

	inline winrt::Windows::Foundation::IAsyncAction RunSynchronous(MainPage* page)
	{
		ScenarioStarted = true;
		ScenarioStart = std::chrono::steady_clock::now();
		MeasurementsCsvPath = TestDataPath + L"\\ui-validation-measurements.csv";
		ResultJsonPath = TestDataPath + L"\\ui-validation-result.json";
		OpenMeasurements();
		int exitCode = 0;
		try
		{
			InitializePageControls(page);
			HWND windowHandle = AppVariables::App.GetWindowHandle();
			Require(windowHandle != NULL, L"Main window handle is not available.");
			Require(IsWindowVisible(windowHandle), L"Main window is not visible after creation.");

			//Pure enum preparation must have populated the combo boxes without touching the GPU
			Require(page->combobox_ThemeStyle().Items().Size() == 3, L"Theme style enum list was not prepared.");
			Require(page->combobox_Display_ScalingMode().Items().Size() > 0, L"Display scaling enum list was not prepared.");
			EnumListsPrepared = true;

			//Switch to the light theme and back so both theme dictionaries and the native theme refresh are exercised
			AppVariables::Settings.Set("ThemeStyle", 2);
			page->combobox_ThemeStyle().SelectedIndex(2);
			page->ThemeAdjustStyle();
			PumpMessagesFor(250);
			Require(page->RequestedTheme() == winrt::Windows::UI::Xaml::ElementTheme::Light, L"Light theme was not applied.");
			AppVariables::Settings.Set("ThemeStyle", 0);
			page->combobox_ThemeStyle().SelectedIndex(0);
			page->ThemeAdjustStyle();
			PumpMessagesFor(250);
			Require(page->RequestedTheme() == winrt::Windows::UI::Xaml::ElementTheme::Dark, L"Dark theme was not restored.");
			ThemeSwitchVerified = true;

			//Let the real page render before the first sample
			PumpMessagesFor(700);
			SampleStage("idle_after_show", 0, -1, L"window shown, before page navigation");

			//Verify the tray setting, persistence, close-to-tray and Explorer rebuild paths
			VerifyTraySettingsAndLifecycle(page, windowHandle);
			SampleStage("tray_verified", 0, -1, L"tray setting and lifecycle checks completed");

			//Navigate all seven content pages through the normal selection handler
			auto checkPage = [&](int pageIndex, std::wstring const& pageName, FrameworkElement const& pageElement)
			{
				page->listview_Main().SelectedIndex(pageIndex);
				page->UpdateLayout();
				PumpMessagesFor(SampleWaitMilliseconds);
				bool pageVisible = pageElement.Visibility() == Visibility::Visible;
				int treeCount = CountVisualTree(pageElement);
				double actualWidth = pageElement.ActualWidth();
				double actualHeight = pageElement.ActualHeight();
				std::wstring detail = pageName + L";visible=" + std::to_wstring(pageVisible ? 1 : 0) + L";width=" + std::to_wstring((int)actualWidth) + L";height=" + std::to_wstring((int)actualHeight) + L";tree=" + std::to_wstring(treeCount);
				AppendCurrentMeasurement("page_switch", 0, pageIndex, detail);
				Require(pageVisible, L"Page is not visible: " + pageName);
				Require(treeCount > 1, L"Page control tree not realized: " + pageName);
				Require(actualWidth > 0 && actualHeight > 0, L"Page did not lay out: " + pageName);
				bool settingsPage = pageIndex == 5;
				bool headerVisible = page->border_Header().Visibility() == Visibility::Visible;
				bool actionBarVisible = page->border_ActionButtons().Visibility() == Visibility::Visible;
				Require(headerVisible == !settingsPage, L"Selection header visibility is wrong on page: " + pageName);
				Require(actionBarVisible == !settingsPage, L"Action bar visibility is wrong on page: " + pageName);
				int flatRows = ValidateFlatPage(pageElement, pageName);
				FlatToggleRowsVerified += flatRows;
				detail += L";flat_rows=" + std::to_wstring(flatRows);
				PagesNavigated++;
			};
			checkPage(0, L"Tuning", page->stackpanel_Tuning());
			checkPage(1, L"Fans", page->stackpanel_Fans());
			checkPage(2, L"Graphics", page->stackpanel_Graphics());
			checkPage(3, L"Display", page->stackpanel_Display());
			checkPage(4, L"Multimedia", page->stackpanel_Multimedia());
			checkPage(5, L"Settings", page->stackpanel_Settings());
			checkPage(6, L"Information", page->stackpanel_Information());
			Require(PagesNavigated == 7, L"Not all seven pages were navigated.");
			Require(FlatToggleRowsVerified > 0, L"No toggle rows were validated.");
			FlatLayoutVerified = true;

			//Exercise gamma/base/RGB and fan-mode events, profile serialization and old-profile compatibility
			VerifyGammaAndFanBehavior(page);

			//Return to the first page before cycling overlays
			page->listview_Main().SelectedIndex(0);
			page->UpdateLayout();
			PumpMessagesFor(SampleWaitMilliseconds);

			//Same deterministic sample list for every popup round
			auto sampleAppList = CreateSampleAppPickerList();

			for (int round = 1; round <= 20; round++)
			{
				std::wstring detail = L"round=" + std::to_wstring(round);

				//Run the real asynchronous ShowMessageBox path and answer it deterministically
				int answerIndex = (round % 2 == 0) ? 1 : 0;
				auto messageBoxOperation = page->ShowMessageBox(L"UI validation message", L"Deterministic MessageBox sample.", { L"OK", L"Cancel" });
				PumpMessagesFor(80);
				Require(page->grid_Overlay_MessageBox().Visibility() == Visibility::Visible, L"MessageBox overlay did not open.");
				int messageBoxTree = CountVisualTree(page->grid_Overlay_MessageBox());
				int messageBoxItems = (int)page->listview_Overlay_MessageBox().Items().Size();
				Require(messageBoxItems == 2, L"MessageBox sample item count mismatch.");
				page->listview_Overlay_MessageBox().SelectedIndex(answerIndex);
				page->listview_Overlay_MessageBox_ItemClick(nullptr, nullptr);
				int messageBoxResult = co_await messageBoxOperation;
				Require(messageBoxResult == answerIndex, L"Real MessageBox returned an unexpected result.");
				Require(page->grid_Overlay_MessageBox().Visibility() == Visibility::Collapsed, L"MessageBox overlay did not close.");
				RealMessageBoxRoundsCompleted++;
				detail += L";messagebox_tree=" + std::to_wstring(messageBoxTree) + L";messagebox_result=" + std::to_wstring(messageBoxResult);

				ShowAppPickerOverlay(page, sampleAppList);
				PumpMessagesFor(60);
				Require(page->grid_Overlay_AppPicker().Visibility() == Visibility::Visible, L"AppPicker overlay did not open.");
				int appPickerTree = CountVisualTree(page->grid_Overlay_AppPicker());
				int appPickerItems = (int)page->listview_Overlay_AppPicker().Items().Size();
				Require(appPickerItems == 5, L"AppPicker sample item count mismatch.");
				HideAppPickerOverlay(page);
				Require(page->grid_Overlay_AppPicker().Visibility() == Visibility::Collapsed, L"AppPicker overlay did not close.");
				detail += L";apppicker_tree=" + std::to_wstring(appPickerTree) + L";apppicker_items=" + std::to_wstring(appPickerItems);

				detail += L";eyefinity_tree=" + std::to_wstring(CycleSimpleOverlay(page->grid_Overlay_ManageEyefinity(), L"ManageEyefinity"));
				detail += L";customres_tree=" + std::to_wstring(CycleSimpleOverlay(page->grid_Overlay_CustomResolution(), L"CustomResolution"));
				detail += L";confirmres_tree=" + std::to_wstring(CycleSimpleOverlay(page->grid_Overlay_ConfirmResolution(), L"ConfirmResolution"));
				detail += L";notification_tree=" + std::to_wstring(CycleSimpleOverlay(page->grid_Overlay_Notification(), L"Notification"));

				SampleStage("popup_round", round, -1, detail);
				PopupRoundsCompleted++;
			}

			//Hide to tray, then let a second process recall this window through the shared matcher
			ShowWindow(windowHandle, SW_HIDE);
			PumpMessagesFor(SampleWaitMilliseconds);
			bool windowHidden = IsWindowVisible(windowHandle) == FALSE;
			AppendCurrentMeasurement("hidden_to_tray", 0, -1, L"ShowWindow(SW_HIDE)");
			Require(windowHidden, L"Window did not hide.");

			int recallChildExitCode = -1;
			uint64_t recallMilliseconds = 0;
			bool recallSucceeded = RunCrossProcessRecall(recallChildExitCode, recallMilliseconds);
			bool recallWindowVisible = IsWindowVisible(windowHandle) != FALSE;
			CrossProcessRecallVerified = recallSucceeded && recallWindowVisible;
			CrossProcessRecallChildExitCode = recallChildExitCode;
			CrossProcessRecallMilliseconds = recallMilliseconds;
			std::wstring recallDetail = L"child_exit=" + std::to_wstring(recallChildExitCode);
			recallDetail += L";elapsed_ms=" + std::to_wstring(recallMilliseconds);
			recallDetail += L";window_visible=" + std::to_wstring(recallWindowVisible ? 1 : 0);
			AppendCurrentMeasurement("cross_process_recall", 0, -1, recallDetail);
			Require(recallSucceeded, L"Cross-process single-instance recall helper failed, exit code: " + std::to_wstring(recallChildExitCode));
			Require(recallWindowVisible, L"Cross-process single-instance recall did not show the existing window.");

			PumpMessagesFor(SampleWaitMilliseconds);
			bool windowShown = IsWindowVisible(windowHandle) != FALSE;
			AppendCurrentMeasurement("shown_again", 0, -1, L"shown by cross-process recall");
			Require(windowShown, L"Window did not show again.");

			page->listview_Main().SelectedIndex(0);
			page->UpdateLayout();
			SampleStage("final_idle", 0, -1, L"final page 0 idle");

			ScenarioCompleted = true;
			WriteResultJson(true, 0);
			exitCode = UnicodePathTestsPassed ? 0 : 2;
			AVDebugWriteLine("UI validation scenario completed.");
		}
		catch (winrt::hresult_error const& exception)
		{
			RecordScenarioFailure(exception.code(), L"RunSynchronous", exception.message().c_str());
			ScenarioCompleted = true;
			WriteResultJson(false, exception.code());
			exitCode = 3;
			AVDebugWriteLine(L"UI validation scenario failed: " << exception.message().c_str());
		}
		catch (std::exception const& exception)
		{
			RecordScenarioFailure(E_FAIL, L"RunSynchronous", Utf8ToWide(exception.what()));
			ScenarioCompleted = true;
			WriteResultJson(false, E_FAIL);
			exitCode = 3;
			AVDebugWriteLine("UI validation scenario failed with a standard exception.");
		}
		catch (...)
		{
			RecordScenarioFailure(E_FAIL, L"RunSynchronous", L"Unknown exception");
			ScenarioCompleted = true;
			WriteResultJson(false, E_FAIL);
			exitCode = 3;
			AVDebugWriteLine("UI validation scenario failed with an unknown exception.");
		}

		CloseMeasurements();
		if (AutoMode)
		{
			AppVariables::App.UiValidationExit(exitCode);
		}
	}
}
