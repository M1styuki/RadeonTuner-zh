#pragma once
#include "pch.h"
#include <winrt/Windows.UI.ViewManagement.h>
#include <winrt/Windows.UI.Xaml.Media.Animation.h>
#include <cmath>
#include <limits>

//Launchers
#include "SteamFuncLibraryFolders.h"
#include "SteamFuncAppInfo.h"
#include "SteamStructures.h"
#include "SteamFunctions.h"

#include "AppVariables.h"

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
			storyboard.Completed([this, generation, weakPage, weakTransform](IInspectable const&, IInspectable const&)
			{
				if (generation != pageTransitionGeneration) { return; }
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
				pageTransitionStoryboard = nullptr;
				pageTransitionTarget = nullptr;
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
			storyboard.Completed([this, key, storyboardKey, weakBorder, weakTransform](IInspectable const&, IInspectable const&)
			{
				auto current = comboPopupAnimations.find(key);
				if (current == comboPopupAnimations.end()) { return; }
				if (static_cast<void*>(winrt::get_abi(current->second)) != storyboardKey) { return; }
				comboPopupAnimations.erase(current);
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
					comboBox.Loaded([this](IInspectable const& sender, RoutedEventArgs const&)
					{
						auto comboBox = sender.as<ComboBox>();
						comboBox.ApplyTemplate();
						PatchComboBoxVisuals(comboBox);
					});
					comboBox.DropDownOpened([this](IInspectable const& sender, IInspectable const&)
					{
						auto comboBox = sender.as<ComboBox>();
						PatchComboBoxVisuals(comboBox);
						AnimateComboBoxPopup(comboBox, true);
					});
					comboBox.DropDownClosed([this](IInspectable const& sender, IInspectable const&)
					{
						AnimateComboBoxPopup(sender.as<ComboBox>(), false);
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
		try
		{
			InitializeValueHints();
			InitializeComboBoxVisuals();

#ifdef RADEONTUNER_UI_VALIDATION
			//UI validation build: only real XAML/theme/lists/defaults, never ADL, registry, autostart or hardware loops
			disable_saving = true;
			disable_saving_settings = true;
			disable_saving_customresolution = true;
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
				if (RadeonTunerUiValidation::AnimationCheckMode)
				{
					co_await AsyncTaskDelay(250, AppVariables::App.GetDispatcher());
					co_await RadeonTunerUiValidation::RunAnimationRegression(this);
				}
				co_return;
			}

			co_await RadeonTunerUiValidation::RunSynchronous(this);
			co_return;
#else
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

			//Sync graphics profile settings
			AdlAppSyncAll();

			//Set default registry values
			AdlSetDefaultSettings();

			//Check admin setttings
			SettingAdmin();

			//Load settings
			co_await SettingLoad();

			//Show or hide experimental settings
			ShowExperimentalSettings(true);

			//Select default indexes
			SelectDefaultIndexes();

			//Start adlx loop metrics
			std::thread threadLoopMetrics(&MainPage::AdlxLoopMetrics, this);
			threadLoopMetrics.detach();

			//Start adlx loop keep active
			std::thread threadLoopKeepActive(&MainPage::AdlxLoopProfile, this);
			threadLoopKeepActive.detach();

			//Check for application update
			std::optional<bool> setStartCheckUpdate = AppVariables::Settings.Load<bool>("StartCheckUpdate");
			if (setStartCheckUpdate.has_value() && setStartCheckUpdate.value())
			{
				UpdateCheckResult updateCheckResult = UpdateCheck(AppVariables::hInstance, "dumbie", "RadeonTuner");
				if (updateCheckResult.UpdateFound)
				{
					//Show update button
					button_Update_Launch().Visibility(Visibility::Visible);

					//Show notification
					AVDebugWriteLine(L"New application update found");
					ShowNotification(L"发现新的应用程序更新");
				}
			}

			//Update first launch setting
			AppVariables::Settings.Set("FirstLaunch", false);
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
		try
		{
			//Set notification text
			grid_Overlay_Notification().Visibility(Visibility::Visible);
			textblock_Notification_Text().Text(text);

			//Notification timer tick
			std::function<void(IInspectable const& sender, IInspectable const& e)> tickFunction = [&](auto, auto)
				{
					try
					{
						//Hide notification text
						grid_Overlay_Notification().Visibility(Visibility::Collapsed);

						//Stop notification timer
						TimerNotification.Stop();
					}
					catch (...) {}
				};

			//Start notification timer
			if (TimerNotification == NULL)
			{
				TimerNotification = winrt::Windows::UI::Xaml::DispatcherTimer();
				TimerNotification.Interval(TimeSpan(40000000));
				TimerNotification.Tick(tickFunction);
			}
			TimerNotification.Start();
		}
		catch (...) {}
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
