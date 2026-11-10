#pragma once
#include "pch.h"
#include <winrt/Windows.UI.ViewManagement.h>
#include <winrt/Windows.UI.Xaml.Media.Animation.h>
#include <cmath>
#include <cerrno>
#include <cwchar>
#include <cwctype>
#include <limits>
#include <utility>
#include <atomic>
#include <condition_variable>
#include <mutex>

//Launchers
#include "SteamFuncLibraryFolders.h"
#include "SteamFuncAppInfo.h"
#include "SteamStructures.h"
#include "SteamFunctions.h"

#include "AppVariables.h"
#include "MainVariables.h"
#include "RadeonTunerBackend.h"
#include "RadeonTunerUiSession.h"

#include "AdlAppsFunc.h"
#include "AdlAppsUnlock.h"
#include "AdlAppsSync.h"
#include "AdlAppsSetDefaults.h"
#include "AdlAppsAdd.h"
#include "AdlAppsLoad.h"
#include "AdlAppsRemove.h"
#include "AdlAppsProperty.h"

#include "AdlGraphicsStatus.h"
#include "AdlOverdrive8.h"
#include "AdlCustomResolution.h"
#include "AdlEyefinity.h"
#include "AdlRegistry.h"
#include "AdlGetDevices.h"
#include "AdlGetIdentifier.h"
#include "AdlInitialize.h"
#include "AdlCheck.h"
#include "AdlFunctions.h"

#include "AdlxInfoLoad.h"
#include "AdlxValuesPrepare.h"
#include "AdlxValuesExportDisplay.h"
#include "AdlxValuesImportDisplay.h"
#include "AdlxValuesExportGraphics.h"
#include "AdlxValuesImportGraphics.h"
#include "AdlxValuesExportTuning.h"
#include "AdlxValuesImportTuning.h"

#include "AdlxValuesLoadDevice.h"
#include "AdlxValuesLoadDisplay.h"
#include "AdlxValuesLoadGraphics.h"
#include "AdlxValuesLoadMultimedia.h"
#include "AdlxValuesLoadTuning.h"

#include "AdlxEventsDisplay.h"
#include "AdlxEventsFans.h"
#include "AdlxEventsGraphics.h"
#include "AdlxEventsMultimedia.h"
#include "AdlxEventsTuning.h"
#include "AdlxEventsSelect.h"

#include "MultimediaSettingsGenerateAdl.h"
#include "MultimediaSettingsConvertUIAdl.h"
#include "MultimediaSettingsConvertUIProfile.h"
#include "MultimediaSettingsApply.h"
#include "MultimediaSettingsProfile.h"
#include "MultimediaSettingsMatch.h"

#include "GraphicsFsrOverrideDll.h"
#include "GraphicsSettingsEnvironment.h"
#include "GraphicsSettingsGenerateAdlApp.h"
#include "GraphicsSettingsGenerateAdlRegistry.h"
#include "GraphicsSettingsConvertUIAdl.h"
#include "GraphicsSettingsConvertUIProfile.h"
#include "GraphicsSettingsProfile.h"
#include "GraphicsSettingsSupport.h"
#include "GraphicsSettingsMatch.h"
#include "GraphicsSettingsApply.h"
#include "AdlxResetShaderCache.h"

#include "DisplaySettingsGenerateAdl.h"
#include "DisplaySettingsConvertUIAdl.h"
#include "DisplaySettingsConvertUIProfile.h"
#include "DisplaySettingsResolution.h"
#include "DisplaySettingsDetails.h"
#include "DisplaySettingsGamma.h"
#include "DisplaySettingsMatch.h"
#include "DisplaySettingsApply.h"
#include "DisplaySettingsProfile.h"
#include "AdlEyefinityEvents.h"
#include "AdlCustomResolutionEvents.h"

#include "TuningFanSettingsGenerateAdl.h"
#include "TuningFanSettingsConvertUIAdl.h"
#include "TuningFanSettingsConvertUIProfile.h"
#include "TuningFanSettingsMatch.h"
#include "TuningFanSettingsApply.h"
#include "TuningFanSettingsProfile.h"
#include "AdlTuningMetrics.h"

#include "SettingFunc.h"
#include "SettingAdmin.h"
#include "SettingSave.h"
#include "SettingLoad.h"

#include "AppPickerInterface.h"
#include "AppPickerEvent.h"

#include "AdlxLoopDisplay.h"
#include "AdlxLoopDisplayEyefinity.h"
#include "AdlxLoopDisplayGamma.h"
#include "AdlxLoopDisplayProfile.h"
#include "AdlxLoopTuning.h"
#include "AdlxLoopTuningProfile.h"
#include "AdlxLoopProfile.h"
#include "AdlxLoopMetrics.h"
#include "AdlxLoopAccess.h"

#include "AdjustCursor.h"
#include "MessageBox.h"
#include "NativeTheme.h"

#include "MainPage.h"
#if __has_include("MainPage.g.cpp")
#include "MainPage.g.cpp"
#endif

#ifdef RADEONTUNER_UI_VALIDATION
#if __has_include("..\tools\UiValidation\UiValidationScenario.h")
#include "..\tools\UiValidation\UiValidationScenario.h"
#else
#include "..\..\RadeonTuner\tools\UiValidation\UiValidationScenario.h"
#endif
#endif

namespace winrt::RadeonTuner::implementation
{
	namespace
	{
		//Notification timer generation, kept outside the page so a queued tick cannot touch a released page.
		uint64_t g_notificationGeneration = 0;

		bool TryParseSliderInput(std::wstring const& text, Slider const& slider, double& value)
		{
			wchar_t* end = nullptr;
			errno = 0;
			double parsed = std::wcstod(text.c_str(), &end);
			if (end == text.c_str() || errno == ERANGE || !std::isfinite(parsed)) { return false; }
			while (*end && std::iswspace(*end)) { ++end; }
			if (*end) { return false; }

			double minimum = slider.Minimum();
			double maximum = slider.Maximum();
			parsed = std::clamp(parsed, minimum, maximum);
			double step = slider.StepFrequency();
			if (step > 0 && std::isfinite(step))
			{
				parsed = minimum + std::round((parsed - minimum) / step) * step;
			}
			value = std::clamp(parsed, minimum, maximum);
			return std::isfinite(value);
		}
	}

	void MainPage::RestoreNumericInputValue(TextBox const& textBox)
	{
		try
		{
			auto slider = textBox.Tag().try_as<Slider>();
			if (!slider) { return; }
			//Restore the live one-way binding after either a committed edit or Escape.
			winrt::Windows::UI::Xaml::Data::Binding binding;
			binding.Source(slider);
			binding.Path(PropertyPath(L"Value"));
			binding.Mode(winrt::Windows::UI::Xaml::Data::BindingMode::OneWay);
			textBox.SetBinding(TextBox::TextProperty(), binding);
			winrt::Windows::UI::Xaml::Data::Binding enabledBinding;
			enabledBinding.Source(slider);
			enabledBinding.Path(PropertyPath(L"IsEnabled"));
			enabledBinding.Mode(winrt::Windows::UI::Xaml::Data::BindingMode::OneWay);
			textBox.SetBinding(Control::IsEnabledProperty(), enabledBinding);
			if (numericInputFocusedBox.get() == textBox) { numericInputOriginalText = textBox.Text(); }
		}
		catch (...) {}
	}

	void MainPage::ApplyNumericInputValue(TextBox const& textBox)
	{
		try
		{
			auto slider = textBox.Tag().try_as<Slider>();
			if (!slider) { return; }
			double value = 0;
			bool edited = numericInputFocusedBox.get() == textBox && numericInputOriginalText != textBox.Text();
			if (edited && textBox.IsEnabled() && !textBox.IsReadOnly() && slider.IsEnabled() &&
				TryParseSliderInput(std::wstring(textBox.Text().c_str()), slider, value) && value != slider.Value())
			{
				//The existing slider handler remains the only model/apply path.
				slider.Value(value);
			}
		}
		catch (...) {}
		RestoreNumericInputValue(textBox);
	}

	void MainPage::textbox_NumericInput_GotFocus(IInspectable const& sender, RoutedEventArgs const& e)
	{
		if (auto textBox = sender.try_as<TextBox>())
		{
			numericInputFocusedBox = winrt::make_weak(textBox);
			numericInputOriginalText = textBox.Text();
		}
	}

	void MainPage::textbox_NumericInput_KeyDown(IInspectable const& sender, KeyRoutedEventArgs const& e)
	{
		auto textBox = sender.try_as<TextBox>();
		if (!textBox) { return; }
		if (e.Key() == winrt::Windows::System::VirtualKey::Enter)
		{
			ApplyNumericInputValue(textBox);
			e.Handled(true);
		}
		else if (e.Key() == winrt::Windows::System::VirtualKey::Escape)
		{
			RestoreNumericInputValue(textBox);
			e.Handled(true);
		}
	}

	void MainPage::textbox_NumericInput_LostFocus(IInspectable const& sender, RoutedEventArgs const& e)
	{
		if (auto textBox = sender.try_as<TextBox>()) { ApplyNumericInputValue(textBox); }
	}

	namespace
	{
		bool SystemAnimationsEnabled()
		{
			try
			{
				static winrt::Windows::UI::ViewManagement::UISettings settings;
				return settings.AnimationsEnabled();
			}
			catch (...)
			{
				return true;
			}
		}

		winrt::Windows::UI::Xaml::CornerRadius UniformCornerRadius(double value)
		{
			return winrt::Windows::UI::Xaml::CornerRadius{ value, value, value, value };
		}

		winrt::Windows::UI::Xaml::Thickness UniformThickness(double value)
		{
			return winrt::Windows::UI::Xaml::Thickness{ value, value, value, value };
		}
	}

	void MainPage::UpdateDependentSettingsInterface(bool animate)
	{
		try
		{
			//Read the existing controls only: no settings, profile or driver values are changed here.
			ApplyDependentRow(stackpanel_FrameGenAlgorithm_Dependent(), toggleswitch_FrameGenEnabled().IsOn(), animate);
			ApplyDependentRow(stackpanel_RadeonChill_Fps(), toggleswitch_RadeonChill().IsOn(), animate);
			ApplyDependentRow(stackpanel_Frtc_Fps(), toggleswitch_Frtc().IsOn(), animate);
			ApplyDependentRow(stackpanel_RadeonBoost_MinResolution(), combobox_RadeonBoost().SelectedIndex() > 0, animate);
			ApplyDependentRow(stackpanel_RadeonImageSharpening1_Sharpening(), toggleswitch_RadeonImageSharpening1().IsOn(), animate);
			ApplyDependentRow(stackpanel_RadeonImageSharpening2_Dependent(), toggleswitch_RadeonImageSharpening2().IsOn(), animate);
			ApplyDependentRow(stackpanel_AntiAliasing_Dependent(), toggleswitch_AntiAliasingOverride().IsOn(), animate);
			ApplyDependentRow(stackpanel_TessellationLevel_Dependent(), combobox_Tessellation_Mode().SelectedIndex() == 2, animate);

			//Display page: fold the rows that depend on each parent toggle.
			ApplyDependentRow(stackpanel_Display_Gamma_Dependent(), toggleswitch_Display_GammaControl().IsOn(), animate);
			ApplyDependentRow(stackpanel_Display_ColorTemperature_Kelvin_Dependent(), toggleswitch_Display_ColorTemperature_Control().IsOn(), animate);
			ApplyDependentRow(stackpanel_Display_CVDC_Dependent(), toggleswitch_Display_CVDC_Control().IsOn(), animate);
			ApplyDependentRow(stackpanel_Display_VariBright_Level_Dependent(), toggleswitch_Display_VariBright().IsOn(), animate);
		}
		catch (...) {}
	}

	void MainPage::FinalizeDependentRow(FrameworkElement const& row, bool visible)
	{
		row.Clip(nullptr);
		row.Height(std::numeric_limits<double>::quiet_NaN());
		row.Opacity(1);
		row.Visibility(visible ? Visibility::Visible : Visibility::Collapsed);
		row.Tag(box_value(visible));
	}

	void MainPage::ApplyDependentRow(FrameworkElement const& row, bool visible, bool animate)
	{
		try
		{
			if (!row) { return; }

			bool previousVisible = false;
			auto tag = row.Tag();
			if (tag)
			{
				try { previousVisible = winrt::unbox_value<bool>(tag); } catch (...) {}
			}

			void* key = static_cast<void*>(winrt::get_abi(row));
			auto running = dependentAnimations.find(key);
			//Capture effective values before Stop removes the previous animation clock.
			bool reversing = running != dependentAnimations.end();
			double displayedHeight = row.Visibility() == Visibility::Visible ? row.ActualHeight() : 0;
			if (row.Visibility() == Visibility::Visible && std::isfinite(row.Height())) { displayedHeight = row.Height(); }
			double displayedOpacity = row.Visibility() == Visibility::Visible ? row.Opacity() : 0;
			if (tag && previousVisible == visible && (running == dependentAnimations.end() || animate))
			{
				//Leave an already-running reveal/collapse to finish; repeated state reads are no-ops.
				return;
			}
			if (running != dependentAnimations.end())
			{
				try { running->second.Stop(); } catch (...) {}
				dependentAnimations.erase(running);
			}

			row.Tag(box_value(visible));

			bool showAnimation = animate && SystemAnimationsEnabled();
			if (!showAnimation)
			{
				FinalizeDependentRow(row, visible);
				return;
			}

			using namespace winrt::Windows::UI::Xaml::Media::Animation;

			winrt::Windows::Foundation::TimeSpan revealTime = std::chrono::milliseconds(170);
			winrt::Windows::UI::Xaml::Duration revealDuration{ revealTime, winrt::Windows::UI::Xaml::DurationType::TimeSpan };
			double heightFrom = 0;
			double heightTo = 0;
			double opacityFrom = 0;
			if (visible)
			{
				//Measure natural content without arranging a full-height intermediate frame.
				row.Visibility(Visibility::Visible);
				row.Height(std::numeric_limits<double>::quiet_NaN());
				double availableWidth = row.ActualWidth();
				if (availableWidth <= 0) { auto parent = VisualTreeHelper::GetParent(row).try_as<FrameworkElement>(); if (parent) { availableWidth = parent.ActualWidth(); } }
				if (availableWidth <= 0) { availableWidth = ActualWidth(); }
				row.Measure(winrt::Windows::Foundation::Size{ (float)availableWidth, std::numeric_limits<float>::infinity() });
				heightTo = row.DesiredSize().Height;
				if (heightTo <= 0.5)
				{
					FinalizeDependentRow(row, true);
					return;
				}
				heightFrom = reversing ? displayedHeight : 0;
				opacityFrom = reversing ? displayedOpacity : 0;
				row.Height(heightFrom);
			}
			else
			{
				heightFrom = displayedHeight;
				if (heightFrom <= 0.5)
				{
					FinalizeDependentRow(row, false);
					return;
				}
				heightTo = 0;
				opacityFrom = displayedOpacity;
			}
			row.Height(heightFrom);
			row.Opacity(opacityFrom);

			Storyboard storyboard;

			CubicEase easing;
			easing.EasingMode(EasingMode::EaseOut);

			DoubleAnimation heightAnimation;
			heightAnimation.From(heightFrom);
			heightAnimation.To(heightTo);
			heightAnimation.Duration(revealDuration);
			heightAnimation.EnableDependentAnimation(true);
			heightAnimation.EasingFunction(easing);
			Storyboard::SetTarget(heightAnimation, row);
			Storyboard::SetTargetProperty(heightAnimation, L"Height");

			DoubleAnimation opacityAnimation;
			opacityAnimation.From(opacityFrom);
			opacityAnimation.To(visible ? 1.0 : 0.0);
			opacityAnimation.Duration(revealDuration);
			opacityAnimation.EasingFunction(easing);
			Storyboard::SetTarget(opacityAnimation, row);
			Storyboard::SetTargetProperty(opacityAnimation, L"Opacity");

			storyboard.Children().Append(heightAnimation);
			storyboard.Children().Append(opacityAnimation);

			//Keep children inside the animated height while neighbouring rows move.
			double clipHeight = (std::max)(heightFrom, heightTo);
			double clipWidth = row.ActualWidth();
			if (clipWidth <= 0) { clipWidth = row.DesiredSize().Width; }
			if (clipWidth <= 0) { clipWidth = ActualWidth(); }
			RectangleGeometry clip;
			clip.Rect(winrt::Windows::Foundation::Rect{ 0, 0, (float)clipWidth, (float)clipHeight });
			ScaleTransform clipScale;
			clipScale.ScaleY(heightFrom / clipHeight);
			clip.Transform(clipScale);
			row.Clip(clip);
			DoubleAnimation clipAnimation;
			clipAnimation.From(heightFrom / clipHeight);
			clipAnimation.To(heightTo / clipHeight);
			clipAnimation.Duration(revealDuration);
			clipAnimation.EasingFunction(easing);
			clipAnimation.EnableDependentAnimation(true);
			Storyboard::SetTarget(clipAnimation, clipScale);
			Storyboard::SetTargetProperty(clipAnimation, L"ScaleY");
			storyboard.Children().Append(clipAnimation);

			winrt::weak_ref<FrameworkElement> weakRow(row);
			auto weakOwner = get_weak();
			storyboard.Completed([weakOwner, weakRow, visible, key](IInspectable const& source, IInspectable const&)
			{
				try
				{
					auto owner = weakOwner.get();
					if (!owner) { return; }
					auto active = owner->dependentAnimations.find(key);
					auto completed = source.try_as<Storyboard>();
					if (active == owner->dependentAnimations.end() || active->second != completed) { return; }
					auto currentRow = weakRow.get();
					active->second.Stop();
					owner->dependentAnimations.erase(active);
					if (!currentRow) { return; }
					bool targetVisible = false;
					auto currentTag = currentRow.Tag();
					if (currentTag)
					{
						try { targetVisible = winrt::unbox_value<bool>(currentTag); } catch (...) {}
					}
					if (targetVisible == visible)
					{
						owner->FinalizeDependentRow(currentRow, visible);
					}
				}
				catch (...) {}
			});

			dependentAnimations[key] = storyboard;
			storyboard.Begin();
		}
		catch (...)
		{
			try { FinalizeDependentRow(row, visible); } catch (...) {}
		}
	}

	void MainPage::FinalizePageTransition()
	{
		++pageTransitionGeneration;
		auto storyboard = pageTransitionStoryboard;
		pageTransitionStoryboard = nullptr;
		if (storyboard)
		{
			try { storyboard.Stop(); } catch (...) {}
		}

		auto target = pageTransitionTarget;
		pageTransitionTarget = nullptr;
		if (target)
		{
			try
			{
				target.Opacity(1);
				if (auto transform = target.RenderTransform().try_as<TranslateTransform>())
				{
					transform.Y(0);
				}
			}
			catch (...) {}
		}
	}

	void MainPage::AnimatePageEntrance(FrameworkElement const& page)
	{
		try
		{
			FinalizePageTransition();
			if (!page) { return; }

			auto transform = page.RenderTransform().try_as<TranslateTransform>();
			if (!transform)
			{
				transform = TranslateTransform();
				page.RenderTransform(transform);
			}

			page.Opacity(1);
			transform.Y(0);
			if (!SystemAnimationsEnabled()) { return; }

			constexpr int pageEntranceMilliseconds = 120;
			constexpr double pageEntranceOffset = 6;

			page.Opacity(0);
			transform.Y(pageEntranceOffset);

			using namespace winrt::Windows::UI::Xaml::Media::Animation;

			winrt::Windows::Foundation::TimeSpan transitionTime = std::chrono::milliseconds(pageEntranceMilliseconds);
			winrt::Windows::UI::Xaml::Duration transitionDuration{ transitionTime, winrt::Windows::UI::Xaml::DurationType::TimeSpan };

			CubicEase easing;
			easing.EasingMode(EasingMode::EaseOut);

			Storyboard storyboard;

			DoubleAnimation opacityAnimation;
			opacityAnimation.From(0.0);
			opacityAnimation.To(1.0);
			opacityAnimation.Duration(transitionDuration);
			opacityAnimation.EasingFunction(easing);
			Storyboard::SetTarget(opacityAnimation, page);
			Storyboard::SetTargetProperty(opacityAnimation, L"Opacity");

			DoubleAnimation offsetAnimation;
			offsetAnimation.From(pageEntranceOffset);
			offsetAnimation.To(0.0);
			offsetAnimation.Duration(transitionDuration);
			offsetAnimation.EasingFunction(easing);
			Storyboard::SetTarget(offsetAnimation, transform);
			Storyboard::SetTargetProperty(offsetAnimation, L"Y");

			storyboard.Children().Append(opacityAnimation);
			storyboard.Children().Append(offsetAnimation);

			uint64_t generation = ++pageTransitionGeneration;
			winrt::weak_ref<FrameworkElement> weakPage(page);
			winrt::weak_ref<TranslateTransform> weakTransform(transform);
			auto weakOwner = get_weak();
			storyboard.Completed([weakOwner, generation, weakPage, weakTransform](IInspectable const&, IInspectable const&)
			{
				auto owner = weakOwner.get();
				if (!owner) { return; }
				if (generation != owner->pageTransitionGeneration) { return; }
				try
				{
					if (auto currentPage = weakPage.get())
					{
						currentPage.Opacity(1);
					}
					if (auto currentTransform = weakTransform.get())
					{
						currentTransform.Y(0);
					}
				}
				catch (...) {}
				owner->pageTransitionStoryboard = nullptr;
				owner->pageTransitionTarget = nullptr;
			});

			pageTransitionStoryboard = storyboard;
			pageTransitionTarget = page;
			storyboard.Begin();
		}
		catch (...)
		{
			try
			{
				page.Opacity(1);
				if (auto transform = page.RenderTransform().try_as<TranslateTransform>())
				{
					transform.Y(0);
				}
			}
			catch (...) {}
		}
	}

	FrameworkElement MainPage::FindComboBoxPopupBorder(ComboBox const& comboBox)
	{
		FrameworkElement result{ nullptr };
		try
		{
			std::function<void(DependencyObject const&)> visit = [&](DependencyObject const& node)
			{
				if (result) { return; }

				if (auto element = node.try_as<FrameworkElement>())
				{
					if (std::wstring(element.Name().c_str()) == L"PopupBorder")
					{
						result = element;
						return;
					}
				}

				if (auto popup = node.try_as<Popup>())
				{
					auto popupChild = popup.Child();
					if (popupChild)
					{
						visit(popupChild);
						if (result) { return; }
					}
				}

				int childCount = VisualTreeHelper::GetChildrenCount(node);
				for (int index = 0; index < childCount; index++)
				{
					visit(VisualTreeHelper::GetChild(node, index));
					if (result) { return; }
				}
			};

			visit(comboBox);
		}
		catch (...) {}
		return result;
	}

	void MainPage::AnimateComboBoxPopup(ComboBox const& comboBox, bool open)
	{
		try
		{
			auto border = FindComboBoxPopupBorder(comboBox);
			if (!border) { return; }

			void* key = static_cast<void*>(winrt::get_abi(comboBox));
			auto running = comboPopupAnimations.find(key);
			if (running != comboPopupAnimations.end())
			{
				try { running->second.Stop(); } catch (...) {}
				comboPopupAnimations.erase(running);
			}

			auto transform = border.RenderTransform().try_as<TranslateTransform>();
			if (!transform)
			{
				transform = TranslateTransform();
				border.RenderTransform(transform);
			}

			if (!SystemAnimationsEnabled())
			{
				border.Opacity(1);
				transform.Y(0);
				return;
			}

			constexpr int popupOpenMilliseconds = 90;
			constexpr int popupCloseMilliseconds = 70;

			border.Opacity(open ? 0 : 1);
			transform.Y(open ? 4 : 0);

			using namespace winrt::Windows::UI::Xaml::Media::Animation;

			winrt::Windows::Foundation::TimeSpan transitionTime = std::chrono::milliseconds(open ? popupOpenMilliseconds : popupCloseMilliseconds);
			winrt::Windows::UI::Xaml::Duration transitionDuration{ transitionTime, winrt::Windows::UI::Xaml::DurationType::TimeSpan };

			CubicEase easing;
			easing.EasingMode(EasingMode::EaseOut);

			Storyboard storyboard;

			DoubleAnimation opacityAnimation;
			opacityAnimation.From(open ? 0.0 : 1.0);
			opacityAnimation.To(open ? 1.0 : 0.0);
			opacityAnimation.Duration(transitionDuration);
			opacityAnimation.EasingFunction(easing);
			Storyboard::SetTarget(opacityAnimation, border);
			Storyboard::SetTargetProperty(opacityAnimation, L"Opacity");
			storyboard.Children().Append(opacityAnimation);

			if (open)
			{
				DoubleAnimation offsetAnimation;
				offsetAnimation.From(4.0);
				offsetAnimation.To(0.0);
				offsetAnimation.Duration(transitionDuration);
				offsetAnimation.EasingFunction(easing);
				Storyboard::SetTarget(offsetAnimation, transform);
				Storyboard::SetTargetProperty(offsetAnimation, L"Y");
				storyboard.Children().Append(offsetAnimation);
			}

			void* storyboardKey = static_cast<void*>(winrt::get_abi(storyboard));
			winrt::weak_ref<FrameworkElement> weakBorder(border);
			winrt::weak_ref<TranslateTransform> weakTransform(transform);
			auto weakOwner = get_weak();
			storyboard.Completed([weakOwner, key, storyboardKey, weakBorder, weakTransform](IInspectable const&, IInspectable const&)
			{
				auto owner = weakOwner.get();
				if (!owner) { return; }
				auto current = owner->comboPopupAnimations.find(key);
				if (current == owner->comboPopupAnimations.end()) { return; }
				if (static_cast<void*>(winrt::get_abi(current->second)) != storyboardKey) { return; }
				owner->comboPopupAnimations.erase(current);
				try
				{
					if (auto currentBorder = weakBorder.get())
					{
						currentBorder.Opacity(1);
					}
					if (auto currentTransform = weakTransform.get())
					{
						currentTransform.Y(0);
					}
				}
				catch (...) {}
			});

			comboPopupAnimations[key] = storyboard;
			storyboard.Begin();
		}
		catch (...) {}
	}

	void MainPage::viewbox_Fan_Plot_SizeChanged(IInspectable const& sender, SizeChangedEventArgs const& e)
	{
		UpdateFanChartAxisLayout();
	}

	void MainPage::UpdateFanChartAxisLayout()
	{
		try
		{
			auto plot = viewbox_Fan_Plot();
			if (!plot) { return; }

			double plotWidth = plot.ActualWidth();
			double plotHeight = plot.ActualHeight();
			if (plotWidth <= 0 || plotHeight <= 0) { return; }

			//The Viewbox content is a 512x112 root with a 500x100 plot inset by 6 DIP.
			constexpr double rootWidth = 512.0;
			constexpr double rootHeight = 112.0;
			constexpr double plotOrigin = 6.0;
			constexpr double chartWidth = 500.0;
			constexpr double chartHeight = 100.0;
			constexpr double xLabelWidth = 30.0;
			constexpr double yLabelHeight = 14.0;

			auto setXLabel = [&](TextBlock const& label, double logicalX)
			{
				if (!label) { return; }
				Canvas::SetLeft(label, logicalX / rootWidth * plotWidth - xLabelWidth / 2.0);
				Canvas::SetTop(label, 0.0);
			};
			auto setYLabel = [&](TextBlock const& label, double logicalY)
			{
				if (!label) { return; }
				Canvas::SetLeft(label, 0.0);
				Canvas::SetTop(label, logicalY / rootHeight * plotHeight - yLabelHeight / 2.0);
			};

			setYLabel(textblock_Fan_Axis_Y_100(), plotOrigin);
			setYLabel(textblock_Fan_Axis_Y_80(), plotOrigin + chartHeight * 0.20);
			setYLabel(textblock_Fan_Axis_Y_60(), plotOrigin + chartHeight * 0.40);
			setYLabel(textblock_Fan_Axis_Y_40(), plotOrigin + chartHeight * 0.60);
			setYLabel(textblock_Fan_Axis_Y_20(), plotOrigin + chartHeight * 0.80);
			setYLabel(textblock_Fan_Axis_Y_0(), plotOrigin + chartHeight);

			setXLabel(textblock_Fan_Axis_X_0(), plotOrigin);
			setXLabel(textblock_Fan_Axis_X_20(), plotOrigin + chartWidth * 0.20);
			setXLabel(textblock_Fan_Axis_X_40(), plotOrigin + chartWidth * 0.40);
			setXLabel(textblock_Fan_Axis_X_60(), plotOrigin + chartWidth * 0.60);
			setXLabel(textblock_Fan_Axis_X_80(), plotOrigin + chartWidth * 0.80);
			setXLabel(textblock_Fan_Axis_X_100(), plotOrigin + chartWidth);
		}
		catch (...) {}
	}

	void MainPage::InitializeValueHints()
	{
		if (valueHintsInitialized) { return; }
		valueHintsInitialized = true;

		try
		{
			auto hintStyle = Resources().Lookup(box_value(L"FlatValueHint")).try_as<winrt::Windows::UI::Xaml::Style>();
			auto valueStyle = Resources().Lookup(box_value(L"FlatValueLabel")).try_as<winrt::Windows::UI::Xaml::Style>();
			if (!hintStyle) { return; }

			std::function<void(DependencyObject const&)> visit = [&](DependencyObject const& node)
			{
				auto textBlock = node.try_as<TextBlock>();
				if (textBlock)
				{
					auto style = textBlock.Style();
					if (style && (winrt::get_abi(style) == winrt::get_abi(hintStyle) ||
						(valueStyle && winrt::get_abi(style) == winrt::get_abi(valueStyle))))
					{
						winrt::weak_ref<TextBlock> weakText(textBlock);
						auto updateNow = [weakText]()
						{
							try
							{
								if (auto current = weakText.get())
								{
									current.Visibility(current.Text().empty() ? Visibility::Collapsed : Visibility::Visible);
								}
							}
							catch (...) {}
						};
						auto updateVisibility = [updateNow](DependencyObject const&, DependencyProperty const&)
						{
							updateNow();
						};
						textBlock.RegisterPropertyChangedCallback(textBlock.TextProperty(), updateVisibility);
						updateNow();
					}
				}

				int childCount = VisualTreeHelper::GetChildrenCount(node);
				for (int index = 0; index < childCount; index++)
				{
					visit(VisualTreeHelper::GetChild(node, index));
				}
			};

			visit(*this);
		}
		catch (...) {}
	}

	static FrameworkElement FindTogglePart(ToggleSwitch const& toggle, wchar_t const* name)
	{
		std::function<FrameworkElement(DependencyObject const&)> find = [&](DependencyObject const& node) -> FrameworkElement
		{
			if (auto element = node.try_as<FrameworkElement>(); element && element.Name() == name) { return element; }
			for (int i = 0; i < VisualTreeHelper::GetChildrenCount(node); ++i) { if (auto found = find(VisualTreeHelper::GetChild(node, i))) { return found; } }
			return nullptr;
		};
		return find(toggle);
	}

	static TranslateTransform FindToggleTransform(ToggleSwitch const& toggle, wchar_t const* name)
	{
		auto element = FindTogglePart(toggle, name);
		return element ? element.RenderTransform().try_as<TranslateTransform>() : nullptr;
	}

	void MainPage::AnimateToggleKnob(ToggleSwitch const& toggle, TranslateTransform const& knob)
	{
		using namespace winrt::Windows::UI::Xaml::Media::Animation;
		void* key = static_cast<void*>(winrt::get_abi(toggle));
		double from = knob.X();
		double target = toggle.IsOn() ? 20.0 : 0.0;
		if (auto running = toggleKnobAnimations.find(key); running != toggleKnobAnimations.end())
		{
			running->second.Stop();
			toggleKnobAnimations.erase(running);
		}
		knob.X(target);
		if (!SystemAnimationsEnabled() || std::abs(from - target) < 0.01) { return; }
		Storyboard storyboard;
		DoubleAnimation movement;
		movement.From(from);
		movement.To(target);
		movement.Duration(winrt::Windows::UI::Xaml::Duration{ std::chrono::milliseconds(100), winrt::Windows::UI::Xaml::DurationType::TimeSpan });
		CubicEase easing;
		easing.EasingMode(EasingMode::EaseOut);
		movement.EasingFunction(easing);
		Storyboard::SetTarget(movement, knob);
		Storyboard::SetTargetProperty(movement, L"X");
		storyboard.Children().Append(movement);
		auto weakOwner = get_weak();
		winrt::weak_ref<TranslateTransform> weakKnob(knob);
		storyboard.Completed([weakOwner, weakKnob, key, target](IInspectable const& sender, IInspectable const&)
		{
			if (auto owner = weakOwner.get())
			{
				auto current = owner->toggleKnobAnimations.find(key);
				if (current == owner->toggleKnobAnimations.end() || current->second != sender.as<Storyboard>()) { return; }
				current->second.Stop();
				owner->toggleKnobAnimations.erase(current);
				if (auto knob = weakKnob.get()) { knob.X(target); }
			}
		});
		toggleKnobAnimations[key] = storyboard;
		storyboard.Begin();
	}

	void MainPage::toggle_Loaded(IInspectable const& sender, RoutedEventArgs const&)
	{
		auto toggle = sender.as<ToggleSwitch>();
		toggle.ApplyTemplate();
		auto knob = FindToggleTransform(toggle, L"SwitchVisualKnob");
		auto thumbElement = FindTogglePart(toggle, L"SwitchThumb");
		if (!knob || !thumbElement) { return; }
		void* key = static_cast<void*>(winrt::get_abi(toggle));
		auto weakOwner = get_weak();
		winrt::weak_ref<ToggleSwitch> weakToggle(toggle);
		if (!toggleKnobInitialized.count(key))
		{
			toggleKnobInitialized[key] = true;
			knob.X(toggle.IsOn() ? 20.0 : 0.0);
			toggle.Toggled([weakOwner](auto const& sender, auto const&)
			{
				if (auto owner = weakOwner.get())
				{
					auto toggle = sender.template as<ToggleSwitch>();
					if (auto knob = FindToggleTransform(toggle, L"SwitchVisualKnob")) { owner->AnimateToggleKnob(toggle, knob); }
				}
			});
			//The native switch consumes the second tap as DoubleTapped without changing IsOn.
			toggle.DoubleTapped([](auto const& sender, auto const& args)
			{
				auto toggle = sender.template as<ToggleSwitch>();
				if (toggle.IsEnabled()) { toggle.IsOn(!toggle.IsOn()); args.Handled(true); }
			});
		}
		auto thumb = thumbElement.as<winrt::Windows::UI::Xaml::Controls::Primitives::Thumb>();
		if (thumb.Tag()) { return; }
		thumb.Tag(box_value(true));
		thumb.DragDelta([weakOwner, weakToggle](auto const&, auto const& args)
		{
			if (std::abs(args.HorizontalChange()) < 0.01) { return; }
			auto owner = weakOwner.get(); auto toggle = weakToggle.get();
			if (!owner || !toggle) { return; }
			auto knob = FindToggleTransform(toggle, L"SwitchVisualKnob");
			auto native = FindToggleTransform(toggle, L"SwitchKnob");
			if (!knob || !native) { return; }
			void* key = static_cast<void*>(winrt::get_abi(toggle));
			if (auto running = owner->toggleKnobAnimations.find(key); running != owner->toggleKnobAnimations.end()) { running->second.Stop(); owner->toggleKnobAnimations.erase(running); }
			knob.X((std::max)(0.0, (std::min)(20.0, native.X())));
		});
		thumb.DragCompleted([weakOwner, weakToggle](auto const&, auto const&)
		{
			auto owner = weakOwner.get(); auto toggle = weakToggle.get();
			if (!owner || !toggle) { return; }
			auto knob = FindToggleTransform(toggle, L"SwitchVisualKnob");
			void* key = static_cast<void*>(winrt::get_abi(toggle));
			if (knob && !owner->toggleKnobAnimations.count(key)) { owner->AnimateToggleKnob(toggle, knob); }
		});
	}

	void MainPage::InitializeComboBoxVisuals()
	{
		if (comboVisualsInitialized) { return; }
		comboVisualsInitialized = true;

		try
		{
			std::function<void(DependencyObject const&)> visit = [&](DependencyObject const& node)
			{
				auto comboBox = node.try_as<ComboBox>();
				if (comboBox)
				{
					comboBox.ApplyTemplate();
					PatchComboBoxVisuals(comboBox);
					comboBox.IsEnabledChanged([](IInspectable const& sender, DependencyPropertyChangedEventArgs const&)
					{
						auto field = sender.as<ComboBox>();
						field.PlaceholderText(field.IsEnabled() ? L"未设置" : L"不可用");
					});
					auto weakOwner = get_weak();
					comboBox.Loaded([weakOwner](IInspectable const& sender, RoutedEventArgs const&)
					{
						if (auto owner = weakOwner.get())
						{
							auto comboBox = sender.as<ComboBox>();
							comboBox.ApplyTemplate();
							owner->PatchComboBoxVisuals(comboBox);
						}
					});
					comboBox.DropDownOpened([weakOwner](IInspectable const& sender, IInspectable const&)
					{
						if (auto owner = weakOwner.get())
						{
							auto comboBox = sender.as<ComboBox>();
							owner->PatchComboBoxVisuals(comboBox);
							owner->AnimateComboBoxPopup(comboBox, true);
						}
					});
					comboBox.DropDownClosed([weakOwner](IInspectable const& sender, IInspectable const&)
					{
						if (auto owner = weakOwner.get())
						{
							owner->AnimateComboBoxPopup(sender.as<ComboBox>(), false);
						}
					});
				}

				int childCount = VisualTreeHelper::GetChildrenCount(node);
				for (int index = 0; index < childCount; index++)
				{
					visit(VisualTreeHelper::GetChild(node, index));
				}
			};

			visit(*this);
		}
		catch (...) {}
	}

	void MainPage::PatchComboBoxVisuals(ComboBox const& comboBox)
	{
		try
		{
			if (!comboBox) { return; }
			comboBox.PlaceholderText(comboBox.IsEnabled() ? L"未设置" : L"不可用");

			auto resources = Application::Current().Resources();
			auto fill = comboBox.Background().try_as<SolidColorBrush>();
			auto borderBrush = comboBox.BorderBrush().try_as<SolidColorBrush>();
			if (!fill) { fill = resources.Lookup(box_value(L"ThemeControlBrush")).try_as<SolidColorBrush>(); }
			if (!borderBrush) { borderBrush = resources.Lookup(box_value(L"ThemeBorderBrush")).try_as<SolidColorBrush>(); }
			auto radius = UniformCornerRadius(6);
			auto borderThickness = UniformThickness(1);

			auto patchBorder = [&](Border const& border)
			{
				if (!border) { return; }
				border.CornerRadius(radius);
				border.BorderThickness(borderThickness);
				if (fill) { border.Background(fill); }
				if (borderBrush) { border.BorderBrush(borderBrush); }
			};

			bool closedPatched = false;
			std::function<void(DependencyObject const&)> visit = [&](DependencyObject const& node)
			{
				if (auto element = node.try_as<FrameworkElement>())
				{
					std::wstring name(element.Name().c_str());
					if (name == L"BorderElement")
					{
						if (auto border = element.try_as<Border>())
						{
							patchBorder(border);
							closedPatched = true;
						}
					}
					else if (name == L"PopupBorder")
					{
						patchBorder(element.try_as<Border>());
					}
				}

				if (auto popup = node.try_as<Popup>())
				{
					try
					{
						auto popupTransitions = popup.ChildTransitions();
						if (popupTransitions && popupTransitions.Size() > 0)
						{
							popupTransitions.Clear();
						}
					}
					catch (...) {}

					auto popupChild = popup.Child();
					if (popupChild)
					{
						try
						{
							if (auto popupElement = popupChild.try_as<UIElement>())
							{
								auto childTransitions = popupElement.Transitions();
								if (childTransitions && childTransitions.Size() > 0)
								{
									childTransitions.Clear();
								}
							}
						}
						catch (...) {}

						bool popupPatched = false;
						std::function<void(DependencyObject const&)> visitPopup = [&](DependencyObject const& popupNode)
						{
							if (auto element = popupNode.try_as<FrameworkElement>())
							{
								if (std::wstring(element.Name().c_str()) == L"PopupBorder")
								{
									if (auto border = element.try_as<Border>())
									{
										patchBorder(border);
										popupPatched = true;
									}
								}
							}
							int popupChildCount = VisualTreeHelper::GetChildrenCount(popupNode);
							for (int index = 0; index < popupChildCount; index++)
							{
								visitPopup(VisualTreeHelper::GetChild(popupNode, index));
							}
						};
						visitPopup(popupChild);
						if (!popupPatched)
						{
							if (auto border = popupChild.try_as<Border>())
							{
								patchBorder(border);
							}
						}
					}
				}

				int childCount = VisualTreeHelper::GetChildrenCount(node);
				for (int index = 0; index < childCount; index++)
				{
					visit(VisualTreeHelper::GetChild(node, index));
				}
			};

			visit(comboBox);
			comboBox.CornerRadius(radius);

			if (!closedPatched)
			{
				int childCount = VisualTreeHelper::GetChildrenCount(comboBox);
				for (int index = 0; index < childCount; index++)
				{
					if (auto border = VisualTreeHelper::GetChild(comboBox, index).try_as<Border>())
					{
						patchBorder(border);
						break;
					}
				}
			}
		}
		catch (...) {}
	}

	winrt::IAsyncAction MainPage::page_Loaded(IInspectable const& sender, RoutedEventArgs const& e)
	{
		//Keep this page alive while its initial load or restore is still running.
		auto strongThis = get_strong();
		AppVariables::App.BeginUiWork();
		struct UiWorkGuard
		{
			~UiWorkGuard() { AppVariables::App.EndUiWork(); }
		} uiWorkGuard;

		try
		{
			InitializeValueHints();
			InitializeComboBoxVisuals();

#ifdef RADEONTUNER_UI_VALIDATION
			//UI validation build: only real XAML/theme/lists/defaults, never ADL, registry, autostart or hardware loops
			disable_saving = true;
			disable_saving_settings = true;
			disable_saving_customresolution = true;
			AppVariables::App.EnsureBackendStarted();
			RadeonTunerUiValidation::InitializePageControls(this);

			//Visual check mode: show the real native Win32 tray menu anchored inside this window only
			if (RadeonTunerUiValidation::ShowTrayMenuOnly)
			{
				co_await AsyncTaskDelay(1200, AppVariables::App.GetDispatcher());

				//Auto mode closes the menu by itself for an automated check, stay mode leaves it for visual inspection
				AppVariables::App.ShowTrayMenuForValidation(RadeonTunerUiValidation::AutoMode ? 1500 : 0);

				//Record that the real native menu path completed without desktop automation
				nlohmann::json trayMenuReport = nlohmann::json::object();
				trayMenuReport["shown_utc"] = RadeonTunerUiValidation::UtcNowString();
				trayMenuReport["native_win32_menu"] = true;
				trayMenuReport["auto_closed"] = RadeonTunerUiValidation::AutoMode;
				trayMenuReport["owner_window"] = (uint64_t)(uintptr_t)AppVariables::App.GetWindowHandle();
				trayMenuReport["theme_style"] = AppVariables::Settings.Load<int>("ThemeStyle").value_or(0);
				trayMenuReport["note"] = "Real CreatePopupMenu/TrackPopupMenuEx menu anchored inside the test window; no owner-draw, external input or desktop automation.";
				RadeonTunerUiValidation::WriteUtf8File(RadeonTunerUiValidation::TestDataPath + L"\\ui-validation-tray-menu.json", trayMenuReport.dump(2));

				if (RadeonTunerUiValidation::AutoMode)
				{
					AppVariables::App.UiValidationExit(0);
				}
				co_return;
			}

			//Lightweight visual preview: real XAML/theme/enums/defaults and navigation, no hardware scenario
			if (RadeonTunerUiValidation::PreviewMode)
			{
				RadeonTunerUiValidation::SeedPreviewMetrics(this);
				if (RadeonTunerUiValidation::LifecycleCheckMode)
				{
					if (!RadeonTunerUiValidation::LifecycleCheckStarted)
					{
						RadeonTunerUiValidation::LifecycleCheckStarted = true;
						RadeonTunerUiValidation::RunLifecycleCheck(get_weak());
					}
					RadeonTunerUiSession::Restore(*this);
					co_return;
				}
				if (RadeonTunerUiValidation::AnimationCheckMode)
				{
					co_await AsyncTaskDelay(250, AppVariables::App.GetDispatcher());
					co_await RadeonTunerUiValidation::RunAnimationRegression(this);
				}
				RadeonTunerUiSession::Restore(*this);
				co_return;
			}

			co_await RadeonTunerUiValidation::RunSynchronous(this);
			co_return;
#else
			//A recreated page only restores UI from the App-owned backend state. It must
			//never initialize ADL, write driver defaults, reload profiles, start loops or
			//run the update check a second time.
			if (BackendInitialized)
			{
				disable_saving = true;
				disable_saving_settings = true;

				ThemeAdjustStyle();
				co_await SettingLoad();
				disable_saving_settings = true;

				//Repopulate the per-page enum lists; this is pure UI and not a driver init.
				AdlxValuesPrepare();

				//Current model and selected profiles survive in the global backend state.
				TuningFanSettings_Convert_ToUI_Profile(tuningFanSettingsProfile, AdlSettingGet::Current);
				DisplaySettings_Convert_ToUI_Profile(displaySettingsProfile, AdlSettingGet::Current);
				GraphicsSettings_Convert_ToUI_Profile(graphicsSettingsProfile, AdlSettingGet::Current);
				MultimediaSettings_Convert_ToUI_Profile(multimediaSettingsProfile, AdlSettingGet::Current);

				//Restore selection labels from the backend state even if no snapshot exists yet.
				try
				{
					auto gpuInfo = AdlGetGpuByAdapterIndex(adl_Gpu_AdapterIndex);
					if (gpuInfo.has_value()) { textblock_GpuSelect().Text(char_to_wstring(gpuInfo.value().strAdapterName)); }
					auto displayInfo = AdlGetDisplayByDisplayIndex(adl_Display_AdapterIndex, adl_Display_DisplayIndex);
					if (displayInfo.has_value()) { textblock_DisplaySelect().Text(char_to_wstring(displayInfo.value().strDisplayName)); }
				}
				catch (...) {}

				ShowExperimentalSettings(true);
				SelectDefaultIndexes();
				AdlxValuesLoadEyefinityDisplays();
				co_await DisplayList_Combined(false);
				AdlxInfoLoad();
				RadeonTunerUiSession::Restore(*this);
				UpdateFanControlModeInterface(tuningFanSettingsProfile, false);

				//Redraw values derived from the restored draft controls without writing settings.
				ValidateFanSettings();
				UpdateFanGraphProfile();
				UpdateFanInputFields();
				UpdateDependentSettingsInterface(false);

				disable_saving = false;
				disable_saving_settings = false;

				AVDebugWriteLine("Restored application UI from backend state.");
				co_return;
			}

			//Fix check if DCH / UWP or downgraded driver is installed and warn not supported
			//Fix check if other overclock applications are running and warn user

			//Adjust theme style
			ThemeAdjustStyle();

			//Check driver software type
			if (AdlCheckDriverOnlySoftware())
			{
				grid_Main().IsHitTestVisible(false);
				std::vector<std::wstring> messageAnswers{ L"仍然继续", L"退出应用程序" };
				int messageResult = co_await ShowMessageBox(L"驱动程序软件类型不兼容", L"强烈建议使用“仅驱动程序”软件类型安装驱动，以避免 RadeonTuner 可能出现的问题。\n\n请使用“仅驱动程序”软件类型和“恢复出厂设置”选项重新安装驱动；可在 Radeon 驱动安装程序的“附加选项”中找到这些选项。", messageAnswers);
				if (!AppVariables::App.IsCurrentUiPage(*this)) { co_return; }
				if (messageResult == 0)
				{
					//Enable interface
					grid_Main().IsHitTestVisible(true);
				}
				else
				{
					//Exit application
					AppVariables::App.Exit(true);
					co_return;
				}
			}

			//Initialize adl api
			std::wstring initResult_Adl = AdlInitialize();
			if (!initResult_Adl.empty())
			{
				grid_Main().IsHitTestVisible(false);
				std::vector<std::wstring> messageAnswers{ L"运行 AMD Cleanup Utility", L"退出应用程序" };
				int messageResult = co_await ShowMessageBox(L"ADL 初始化失败", L"请重新安装或更新 AMD 驱动程序；如果此消息持续出现，请尝试使用 AMD Cleanup Utility 并重新安装驱动程序。（" + initResult_Adl + L"）", messageAnswers);
				if (!AppVariables::App.IsCurrentUiPage(*this)) { co_return; }
				if (messageResult == 0)
				{
					//Launch driver cleanup utility
					LaunchDriverCleanup();

					//Exit application
					AppVariables::App.Exit(true);
					co_return;
				}
				else
				{
					//Exit application
					AppVariables::App.Exit(true);
					co_return;
				}
			}

			//Initialization is one-shot for this process; a recreated page must not
			//initialize ADL or write driver defaults again.
			BackendInitialized = true;

			//Prepare adlx values
			AdlxValuesPrepare();

			//Load application profiles
			TuningFanSettings_Profiles_LoadFromFile();
			GraphicsSettings_Profiles_LoadFromFile();
			DisplaySettings_Profiles_LoadFromFile();
			MultimediaSettings_Profiles_LoadFromFile();

			//Get all available GPU's
			std::vector<AdapterInfo> listGpus = AdlGetGpuAll(false);
			if (listGpus.size() == 0)
			{
				grid_Main().IsHitTestVisible(false);
				std::vector<std::wstring> messageAnswers{ L"运行 AMD Cleanup Utility", L"退出应用程序" };
				int messageResult = co_await ShowMessageBox(L"未找到任何 GPU", L"请重新安装或更新 AMD 驱动程序；如果此消息持续出现，请尝试使用 AMD Cleanup Utility 并重新安装驱动程序。", messageAnswers);
				if (!AppVariables::App.IsCurrentUiPage(*this)) { co_return; }
				if (messageResult == 0)
				{
					//Launch driver cleanup utility
					LaunchDriverCleanup();

					//Exit application
					AppVariables::App.Exit(true);
					co_return;
				}
				else
				{
					//Exit application
					AppVariables::App.Exit(true);
					co_return;
				}
			}
			else
			{
				//Select defaults
				co_await AdlxValuesLoadSelectGpu(listGpus[0]);
			}

			//Get all displays
			//Note: No longer blocking launch when no displays are found, allowing workstations without a display.
			std::vector<ADLDisplayInfo> displayList = AdlGetDisplayAll();
			if (displayList.size() > 0)
			{
				//Select defaults
				co_await AdlxValuesLoadSelectDisplay(displayList[0]);
			}

			//The host performed the one-time application sync; a worker must not repeat it.
			if (!AppVariables::UiWorkerMode)
			{
				AdlAppSyncAll();
			}

			//The host performed the one-time registry defaults; a worker must not repeat them.
			if (!AppVariables::UiWorkerMode)
			{
				AdlSetDefaultSettings();
			}

			//Check admin setttings
			SettingAdmin();

			//Load settings
			co_await SettingLoad();

			//Show or hide experimental settings
			ShowExperimentalSettings(true);

			//Select default indexes
			SelectDefaultIndexes();

			//Restore the host's full model/selection and UI draft before the update check.
			if (AppVariables::UiWorkerMode)
			{
				disable_saving = true;
				disable_saving_settings = true;
				RadeonTunerBackend::ApplyStateJson(AppVariables::UiWorkerStateJson);
				TuningFanSettings_Convert_ToUI_Profile(tuningFanSettingsProfile, AdlSettingGet::Current);
				DisplaySettings_Convert_ToUI_Profile(displaySettingsProfile, AdlSettingGet::Current);
				GraphicsSettings_Convert_ToUI_Profile(graphicsSettingsProfile, AdlSettingGet::Current);
				MultimediaSettings_Convert_ToUI_Profile(multimediaSettingsProfile, AdlSettingGet::Current);
				RadeonTunerUiSession::Restore(*this);
				UpdateFanControlModeInterface(tuningFanSettingsProfile, false);
				disable_saving = false;
				disable_saving_settings = false;
			}

			//Backend stays alive when this page is released. Start it once.
			AppVariables::App.EnsureBackendStarted();

			//Check for application update, only on the first initialization
			std::optional<bool> setStartCheckUpdate = AppVariables::Settings.Load<bool>("StartCheckUpdate");
			if ((!AppVariables::UiWorkerMode || AppVariables::UiWorkerFirst) && setStartCheckUpdate.has_value() && setStartCheckUpdate.value())
			{
				UpdateCheckResult updateCheckResult = UpdateCheck(AppVariables::hInstance, "dumbie", "RadeonTuner");
				if (updateCheckResult.UpdateFound)
				{
					//Set update version
					vUpdateVersion = updateCheckResult.UpdateVersion;

					//Show update button
					button_Update_Launch().Visibility(Visibility::Visible);

					//Show notification
					AVDebugWriteLine(L"New application update found");
					ShowNotification(L"发现新的应用程序更新");
				}
			}

			//Update first launch setting
			AppVariables::Settings.Set("FirstLaunch", false);

			//Starting hidden is the same tray state as closing later: keep backend, release UI.
			if (!AppVariables::App.GetIsWindowVisible())
			{
				AppVariables::App.HideToTray();
			}
#endif
		}
		catch (...) {}
	}
	void MainPage::SelectDefaultIndexes()
	{
		try
		{
			//Select Eyefinity orientation
			combobox_EyefinityMonitorOrientation().SelectedIndex(0);

			//Select previous menu index
			int mainSelectIndex = 0;
			std::optional<int> prevMenuIndex = AppVariables::Settings.Load<int>("MenuIndex");
			if (prevMenuIndex.has_value())
			{
				mainSelectIndex = prevMenuIndex.value();
				//Check if selected index is exit button
				int exitButtonIndex = listview_Main().Items().Size() - 1;
				if (mainSelectIndex >= exitButtonIndex) { mainSelectIndex = 0; }
			}
			try
			{
				listview_Main().SelectedIndex(mainSelectIndex);
			}
			catch (...)
			{
				listview_Main().SelectedIndex(0);
			}
		}
		catch (...) {}
	}

	void MainPage::ShowExperimentalSettings(BOOL silent)
	{
		try
		{
			std::optional<bool> ShowExperimental = AppVariables::Settings.Load<bool>("ShowExperimental");
			if (ShowExperimental.has_value())
			{
				if (ShowExperimental.value())
				{
					//Enable or disable graphics settings
					stackpanel_FsrMultiFrameGenerationRatio().Visibility(Visibility::Visible);
					stackpanel_FsrOverrideMultiFrameGeneration().Visibility(Visibility::Visible);
					stackpanel_FsrOverrideRayRegeneration().Visibility(Visibility::Visible);
					stackpanel_FsrOverrideNeuralRadianceCaching().Visibility(Visibility::Visible);
					stackpanel_FluidMotion_Options().Visibility(Visibility::Visible);
					stackpanel_RadeonBoost_Options().Visibility(Visibility::Visible);
					stackpanel_FsrShowInformation().Visibility(Visibility::Visible);
					stackpanel_FsrDisableMLSwapchain().Visibility(Visibility::Visible);

					textblock_FsrDllLoadPath().Visibility(Visibility::Collapsed);
					textblock_FsrDllLoadPathExperimental().Visibility(Visibility::Visible);

					//Show notification
					if (!silent)
					{
						ShowNotification(L"正在显示实验性设置");
						AVDebugWriteLine(L"Showing experimental settings");
					}
				}
				else
				{
					//Enable or disable graphics settings
					stackpanel_FsrMultiFrameGenerationRatio().Visibility(Visibility::Collapsed);
					stackpanel_FsrOverrideMultiFrameGeneration().Visibility(Visibility::Collapsed);
					stackpanel_FsrOverrideRayRegeneration().Visibility(Visibility::Collapsed);
					stackpanel_FsrOverrideNeuralRadianceCaching().Visibility(Visibility::Collapsed);
					stackpanel_FluidMotion_Options().Visibility(Visibility::Collapsed);
					stackpanel_RadeonBoost_Options().Visibility(Visibility::Collapsed);
					stackpanel_FsrShowInformation().Visibility(Visibility::Collapsed);
					stackpanel_FsrDisableMLSwapchain().Visibility(Visibility::Collapsed);

					textblock_FsrDllLoadPath().Visibility(Visibility::Visible);
					textblock_FsrDllLoadPathExperimental().Visibility(Visibility::Collapsed);

					//Show notification
					if (!silent)
					{
						ShowNotification(L"正在隐藏实验性设置");
						AVDebugWriteLine(L"Hiding experimental settings");
					}
				}
			}
		}
		catch (...) {}
	}

	void MainPage::listview_Main_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e)
	{
		try
		{
			//Get selected index
			int selectedIndex = sender.as<ListView>().SelectedIndex();

			//Exit application
			if (selectedIndex == 7)
			{
				//Exit application
				AppVariables::App.Exit(true);
				return;
			}

			//Save selected index
			AppVariables::Settings.Set("MenuIndex", selectedIndex);

			//Hide all buttons
			stackpanel_Tuning_Buttons().Visibility(Visibility::Collapsed);
			stackpanel_Fan_Buttons().Visibility(Visibility::Collapsed);
			stackpanel_Graphics_Buttons().Visibility(Visibility::Collapsed);
			stackpanel_Display_Buttons().Visibility(Visibility::Collapsed);
			stackpanel_Multimedia_Buttons().Visibility(Visibility::Collapsed);
			stackpanel_Information_Buttons().Visibility(Visibility::Collapsed);

			//Hide all pages
			stackpanel_Graphics().Visibility(Visibility::Collapsed);
			stackpanel_Tuning().Visibility(Visibility::Collapsed);
			stackpanel_Fans().Visibility(Visibility::Collapsed);
			stackpanel_Display().Visibility(Visibility::Collapsed);
			stackpanel_Multimedia().Visibility(Visibility::Collapsed);
			stackpanel_Settings().Visibility(Visibility::Collapsed);
			stackpanel_Information().Visibility(Visibility::Collapsed);

			//Hide selection boxes
			button_GpuSelect().Visibility(Visibility::Collapsed);
			button_DisplaySelect().Visibility(Visibility::Collapsed);
			stackpanel_AppSelect_Graphics().Visibility(Visibility::Collapsed);
			stackpanel_AppSelect_Tuning().Visibility(Visibility::Collapsed);
			stackpanel_AppSelect_Display().Visibility(Visibility::Collapsed);
			stackpanel_AppSelect_Multimedia().Visibility(Visibility::Collapsed);

			//Hide GPU status bar, header selection area and action bar
			border_Gpu_Status().Visibility(Visibility::Collapsed);
			border_Header().Visibility(Visibility::Collapsed);
			border_ActionButtons().Visibility(Visibility::Collapsed);

			//Make selected page visible
			FrameworkElement selectedPage{ nullptr };
			if (selectedIndex == 0)
			{
				selectedPage = stackpanel_Tuning();
				button_GpuSelect().Visibility(Visibility::Visible);
				stackpanel_AppSelect_Tuning().Visibility(Visibility::Visible);
				stackpanel_Tuning().Visibility(Visibility::Visible);
				stackpanel_Tuning_Buttons().Visibility(Visibility::Visible);

				//Show GPU status bar
				border_Gpu_Status().Visibility(Visibility::Visible);
			}
			else if (selectedIndex == 1)
			{
				selectedPage = stackpanel_Fans();
				button_GpuSelect().Visibility(Visibility::Visible);
				stackpanel_AppSelect_Tuning().Visibility(Visibility::Visible);
				stackpanel_Fans().Visibility(Visibility::Visible);
				stackpanel_Fan_Buttons().Visibility(Visibility::Visible);

				//Show GPU status bar
				border_Gpu_Status().Visibility(Visibility::Visible);
			}
			else if (selectedIndex == 2)
			{
				selectedPage = stackpanel_Graphics();
				button_GpuSelect().Visibility(Visibility::Visible);
				stackpanel_AppSelect_Graphics().Visibility(Visibility::Visible);
				stackpanel_Graphics().Visibility(Visibility::Visible);
				stackpanel_Graphics_Buttons().Visibility(Visibility::Visible);

				//Show GPU status bar
				border_Gpu_Status().Visibility(Visibility::Visible);
			}
			else if (selectedIndex == 3)
			{
				selectedPage = stackpanel_Display();
				button_DisplaySelect().Visibility(Visibility::Visible);
				stackpanel_AppSelect_Display().Visibility(Visibility::Visible);
				stackpanel_Display().Visibility(Visibility::Visible);
				stackpanel_Display_Buttons().Visibility(Visibility::Visible);
			}
			else if (selectedIndex == 4)
			{
				selectedPage = stackpanel_Multimedia();
				button_GpuSelect().Visibility(Visibility::Visible);
				stackpanel_AppSelect_Multimedia().Visibility(Visibility::Visible);
				stackpanel_Multimedia().Visibility(Visibility::Visible);
				stackpanel_Multimedia_Buttons().Visibility(Visibility::Visible);
			}
			else if (selectedIndex == 5)
			{
				selectedPage = stackpanel_Settings();
				stackpanel_Settings().Visibility(Visibility::Visible);
			}
			else if (selectedIndex == 6)
			{
				selectedPage = stackpanel_Information();
				button_GpuSelect().Visibility(Visibility::Visible);
				stackpanel_Information().Visibility(Visibility::Visible);
				stackpanel_Information_Buttons().Visibility(Visibility::Visible);

				//Show GPU status bar
				border_Gpu_Status().Visibility(Visibility::Visible);
			}

			//The settings page has no selection or action controls, so both empty rows are removed
			//and its content starts at the top. Every other page keeps the original header/action bar.
			if (selectedIndex != 5)
			{
				border_Header().Visibility(Visibility::Visible);
				border_ActionButtons().Visibility(Visibility::Visible);
			}

			//Animate only the final selected page; a newer selection stops the previous storyboard.
			if (selectedPage)
			{
				AnimatePageEntrance(selectedPage);
			}
			else
			{
				FinalizePageTransition();
			}
		}
		catch (...) {}
	}

	void MainPage::ShowNotification(std::wstring text)
	{
		if (!AppVariables::App.IsCurrentUiPage(*this)) { return; }
		try
		{
			//Set notification text
			grid_Overlay_Notification().Visibility(Visibility::Visible);
			textblock_Notification_Text().Text(text);

			//Only a weak page is captured by the timer tick. A queued tick from an
			//older page cannot touch a released or recreated page.
			winrt::weak_ref<MainPage> weakPage = get_weak();
			uint64_t generation = g_notificationGeneration;
			auto tickFunction = [weakPage, generation](IInspectable const&, IInspectable const&)
				{
					try
					{
						if (generation != g_notificationGeneration)
						{
							return;
						}
						if (auto page = weakPage.get())
						{
							page->grid_Overlay_Notification().Visibility(Visibility::Collapsed);
						}
						if (TimerNotification != NULL)
						{
							TimerNotification.Stop();
							TimerNotification = NULL;
						}
					}
					catch (...) {}
				};

			//Start notification timer
			if (TimerNotification == NULL)
			{
				TimerNotification = winrt::Windows::UI::Xaml::DispatcherTimer();
				TimerNotification.Interval(TimeSpan(40000000));
			}
			TimerNotification.Tick(tickFunction);
			TimerNotification.Start();
		}
		catch (...) {}
	}

	winrt::Windows::Foundation::IInspectable MainPage::KeepAliveForUi()
	{
		winrt::RadeonTuner::MainPage projected = *this;
		return projected;
	}

	void MainPage::StopNotificationTimer()
	{
		//Invalidate any queued notification tick before releasing its handler.
		g_notificationGeneration++;
		try { grid_Overlay_Notification().Visibility(Visibility::Collapsed); } catch (...) {}
		if (TimerNotification != NULL)
		{
			try { TimerNotification.Stop(); } catch (...) {}
			try { TimerNotification.Tick(nullptr); } catch (...) {}
			TimerNotification = NULL;
		}
	}

	bool MainPage::HasActiveResolutionTimer()
	{
		try
		{
			if (TimerResolutionSwitch != NULL && TimerResolutionSwitch.IsEnabled()) { return true; }
			if (TimerResolutionCustom != NULL && TimerResolutionCustom.IsEnabled()) { return true; }
		}
		catch (...) {}
		return false;
	}

	void MainPage::StopUiTimersForRelease()
	{
		StopNotificationTimer();
		if (TimerResolutionSwitch != NULL)
		{
			try { TimerResolutionSwitch.Stop(); } catch (...) {}
			try { TimerResolutionSwitch.Tick(nullptr); } catch (...) {}
			TimerResolutionSwitch = NULL;
		}
		if (TimerResolutionCustom != NULL)
		{
			try { TimerResolutionCustom.Stop(); } catch (...) {}
			try { TimerResolutionCustom.Tick(nullptr); } catch (...) {}
			TimerResolutionCustom = NULL;
		}
	}

	void MainPage::StopUiAnimationsForRelease()
	{
		++pageTransitionGeneration;
		if (pageTransitionStoryboard)
		{
			try { pageTransitionStoryboard.Stop(); } catch (...) {}
		}
		pageTransitionStoryboard = nullptr;
		pageTransitionTarget = nullptr;

		auto dependentAnimationsStop = std::move(dependentAnimations);
		dependentAnimations.clear();
		for (auto& entry : dependentAnimationsStop)
		{
			try { entry.second.Stop(); } catch (...) {}
		}
		auto comboPopupAnimationsStop = std::move(comboPopupAnimations);
		comboPopupAnimations.clear();
		for (auto& entry : comboPopupAnimationsStop)
		{
			try { entry.second.Stop(); } catch (...) {}
		}
		auto toggleKnobAnimationsStop = std::move(toggleKnobAnimations);
		toggleKnobAnimations.clear();
		for (auto& entry : toggleKnobAnimationsStop)
		{
			try { entry.second.Stop(); } catch (...) {}
		}
	}
	std::wstring MainPage::ThemeGetImagePath(std::wstring imageName, bool includeAppx)
	{
		std::wstring imagePath = L"";
		try
		{
			//Get theme style setting
			int themeStyle = 0;
			std::optional<int> setThemeStyle = AppVariables::Settings.Load<int>("ThemeStyle");
			if (setThemeStyle.has_value())
			{
				themeStyle = setThemeStyle.value();
			}

			//Check theme style and set image path
			if (themeStyle == 0 || themeStyle == 1)
			{
				imagePath = L"/Assets/" + imageName + L".png";
			}
			else
			{
				imagePath = L"/Assets/" + imageName + L"Dark.png";
			}

			//Include appx prefix
			if (includeAppx)
			{
				imagePath = L"ms-appx://" + imagePath;
			}

			//AVDebugWriteLine("Theme image path: " << imagePath);
		}
		catch (...) {}
		//Return image path
		return imagePath;
	}

	void MainPage::ThemeAdjustStyle()
	{
		try
		{
			//Get theme style setting
			int themeStyle = 0;
			std::optional<int> setThemeStyle = AppVariables::Settings.Load<int>("ThemeStyle");
			if (setThemeStyle.has_value())
			{
				themeStyle = setThemeStyle.value();
			}

			AVDebugWriteLine(L"Adjusting theme style: " << themeStyle);

			//Apply theme style
			if (themeStyle == 0)
			{
				SolidColorBrush colorGray = Application::Current().Resources().Lookup(box_value(L"ApplicationBlackBrush")).as<SolidColorBrush>();
				RequestedTheme(ElementTheme::Dark);
				Background(colorGray);
			}
			else if (themeStyle == 1)
			{
				SolidColorBrush colorBlack = SolidColorBrush(Windows::UI::Colors::Black());
				RequestedTheme(ElementTheme::Dark);
				Background(colorBlack);
			}
			else
			{
				SolidColorBrush colorWhite = Application::Current().Resources().Lookup(box_value(L"ApplicationWhiteBrush")).as<SolidColorBrush>();
				RequestedTheme(ElementTheme::Light);
				Background(colorWhite);
			}

			//Keep the native title bar and Win32 menu in the same light/dark mode
			AppVariables::App.RefreshNativeTheme();
		}
		catch (...) {}
	}
}


//UI session helpers. These are intentionally outside the page type so the
//snapshot can outlive the page and contains no XAML reference.
namespace
{
	struct UiSnapshotEntry
	{
		bool HasEnabled = false;
		bool Enabled = true;
		bool HasVisibility = false;
		int Visibility = 0;
		bool HasIsOn = false;
		bool IsOn = false;
		bool HasValue = false;
		double Value = 0;
		bool HasIndex = false;
		int Index = -1;
		bool HasText = false;
		std::wstring Text;
	};

	std::unordered_map<std::wstring, UiSnapshotEntry> g_uiSnapshot;

	void VisitUiElements(winrt::Windows::UI::Xaml::DependencyObject const& node, std::function<void(winrt::Windows::UI::Xaml::FrameworkElement const&)> const& action)
	{
		using winrt::Windows::UI::Xaml::FrameworkElement;
		using winrt::Windows::UI::Xaml::Media::VisualTreeHelper;

		if (auto element = node.try_as<FrameworkElement>())
		{
			action(element);
		}
		int childCount = VisualTreeHelper::GetChildrenCount(node);
		for (int index = 0; index < childCount; index++)
		{
			auto child = VisualTreeHelper::GetChild(node, index);
			if (child)
			{
				VisitUiElements(child, action);
			}
		}
	}
}

namespace RadeonTunerUiSession
{
	using winrt::Windows::UI::Xaml::Controls::ComboBox;
	using winrt::Windows::UI::Xaml::Controls::ListView;
	using winrt::Windows::UI::Xaml::Controls::Slider;
	using winrt::Windows::UI::Xaml::Controls::TextBox;
	using winrt::Windows::UI::Xaml::Controls::TextBlock;
	using winrt::Windows::UI::Xaml::Controls::ToggleSwitch;
	using winrt::Windows::UI::Xaml::FrameworkElement;

	void Capture(winrt::RadeonTuner::implementation::MainPage& page)
	{
		try
		{
			g_uiSnapshot.clear();
			VisitUiElements(page, [&page](FrameworkElement const& element)
				{
					try
					{
						std::wstring name(element.Name().c_str());
						//Template parts have their own namescopes and repeat across controls.
						//Only page-owned elements can be restored by a page-level name.
						if (name.empty() || page.FindName(element.Name()) != element)
						{
							return;
						}

						//Transient modal/overlay content is never restored as a draft.
						if (name.find(L"_Overlay_") != std::wstring::npos || name.rfind(L"grid_Overlay_", 0) == 0)
						{
							return;
						}

						UiSnapshotEntry entry{};
						if (auto control = element.try_as<winrt::Windows::UI::Xaml::Controls::Control>())
						{
							entry.HasEnabled = true;
							entry.Enabled = control.IsEnabled();
						}
						entry.HasVisibility = true;
						entry.Visibility = (int)element.Visibility();

						if (auto toggle = element.try_as<ToggleSwitch>())
						{
							entry.HasIsOn = true;
							entry.IsOn = toggle.IsOn();
						}
						if (auto slider = element.try_as<Slider>())
						{
							entry.HasValue = true;
							entry.Value = slider.Value();
						}
						if (auto comboBox = element.try_as<ComboBox>())
						{
							entry.HasIndex = true;
							entry.Index = comboBox.SelectedIndex();
						}
						if (auto textBox = element.try_as<TextBox>())
						{
							entry.HasText = true;
							entry.Text = textBox.Text().c_str();
						}
						else if (auto textBlock = element.try_as<TextBlock>())
						{
							entry.HasText = true;
							entry.Text = textBlock.Text().c_str();
						}
						if (auto listView = element.try_as<ListView>())
						{
							entry.HasIndex = true;
							entry.Index = listView.SelectedIndex();
						}

						g_uiSnapshot[name] = std::move(entry);
					}
					catch (...) {}
				});
		}
		catch (...) {}
	}

	void PrepareForRelease(winrt::RadeonTuner::implementation::MainPage& page)
	{
		try
		{
			page.StopUiAnimationsForRelease();
			page.StopUiTimersForRelease();
		}
		catch (...) {}
	}

	void Restore(winrt::RadeonTuner::implementation::MainPage& page)
	{
		if (g_uiSnapshot.empty())
		{
			return;
		}
		try
		{
			VisitUiElements(page, [&page](FrameworkElement const& element)
				{
					try
					{
						std::wstring name(element.Name().c_str());
						if (name.empty() || page.FindName(element.Name()) != element)
						{
							return;
						}
						auto entry = g_uiSnapshot.find(name);
						if (entry == g_uiSnapshot.end())
						{
							return;
						}
						const UiSnapshotEntry& state = entry->second;

						//Restore values first; the real handlers run with saving disabled
						//and only refresh dependent presentation.
						if (state.HasIsOn)
						{
							if (auto toggle = element.try_as<ToggleSwitch>())
							{
								toggle.IsOn(state.IsOn);
							}
						}
						if (state.HasValue)
						{
							if (auto slider = element.try_as<Slider>())
							{
								slider.Value(state.Value);
							}
						}
						if (state.HasIndex)
						{
							if (auto comboBox = element.try_as<ComboBox>())
							{
								comboBox.SelectedIndex(state.Index);
							}
							else if (auto listView = element.try_as<ListView>())
							{
								listView.SelectedIndex(state.Index);
							}
						}
						if (state.HasText)
						{
							if (auto textBox = element.try_as<TextBox>())
							{
								textBox.Text(state.Text);
							}
							else if (auto textBlock = element.try_as<TextBlock>())
							{
								textBlock.Text(state.Text);
							}
						}
						if (state.HasEnabled)
						{
							if (auto control = element.try_as<winrt::Windows::UI::Xaml::Controls::Control>())
							{
								control.IsEnabled(state.Enabled);
							}
						}
						if (state.HasVisibility)
						{
							element.Visibility((winrt::Windows::UI::Xaml::Visibility)state.Visibility);
						}
						//Local snapshot values replace Binding objects. Restore the live
						//value/capability bindings of the editable slider number boxes.
						if (auto textBox = element.try_as<TextBox>())
						{
							if (textBox.Tag().try_as<Slider>()) { page.RestoreNumericInputValue(textBox); }
						}
					}
					catch (...) {}
				});
		}
		catch (...) {}
	}

	bool HasSnapshot()
	{
		return !g_uiSnapshot.empty();
	}

	void ClearSnapshot()
	{
		g_uiSnapshot.clear();
	}

	std::string CaptureSnapshotJson()
	{
		try
		{
			nlohmann::json snapshot = nlohmann::json::array();
			for (auto const& pair : g_uiSnapshot)
			{
				const UiSnapshotEntry& state = pair.second;
				nlohmann::json entry = nlohmann::json::object();
				entry["name"] = wstring_to_string_utf8(pair.first);
				entry["has_enabled"] = state.HasEnabled;
				entry["enabled"] = state.Enabled;
				entry["has_visibility"] = state.HasVisibility;
				entry["visibility"] = state.Visibility;
				entry["has_ison"] = state.HasIsOn;
				entry["ison"] = state.IsOn;
				entry["has_value"] = state.HasValue;
				entry["value"] = state.Value;
				entry["has_index"] = state.HasIndex;
				entry["index"] = state.Index;
				entry["has_text"] = state.HasText;
				entry["text"] = wstring_to_string_utf8(state.Text);
				snapshot.push_back(std::move(entry));
			}
			return snapshot.dump();
		}
		catch (...)
		{
			return "[]";
		}
	}

	bool RestoreSnapshotJson(std::string const& json)
	{
		try
		{
			nlohmann::json snapshot = nlohmann::json::parse(json);
			if (!snapshot.is_array())
			{
				return false;
			}
			g_uiSnapshot.clear();
			for (auto const& item : snapshot)
			{
				try
				{
					UiSnapshotEntry state{};
					state.HasEnabled = item.value("has_enabled", false);
					state.Enabled = item.value("enabled", true);
					state.HasVisibility = item.value("has_visibility", false);
					state.Visibility = item.value("visibility", 0);
					state.HasIsOn = item.value("has_ison", false);
					state.IsOn = item.value("ison", false);
					state.HasValue = item.value("has_value", false);
					state.Value = item.value("value", 0.0);
					state.HasIndex = item.value("has_index", false);
					state.Index = item.value("index", -1);
					state.HasText = item.value("has_text", false);
					state.Text = string_to_wstring_utf8(item.value("text", std::string()));
					g_uiSnapshot[string_to_wstring_utf8(item.value("name", std::string()))] = std::move(state);
				}
				catch (...) {}
			}
			return true;
		}
		catch (...)
		{
			return false;
		}
	}
}


namespace RadeonTunerBackend
{
	using winrt::RadeonTuner::implementation::MainPage;
	namespace
	{
		std::mutex g_profileLoopMutex;
		std::condition_variable g_profileLoopCv;
		bool g_profileLoopPaused = false;
		int g_profileLoopActiveBatches = 0;
		std::atomic<bool> g_defaultsApplied{ false };
		std::atomic<bool> g_syncApplied{ false };

		void WaitForProfileLoopResume()
		{
			std::unique_lock<std::mutex> lock(g_profileLoopMutex);
			g_profileLoopCv.wait(lock, []()
				{
					return !g_profileLoopPaused || AppVariables::ApplicationExiting;
				});
		}

		bool BeginProfileBatch()
		{
			std::lock_guard<std::mutex> lock(g_profileLoopMutex);
			if (g_profileLoopPaused)
			{
				return false;
			}
			++g_profileLoopActiveBatches;
			return true;
		}

		void EndProfileBatch()
		{
			{
				std::lock_guard<std::mutex> lock(g_profileLoopMutex);
				if (g_profileLoopActiveBatches > 0)
				{
					--g_profileLoopActiveBatches;
				}
			}
			g_profileLoopCv.notify_all();
		}
	}

	void PauseProfileLoop()
	{
		std::unique_lock<std::mutex> lock(g_profileLoopMutex);
		g_profileLoopPaused = true;
		g_profileLoopCv.wait(lock, []()
			{
				return g_profileLoopActiveBatches == 0;
			});
	}

	void ResumeProfileLoop()
	{
		{
			std::lock_guard<std::mutex> lock(g_profileLoopMutex);
			g_profileLoopPaused = false;
		}
		g_profileLoopCv.notify_all();
	}

	std::wstring InitializeHeadless(HeadlessOptions const& options)
	{
		try
		{
			if (options.InitializeAdl && adl_Context == NULL)
			{
				std::wstring initResult = MainPage::AdlInitialize();
				if (!initResult.empty())
				{
					return initResult;
				}
			}
			if (adl_Context == NULL)
			{
				return L"ADL 未初始化";
			}

			//Profiles are loaded before any default application sync, matching the UI startup order.
			if (options.LoadProfiles)
			{
				ReloadProfiles();
			}

			if (options.SelectDefaults)
			{
				std::vector<AdapterInfo> gpuList = MainPage::AdlGetGpuAll(false);
				if (gpuList.empty())
				{
					return L"未找到任何 GPU";
				}
				if (adl_Gpu_AdapterIndex < 0)
				{
					AdapterInfo& adapterInfo = gpuList[0];
					adl_Gpu_AdapterIndex = adapterInfo.iAdapterIndex;
					adl_Gpu_RegistryPath = string_to_wstring(adapterInfo.strDriverPathExt);
					adl_Gpu_UniqueIdentifierHex = L"0x0001";
					adl_Gpu_DeviceIdentifier = MainPage::AdlxGetGpuIdentifier(adl_Gpu_AdapterIndex);
				}

				std::vector<ADLDisplayInfo> displayList = MainPage::AdlGetDisplayAll();
				if (!displayList.empty() && adl_Display_AdapterIndex < 0)
				{
					ADLDisplayInfo& displayInfo = displayList[0];
					adl_Display_AdapterIndex = displayInfo.displayID.iDisplayLogicalAdapterIndex;
					adl_Display_DisplayIndex = displayInfo.displayID.iDisplayLogicalIndex;
					adl_Display_DeviceIdentifier = MainPage::AdlxGetDisplayIdentifier(adl_Display_AdapterIndex, adl_Display_DisplayIndex);
				}
			}

			if ((options.SyncApplications || options.WriteDefaults) && adl_Gpu_AdapterIndex < 0)
			{
				return L"未选择 GPU";
			}

			//The original one-time operations are guarded per process. A later worker
			//passes false for both flags and never repeats global registry defaults.
			if (options.SyncApplications && !g_syncApplied.exchange(true))
			{
				MainPage::AdlAppSyncAll();
			}
			if (options.WriteDefaults && !g_defaultsApplied.exchange(true))
			{
				MainPage::AdlSetDefaultSettings();
			}

			BackendInitialized = true;
			return L"";
		}
		catch (...)
		{
			return L"后台初始化失败（异常）";
		}
	}

	bool ReloadProfiles()
	{
		try
		{
			MainPage::TuningFanSettings_Profiles_LoadFromFile();
			MainPage::GraphicsSettings_Profiles_LoadFromFile();
			MainPage::DisplaySettings_Profiles_LoadFromFile();
			MainPage::MultimediaSettings_Profiles_LoadFromFile();
			return true;
		}
		catch (...)
		{
			return false;
		}
	}

	void ReloadSettings()
	{
		try
		{
			if (!AppVariables::SaveDataPath.empty())
			{
				//AVSettingsJson caches the whole file at construction; rebuild the object
				//instead of calling Load and assuming it re-reads the disk.
				AppVariables::Settings = AVSettingsJson(PathMerge(AppVariables::SaveDataPath, L"Settings\\Settings.json"));
			}
		}
		catch (...) {}
	}

	namespace
	{
		nlohmann::json IntJson(AdlSettingInt const& value)
		{
			nlohmann::json j = nlohmann::json::object();
			j["c"] = value.Current; j["d"] = value.Default; j["n"] = value.Minimum; j["x"] = value.Maximum; j["s"] = value.Step; j["u"] = value.Support;
			return j;
		}
		void IntLoad(nlohmann::json const& j, AdlSettingInt& value)
		{
			try { if (j.contains("c")) value.Current = j["c"].is_null() ? std::optional<int>() : std::optional<int>(j["c"].get<int>()); } catch (...) {}
			try { if (j.contains("d")) value.Default = j["d"].is_null() ? std::optional<int>() : std::optional<int>(j["d"].get<int>()); } catch (...) {}
			try { if (j.contains("n")) value.Minimum = j["n"].is_null() ? std::optional<int>() : std::optional<int>(j["n"].get<int>()); } catch (...) {}
			try { if (j.contains("x")) value.Maximum = j["x"].is_null() ? std::optional<int>() : std::optional<int>(j["x"].get<int>()); } catch (...) {}
			try { if (j.contains("s")) value.Step = j["s"].is_null() ? std::optional<int>() : std::optional<int>(j["s"].get<int>()); } catch (...) {}
			try { if (j.contains("u")) value.Support = j["u"].get<bool>(); } catch (...) {}
		}
		nlohmann::json FloatJson(AdlSettingFloat const& value)
		{
			nlohmann::json j = nlohmann::json::object();
			j["c"] = value.Current; j["d"] = value.Default; j["n"] = value.Minimum; j["x"] = value.Maximum; j["s"] = value.Step; j["u"] = value.Support;
			return j;
		}
		void FloatLoad(nlohmann::json const& j, AdlSettingFloat& value)
		{
			try { if (j.contains("c")) value.Current = j["c"].is_null() ? std::optional<float>() : std::optional<float>(j["c"].get<float>()); } catch (...) {}
			try { if (j.contains("d")) value.Default = j["d"].is_null() ? std::optional<float>() : std::optional<float>(j["d"].get<float>()); } catch (...) {}
			try { if (j.contains("n")) value.Minimum = j["n"].is_null() ? std::optional<float>() : std::optional<float>(j["n"].get<float>()); } catch (...) {}
			try { if (j.contains("x")) value.Maximum = j["x"].is_null() ? std::optional<float>() : std::optional<float>(j["x"].get<float>()); } catch (...) {}
			try { if (j.contains("s")) value.Step = j["s"].is_null() ? std::optional<float>() : std::optional<float>(j["s"].get<float>()); } catch (...) {}
			try { if (j.contains("u")) value.Support = j["u"].get<bool>(); } catch (...) {}
		}
		nlohmann::json StringJson(AdlSettingString const& value)
		{
			nlohmann::json j = nlohmann::json::object();
			j["c"] = value.Current.has_value() ? nlohmann::json(wstring_to_string_utf8(value.Current.value())) : nlohmann::json(nullptr);
			j["d"] = value.Default.has_value() ? nlohmann::json(wstring_to_string_utf8(value.Default.value())) : nlohmann::json(nullptr);
			j["u"] = value.Support;
			return j;
		}
		void StringLoad(nlohmann::json const& j, AdlSettingString& value)
		{
			try { if (j.contains("c")) { value.Current = j["c"].is_null() ? std::optional<std::wstring>() : std::optional<std::wstring>(string_to_wstring_utf8(j["c"].get<std::string>())); } } catch (...) {}
			try { if (j.contains("d")) { value.Default = j["d"].is_null() ? std::optional<std::wstring>() : std::optional<std::wstring>(string_to_wstring_utf8(j["d"].get<std::string>())); } } catch (...) {}
			try { if (j.contains("u")) value.Support = j["u"].get<bool>(); } catch (...) {}
		}
	}

	static nlohmann::json TuningToJson(TuningFanSettings const& v)
	{
		nlohmann::json j = nlohmann::json::object();
		j["DeviceId"] = v.DeviceId.has_value() ? wstring_to_string_utf8(v.DeviceId.value()) : std::string();
		j["Application"] = v.Application.has_value() ? wstring_to_string_utf8(v.Application.value()) : std::string();
#define RT_T_INT(name) j[#name] = IntJson(v.name);
		RT_T_INT(CoreMin) RT_T_INT(CoreMax) RT_T_INT(MemoryTiming) RT_T_INT(MemoryEcc) RT_T_INT(MemoryMax)
		RT_T_INT(PowerLimit) RT_T_INT(PowerVoltage) RT_T_INT(PowerTDC) RT_T_INT(FanSpeedMin) RT_T_INT(FanSpeedMax)
		RT_T_INT(FanZeroRpm) RT_T_INT(FanSpeed0) RT_T_INT(FanTemp0) RT_T_INT(FanSpeed1) RT_T_INT(FanTemp1)
		RT_T_INT(FanSpeed2) RT_T_INT(FanTemp2) RT_T_INT(FanSpeed3) RT_T_INT(FanTemp3) RT_T_INT(FanSpeed4) RT_T_INT(FanTemp4)
		RT_T_INT(KeepActive) RT_T_INT(FanControlMode) RT_T_INT(FanRangedCurve)
#undef RT_T_INT
		j["FanZeroTemp"] = v.FanZeroTemp; j["FanSupport"] = v.FanSupport; j["TuningSupport"] = v.TuningSupport; j["UsingProfile"] = v.UsingProfile;
		return j;
	}
	static TuningFanSettings TuningFromJson(nlohmann::json const& j)
	{
		TuningFanSettings v{};
		try { v.DeviceId = string_to_wstring_utf8(j.value("DeviceId", std::string())); } catch (...) {}
		try { v.Application = string_to_wstring_utf8(j.value("Application", std::string())); } catch (...) {}
#define RT_T_LOAD(name) if (j.contains(#name)) { IntLoad(j[#name], v.name); }
		RT_T_LOAD(CoreMin) RT_T_LOAD(CoreMax) RT_T_LOAD(MemoryTiming) RT_T_LOAD(MemoryEcc) RT_T_LOAD(MemoryMax)
		RT_T_LOAD(PowerLimit) RT_T_LOAD(PowerVoltage) RT_T_LOAD(PowerTDC) RT_T_LOAD(FanSpeedMin) RT_T_LOAD(FanSpeedMax)
		RT_T_LOAD(FanZeroRpm) RT_T_LOAD(FanSpeed0) RT_T_LOAD(FanTemp0) RT_T_LOAD(FanSpeed1) RT_T_LOAD(FanTemp1)
		RT_T_LOAD(FanSpeed2) RT_T_LOAD(FanTemp2) RT_T_LOAD(FanSpeed3) RT_T_LOAD(FanTemp3) RT_T_LOAD(FanSpeed4) RT_T_LOAD(FanTemp4)
		RT_T_LOAD(KeepActive) RT_T_LOAD(FanControlMode) RT_T_LOAD(FanRangedCurve)
#undef RT_T_LOAD
		try { v.FanZeroTemp = j.value("FanZeroTemp", -1); } catch (...) {}
		try { v.FanSupport = j.value("FanSupport", false); } catch (...) {}
		try { v.TuningSupport = j.value("TuningSupport", false); } catch (...) {}
		try { v.UsingProfile = j.value("UsingProfile", false); } catch (...) {}
		return v;
	}

	static nlohmann::json DisplayToJson(DisplaySettings const& v)
	{
		nlohmann::json j = nlohmann::json::object();
		j["DeviceId"] = v.DeviceId.has_value() ? wstring_to_string_utf8(v.DeviceId.value()) : std::string();
		j["Application"] = v.Application.has_value() ? wstring_to_string_utf8(v.Application.value()) : std::string();
#define RT_D_INT(name) j[#name] = IntJson(v.name);
		RT_D_INT(HdrEnabled) RT_D_INT(FreeSyncMode) RT_D_INT(VsrEnabled) RT_D_INT(GpuScalingEnabled) RT_D_INT(IntegerScalingEnabled)
		RT_D_INT(ScalingMode) RT_D_INT(VariBrightEnabled) RT_D_INT(VariBrightLevel) RT_D_INT(ColorDepth) RT_D_INT(PixelFormat)
		RT_D_INT(ColorEnhancement) RT_D_INT(Brightness) RT_D_INT(Contrast) RT_D_INT(Saturation) RT_D_INT(Hue)
		RT_D_INT(ColorTemperatureControl) RT_D_INT(ColorTemperatureKelvin) RT_D_INT(CVDCControl) RT_D_INT(CVDCProtanopia)
		RT_D_INT(CVDCDeuteranopia) RT_D_INT(CVDCTritanopia) RT_D_INT(HDCPEnabled) RT_D_INT(GammaControl) RT_D_INT(EyefinityAutomatic)
#undef RT_D_INT
		j["GammaBase"] = FloatJson(v.GammaBase); j["GammaRed"] = FloatJson(v.GammaRed); j["GammaGreen"] = FloatJson(v.GammaGreen); j["GammaBlue"] = FloatJson(v.GammaBlue);
		j["UsingProfile"] = v.UsingProfile;
		return j;
	}
	static DisplaySettings DisplayFromJson(nlohmann::json const& j)
	{
		DisplaySettings v{};
		try { v.DeviceId = string_to_wstring_utf8(j.value("DeviceId", std::string())); } catch (...) {}
		try { v.Application = string_to_wstring_utf8(j.value("Application", std::string())); } catch (...) {}
#define RT_D_LOAD(name) if (j.contains(#name)) { IntLoad(j[#name], v.name); }
		RT_D_LOAD(HdrEnabled) RT_D_LOAD(FreeSyncMode) RT_D_LOAD(VsrEnabled) RT_D_LOAD(GpuScalingEnabled) RT_D_LOAD(IntegerScalingEnabled)
		RT_D_LOAD(ScalingMode) RT_D_LOAD(VariBrightEnabled) RT_D_LOAD(VariBrightLevel) RT_D_LOAD(ColorDepth) RT_D_LOAD(PixelFormat)
		RT_D_LOAD(ColorEnhancement) RT_D_LOAD(Brightness) RT_D_LOAD(Contrast) RT_D_LOAD(Saturation) RT_D_LOAD(Hue)
		RT_D_LOAD(ColorTemperatureControl) RT_D_LOAD(ColorTemperatureKelvin) RT_D_LOAD(CVDCControl) RT_D_LOAD(CVDCProtanopia)
		RT_D_LOAD(CVDCDeuteranopia) RT_D_LOAD(CVDCTritanopia) RT_D_LOAD(HDCPEnabled) RT_D_LOAD(GammaControl) RT_D_LOAD(EyefinityAutomatic)
#undef RT_D_LOAD
		if (j.contains("GammaBase")) { FloatLoad(j["GammaBase"], v.GammaBase); } if (j.contains("GammaRed")) { FloatLoad(j["GammaRed"], v.GammaRed); } if (j.contains("GammaGreen")) { FloatLoad(j["GammaGreen"], v.GammaGreen); } if (j.contains("GammaBlue")) { FloatLoad(j["GammaBlue"], v.GammaBlue); }
		try { v.UsingProfile = j.value("UsingProfile", false); } catch (...) {}
		return v;
	}

	static nlohmann::json GraphicsToJson(GraphicsSettings const& v)
	{
		nlohmann::json j = nlohmann::json::object();
		j["DeviceId"] = v.DeviceId.has_value() ? wstring_to_string_utf8(v.DeviceId.value()) : std::string();
		j["Application"] = v.Application.has_value() ? wstring_to_string_utf8(v.Application.value()) : std::string();
#define RT_G_INT(name) j[#name] = IntJson(v.name);
		RT_G_INT(DeLagEnabled) RT_G_INT(MfgRatio) RT_G_INT(FsrOverride) RT_G_INT(MlfiOverride) RT_G_INT(MfgOverride) RT_G_INT(MldOverride) RT_G_INT(NrcOverride)
		RT_G_INT(ChillEnabled) RT_G_INT(ChillMinFps) RT_G_INT(ChillMaxFps) RT_G_INT(FrtcEnabled) RT_G_INT(FrtcFrameRateTarget) RT_G_INT(BoostMode) RT_G_INT(BoostMinResolution)
		RT_G_INT(RisEnabled) RT_G_INT(RisDesktopEnabled) RT_G_INT(RisSharpeningDegree) RT_G_INT(VerticalSync) RT_G_INT(EnhancedSync) RT_G_INT(OpenGLTripleBuffering)
		RT_G_INT(AntiAliasingOverride) RT_G_INT(AntiAliasingMethod) RT_G_INT(AntiAliasingLevel) RT_G_INT(AntiAliasingEnhancedQuality) RT_G_INT(AntiAliasingMorphological)
		RT_G_INT(AnisotropicOverride) RT_G_INT(TextureFilteringQuality) RT_G_INT(SurfaceFormatOptimization) RT_G_INT(OpenGL10BitPixelFormat) RT_G_INT(TessellationMode) RT_G_INT(TessellationLevel) RT_G_INT(FrameGenEnabled) RT_G_INT(FrameGenAlgorithm)
		RT_G_INT(FsrShowInformation) RT_G_INT(FsrDisableMLSwapchain)
#undef RT_G_INT
		j["FsrOvrDLLPath"] = StringJson(v.FsrOvrDLLPath); j["RisVersion"] = v.RisVersion.Version;
		return j;
	}
	static GraphicsSettings GraphicsFromJson(nlohmann::json const& j)
	{
		GraphicsSettings v{};
		try { v.DeviceId = string_to_wstring_utf8(j.value("DeviceId", std::string())); } catch (...) {}
		try { v.Application = string_to_wstring_utf8(j.value("Application", std::string())); } catch (...) {}
#define RT_G_LOAD(name) if (j.contains(#name)) { IntLoad(j[#name], v.name); }
		RT_G_LOAD(DeLagEnabled) RT_G_LOAD(MfgRatio) RT_G_LOAD(FsrOverride) RT_G_LOAD(MlfiOverride) RT_G_LOAD(MfgOverride) RT_G_LOAD(MldOverride) RT_G_LOAD(NrcOverride)
		RT_G_LOAD(ChillEnabled) RT_G_LOAD(ChillMinFps) RT_G_LOAD(ChillMaxFps) RT_G_LOAD(FrtcEnabled) RT_G_LOAD(FrtcFrameRateTarget) RT_G_LOAD(BoostMode) RT_G_LOAD(BoostMinResolution)
		RT_G_LOAD(RisEnabled) RT_G_LOAD(RisDesktopEnabled) RT_G_LOAD(RisSharpeningDegree) RT_G_LOAD(VerticalSync) RT_G_LOAD(EnhancedSync) RT_G_LOAD(OpenGLTripleBuffering)
		RT_G_LOAD(AntiAliasingOverride) RT_G_LOAD(AntiAliasingMethod) RT_G_LOAD(AntiAliasingLevel) RT_G_LOAD(AntiAliasingEnhancedQuality) RT_G_LOAD(AntiAliasingMorphological)
		RT_G_LOAD(AnisotropicOverride) RT_G_LOAD(TextureFilteringQuality) RT_G_LOAD(SurfaceFormatOptimization) RT_G_LOAD(OpenGL10BitPixelFormat) RT_G_LOAD(TessellationMode) RT_G_LOAD(TessellationLevel) RT_G_LOAD(FrameGenEnabled) RT_G_LOAD(FrameGenAlgorithm)
		RT_G_LOAD(FsrShowInformation) RT_G_LOAD(FsrDisableMLSwapchain)
#undef RT_G_LOAD
		if (j.contains("FsrOvrDLLPath")) { StringLoad(j["FsrOvrDLLPath"], v.FsrOvrDLLPath); } try { v.RisVersion.Version = j.value("RisVersion", 0); } catch (...) {}
		return v;
	}

	static nlohmann::json MultimediaToJson(MultimediaSettings const& v) { nlohmann::json j = nlohmann::json::object(); j["DeviceId"] = v.DeviceId.has_value() ? wstring_to_string_utf8(v.DeviceId.value()) : std::string(); j["Application"] = v.Application.has_value() ? wstring_to_string_utf8(v.Application.value()) : std::string(); j["VideoUpscaling"] = IntJson(v.VideoUpscaling); j["VideoSharpening"] = IntJson(v.VideoSharpening); j["VideoBrightness"] = IntJson(v.VideoBrightness); j["UsingProfile"] = v.UsingProfile; return j; }
	static MultimediaSettings MultimediaFromJson(nlohmann::json const& j) { MultimediaSettings v{}; try { v.DeviceId = string_to_wstring_utf8(j.value("DeviceId", std::string())); } catch (...) {} try { v.Application = string_to_wstring_utf8(j.value("Application", std::string())); } catch (...) {} if (j.contains("VideoUpscaling")) { IntLoad(j["VideoUpscaling"], v.VideoUpscaling); } if (j.contains("VideoSharpening")) { IntLoad(j["VideoSharpening"], v.VideoSharpening); } if (j.contains("VideoBrightness")) { IntLoad(j["VideoBrightness"], v.VideoBrightness); } try { v.UsingProfile = j.value("UsingProfile", false); } catch (...) {} return v; }

	static void RestoreGammaTargets(nlohmann::json const& state)
	{
		try
		{
			if (!state.contains("gamma_restore_targets") || !state["gamma_restore_targets"].is_array()) { return; }
			std::lock_guard<std::mutex> lock(gammaAppliedTargetsMutex);
			gammaAppliedTargets.clear();
			for (auto const& item : state["gamma_restore_targets"])
			{
				std::wstring target = string_to_wstring_utf8(item.get<std::string>());
				if (!target.empty()) { gammaAppliedTargets.insert(target); }
			}
		}
		catch (...) {}
	}

	static void RestoreCacheFlags(nlohmann::json const& state)
	{
		auto restore = [](nlohmann::json const& array, auto& cache)
		{
			for (auto const& item : array)
			{
				try
				{
					std::wstring deviceId = string_to_wstring_utf8(item.value("DeviceId", std::string()));
					std::wstring application = string_to_wstring_utf8(item.value("Application", std::string()));
					bool usingProfile = item.value("UsingProfile", false);
					for (auto& model : cache)
					{
						if (model.DeviceId.has_value() && model.DeviceId.value() == deviceId && model.Application.has_value() && model.Application.value() == application)
						{
							model.UsingProfile = usingProfile;
							break;
						}
					}
				}
				catch (...) {}
			}
		};
		if (state.contains("tuning_cache")) { restore(state["tuning_cache"], tuningFanSettingsCache); }
		if (state.contains("display_cache")) { restore(state["display_cache"], displaySettingsCache); }
		if (state.contains("multimedia_cache")) { restore(state["multimedia_cache"], multimediaSettingsCache); }
	}

	std::string MergeRuntimeStateJson(std::string const& draftJson, std::string const& runtimeJson)
	{
		try
		{
			if (draftJson.empty()) { return runtimeJson; }
			if (runtimeJson.empty()) { return draftJson; }
			nlohmann::json draft = nlohmann::json::parse(draftJson);
			nlohmann::json runtime = nlohmann::json::parse(runtimeJson);

			for (const char* key : { "tuning_cache", "display_cache", "multimedia_cache", "gamma_restore_targets" })
			{
				if (runtime.contains(key)) { draft[key] = runtime[key]; }
			}

			auto alignUsing = [&](const char* modelKey, const char* cacheKey)
			{
				try
				{
					if (!draft.contains(modelKey) || !draft[modelKey].is_object()) { return; }
					std::wstring deviceId = string_to_wstring_utf8(draft[modelKey].value("DeviceId", std::string()));
					std::wstring application = string_to_wstring_utf8(draft[modelKey].value("Application", std::string()));
					bool usingProfile = false;
					if (runtime.contains(cacheKey) && runtime[cacheKey].is_array())
					{
						for (auto const& item : runtime[cacheKey])
						{
							if (string_to_wstring_utf8(item.value("DeviceId", std::string())) == deviceId && string_to_wstring_utf8(item.value("Application", std::string())) == application)
							{
								usingProfile = item.value("UsingProfile", false);
								break;
							}
						}
					}
					draft[modelKey]["UsingProfile"] = usingProfile;
				}
				catch (...) {}
			};
			alignUsing("tuning", "tuning_cache");
			alignUsing("display", "display_cache");
			alignUsing("multimedia", "multimedia_cache");
			return draft.dump();
		}
		catch (...) { return draftJson; }
	}

	bool ApplyRuntimeStateJson(std::string const& json)
	{
		try
		{
			if (json.empty()) { return false; }
			nlohmann::json state = nlohmann::json::parse(json);
			RestoreCacheFlags(state);
			RestoreGammaTargets(state);
			return true;
		}
		catch (...) { return false; }
	}

	std::string CaptureStateJson()
	{
		try
		{
			nlohmann::json state = nlohmann::json::object();
			state["gpu_device"] = wstring_to_string_utf8(adl_Gpu_DeviceIdentifier);
			state["display_device"] = wstring_to_string_utf8(adl_Display_DeviceIdentifier);
			state["gpu_index"] = adl_Gpu_AdapterIndex;
			state["display_adapter_index"] = adl_Display_AdapterIndex;
			state["display_index"] = adl_Display_DisplayIndex;
			state["tuning_app"] = wstring_to_string_utf8(tuningFanSettingsProfile.Application.has_value() ? tuningFanSettingsProfile.Application.value() : std::wstring(L"Global"));
			state["display_app"] = wstring_to_string_utf8(displaySettingsProfile.Application.has_value() ? displaySettingsProfile.Application.value() : std::wstring(L"Global"));
			state["graphics_app"] = wstring_to_string_utf8(graphicsSettingsProfile.Application.has_value() ? graphicsSettingsProfile.Application.value() : std::wstring(L"Global"));
			state["multimedia_app"] = wstring_to_string_utf8(multimediaSettingsProfile.Application.has_value() ? multimediaSettingsProfile.Application.value() : std::wstring(L"Global"));
			state["tuning"] = TuningToJson(tuningFanSettingsProfile);
			state["display"] = DisplayToJson(displaySettingsProfile);
			state["graphics"] = GraphicsToJson(graphicsSettingsProfile);
			state["multimedia"] = MultimediaToJson(multimediaSettingsProfile);

			auto appendCache = [](nlohmann::json& array, auto const& cache)
			{
				for (auto const& model : cache)
				{
					nlohmann::json item = nlohmann::json::object();
					item["DeviceId"] = model.DeviceId.has_value() ? wstring_to_string_utf8(model.DeviceId.value()) : std::string();
					item["Application"] = model.Application.has_value() ? wstring_to_string_utf8(model.Application.value()) : std::string();
					item["UsingProfile"] = model.UsingProfile;
					array.push_back(std::move(item));
				}
			};
			state["tuning_cache"] = nlohmann::json::array(); appendCache(state["tuning_cache"], tuningFanSettingsCache);
			state["display_cache"] = nlohmann::json::array(); appendCache(state["display_cache"], displaySettingsCache);
			state["multimedia_cache"] = nlohmann::json::array(); appendCache(state["multimedia_cache"], multimediaSettingsCache);
			state["gamma_restore_targets"] = nlohmann::json::array();
			{
				std::lock_guard<std::mutex> lock(gammaAppliedTargetsMutex);
				for (auto const& target : gammaAppliedTargets) { state["gamma_restore_targets"].push_back(wstring_to_string_utf8(target)); }
			}
			state["snapshot"] = nlohmann::json::parse(RadeonTunerUiSession::CaptureSnapshotJson());
			return state.dump();
		}
		catch (...)
		{
			return std::string();
		}
	}

	bool ApplyStateJson(std::string const& json)
	{
		try
		{
			if (json.empty())
			{
				return false;
			}
			nlohmann::json state = nlohmann::json::parse(json);

			//Resolve the stable device identity against the current enumeration first.
			bool stateMatchesDevices = true;
			std::wstring gpuDevice = string_to_wstring_utf8(state.value("gpu_device", std::string()));
			if (!gpuDevice.empty())
			{
				auto gpuInfo = MainPage::AdlGetGpuByDeviceId(gpuDevice);
				if (gpuInfo.has_value())
				{
					adl_Gpu_AdapterIndex = gpuInfo.value().iAdapterIndex;
					adl_Gpu_RegistryPath = string_to_wstring(gpuInfo.value().strDriverPathExt);
					adl_Gpu_UniqueIdentifierHex = L"0x0001";
					adl_Gpu_DeviceIdentifier = gpuDevice;
				}
				else
				{
					stateMatchesDevices = false;
				}
			}
			std::wstring displayDevice = string_to_wstring_utf8(state.value("display_device", std::string()));
			if (!displayDevice.empty())
			{
				std::vector<ADLDisplayInfo> displayList = MainPage::AdlGetDisplayAll();
				bool displayFound = false;
				for (ADLDisplayInfo& displayInfo : displayList)
				{
					int adapterIndex = displayInfo.displayID.iDisplayLogicalAdapterIndex;
					int displayIndex = displayInfo.displayID.iDisplayLogicalIndex;
					if (MainPage::AdlxGetDisplayIdentifier(adapterIndex, displayIndex) == displayDevice)
					{
						adl_Display_AdapterIndex = adapterIndex;
						adl_Display_DisplayIndex = displayIndex;
						adl_Display_DeviceIdentifier = displayDevice;
						displayFound = true;
						break;
					}
				}
				if (!displayFound)
				{
					stateMatchesDevices = false;
				}
			}

			if (stateMatchesDevices)
			{
			if (state.contains("tuning")) { auto value = TuningFromJson(state["tuning"]); if (value.DeviceId.has_value() && !value.DeviceId.value().empty()) { tuningFanSettingsProfile = value; } }
			if (state.contains("display")) { auto value = DisplayFromJson(state["display"]); if (value.DeviceId.has_value() && !value.DeviceId.value().empty()) { displaySettingsProfile = value; } }
			if (state.contains("graphics")) { auto value = GraphicsFromJson(state["graphics"]); if (value.DeviceId.has_value() && !value.DeviceId.value().empty()) { graphicsSettingsProfile = value; } }
			if (state.contains("multimedia")) { auto value = MultimediaFromJson(state["multimedia"]); if (value.DeviceId.has_value() && !value.DeviceId.value().empty()) { multimediaSettingsProfile = value; } }
			}

			if (stateMatchesDevices && state.contains("snapshot"))
			{
				RadeonTunerUiSession::RestoreSnapshotJson(state["snapshot"].dump());
			}
			RestoreCacheFlags(state);
			RestoreGammaTargets(state);
			return true;
		}
		catch (...)
		{
			return false;
		}
	}

	void MetricsLoop()
	{
		using winrt::RadeonTuner::implementation::MainPage;
		while (true)
		{
			try
			{
				Sleep(1000);
				if (AppVariables::ApplicationExiting)
				{
					return;
				}

				//Metrics are UI-only: skip while the page is released or hidden.
				if (!AppVariables::App.GetIsWindowVisible())
				{
					continue;
				}

				//Collect metrics on this background thread; only text updates are dispatched
				//to the current UI session. The session ID is checked at dispatch time.
				uint64_t expectedSession = AppVariables::App.CurrentUiSessionId();
				MainPage::TuningMetrics_Update(expectedSession);
				MainPage::GraphicsStatus_Update(expectedSession);
				MainPage::AdlxLoopSelectedDeviceAccess();
			}
			catch (...) {}
		}
	}

	void ProfileLoop()
	{
		using winrt::RadeonTuner::implementation::MainPage;

		while (true)
		{
			bool batchActive = false;
			try
			{
				if (AppVariables::ApplicationExiting)
				{
					return;
				}

				//Delay outside the batch so PauseProfileLoop does not wait for a sleep.
				if (!AppVariables::LaunchLoopProfile)
				{
					Sleep(10000);
				}
				else
				{
					Sleep(2000);
					AppVariables::LaunchLoopProfile = false;
				}

				WaitForProfileLoopResume();
				if (AppVariables::ApplicationExiting)
				{
					return;
				}
				if (!BeginProfileBatch())
				{
					continue;
				}
				batchActive = true;

				std::vector<std::wstring> processExeRunning{};
				std::vector<AVProcess> processAll = Get_ProcessAll();
				processExeRunning.reserve(processAll.size());
				for (AVProcess& process : processAll)
				{
					std::wstring exeNameLower = wstring_to_lower(process.ExeNameW());
					processExeRunning.push_back(std::move(exeNameLower));
				}

				MainPage::AdlxLoopDisplayEyefinityAutomatic(processExeRunning);
				MainPage::AdlxLoopDisplayApplicationProfile(processExeRunning);
				MainPage::AdlxLoopTuningApplicationProfile(processExeRunning);
			}
			catch (...) {}
			if (batchActive)
			{
				EndProfileBatch();
			}
		}
	}
}
