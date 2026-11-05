#pragma once
#include <cmath>
#include <filesystem>
#include <optional>
#include <string>
#include "pch.h"
#include "MainPage.h"
#include "AppVariables.h"
#include "MainVariables.h"
#include "UiValidationCommon.h"

namespace RadeonTunerUiValidation
{
	using winrt::Windows::UI::Xaml::Visibility;
	using winrt::RadeonTuner::implementation::MainPage;

	inline bool NearlyEqual(float left, float right)
	{
		return std::fabs(left - right) < 0.0001F;
	}

	inline void VerifyGammaAndFanBehavior(MainPage* page)
	{
		//Exercise the real isolated event paths without touching a GPU or writing application settings
		DisplaySettings savedDisplay = displaySettingsProfile;
		TuningFanSettings savedTuning = tuningFanSettingsProfile;
		bool previousDisableSaving = disable_saving;
		try
		{
			disable_saving = false;

			//Gamma control and RGB/base event updates
			page->toggleswitch_Display_GammaControl().IsOn(true);
			page->toggleswitch_Display_GammaControl_Toggled(page->toggleswitch_Display_GammaControl(), winrt::Windows::UI::Xaml::RoutedEventArgs{ nullptr });
			Require(page->slider_Display_GammaBase().IsEnabled(), L"Gamma base slider was not enabled by the event.");
			Require(page->slider_Display_GammaRed().IsEnabled(), L"Gamma red slider was not enabled by the event.");
			Require(displaySettingsProfile.GammaControl.Current.value_or(false), L"Gamma control event did not update the profile.");
			Require(SomeText(page->textblock_Display_GammaControl_Value().Text()) == L"已启用", L"Gamma enabled value is not localized.");
			page->slider_Display_GammaBase().Value(1.25F);
			page->slider_Display_GammaRed().Value(1.10F);
			page->slider_Display_GammaGreen().Value(1.05F);
			page->slider_Display_GammaBlue().Value(0.90F);
			page->slider_Display_GammaRGB_ValueChanged(page->slider_Display_GammaBase(), winrt::Windows::UI::Xaml::Controls::Primitives::RangeBaseValueChangedEventArgs{ nullptr });
			Require(NearlyEqual(displaySettingsProfile.GammaBase.Current.value_or(0.0F), 1.25F), L"Gamma base event did not update the profile.");
			Require(NearlyEqual(displaySettingsProfile.GammaRed.Current.value_or(0.0F), 1.10F), L"Gamma red event did not update the profile.");
			Require(NearlyEqual(displaySettingsProfile.GammaGreen.Current.value_or(0.0F), 1.05F), L"Gamma green event did not update the profile.");
			Require(NearlyEqual(displaySettingsProfile.GammaBlue.Current.value_or(0.0F), 0.90F), L"Gamma blue event did not update the profile.");

			page->toggleswitch_Display_GammaControl().IsOn(false);
			page->toggleswitch_Display_GammaControl_Toggled(page->toggleswitch_Display_GammaControl(), winrt::Windows::UI::Xaml::RoutedEventArgs{ nullptr });
			Require(!page->slider_Display_GammaBase().IsEnabled(), L"Gamma base slider was not disabled by the event.");
			Require(SomeText(page->textblock_Display_GammaControl_Value().Text()) == L"已禁用", L"Gamma disabled value is not localized.");

			//Display dependent rows still fold when saving is disabled, as in the isolated preview.
			bool previousPreviewSaving = disable_saving;
			disable_saving = true;

			page->toggleswitch_Display_GammaControl().IsOn(true);
			page->UpdateDependentSettingsInterface(false);
			Require(page->stackpanel_Display_Gamma_Dependent().Visibility() == Visibility::Visible, L"Gamma dependent rows did not fold in the preview path.");
			page->toggleswitch_Display_GammaControl().IsOn(false);
			page->UpdateDependentSettingsInterface(false);
			Require(page->stackpanel_Display_Gamma_Dependent().Visibility() == Visibility::Collapsed, L"Gamma dependent rows did not collapse in the preview path.");
			Require(NearlyEqual(page->slider_Display_GammaBase().Value(), 1.25F), L"Collapsing the gamma rows reset a value.");

			page->toggleswitch_Display_ColorTemperature_Control().IsOn(true);
			page->UpdateDependentSettingsInterface(false);
			Require(page->stackpanel_Display_ColorTemperature_Kelvin_Dependent().Visibility() == Visibility::Visible, L"Color temperature row did not fold in the preview path.");
			page->toggleswitch_Display_ColorTemperature_Control().IsOn(false);
			page->UpdateDependentSettingsInterface(false);
			Require(page->stackpanel_Display_ColorTemperature_Kelvin_Dependent().Visibility() == Visibility::Collapsed, L"Color temperature row did not collapse in the preview path.");

			page->toggleswitch_Display_CVDC_Control().IsOn(true);
			page->UpdateDependentSettingsInterface(false);
			Require(page->stackpanel_Display_CVDC_Dependent().Visibility() == Visibility::Visible, L"CVDC dependent rows did not fold in the preview path.");
			page->toggleswitch_Display_CVDC_Control().IsOn(false);
			page->UpdateDependentSettingsInterface(false);
			Require(page->stackpanel_Display_CVDC_Dependent().Visibility() == Visibility::Collapsed, L"CVDC dependent rows did not collapse in the preview path.");

			disable_saving = previousPreviewSaving;

			//Gamma ramp generation is deterministic and base/RGB changes affect different channels
			AdlGammaRamp neutral = page->AdlGammaRampBuild(1.0F, 1.0F, 1.0F, 1.0F);
			AdlGammaRamp neutralCopy = page->AdlGammaRampBuild(1.0F, 1.0F, 1.0F, 1.0F);
			AdlGammaRamp baseChanged = page->AdlGammaRampBuild(1.1F, 1.0F, 1.0F, 1.0F);
			AdlGammaRamp redChanged = page->AdlGammaRampBuild(1.0F, 1.1F, 1.0F, 1.0F);
			Require(page->AdlGammaRampMatch(neutral, neutralCopy), L"Neutral gamma ramps are not deterministic.");
			Require(!page->AdlGammaRampMatch(neutral, baseChanged), L"Gamma base did not change the ramp.");
			Require(!page->AdlGammaRampMatch(neutral, redChanged), L"Gamma red did not change the ramp.");

			//Fan mode combobox event updates the profile and the visible mode sections
			auto selectFanMode = [&](FanControlModes mode)
			{
				page->combobox_FanControl_Mode().SelectedIndex((int)mode);
				page->combobox_FanControl_Mode_SelectionChanged(page->combobox_FanControl_Mode(), winrt::Windows::UI::Xaml::Controls::SelectionChangedEventArgs{ nullptr });
				//Settle the reveal animation for deterministic state assertions; the event path above remains the production animation path.
				page->UpdateFanControlModeInterface(tuningFanSettingsProfile, false);
			};
			selectFanMode(FanControlModes::NoFanControl);
			Require(tuningFanSettingsProfile.FanControlMode.Current.value_or(-1) == (int)FanControlModes::NoFanControl, L"NoFanControl was not stored.");
			Require(page->stackpanel_FanControl_RangedFanSpeed().Visibility() == Visibility::Collapsed, L"NoFanControl showed the ranged fan section.");
			Require(page->stackpanel_FanControl_Curve().Visibility() == Visibility::Collapsed, L"NoFanControl showed the fan curve section.");
			Require(page->stackpanel_FanControl_FanZeroRpm().Visibility() == Visibility::Collapsed, L"NoFanControl showed the zero RPM section.");

			selectFanMode(FanControlModes::FanZeroRpmOnly);
			Require(tuningFanSettingsProfile.FanControlMode.Current.value_or(-1) == (int)FanControlModes::FanZeroRpmOnly, L"FanZeroRpmOnly was not stored.");
			Require(page->stackpanel_FanControl_FanZeroRpm().Visibility() == Visibility::Visible, L"FanZeroRpmOnly hid the zero RPM section.");
			Require(page->stackpanel_FanControl_RangedFanSpeed().Visibility() == Visibility::Collapsed, L"FanZeroRpmOnly showed the ranged fan section.");
			Require(page->stackpanel_FanControl_Curve().Visibility() == Visibility::Collapsed, L"FanZeroRpmOnly showed the fan curve section.");

			selectFanMode(FanControlModes::RangedFanSpeed);
			Require(tuningFanSettingsProfile.FanControlMode.Current.value_or(-1) == (int)FanControlModes::RangedFanSpeed, L"RangedFanSpeed was not stored.");
			Require(page->stackpanel_FanControl_RangedFanSpeed().Visibility() == Visibility::Visible, L"RangedFanSpeed hid the ranged fan section.");
			Require(page->stackpanel_FanControl_Curve().Visibility() == Visibility::Collapsed, L"RangedFanSpeed showed the fan curve section.");
			Require(page->stackpanel_FanControl_FanZeroRpm().Visibility() == Visibility::Visible, L"RangedFanSpeed hid the zero RPM section.");

			selectFanMode(FanControlModes::CustomFanCurve);
			Require(tuningFanSettingsProfile.FanControlMode.Current.value_or(-1) == (int)FanControlModes::CustomFanCurve, L"CustomFanCurve was not stored.");
			Require(page->stackpanel_FanControl_Curve().Visibility() == Visibility::Visible, L"CustomFanCurve hid the fan curve section.");
			Require(page->stackpanel_FanControl_RangedFanSpeed().Visibility() == Visibility::Collapsed, L"CustomFanCurve showed the ranged fan section.");
			Require(page->stackpanel_FanControl_FanZeroRpm().Visibility() == Visibility::Visible, L"CustomFanCurve hid the zero RPM section.");

			//Fan fields survive an actual profile file roundtrip
			TuningFanSettings fanProfile{};
			fanProfile.DeviceId = L"test-device";
			fanProfile.Application = L"Global";
			fanProfile.CoreMin.Current = 500;
			fanProfile.MemoryMax.Current = 1200;
			fanProfile.FanZeroRpm.Current = true;
			fanProfile.FanSpeedMin.Current = 20;
			fanProfile.FanSpeedMax.Current = 80;
			fanProfile.KeepActive.Current = true;
			fanProfile.FanControlMode.Current = (int)FanControlModes::CustomFanCurve;
			std::wstring fanPath = TestDataPath + L"\\fan-profile-roundtrip.radt";
			Require(page->TuningFanSettings_Profile_SaveToFile(fanProfile, fanPath), L"Fan profile save failed.");
			auto fanLoaded = page->TuningFanSettings_Profile_LoadFromFile(fanPath);
			Require(fanLoaded.has_value(), L"Fan profile load failed.");
			Require(fanLoaded->CoreMin.Current.value_or(0) == 500, L"Fan profile lost CoreMin.");
			Require(fanLoaded->MemoryMax.Current.value_or(0) == 1200, L"Fan profile lost MemoryMax.");
			Require(fanLoaded->FanZeroRpm.Current.value_or(false), L"Fan profile lost FanZeroRpm.");
			Require(fanLoaded->FanSpeedMin.Current.value_or(0) == 20, L"Fan profile lost FanSpeedMin.");
			Require(fanLoaded->FanSpeedMax.Current.value_or(0) == 80, L"Fan profile lost FanSpeedMax.");
			Require(fanLoaded->FanControlMode.Current.value_or(-1) == (int)FanControlModes::CustomFanCurve, L"Fan profile lost FanControlMode.");
			Require(fanLoaded->KeepActive.Current.value_or(false), L"Fan profile lost KeepActive.");

			//An older profile without the new fields keeps old values and receives sensible defaults
			nlohmann::json fanLegacy = nlohmann::json::parse(wstring_to_string_utf8(struct_to_jsonstring(fanProfile, true)));
			fanLegacy.erase("FanControlMode.Current");
			fanLegacy.erase("FanSpeedMin.Current");
			fanLegacy.erase("FanSpeedMax.Current");
			fanLegacy.erase("KeepActive.Current");
			TuningFanSettings fanOld = jsonstring_to_struct<TuningFanSettings>(fanLegacy.dump());
			Require(fanOld.CoreMin.Current.value_or(0) == 500, L"Old fan profile lost existing CoreMin.");
			Require(fanOld.FanControlMode.Current.value_or(-1) == (int)FanControlModes::NoFanControl, L"Old fan profile did not default FanControlMode.");
			Require(!fanOld.KeepActive.Current.value_or(true), L"Old fan profile did not default KeepActive.");

			//Gamma and new graphics fields roundtrip, while old profiles keep their values and defaults
			DisplaySettings gammaProfile{};
			gammaProfile.DeviceId = L"test-display";
			gammaProfile.Application = L"Global";
			gammaProfile.Brightness.Current = 57;
			gammaProfile.GammaControl.Current = true;
			gammaProfile.GammaBase.Current = 1.25F;
			gammaProfile.GammaRed.Current = 1.10F;
			gammaProfile.GammaGreen.Current = 1.05F;
			gammaProfile.GammaBlue.Current = 0.90F;
			auto gammaLoaded = jsonstring_to_struct<DisplaySettings>(struct_to_jsonstring(gammaProfile, true));
			Require(gammaLoaded.GammaControl.Current.value_or(false), L"Gamma control did not serialize.");
			Require(NearlyEqual(gammaLoaded.GammaBase.Current.value_or(0.0F), 1.25F), L"Gamma base did not serialize.");
			Require(NearlyEqual(gammaLoaded.GammaRed.Current.value_or(0.0F), 1.10F), L"Gamma red did not serialize.");
			Require(gammaLoaded.Brightness.Current.value_or(0) == 57, L"Gamma profile lost Brightness.");

			nlohmann::json gammaLegacy = nlohmann::json::parse(wstring_to_string_utf8(struct_to_jsonstring(gammaProfile, true)));
			gammaLegacy.erase("GammaControl.Current");
			gammaLegacy.erase("GammaBase.Current");
			gammaLegacy.erase("GammaRed.Current");
			gammaLegacy.erase("GammaGreen.Current");
			gammaLegacy.erase("GammaBlue.Current");
			DisplaySettings gammaOld = jsonstring_to_struct<DisplaySettings>(gammaLegacy.dump());
			Require(!gammaOld.GammaControl.Current.value_or(true), L"Old display profile did not default GammaControl.");
			Require(NearlyEqual(gammaOld.GammaBase.Current.value_or(0.0F), 1.0F), L"Old display profile did not default GammaBase.");
			Require(gammaOld.Brightness.Current.value_or(0) == 57, L"Old display profile lost Brightness.");

			GraphicsSettings graphicsProfile{};
			graphicsProfile.DeviceId = L"test-gpu";
			graphicsProfile.Application = L"Global";
			graphicsProfile.DeLagEnabled.Current = true;
			graphicsProfile.FsrShowInformation.Current = true;
			graphicsProfile.FsrDisableMLSwapchain.Current = true;
			auto graphicsLoaded = jsonstring_to_struct<GraphicsSettings>(struct_to_jsonstring(graphicsProfile, true));
			Require(graphicsLoaded.DeLagEnabled.Current.value_or(false), L"Graphics profile lost DeLagEnabled.");
			Require(graphicsLoaded.FsrShowInformation.Current.value_or(false), L"Graphics profile lost FsrShowInformation.");
			Require(graphicsLoaded.FsrDisableMLSwapchain.Current.value_or(false), L"Graphics profile lost FsrDisableMLSwapchain.");

			nlohmann::json graphicsLegacy = nlohmann::json::parse(wstring_to_string_utf8(struct_to_jsonstring(graphicsProfile, true)));
			graphicsLegacy.erase("FsrShowInformation.Current");
			graphicsLegacy.erase("FsrDisableMLSwapchain.Current");
			GraphicsSettings graphicsOld = jsonstring_to_struct<GraphicsSettings>(graphicsLegacy.dump());
			Require(graphicsOld.DeLagEnabled.Current.value_or(false), L"Old graphics profile lost DeLagEnabled.");
			Require(!graphicsOld.FsrShowInformation.Current.value_or(true), L"Old graphics profile did not default FsrShowInformation.");
			Require(!graphicsOld.FsrDisableMLSwapchain.Current.value_or(true), L"Old graphics profile did not default FsrDisableMLSwapchain.");

			//Existing settings keys and unknown data survive a settings write
			std::wstring settingsPath = TestDataPath + L"\\settings-compat.json";
			std::wstring settingsInitial = LR"({"HideTrayIcon":true,"ThemeStyle":2,"LegacyUnknown":"keep-me","StartCheckUpdate":false})";
			Require(string_to_file(settingsPath, settingsInitial), L"Could not create the compatibility settings file.");
			AVSettingsJson compatSettings(settingsPath);
			Require(compatSettings.Load<bool>("HideTrayIcon").value_or(false), L"Existing HideTrayIcon was not loaded.");
			Require(compatSettings.Load<int>("ThemeStyle").value_or(-1) == 2, L"Existing ThemeStyle was not loaded.");
			Require(compatSettings.Set("FirstLaunch", false), L"Could not add the new settings field.");
			nlohmann::json settingsJson = nlohmann::json::parse(wstring_to_string_utf8(file_to_string(settingsPath)));
			Require(settingsJson["HideTrayIcon"] == true, L"Settings write lost HideTrayIcon.");
			Require(settingsJson["ThemeStyle"] == 2, L"Settings write lost ThemeStyle.");
			Require(settingsJson["LegacyUnknown"] == "keep-me", L"Settings write lost an unknown existing key.");
			Require(settingsJson["FirstLaunch"] == false, L"Settings write did not add FirstLaunch.");

			UiCompatibilityVerified = true;
			std::filesystem::remove(fanPath);
			std::filesystem::remove(settingsPath);
		}
		catch (...)
		{
			displaySettingsProfile = savedDisplay;
			tuningFanSettingsProfile = savedTuning;
			disable_saving = previousDisableSaving;
			throw;
		}
		displaySettingsProfile = savedDisplay;
		tuningFanSettingsProfile = savedTuning;
		disable_saving = previousDisableSaving;
	}
}
