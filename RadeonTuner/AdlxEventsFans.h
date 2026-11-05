#pragma once
#include "pch.h"
#include "MainPage.h"
#include "MainVariables.h"
#include "FanInputValue.h"
#include <cmath>

namespace winrt::RadeonTuner::implementation
{
	void MainPage::toggleswitch_Fan_Zero_Rpm_Toggled(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving) { return; }

			//Get setting value
			auto newSender = sender.as<ToggleSwitch>();
			bool newValue = newSender.IsOn();

			if (newValue)
			{
				//Show or hide Zero RPM line
				grid_Fan_Zero_Rpm_Line_Profile().Visibility(Visibility::Visible);
				border_Fan_Zero_Rpm_Range().Visibility(Visibility::Visible);
			}
			else
			{
				//Show or hide Zero RPM line
				grid_Fan_Zero_Rpm_Line_Profile().Visibility(Visibility::Collapsed);
				border_Fan_Zero_Rpm_Range().Visibility(Visibility::Collapsed);
			}

			//Adjust button colors
			SolidColorBrush colorIgnored = Application::Current().Resources().Lookup(box_value(L"ApplicationIgnoredBrush")).as<SolidColorBrush>();
			button_Tuning_Apply().Background(colorIgnored);
			button_Fan_Apply().Background(colorIgnored);

			//Update current value
			tuningFanSettingsProfile.FanZeroRpm.Current = newValue;
		}
		catch (...) {}
	}

	void MainPage::combobox_FanControl_Mode_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving) { return; }

			//Get setting value
			int newValue = combobox_FanControl_Mode().SelectedIndex();

			//Adjust button colors
			SolidColorBrush colorIgnored = Application::Current().Resources().Lookup(box_value(L"ApplicationIgnoredBrush")).as<SolidColorBrush>();
			button_Tuning_Apply().Background(colorIgnored);
			button_Fan_Apply().Background(colorIgnored);

			//Update current value
			tuningFanSettingsProfile.FanControlMode.Current = newValue;

			//Update fan control mode interface
			UpdateFanControlModeInterface(tuningFanSettingsProfile, true);
		}
		catch (...) {}
	}

	void MainPage::slider_FanRanged_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving) { return; }

			//Get setting value
			int minValue = (int)slider_Fan_Speed_Min().Value();
			int maxValue = (int)slider_Fan_Speed_Max().Value();

			//Match values
			if (minValue > maxValue)
			{
				slider_Fan_Speed_Max().Value(minValue);
			}
			if (maxValue < minValue)
			{
				slider_Fan_Speed_Min().Value(maxValue);
			}

			//Adjust button colors
			SolidColorBrush colorIgnored = Application::Current().Resources().Lookup(box_value(L"ApplicationIgnoredBrush")).as<SolidColorBrush>();
			button_Tuning_Apply().Background(colorIgnored);
			button_Fan_Apply().Background(colorIgnored);

			//Update current value
			tuningFanSettingsProfile.FanSpeedMin.Current = minValue;
			tuningFanSettingsProfile.FanSpeedMax.Current = maxValue;

			//Keep the editable number boxes aligned with the adapter values
			UpdateFanInputFields();
		}
		catch (...) {}
	}

	void MainPage::slider_FanCurve_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e)
	{
		try
		{
			//Check if saving is disabled
			if (disable_saving) { return; }

			//Validate and update fan graph
			ValidateFanSettings();
			UpdateFanGraphProfile();

			//Adjust button colors
			SolidColorBrush colorIgnored = Application::Current().Resources().Lookup(box_value(L"ApplicationIgnoredBrush")).as<SolidColorBrush>();
			button_Tuning_Apply().Background(colorIgnored);
			button_Fan_Apply().Background(colorIgnored);

			//Update current value
			tuningFanSettingsProfile.FanSpeed0.Current = (int)slider_Fan_Speed_0().Value();
			tuningFanSettingsProfile.FanSpeed1.Current = (int)slider_Fan_Speed_1().Value();
			tuningFanSettingsProfile.FanSpeed2.Current = (int)slider_Fan_Speed_2().Value();
			tuningFanSettingsProfile.FanSpeed3.Current = (int)slider_Fan_Speed_3().Value();
			tuningFanSettingsProfile.FanSpeed4.Current = (int)slider_Fan_Speed_4().Value();
			tuningFanSettingsProfile.FanTemp0.Current = (int)slider_Fan_Temp_0().Value();
			tuningFanSettingsProfile.FanTemp1.Current = (int)slider_Fan_Temp_1().Value();
			tuningFanSettingsProfile.FanTemp2.Current = (int)slider_Fan_Temp_2().Value();
			tuningFanSettingsProfile.FanTemp3.Current = (int)slider_Fan_Temp_3().Value();
			tuningFanSettingsProfile.FanTemp4.Current = (int)slider_Fan_Temp_4().Value();

			//Keep the editable number boxes aligned with the adapter values after validation
			UpdateFanInputFields();
		}
		catch (...) {}
	}

	void MainPage::textbox_FanInput_KeyDown(IInspectable const& sender, KeyRoutedEventArgs const& e)
	{
		try
		{
			if (e.Key() != winrt::Windows::System::VirtualKey::Enter) { return; }

			auto textBox = sender.try_as<TextBox>();
			if (!textBox) { return; }

			ApplyFanInputValue(textBox);
			e.Handled(true);
		}
		catch (...) {}
	}

	void MainPage::textbox_FanInput_LostFocus(IInspectable const& sender, RoutedEventArgs const& e)
	{
		try
		{
			auto textBox = sender.try_as<TextBox>();
			if (!textBox) { return; }

			ApplyFanInputValue(textBox);
		}
		catch (...) {}
	}

	void MainPage::ApplyFanInputValue(TextBox const& textBox)
	{
		try
		{
			//The hidden sliders remain the single adapter to the established fan handlers
			Slider adapter = FanInputAdapter(textBox);
			if (!adapter) { return; }

			//Only a complete integer is accepted; invalid edits restore the current adapter value
			int value = 0;
			if (!::RadeonTuner::FanInputValue::TryParse(std::wstring(textBox.Text().c_str()), adapter.Minimum(), adapter.Maximum(), adapter.StepFrequency(), value))
			{
				UpdateFanInputFields();
				return;
			}

			//Enter and LostFocus are both handled without applying the same value twice
			int currentValue = (int)std::lround(adapter.Value());
			if (value != currentValue)
			{
				adapter.Value((double)value);
			}
			UpdateFanInputFields();
		}
		catch (...) {}
	}

	Slider MainPage::FanInputAdapter(TextBox const& textBox)
	{
		try
		{
			std::wstring name(textBox.Name().c_str());
			if (name == L"textbox_Fan_Speed_Min") { return slider_Fan_Speed_Min(); }
			if (name == L"textbox_Fan_Speed_Max") { return slider_Fan_Speed_Max(); }
			if (name == L"textbox_Fan_Temp_0") { return slider_Fan_Temp_0(); }
			if (name == L"textbox_Fan_Temp_1") { return slider_Fan_Temp_1(); }
			if (name == L"textbox_Fan_Temp_2") { return slider_Fan_Temp_2(); }
			if (name == L"textbox_Fan_Temp_3") { return slider_Fan_Temp_3(); }
			if (name == L"textbox_Fan_Temp_4") { return slider_Fan_Temp_4(); }
			if (name == L"textbox_Fan_Speed_0") { return slider_Fan_Speed_0(); }
			if (name == L"textbox_Fan_Speed_1") { return slider_Fan_Speed_1(); }
			if (name == L"textbox_Fan_Speed_2") { return slider_Fan_Speed_2(); }
			if (name == L"textbox_Fan_Speed_3") { return slider_Fan_Speed_3(); }
			if (name == L"textbox_Fan_Speed_4") { return slider_Fan_Speed_4(); }
		}
		catch (...) {}
		return Slider{ nullptr };
	}

	void MainPage::UpdateFanInputFields()
	{
		try
		{
			auto update = [](TextBox const& textBox, Slider const& slider)
			{
				textBox.Text(winrt::hstring(std::to_wstring((int)std::lround(slider.Value()))));
			};
			auto updateHint = [](TextBlock const& textBlock, Slider const& slider, std::wstring const& suffix)
			{
				textBlock.Text(winrt::hstring(std::to_wstring((int)std::lround(slider.Value())) + suffix));
			};

			update(textbox_Fan_Speed_Min(), slider_Fan_Speed_Min());
			update(textbox_Fan_Speed_Max(), slider_Fan_Speed_Max());
			update(textbox_Fan_Temp_0(), slider_Fan_Temp_0());
			update(textbox_Fan_Temp_1(), slider_Fan_Temp_1());
			update(textbox_Fan_Temp_2(), slider_Fan_Temp_2());
			update(textbox_Fan_Temp_3(), slider_Fan_Temp_3());
			update(textbox_Fan_Temp_4(), slider_Fan_Temp_4());
			update(textbox_Fan_Speed_0(), slider_Fan_Speed_0());
			update(textbox_Fan_Speed_1(), slider_Fan_Speed_1());
			update(textbox_Fan_Speed_2(), slider_Fan_Speed_2());
			update(textbox_Fan_Speed_3(), slider_Fan_Speed_3());
			update(textbox_Fan_Speed_4(), slider_Fan_Speed_4());

			//The named hint groups stay collapsed; their text is exposed through the input tooltips.
			updateHint(textblock_Fan_Speed_Min_Value(), slider_Fan_Speed_Min(), L"%");
			updateHint(textblock_Fan_Speed_Max_Value(), slider_Fan_Speed_Max(), L"%");
			updateHint(textblock_Fan_Curve_0_Temp_Value(), slider_Fan_Temp_0(), L"°C");
			updateHint(textblock_Fan_Curve_1_Temp_Value(), slider_Fan_Temp_1(), L"°C");
			updateHint(textblock_Fan_Curve_2_Temp_Value(), slider_Fan_Temp_2(), L"°C");
			updateHint(textblock_Fan_Curve_3_Temp_Value(), slider_Fan_Temp_3(), L"°C");
			updateHint(textblock_Fan_Curve_4_Temp_Value(), slider_Fan_Temp_4(), L"°C");
			updateHint(textblock_Fan_Curve_0_Speed_Value(), slider_Fan_Speed_0(), L"%");
			updateHint(textblock_Fan_Curve_1_Speed_Value(), slider_Fan_Speed_1(), L"%");
			updateHint(textblock_Fan_Curve_2_Speed_Value(), slider_Fan_Speed_2(), L"%");
			updateHint(textblock_Fan_Curve_3_Speed_Value(), slider_Fan_Speed_3(), L"%");
			updateHint(textblock_Fan_Curve_4_Speed_Value(), slider_Fan_Speed_4(), L"%");
		}
		catch (...) {}
	}

	void MainPage::ValidateFanSettings()
	{
		try
		{
			//Fix cleanup and improve method

			//Validate fan speed
			float fanSpeed0 = slider_Fan_Speed_0().Value();
			float fanSpeed1 = slider_Fan_Speed_1().Value();
			float fanSpeed2 = slider_Fan_Speed_2().Value();
			float fanSpeed3 = slider_Fan_Speed_3().Value();
			float fanSpeed4 = slider_Fan_Speed_4().Value();

			if (fanSpeed4 < fanSpeed3)
			{
				slider_Fan_Speed_4().Value(fanSpeed3);
			}
			if (fanSpeed3 < fanSpeed2)
			{
				slider_Fan_Speed_3().Value(fanSpeed2);
			}
			if (fanSpeed3 > fanSpeed4)
			{
				slider_Fan_Speed_3().Value(fanSpeed4);
			}
			if (fanSpeed2 < fanSpeed1)
			{
				slider_Fan_Speed_2().Value(fanSpeed1);
			}
			if (fanSpeed2 > fanSpeed3)
			{
				slider_Fan_Speed_2().Value(fanSpeed3);
			}
			if (fanSpeed1 < fanSpeed0)
			{
				slider_Fan_Speed_1().Value(fanSpeed0);
			}
			if (fanSpeed1 > fanSpeed2)
			{
				slider_Fan_Speed_1().Value(fanSpeed2);
			}
			if (fanSpeed0 > fanSpeed1)
			{
				slider_Fan_Speed_0().Value(fanSpeed1);
			}

			//Validate fan temperature
			float fanTempOffset = 5;
			float fanTemp0 = slider_Fan_Temp_0().Value();
			float fanTemp1 = slider_Fan_Temp_1().Value();
			float fanTemp2 = slider_Fan_Temp_2().Value();
			float fanTemp3 = slider_Fan_Temp_3().Value();
			float fanTemp4 = slider_Fan_Temp_4().Value();

			float fanTempMax0 = fanTemp1 - fanTempOffset;
			float fanTempMin1 = fanTemp0 + fanTempOffset;
			float fanTempMax1 = fanTemp2 - fanTempOffset;
			float fanTempMin2 = fanTemp1 + fanTempOffset;
			float fanTempMax2 = fanTemp3 - fanTempOffset;
			float fanTempMin3 = fanTemp2 + fanTempOffset;
			float fanTempMax3 = fanTemp4 - fanTempOffset;
			float fanTempMin4 = fanTemp3 + fanTempOffset;

			if (fanTemp4 < fanTempMin4)
			{
				slider_Fan_Temp_4().Value(fanTempMin4);
			}
			if (fanTemp3 < fanTempMin3)
			{
				slider_Fan_Temp_3().Value(fanTempMin3);
			}
			if (fanTemp3 > fanTempMax3)
			{
				slider_Fan_Temp_3().Value(fanTempMax3);
			}
			if (fanTemp2 < fanTempMin2)
			{
				slider_Fan_Temp_2().Value(fanTempMin2);
			}
			if (fanTemp2 > fanTempMax2)
			{
				slider_Fan_Temp_2().Value(fanTempMax2);
			}
			if (fanTemp1 < fanTempMin1)
			{
				slider_Fan_Temp_1().Value(fanTempMin1);
			}
			if (fanTemp1 > fanTempMax1)
			{
				slider_Fan_Temp_1().Value(fanTempMax1);
			}
			if (fanTemp0 > fanTempMax0)
			{
				slider_Fan_Temp_0().Value(fanTempMax0);
			}

			//Set result
			//AVDebugWriteLine("ADLX validated fan settings.");
		}
		catch (...) {}
	}

	void MainPage::UpdateFanControlModeInterface(TuningFanSettings tuningFanSettings, bool animate)
	{
		try
		{
			//Get setting
			FanControlModes valueObject = (FanControlModes)tuningFanSettings.FanControlMode.Current.value();

			//Keep the official modes mutually exclusive while revealing the relevant section
			bool showRanged = valueObject == FanControlModes::RangedFanSpeed;
			bool showCurve = valueObject == FanControlModes::CustomFanCurve;
			bool showZeroRpm = valueObject == FanControlModes::RangedFanSpeed ||
				valueObject == FanControlModes::CustomFanCurve ||
				valueObject == FanControlModes::FanZeroRpmOnly;

			ApplyDependentRow(stackpanel_FanControl_RangedFanSpeed(), showRanged, animate);
			ApplyDependentRow(stackpanel_FanControl_Curve(), showCurve, animate);
			ApplyDependentRow(stackpanel_FanControl_FanZeroRpm(), showZeroRpm, animate);
		}
		catch (...) {}
	}

	void MainPage::UpdateFanGraphGpu(TuningFanSettings tuningFanSettings)
	{
		try
		{
			//Speed vertical Y
			//Temperature horizontal X
			PointCollection fanPoints;

			//Get minimum and maximum
			int fanSpeedMinimum = tuningFanSettings.FanSpeed0.Minimum.value();
			int fanSpeedMaximum = tuningFanSettings.FanSpeed0.Maximum.value();

			//Get graph size
			int graphWidth = 500;
			int graphHeight = 100;

			//Add fan points
			float fanSpeedStart = fanSpeedMaximum - tuningFanSettings.FanSpeed0.Current.value();
			float fanTempStart = 0;
			fanPoints.Append(Point{ fanTempStart, fanSpeedStart });

			float fanSpeed0 = fanSpeedMaximum - tuningFanSettings.FanSpeed0.Current.value();
			float fanTemp0 = (double)tuningFanSettings.FanTemp0.Current.value() / 100 * graphWidth;
			fanPoints.Append(Point{ fanTemp0, fanSpeed0 });

			float fanSpeed1 = fanSpeedMaximum - tuningFanSettings.FanSpeed1.Current.value();
			float fanTemp1 = (double)tuningFanSettings.FanTemp1.Current.value() / 100 * graphWidth;
			fanPoints.Append(Point{ fanTemp1, fanSpeed1 });

			float fanSpeed2 = fanSpeedMaximum - tuningFanSettings.FanSpeed2.Current.value();
			float fanTemp2 = (double)tuningFanSettings.FanTemp2.Current.value() / 100 * graphWidth;
			fanPoints.Append(Point{ fanTemp2, fanSpeed2 });

			float fanSpeed3 = fanSpeedMaximum - tuningFanSettings.FanSpeed3.Current.value();
			float fanTemp3 = (double)tuningFanSettings.FanTemp3.Current.value() / 100 * graphWidth;
			fanPoints.Append(Point{ fanTemp3, fanSpeed3 });

			float fanSpeed4 = fanSpeedMaximum - tuningFanSettings.FanSpeed4.Current.value();
			float fanTemp4 = (double)tuningFanSettings.FanTemp4.Current.value() / 100 * graphWidth;
			fanPoints.Append(Point{ fanTemp4, fanSpeed4 });

			float fanSpeedEnd = fanSpeedMaximum - tuningFanSettings.FanSpeed4.Current.value();
			float fanTempEnd = graphWidth;
			fanPoints.Append(Point{ fanTempEnd, fanSpeedEnd });

			//Set zero rpm line position
			if (tuningFanSettings.FanZeroTemp >= 0)
			{
				float fanZeroLinePosition = (double)tuningFanSettings.FanZeroTemp / 100 * graphWidth;
				Canvas::SetLeft(grid_Fan_Zero_Rpm_Line_Gpu(), fanZeroLinePosition);
				Canvas::SetLeft(grid_Fan_Zero_Rpm_Line_Profile(), fanZeroLinePosition);
				border_Fan_Zero_Rpm_Range().Width(fanZeroLinePosition);
			}

			//Set fan points to polyline
			polyline_Fan_Lines_Gpu().Points(fanPoints);

			//Set result
			//AVDebugWriteLine("Updated gpu fan graph.");
		}
		catch (...) {}
	}

	void MainPage::UpdateFanGraphProfile()
	{
		try
		{
			//Speed vertical Y
			//Temperature horizontal X
			PointCollection fanPoints;

			//Get minimum and maximum
			int fanSpeedMinimum = slider_Fan_Speed_0().Minimum();
			int fanSpeedMaximum = slider_Fan_Speed_0().Maximum();

			//Get graph size
			int graphWidth = 500;
			int graphHeight = 100;

			//Add fan points
			float fanSpeedStart = fanSpeedMaximum - slider_Fan_Speed_0().Value();
			float fanTempStart = 0;
			fanPoints.Append(Point{ fanTempStart, fanSpeedStart });

			float fanSpeed0 = fanSpeedMaximum - slider_Fan_Speed_0().Value();
			float fanTemp0 = slider_Fan_Temp_0().Value() / 100 * graphWidth;
			ellipse_Fan_Point_0().Margin(Thickness(fanTemp0 + 3, fanSpeed0 + 3, 0, 0));
			fanPoints.Append(Point{ fanTemp0, fanSpeed0 });

			float fanSpeed1 = fanSpeedMaximum - slider_Fan_Speed_1().Value();
			float fanTemp1 = slider_Fan_Temp_1().Value() / 100 * graphWidth;
			ellipse_Fan_Point_1().Margin(Thickness(fanTemp1 + 3, fanSpeed1 + 3, 0, 0));
			fanPoints.Append(Point{ fanTemp1, fanSpeed1 });

			float fanSpeed2 = fanSpeedMaximum - slider_Fan_Speed_2().Value();
			float fanTemp2 = slider_Fan_Temp_2().Value() / 100 * graphWidth;
			ellipse_Fan_Point_2().Margin(Thickness(fanTemp2 + 3, fanSpeed2 + 3, 0, 0));
			fanPoints.Append(Point{ fanTemp2, fanSpeed2 });

			float fanSpeed3 = fanSpeedMaximum - slider_Fan_Speed_3().Value();
			float fanTemp3 = slider_Fan_Temp_3().Value() / 100 * graphWidth;
			ellipse_Fan_Point_3().Margin(Thickness(fanTemp3 + 3, fanSpeed3 + 3, 0, 0));
			fanPoints.Append(Point{ fanTemp3, fanSpeed3 });

			float fanSpeed4 = fanSpeedMaximum - slider_Fan_Speed_4().Value();
			float fanTemp4 = slider_Fan_Temp_4().Value() / 100 * graphWidth;
			ellipse_Fan_Point_4().Margin(Thickness(fanTemp4 + 3, fanSpeed4 + 3, 0, 0));
			fanPoints.Append(Point{ fanTemp4, fanSpeed4 });

			float fanSpeedEnd = fanSpeedMaximum - slider_Fan_Speed_4().Value();
			float fanTempEnd = graphWidth;
			fanPoints.Append(Point{ fanTempEnd, fanSpeedEnd });

			//Set fan points to polyline
			polyline_Fan_Lines_Profile().Points(fanPoints);

			//Set result
			//AVDebugWriteLine("Updated profile fan graph.");
		}
		catch (...) {}
	}
}