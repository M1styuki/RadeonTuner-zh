#pragma once
#include <Windows.h>
#include <chrono>
#include <cmath>
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
	using winrt::Windows::UI::Xaml::HorizontalAlignment;
	using winrt::Windows::UI::Xaml::TextAlignment;
	using winrt::Windows::UI::Xaml::GridUnitType;
	using winrt::Windows::UI::Xaml::Media::VisualTreeHelper;
	using winrt::Windows::UI::Xaml::Controls::ComboBox;
	using winrt::Windows::UI::Xaml::Controls::Grid;
	using winrt::Windows::UI::Xaml::Controls::Image;
	using winrt::Windows::UI::Xaml::Controls::Orientation;
	using winrt::Windows::UI::Xaml::Controls::Slider;
	using winrt::Windows::UI::Xaml::Controls::StackPanel;
	using winrt::Windows::UI::Xaml::Controls::TextBlock;
	using winrt::Windows::UI::Xaml::Controls::TextBox;
	using winrt::Windows::UI::Xaml::Controls::ToggleSwitch;
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
		int fanInputsValidated = 0;
		int fanCurveInputsValidated = 0;
		int fanRangeInputsValidated = 0;
		int fanRangeSlidersValidated = 0;
		int hiddenFanAdapters = 0;
		auto headingStyle = winrt::Windows::UI::Xaml::Application::Current().Resources().Lookup(winrt::box_value(L"ApplicationSectionHeader")).as<winrt::Windows::UI::Xaml::Style>();
		std::vector<double> sliderStarts;
		std::vector<double> sliderEnds;
		std::vector<double> sliderTextboxEdges;
		std::vector<double> controlRightEdges;
		std::vector<double> rangeSliderWidths;
		std::vector<double> rangeGroupWidths;

		auto nearlyEqual = [](double left, double right)
		{
			return std::abs(left - right) <= 0.75;
		};

		auto findChildAtColumn = [](Grid const& grid, int column) -> FrameworkElement
		{
			for (uint32_t index = 0; index < grid.Children().Size(); index++)
			{
				auto child = grid.Children().GetAt(index).try_as<FrameworkElement>();
				if (child && Grid::GetColumn(child) == column) { return child; }
			}
			return FrameworkElement{ nullptr };
		};

		auto findChildAtCell = [](Grid const& grid, int column, int row) -> FrameworkElement
		{
			for (uint32_t index = 0; index < grid.Children().Size(); index++)
			{
				auto child = grid.Children().GetAt(index).try_as<FrameworkElement>();
				if (child && Grid::GetColumn(child) == column && Grid::GetRow(child) == row) { return child; }
			}
			return FrameworkElement{ nullptr };
		};

		auto tryGetRowContext = [&](FrameworkElement const& control, Grid& outer, StackPanel& left) -> bool
		{
			auto parent = VisualTreeHelper::GetParent(control).try_as<Grid>();
			if (!parent || parent.ColumnDefinitions().Size() != 2) { return false; }
			if (parent.ColumnDefinitions().GetAt(0).Width().Value != 300) { return false; }
			if (Grid::GetColumn(control) != 1) { return false; }
			auto leftChild = findChildAtColumn(parent, 0).try_as<StackPanel>();
			if (!leftChild) { return false; }
			outer = parent;
			left = leftChild;
			return true;
		};

		auto validateLeftDescription = [&](StackPanel const& left)
		{
			bool labelFound = false;
			int helpIndex = -1;
			int warningIndex = -1;
			for (uint32_t index = 0; index < left.Children().Size(); index++)
			{
				auto line = left.Children().GetAt(index).try_as<StackPanel>();
				if (!line || line.Orientation() != Orientation::Horizontal) { continue; }
				int iconIndex = 0;
				for (uint32_t childIndex = 0; childIndex < line.Children().Size(); childIndex++)
				{
					auto child = line.Children().GetAt(childIndex);
					auto text = child.try_as<TextBlock>();
					if (text && !text.Text().empty()) { labelFound = true; }
					auto icons = child.try_as<StackPanel>();
					if (!icons) { continue; }
					for (uint32_t iconChildIndex = 0; iconChildIndex < icons.Children().Size(); iconChildIndex++)
					{
						auto icon = icons.Children().GetAt(iconChildIndex).try_as<Image>();
						if (!icon) { continue; }
						std::wstring tooltip = TooltipText(icon);
						if (tooltip.empty()) { continue; }
						if (tooltip[0] == L'*')
						{
							if (warningIndex < 0) { warningIndex = iconIndex; }
						}
						else if (helpIndex < 0)
						{
							helpIndex = iconIndex;
						}
						iconIndex++;
					}
				}
			}
			if (!labelFound)
			{
				throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Setting row has no leading label on page: " + pageName));
			}
			if (helpIndex >= 0 && warningIndex >= 0 && warningIndex < helpIndex)
			{
				throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Help icon does not precede warning icon on page: " + pageName));
			}
		};

		auto rightEdge = [&](FrameworkElement const& element)
		{
			auto point = element.TransformToVisual(pageRoot).TransformPoint(winrt::Windows::Foundation::Point(element.ActualWidth(), 0));
			return point.X;
		};

		auto validateRightEdge = [&](FrameworkElement const& element)
		{
			if (!IsEffectivelyVisible(element, pageRoot) || element.ActualWidth() < 1) { return; }
			controlRightEdges.push_back(rightEdge(element));
		};

		std::function<void(DependencyObject const&)> inspect = [&](DependencyObject const& node)
		{
			auto textBlock = node.try_as<TextBlock>();
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

			auto toggle = node.try_as<ToggleSwitch>();
			if (toggle && IsEffectivelyVisible(toggle, pageRoot))
			{
				Grid outer{ nullptr };
				StackPanel left{ nullptr };
				if (tryGetRowContext(toggle, outer, left))
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
					if (toggle.HorizontalAlignment() != HorizontalAlignment::Right || std::abs(toggle.Width() - 44) > 0.1 || toggle.MinWidth() > 0.1)
					{
						throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Toggle is not a compact 44 DIP right-aligned control on page: " + pageName));
					}
					validateLeftDescription(left);
					validateRightEdge(toggle);
					validatedRows++;
				}
			}

			auto combo = node.try_as<ComboBox>();
			if (combo && IsEffectivelyVisible(combo, pageRoot))
			{
				Grid outer{ nullptr };
				StackPanel left{ nullptr };
				if (tryGetRowContext(combo, outer, left))
				{
					if (std::abs(combo.Width() - 160) > 0.1 || std::abs(combo.Height() - 32) > 0.1 || combo.HorizontalAlignment() != HorizontalAlignment::Right)
					{
						throw winrt::hresult_error(E_FAIL, winrt::hstring(L"ComboBox is not a 160x32 right-aligned control on page: " + pageName));
					}
					validateLeftDescription(left);
					validateRightEdge(combo);
					validatedRows++;
				}
			}

			auto slider = node.try_as<Slider>();
			if (slider)
			{
				std::wstring sliderName(slider.Name().c_str());
				if (sliderName.rfind(L"slider_Fan_", 0) == 0 &&
					sliderName != L"slider_Fan_Speed_Min" &&
					sliderName != L"slider_Fan_Speed_Max" &&
					!IsEffectivelyVisible(slider, pageRoot))
				{
					hiddenFanAdapters++;
				}
			}
			if (slider && IsEffectivelyVisible(slider, pageRoot))
			{
				std::wstring sliderName(slider.Name().c_str());
				bool isRangeSlider = sliderName == L"slider_Fan_Speed_Min" || sliderName == L"slider_Fan_Speed_Max";
				if (isRangeSlider)
				{
					auto group = VisualTreeHelper::GetParent(slider).try_as<Grid>();
					auto rangeGrid = group ? VisualTreeHelper::GetParent(group).try_as<Grid>() : Grid{ nullptr };
					if (!group || !rangeGrid ||
						group.ColumnDefinitions().Size() != 4 ||
						Grid::GetColumn(slider) != 1 ||
						std::wstring(rangeGrid.Name().c_str()) != L"grid_Fan_Range_Inputs" ||
						rangeGrid.ColumnDefinitions().Size() != 2 ||
						std::abs(rangeGrid.ColumnSpacing() - 16) > 0.1)
					{
						throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Fan range slider is not in the shared two-star layout on page: " + pageName));
					}
					auto firstWidth = group.ColumnDefinitions().GetAt(0).Width();
					auto secondWidth = group.ColumnDefinitions().GetAt(1).Width();
					auto thirdWidth = group.ColumnDefinitions().GetAt(2).Width();
					auto fourthWidth = group.ColumnDefinitions().GetAt(3).Width();
					if (firstWidth.GridUnitType != GridUnitType::Auto ||
						secondWidth.GridUnitType != GridUnitType::Star ||
						std::abs(thirdWidth.Value - 8) > 0.1 ||
						std::abs(fourthWidth.Value - 160) > 0.1)
					{
						throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Fan range group does not use Auto/star/8/160 columns on page: " + pageName));
					}
					auto label = findChildAtCell(group, 0, 0).try_as<TextBlock>();
					auto numberBox = findChildAtCell(group, 3, 0).try_as<TextBox>();
					std::wstring expectedLabel = sliderName == L"slider_Fan_Speed_Min" ? L"最低转速（%）" : L"最高转速（%）";
					std::wstring expectedNumberBox = sliderName == L"slider_Fan_Speed_Min" ? L"textbox_Fan_Speed_Min" : L"textbox_Fan_Speed_Max";
					std::wstring labelText(label ? label.Text().c_str() : L"");
					std::wstring numberBoxName(numberBox ? numberBox.Name().c_str() : L"");
					if (!label || labelText != expectedLabel ||
						!numberBox || numberBoxName != expectedNumberBox ||
						std::abs(numberBox.Width() - 160) > 0.1 ||
						std::abs(numberBox.Height() - 32) > 0.1 ||
						numberBox.HorizontalAlignment() != HorizontalAlignment::Right ||
						numberBox.TextAlignment() != TextAlignment::Center)
					{
						throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Fan range label or number box layout is not usable on page: " + pageName));
					}
					if (slider.HorizontalAlignment() != HorizontalAlignment::Stretch ||
						slider.MinWidth() < 56 ||
						slider.ActualWidth() < 56)
					{
						throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Fan range slider is not a stretching >=56 DIP control on page: " + pageName));
					}
					if (slider.ActualWidth() > 1)
					{
						auto finish = slider.TransformToVisual(pageRoot).TransformPoint(winrt::Windows::Foundation::Point(slider.ActualWidth(), 0));
						auto boxStart = numberBox.TransformToVisual(pageRoot).TransformPoint(winrt::Windows::Foundation::Point(0, 0));
						if (!nearlyEqual(finish.X + 8, boxStart.X))
						{
							throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Fan range slider does not end 8 DIP before its number box on page: " + pageName));
						}
						rangeSliderWidths.push_back(slider.ActualWidth());
						rangeGroupWidths.push_back(group.ActualWidth());
						if (sliderName == L"slider_Fan_Speed_Max")
						{
							validateRightEdge(numberBox);
						}
					}
					fanRangeSlidersValidated++;
					validatedRows++;
				}
				else
				{
					auto sliderGrid = VisualTreeHelper::GetParent(slider).try_as<Grid>();
					if (sliderGrid && sliderGrid.ColumnDefinitions().Size() == 3)
					{
						auto outer = VisualTreeHelper::GetParent(sliderGrid).try_as<Grid>();
						if (outer && outer.ColumnDefinitions().Size() == 2 && outer.ColumnDefinitions().GetAt(0).Width().Value == 300)
						{
							auto left = findChildAtColumn(outer, 0).try_as<StackPanel>();
							if (!left)
							{
								throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Slider row has no left description stack on page: " + pageName));
							}
							auto firstWidth = sliderGrid.ColumnDefinitions().GetAt(0).Width();
							auto secondWidth = sliderGrid.ColumnDefinitions().GetAt(1).Width();
							auto thirdWidth = sliderGrid.ColumnDefinitions().GetAt(2).Width();
							if (firstWidth.GridUnitType != GridUnitType::Star || std::abs(secondWidth.Value - 12) > 0.1 || std::abs(thirdWidth.Value - 160) > 0.1)
							{
								throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Slider row does not use the shared star/12/160 columns on page: " + pageName));
							}
							auto numberBox = findChildAtColumn(sliderGrid, 2).try_as<TextBox>();
							if (!numberBox ||
								slider.HorizontalAlignment() != HorizontalAlignment::Stretch ||
								slider.MinWidth() < 120 ||
								std::abs(numberBox.Width() - 160) > 0.1 ||
								std::abs(numberBox.Height() - 32) > 0.1 ||
								numberBox.HorizontalAlignment() != HorizontalAlignment::Right ||
								numberBox.TextAlignment() != TextAlignment::Center)
							{
								throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Slider/number box layout is not usable on page: " + pageName));
							}
							if (IsEffectivelyVisible(numberBox, pageRoot) && slider.ActualWidth() > 1)
							{
								auto start = slider.TransformToVisual(pageRoot).TransformPoint(winrt::Windows::Foundation::Point(0, 0));
								auto finish = slider.TransformToVisual(pageRoot).TransformPoint(winrt::Windows::Foundation::Point(slider.ActualWidth(), 0));
								double textboxEdge = rightEdge(numberBox);
								sliderStarts.push_back(start.X);
								sliderEnds.push_back(finish.X);
								sliderTextboxEdges.push_back(textboxEdge);
								validateRightEdge(numberBox);
							}
							validateLeftDescription(left);
							validatedRows++;
						}
					}
				}
			}

			auto textBox = node.try_as<TextBox>();
			if (textBox && IsEffectivelyVisible(textBox, pageRoot))
			{
				std::wstring textBoxName(textBox.Name().c_str());
				if (textBoxName == L"textbox_Fan_Speed_Min" || textBoxName == L"textbox_Fan_Speed_Max")
				{
					auto group = VisualTreeHelper::GetParent(textBox).try_as<Grid>();
					if (!group ||
						group.ColumnDefinitions().Size() != 4 ||
						Grid::GetColumn(textBox) != 3 ||
						std::abs(textBox.Width() - 160) > 0.1 ||
						std::abs(textBox.Height() - 32) > 0.1 ||
						textBox.HorizontalAlignment() != HorizontalAlignment::Right ||
						textBox.TextAlignment() != TextAlignment::Center)
					{
						throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Range fan number box layout is not usable on page: " + pageName));
					}
					if (textBoxName == L"textbox_Fan_Speed_Max")
					{
						validateRightEdge(textBox);
					}
					fanRangeInputsValidated++;
					fanInputsValidated++;
					validatedRows++;
				}
				else if (textBoxName.rfind(L"textbox_Fan_", 0) == 0)
				{
					auto pairGrid = VisualTreeHelper::GetParent(textBox).try_as<Grid>();
					int column = pairGrid ? Grid::GetColumn(textBox) : -1;
					if (!pairGrid || pairGrid.ColumnDefinitions().Size() != 5 ||
						(column != 1 && column != 4) ||
						pairGrid.HorizontalAlignment() != HorizontalAlignment::Right ||
						std::abs(textBox.Width() - 160) > 0.1 ||
						std::abs(textBox.Height() - 32) > 0.1 ||
						textBox.HorizontalAlignment() != HorizontalAlignment::Right ||
						textBox.TextAlignment() != TextAlignment::Center)
					{
						throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Paired fan number box layout is not usable on page: " + pageName));
					}
					if (column == 4)
					{
						validateRightEdge(textBox);
					}
					fanCurveInputsValidated++;
					fanInputsValidated++;
					validatedRows++;
				}
			}

			int childCount = VisualTreeHelper::GetChildrenCount(node);
			for (int index = 0; index < childCount; index++)
			{
				inspect(VisualTreeHelper::GetChild(node, index));
			}
		};
		inspect(pageRoot);

		if (pageName == L"Fans")
		{
			const bool curveMode = fanCurveInputsValidated == 10 && fanRangeInputsValidated == 0;
			const bool rangeMode = fanCurveInputsValidated == 0 && fanRangeInputsValidated == 2;
			const bool emptyMode = fanCurveInputsValidated == 0 && fanRangeInputsValidated == 0;
			if (!curveMode && !rangeMode && !emptyMode)
			{
				throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Fan mode sections are not mutually exclusive on page: " + pageName));
			}
			if (fanInputsValidated != fanCurveInputsValidated + fanRangeInputsValidated)
			{
				throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Fan number box count does not match the visible mode on page: " + pageName));
			}
			if (fanRangeSlidersValidated != fanRangeInputsValidated)
			{
				throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Fan range sliders do not match the visible range mode on page: " + pageName));
			}
			if (hiddenFanAdapters != 10)
			{
				throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Expected 10 hidden fan curve adapter sliders, found " + std::to_wstring(hiddenFanAdapters)));
			}
			if (rangeSliderWidths.size() == 2 && !nearlyEqual(rangeSliderWidths[0], rangeSliderWidths[1]))
			{
				throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Fan range sliders do not have equal lengths on page: " + pageName));
			}
			if (rangeGroupWidths.size() == 2 && !nearlyEqual(rangeGroupWidths[0], rangeGroupWidths[1]))
			{
				throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Fan range groups do not have equal widths on page: " + pageName));
			}
		}

		for (size_t index = 1; index < sliderStarts.size(); index++)
		{
			if (!nearlyEqual(sliderStarts[index], sliderStarts[0]) || !nearlyEqual(sliderEnds[index], sliderEnds[0]))
			{
				throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Slider positions differ on page: " + pageName));
			}
		}
		for (size_t index = 0; index < sliderStarts.size(); index++)
		{
			if (!nearlyEqual(sliderEnds[index] + 12, sliderTextboxEdges[index]))
			{
				throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Slider does not end 12 DIP before its number box on page: " + pageName));
			}
		}
		for (size_t index = 1; index < controlRightEdges.size(); index++)
		{
			if (!nearlyEqual(controlRightEdges[index], controlRightEdges[0]))
			{
				throw winrt::hresult_error(E_FAIL, winrt::hstring(L"Right control edges differ on page: " + pageName));
			}
		}
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

		if (PreviewMode)
		{
			//Preview only: replace native XAML defaults with explicit illustrative values and ranges.
			auto setPreviewSlider = [](auto const& slider, double minimum, double maximum, double value, double step)
			{
				slider.Minimum(minimum);
				slider.Maximum(maximum);
				slider.StepFrequency(step);
				slider.SmallChange(step);
				slider.LargeChange(step * 10.0);
				slider.TickFrequency(0);
				slider.Value(value);
			};

			page->textblock_GpuSelect().Text(L"示例显卡（预览数据）");
			page->textblock_DisplaySelect().Text(L"示例显示器（预览数据）");

			setPreviewSlider(page->slider_RadeonChill_Min(), 30, 240, 60, 1);
			setPreviewSlider(page->slider_RadeonChill_Max(), 30, 240, 120, 1);
			setPreviewSlider(page->slider_Frtc_FrameRateTarget(), 30, 300, 144, 1);
			setPreviewSlider(page->slider_RadeonBoost_MinResolution(), 50, 100, 80, 1);
			setPreviewSlider(page->slider_RadeonImageSharpening1_Sharpening(), 0, 100, 50, 1);
			setPreviewSlider(page->slider_RadeonImageSharpening2_Sharpening(), 0, 100, 50, 1);
			setPreviewSlider(page->slider_Core_Min(), 500, 3000, 1200, 10);
			setPreviewSlider(page->slider_Core_Max(), -500, 500, 0, 5);
			setPreviewSlider(page->slider_Memory_Max(), -500, 500, 0, 5);
			setPreviewSlider(page->slider_Power_Limit(), -20, 20, 0, 1);
			setPreviewSlider(page->slider_Power_Voltage(), -100, 100, 0, 5);
			setPreviewSlider(page->slider_Power_TDC(), -20, 20, 0, 1);
			setPreviewSlider(page->slider_Fan_Speed_Min(), 20, 100, 30, 1);
			setPreviewSlider(page->slider_Fan_Speed_Max(), 40, 100, 80, 1);
			setPreviewSlider(page->slider_Fan_Speed_0(), 0, 100, 30, 1);
			setPreviewSlider(page->slider_Fan_Temp_0(), 20, 100, 30, 1);
			setPreviewSlider(page->slider_Fan_Speed_1(), 0, 100, 40, 1);
			setPreviewSlider(page->slider_Fan_Temp_1(), 20, 100, 45, 1);
			setPreviewSlider(page->slider_Fan_Speed_2(), 0, 100, 50, 1);
			setPreviewSlider(page->slider_Fan_Temp_2(), 20, 100, 60, 1);
			setPreviewSlider(page->slider_Fan_Speed_3(), 0, 100, 65, 1);
			setPreviewSlider(page->slider_Fan_Temp_3(), 20, 100, 75, 1);
			setPreviewSlider(page->slider_Fan_Speed_4(), 0, 100, 80, 1);
			setPreviewSlider(page->slider_Fan_Temp_4(), 20, 100, 90, 1);
			setPreviewSlider(page->slider_Display_GammaBase(), 0.5, 2.5, 1.0, 0.01);
			setPreviewSlider(page->slider_Display_GammaRed(), 0.5, 2.0, 1.0, 0.01);
			setPreviewSlider(page->slider_Display_GammaGreen(), 0.5, 2.0, 1.0, 0.01);
			setPreviewSlider(page->slider_Display_GammaBlue(), 0.5, 2.0, 1.0, 0.01);
			setPreviewSlider(page->slider_Display_Brightness(), -100, 100, 0, 1);
			setPreviewSlider(page->slider_Display_Contrast(), -100, 100, 0, 1);
			setPreviewSlider(page->slider_Display_Saturation(), 0, 200, 100, 1);
			setPreviewSlider(page->slider_Display_Hue(), -180, 180, 0, 1);
			setPreviewSlider(page->slider_Display_ColorTemperature_Kelvin(), 4000, 10000, 6500, 100);
			setPreviewSlider(page->slider_Display_Protanopia(), 0, 100, 50, 1);
			setPreviewSlider(page->slider_Display_Deuteranopia(), 0, 100, 50, 1);
			setPreviewSlider(page->slider_Display_Tritanopia(), 0, 100, 50, 1);
			setPreviewSlider(page->slider_Video_Sharpening(), 0, 100, 50, 1);
			setPreviewSlider(page->slider_Video_Brightness(), -100, 100, 0, 1);
			setPreviewSlider(page->slider_Eyefinity_Rows(), 1, 6, 2, 1);
			setPreviewSlider(page->slider_Eyefinity_Columns(), 1, 6, 2, 1);

			//ECC availability is hardware-dependent; the preview shows an explicit unavailable state instead of a selected enum.
			page->combobox_Memory_Ecc().IsEnabled(false);
			page->combobox_Memory_Ecc().SelectedIndex(-1);
			page->combobox_Memory_Ecc().PlaceholderText(L"不可用");
			page->textblock_Memory_Ecc_Value().Text(L"不可用");

			//Representative enabled parent states so every dependent editor is visible in the isolated preview.
			page->toggleswitch_FrameGenEnabled().IsOn(true);
			page->toggleswitch_RadeonChill().IsOn(true);
			page->toggleswitch_Frtc().IsOn(true);
			page->combobox_RadeonBoost().SelectedIndex(1);
			page->toggleswitch_RadeonImageSharpening1().IsOn(true);
			page->toggleswitch_RadeonImageSharpening2().IsOn(true);
			page->toggleswitch_RadeonImageSharpening2_Desktop().IsOn(true);
			page->toggleswitch_AntiAliasingOverride().IsOn(true);
			page->combobox_Tessellation_Mode().SelectedIndex(2);
			page->toggleswitch_Display_GammaControl().IsOn(true);
			page->toggleswitch_Display_ColorTemperature_Control().IsOn(true);
			page->toggleswitch_Display_CVDC_Control().IsOn(true);
			if (page->combobox_FrameGenAlgorithm().Items().Size() > 0) { page->combobox_FrameGenAlgorithm().SelectedIndex(0); }
			if (page->combobox_AntiAliasingMethod().Items().Size() > 0) { page->combobox_AntiAliasingMethod().SelectedIndex(0); }
			if (page->combobox_AntiAliasingLevel().Items().Size() > 0) { page->combobox_AntiAliasingLevel().SelectedIndex(0); }
			if (page->combobox_Tessellation_Level().Items().Size() > 0) { page->combobox_Tessellation_Level().SelectedIndex(0); }
			page->UpdateDependentSettingsInterface(false);

			//Preview the target curve and the adapter-backed number boxes with deterministic samples
			page->ValidateFanSettings();
			page->UpdateFanGraphProfile();
			page->UpdateFanInputFields();

			//Preview the real mutually exclusive fan mode UI without ADL or a mode profile write.
			page->combobox_FanControl_Mode().SelectedIndex((int)FanControlModes::CustomFanCurve);
			tuningFanSettingsProfile.FanControlMode.Current = (int)FanControlModes::CustomFanCurve;
			page->UpdateFanControlModeInterface(tuningFanSettingsProfile, false);
		}
		else
		{
			//Keep the layout run deterministic without touching ADL: show one official mode section.
			page->combobox_FanControl_Mode().SelectedIndex((int)FanControlModes::CustomFanCurve);
			tuningFanSettingsProfile.FanControlMode.Current = (int)FanControlModes::CustomFanCurve;
			page->UpdateFanControlModeInterface(tuningFanSettingsProfile, false);
			page->UpdateDependentSettingsInterface(false);
		}

		page->UpdateLayout();
	}

	inline void SeedPreviewMetrics(MainPage* page)
	{
		//Test-only visual samples; the metric text blocks contain values only, units are static XAML.
		page->textblock_Current_Gpu_Activity().Text(L"36");
		page->textblock_Current_Core_Speed().Text(L"2410");
		page->textblock_Current_Memory_Usage().Text(L"2.8");
		page->textblock_Current_Memory_Speed().Text(L"2500");
		page->textblock_Current_Memory_Activity().Text(L"12");
		page->textblock_Current_Power_Wattage().Text(L"92");
		page->textblock_Current_Power_Voltage().Text(L"950");
		page->textblock_Current_Temp_Core().Text(L"52");
		page->textblock_Current_Temp_Hotspot().Text(L"63");
		page->textblock_Current_Temp_Memory().Text(L"58");
		page->textblock_Current_Fan_Speed_RPM().Text(L"1050");
		page->textblock_Current_Fan_Speed_Percentage().Text(L"30");
		page->textblock_Current_Temp_Intake().Text(L"28");
	}

	inline void SendValidationMouseClick(int screenX, int screenY)
	{
		HWND testWindow = AppVariables::App.GetWindowHandle();
		POINT point{screenX, screenY};
		Require(GetForegroundWindow() == testWindow && GetAncestor(WindowFromPoint(point), GA_ROOT) == testWindow, L"Isolated input target changed; pointer check stopped.");
		SetCursorPos(screenX, screenY);
		INPUT inputs[2] = {};
		inputs[0].type = INPUT_MOUSE;
		inputs[0].mi.dwFlags = MOUSEEVENTF_LEFTDOWN;
		inputs[1].type = INPUT_MOUSE;
		inputs[1].mi.dwFlags = MOUSEEVENTF_LEFTUP;
		Require(SendInput(2, inputs, sizeof(INPUT)) == 2, L"Isolated pointer input was not delivered.");
	}

	inline bool WindowBelongsTo(HWND window, HWND root)
	{
		return window != NULL && GetAncestor(window, GA_ROOT) == root;
	}

	inline winrt::Windows::Foundation::IAsyncAction RunAnimationRegression(MainPage* page)
	{
		nlohmann::json report;
		report["started_utc"] = UtcNowString();
		bool savedDisableSaving = disable_saving;
		bool savedDisableSavingSettings = disable_saving_settings;
		report["guards_before"] = { {"disable_saving", savedDisableSaving}, {"disable_saving_settings", savedDisableSavingSettings} };
#ifdef RADEONTUNER_UI_VALIDATION
		report["adl_blocked_before_load"] = true;
#else
		report["adl_blocked_before_load"] = false;
#endif
		try
		{
			using namespace winrt::Windows::UI::Xaml::Media;
			using namespace winrt::Windows::UI::Xaml::Media::Animation;
			winrt::Windows::UI::ViewManagement::UISettings uiSettings;
			bool animated = uiSettings.AnimationsEnabled();
			report["animations_enabled"] = animated;
			ShowWindow(AppVariables::App.GetWindowHandle(), SW_SHOWNOACTIVATE);
			if (AnimationCheckStay)
			{
				UINT dpi = GetDpiForWindow(AppVariables::App.GetWindowHandle());
				SetWindowPos(AppVariables::App.GetWindowHandle(), HWND_TOP, 48, 48, MulDiv(1040, dpi, 96), MulDiv(900, dpi, 96), SWP_NOACTIVATE);
			}
			Require(disable_saving && disable_saving_settings, L"UI test saving guards must be enabled.");
			page->listview_Main().SelectedIndex(3);
			page->UpdateLayout();
			co_await AsyncTaskDelay(180, AppVariables::App.GetDispatcher());
			auto toggle = page->toggleswitch_Display_GammaControl();
			report["loaded_switches"] = page->UiValidationToggleCount();
			auto rows = page->stackpanel_Display_Gamma_Dependent();
			winrt::Windows::UI::Xaml::BringIntoViewOptions bringOptions;
			bringOptions.VerticalAlignmentRatio(0);
			toggle.StartBringIntoView(bringOptions);
			co_await AsyncTaskDelay(250, AppVariables::App.GetDispatcher());
			std::function<FrameworkElement(DependencyObject const&)> findKnob = [&](DependencyObject const& node) -> FrameworkElement
			{
				if (auto element = node.try_as<FrameworkElement>()) { if (element.Name() == L"SwitchVisualKnob") { return element; } }
				for (int i = 0; i < VisualTreeHelper::GetChildrenCount(node); ++i) { auto found = findKnob(VisualTreeHelper::GetChild(node, i)); if (found) { return found; } }
				return nullptr;
			};
			auto knob = findKnob(toggle);
			Require(knob != nullptr, L"Switch knob was not realized.");
			auto position = knob.RenderTransform().as<TranslateTransform>();
			auto checkEnd = [&](bool on) -> winrt::Windows::Foundation::IAsyncAction
			{
				co_await AsyncTaskDelay(250, AppVariables::App.GetDispatcher());
				page->UpdateLayout();
				report["endpoints"].push_back({ {"expected_on", on}, {"actual_on", toggle.IsOn()}, {"x", position.X()}, {"row_height", rows.ActualHeight()}, {"running", page->UiValidationAnimationCount()}, {"loaded", page->UiValidationToggleCount()}, {"knob_clocks", page->UiValidationKnobAnimationCount()} });
				for (auto const& group : winrt::Windows::UI::Xaml::VisualStateManager::GetVisualStateGroups(VisualTreeHelper::GetChild(toggle, 0).as<FrameworkElement>()))
				{
					if (group.Name() == L"ToggleStates") { report["toggle_state"] = group.CurrentState() ? winrt::to_string(group.CurrentState().Name()) : "none"; for (auto const& state : group.States()) { report[winrt::to_string(state.Name()) + "_setters"] = state.Setters().Size(); } }
				}
				Require(toggle.IsOn() == on, L"Latest switch state was lost.");
				Require(std::abs(position.X() - (on ? 20.0 : 0.0)) < 0.1, L"Switch ended at the wrong position.");
				Require(rows.Visibility() == (on ? Visibility::Visible : Visibility::Collapsed), L"Dependent rows ended in the wrong state.");
				Require(std::isnan(rows.Height()) && !rows.Clip(), L"Completed row retained animation geometry.");
				Require(page->UiValidationAnimationCount() == 0, L"Completed animation clock was retained.");
			};
			toggle.IsOn(false);
			page->UpdateDependentSettingsInterface(false);
			co_await checkEnd(false);
			report["reversals"] = nlohmann::json::array();
			for (bool finalState : { true, false })
			{
				for (int i = 0; i < 5; ++i)
				{
					bool next = i == 4 ? finalState : !toggle.IsOn();
					bool changed = next != toggle.IsOn();
					double before = std::isfinite(rows.Height()) ? rows.Height() : rows.ActualHeight();
					bool inFlight = page->UiValidationAnimationFrom(rows).has_value();
					toggle.IsOn(next);
					auto activeFrom = page->UiValidationAnimationFrom(rows);
					if (animated && changed && inFlight && before > 0.5 && activeFrom.has_value())
					{
						double from = activeFrom.value();
						Require(std::abs(from - before) < 0.1, L"Reversal jumped away from the displayed height.");
						report["reversals"].push_back({ {"before", before}, {"from", from}, {"target", next} });
					}
					co_await AsyncTaskDelay(30, AppVariables::App.GetDispatcher());
				}
				co_await checkEnd(finalState);
			}
			for (bool on : { true, false })
			{
				double fullHeight = rows.ActualHeight();
				ULONGLONG sampleStarted = GetTickCount64();
				toggle.IsOn(on);
				co_await AsyncTaskDelay(45, AppVariables::App.GetDispatcher());
				double knobX = position.X();
				ULONGLONG sampleElapsed = GetTickCount64() - sampleStarted;
				report["slow_samples"].push_back({ {"on", on}, {"height", rows.Height()}, {"actual", rows.ActualHeight()}, {"full", fullHeight}, {"opacity", rows.Opacity()}, {"knob_x", knobX}, {"elapsed_ms", sampleElapsed} });
				if (animated && sampleElapsed < 85) { Require(knobX > 0.1 && knobX < 19.9, L"Switch skipped the intermediate ON/OFF movement."); }
				if (animated) { Require(rows.Visibility() == Visibility::Visible && rows.Height() > 0.5, L"Open/close animation disappeared immediately."); }
				if (animated && !on) { Require(rows.Height() < fullHeight, L"OFF animation did not shrink the row."); }
				co_await checkEnd(on);
			}
			report["slow_on_off"] = "passed";
			page->listview_Main().SelectedIndex(0);
			co_await AsyncTaskDelay(180, AppVariables::App.GetDispatcher());
			auto plain = page->toggleswitch_KeepActive();
			auto plainPosition = findKnob(plain).RenderTransform().as<TranslateTransform>();
			for (bool on : { true, false, true, false }) { plain.IsOn(on); co_await AsyncTaskDelay(30, AppVariables::App.GetDispatcher()); }
			co_await AsyncTaskDelay(150, AppVariables::App.GetDispatcher());
			Require(!plain.IsOn() && std::abs(plainPosition.X()) < 0.1, L"Ordinary switch rapid OFF ended incorrectly.");
			plain.IsOn(true);
			co_await AsyncTaskDelay(150, AppVariables::App.GetDispatcher());
			Require(std::abs(plainPosition.X() - 20) < 0.1, L"Ordinary switch ON ended incorrectly.");
			plain.IsOn(false);
			co_await AsyncTaskDelay(150, AppVariables::App.GetDispatcher());
			Require(std::abs(plainPosition.X()) < 0.1, L"Ordinary switch OFF ended incorrectly.");
			report["ordinary_switch_rapid_and_slow"] = "passed";
			//Actual pointer path on the native Thumb: rapid clicks must reverse IsOn each time.
			HWND pointerWindow = AppVariables::App.GetWindowHandle();
			ShowWindow(pointerWindow, SW_SHOW);
			SetForegroundWindow(pointerWindow);
			SetActiveWindow(pointerWindow);
			plain.StartBringIntoView(bringOptions);
			co_await AsyncTaskDelay(240, AppVariables::App.GetDispatcher());
			auto pointerCenter = plain.TransformToVisual(nullptr).TransformPoint(winrt::Windows::Foundation::Point{ (float)(plain.ActualWidth() / 2.0), (float)(plain.ActualHeight() / 2.0) });
			UINT pointerDpi = GetDpiForWindow(pointerWindow);
			POINT clientOrigin{ 0, 0 };
			ClientToScreen(pointerWindow, &clientOrigin);
			int pointerScreenX = clientOrigin.x + MulDiv((int)pointerCenter.X, pointerDpi, 96);
			int pointerScreenY = clientOrigin.y + MulDiv((int)pointerCenter.Y, pointerDpi, 96);
			POINT pointerPoint{ pointerScreenX, pointerScreenY };
			HWND pointerHit = WindowFromPoint(pointerPoint);
			report["pointer_hit_test"] = {
				{ "screen_x", pointerScreenX },
				{ "screen_y", pointerScreenY },
				{ "foreground", GetForegroundWindow() == pointerWindow },
				{ "hit_matches_window", WindowBelongsTo(pointerHit, pointerWindow) },
				{ "click_width", plain.ActualWidth() },
				{ "click_height", plain.ActualHeight() }
			};
			Require(GetForegroundWindow() == pointerWindow, L"Pointer test window is not foreground.");
			Require(WindowBelongsTo(pointerHit, pointerWindow), L"Pointer test point does not hit the isolated window.");

			plain.IsOn(false);
			co_await AsyncTaskDelay(180, AppVariables::App.GetDispatcher());
			int pointerPressedCount = 0;
			int pointerReleasedCount = 0;
			int tappedCount = 0;
			int doubleTappedCount = 0;
			auto pressedToken = plain.PointerPressed([&](auto const&, auto const&) { ++pointerPressedCount; });
			auto releasedToken = plain.PointerReleased([&](auto const&, auto const&) { ++pointerReleasedCount; });
			auto tappedToken = plain.Tapped([&](auto const&, auto const&) { ++tappedCount; });
			auto doubleToken = plain.DoubleTapped([&](auto const&, auto const&) { ++doubleTappedCount; });
			int toggleCount = 0;
			auto toggledToken = plain.Toggled([&](auto const&, auto const&) { ++toggleCount; });
			report["pointer_clicks"] = nlohmann::json::array();
			for (int clickIndex = 0; clickIndex < 12; clickIndex++)
			{
				bool expectedOn = !plain.IsOn();
				ULONGLONG clickStarted = GetTickCount64();
				SendValidationMouseClick(pointerScreenX, pointerScreenY);
				co_await AsyncTaskDelay(40, AppVariables::App.GetDispatcher());
				report["pointer_clicks"].push_back({
					{ "index", clickIndex },
					{ "expected_on", expectedOn },
					{ "actual_on", plain.IsOn() },
					{ "knob_x", plainPosition.X() },
					{ "elapsed_ms", GetTickCount64() - clickStarted },
					{ "pointer_pressed", pointerPressedCount },
					{ "pointer_released", pointerReleasedCount },
					{ "tapped", tappedCount },
					{ "double_tapped", doubleTappedCount }
				});
				Require(plain.IsOn() == expectedOn, L"Rapid pointer click was swallowed.");
				Require(toggleCount == clickIndex + 1, L"Pointer click produced missing or duplicate state changes.");
			}
			plain.Toggled(toggledToken);

			plain.IsOn(false);
			co_await AsyncTaskDelay(180, AppVariables::App.GetDispatcher());
			SendValidationMouseClick(pointerScreenX, pointerScreenY);
			co_await AsyncTaskDelay(45, AppVariables::App.GetDispatcher());
			Require(plain.IsOn(), L"Pointer click did not turn the switch on.");
			double displayedBeforeReverse = plainPosition.X();
			Require(displayedBeforeReverse > 0.5 && displayedBeforeReverse < 19.5, L"Switch was not observed mid-transition.");
			SendValidationMouseClick(pointerScreenX, pointerScreenY);
			ULONGLONG reverseStarted = GetTickCount64();
			report["reversal_samples"] = nlohmann::json::array();
			double displayedAfterReverse = displayedBeforeReverse;
			for (int sampleIndex = 0; sampleIndex < 5; sampleIndex++)
			{
				co_await AsyncTaskDelay(12, AppVariables::App.GetDispatcher());
				displayedAfterReverse = plainPosition.X();
				report["reversal_samples"].push_back({ { "index", sampleIndex }, { "x", displayedAfterReverse }, { "on", plain.IsOn() }, { "elapsed_ms", GetTickCount64() - reverseStarted } });
				if (sampleIndex == 1 && animated && GetTickCount64() - reverseStarted < 70) { Require(displayedAfterReverse > 0.1 && displayedAfterReverse < displayedBeforeReverse, L"Pointer reversal jumped or continued in the previous direction."); }
			}
			report["pointer_reversal"] = {
				{ "expected_on", false },
				{ "actual_on", plain.IsOn() },
				{ "before_x", displayedBeforeReverse },
				{ "after_x", displayedAfterReverse }
			};
			Require(!plain.IsOn(), L"Mid-animation pointer click did not reverse IsOn.");
			Require(displayedAfterReverse < displayedBeforeReverse - 0.1, L"Switch thumb did not reverse from its displayed position.");
			co_await AsyncTaskDelay(180, AppVariables::App.GetDispatcher());
			Require(!plain.IsOn() && std::abs(plainPosition.X()) < 0.1, L"Reversed switch did not finish at OFF.");
			report["pointer_actual"] = "passed";
			plain.PointerPressed(pressedToken);
			plain.PointerReleased(releasedToken);
			plain.Tapped(tappedToken);
			plain.DoubleTapped(doubleToken);
			page->listview_Main().SelectedIndex(1);
			page->ApplyDependentRow(page->stackpanel_FanControl_Curve(), true, false);
			page->slider_Fan_Temp_4().Value(100);
			page->slider_Fan_Speed_4().Value(100);
			page->UpdateFanGraphProfile();
			page->UpdateFanInputFields();
			page->UpdateLayout();
			co_await AsyncTaskDelay(180, AppVariables::App.GetDispatcher());
			page->UpdateFanChartAxisLayout();
			auto frame = page->grid_Fan_Graph();
			Require(frame.ActualWidth() > 500 && frame.ActualHeight() > 100, L"Chart did not receive a real layout.");
			auto point = page->ellipse_Fan_Point_4();
			auto bounds = point.TransformToVisual(frame).TransformBounds(winrt::Windows::Foundation::Rect{0, 0, (float)point.ActualWidth(), (float)point.ActualHeight()});
			Require(bounds.X >= 0 && bounds.Y >= 0 && bounds.X + bounds.Width <= frame.ActualWidth() && bounds.Y + bounds.Height <= frame.ActualHeight(), L"Top-right endpoint is clipped by the chart frame.");
			auto plot = page->viewbox_Fan_Plot();
			auto title = page->textblock_Fan_Y_Title();
			auto titleBounds = title.TransformToVisual(plot).TransformBounds(winrt::Windows::Foundation::Rect{0, 0, (float)title.ActualWidth(), (float)title.ActualHeight()});
			Require(std::abs(titleBounds.Y + titleBounds.Height / 2 - plot.ActualHeight() / 2) < 1.0, L"Y axis title is not vertically centered.");
			report["chart_endpoint_inside_frame"] = true;
			report["y_title_centered"] = true;
			report["guards_after"] = { {"disable_saving", disable_saving}, {"disable_saving_settings", disable_saving_settings} };
			disable_saving = savedDisableSaving;
			disable_saving_settings = savedDisableSavingSettings;
			report["status"] = "passed";
			WriteUtf8File(TestDataPath + L"\\animation-regression.json", report.dump(2));
			if (!AnimationCheckStay) { AppVariables::App.UiValidationExit(0); }
		}
		catch (winrt::hresult_error const& error)
		{
			report["guards_after"] = { {"disable_saving", disable_saving}, {"disable_saving_settings", disable_saving_settings} };
			disable_saving = savedDisableSaving;
			disable_saving_settings = savedDisableSavingSettings;
			report["status"] = "failed";
			report["error"] = winrt::to_string(error.message());
			WriteUtf8File(TestDataPath + L"\\animation-regression.json", report.dump(2));
			AppVariables::App.UiValidationExit(2);
		}
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
		report["flat_rows_verified"] = FlatRowsVerified;
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
				FlatRowsVerified += flatRows;
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
			Require(FlatRowsVerified > 0, L"No setting rows were validated.");
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
