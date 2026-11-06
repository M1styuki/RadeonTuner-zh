#pragma once
#include "MainPage.g.h"
#include <unordered_map>
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.UI.Xaml.Media.Animation.h>

namespace winrt::RadeonTuner::implementation
{
	struct MainPage : MainPageT<MainPage>
	{
		MainPage() {}

		//UI session lifecycle used by App::ReleaseUI; no page reference is kept globally.
		void StopNotificationTimer();
		bool HasActiveResolutionTimer();
		void StopUiTimersForRelease();
		void StopUiAnimationsForRelease();
		winrt::Windows::Foundation::IInspectable KeepAliveForUi();

		void PointerMoved_AdjustCursor(IInspectable const& sender, PointerRoutedEventArgs const& e);
		winrt::IAsyncAction page_Loaded(IInspectable const& sender, RoutedEventArgs const& e);
		void listview_Main_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void button_Website_Project_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void button_Website_Donation_Click(IInspectable const& sender, RoutedEventArgs const& e);

		static std::wstring AdlInitialize();
		static void ADL_MemoryFree_Customizations(CUSTOMISATIONS* pCustomisations);
		static void AdlSetDefaultSettings();
		static bool AdlCheckDriverOnlySoftware();

		static std::vector<AdapterInfo> AdlGetGpuAll(bool ignoreDuplicate);
		static std::optional<AdapterInfo> AdlGetGpuByDeviceId(std::wstring deviceId);
		static std::optional<AdapterInfo> AdlGetGpuByAdapterIndex(int adapterIndex);
		static std::vector<ADLDisplayInfo> AdlGetDisplayAll();
		static std::vector<ADLDisplayInfo> AdlGetDisplayByAdapterIndex(int adapterIndex);
		static std::optional<ADLDisplayInfo> AdlGetDisplayByDisplayIndex(int adapterIndex, int displayIndex);

		winrt::IAsyncOperation<winrt::IVector<RadeonTuner::AppPickerIdl>> AdlAppPickerAdd();
		winrt::IAsyncOperation<winrt::IVector<RadeonTuner::AppPickerIdl>> AdlAppPickerAddProcess();
		winrt::IAsyncOperation<winrt::IVector<RadeonTuner::AppPickerIdl>> AdlAppPickerAddLauncher();
		winrt::IAsyncOperation<winrt::IVector<RadeonTuner::AppPickerIdl>> AdlAppPickerRemoveAppGraphics();
		winrt::IAsyncOperation<winrt::IVector<RadeonTuner::AppPickerIdl>> AdlAppPickerRemoveAppTuning();
		winrt::IAsyncOperation<winrt::IVector<RadeonTuner::AppPickerIdl>> AdlAppPickerRemoveAppDisplay();
		winrt::IVector<RadeonTuner::AppPickerIdl> AdlAppPickerAddFile();

		static std::vector<AdlApplication> AdlAppLoadAll(std::wstring driverArea, bool loadProperties);
		static std::optional<AdlApplication> AdlAppLoadSearch(std::wstring driverArea, std::wstring fileName, std::wstring filePath);
		std::wstring AdlAppAdd(std::wstring filePath, std::wstring driverArea);
		static std::wstring AdlAppRemove(AdlApplication adlApp);
		static bool AdlAppRemoveAll();
		static bool AdlAppSyncAll();
		bool AdlAppUnlock(AdlApplication& adlApp, bool unlock);
		bool AdlAppSetDefaults(AdlApplication& adlApp, bool clearProperties, bool addOnly);
		static std::wstring AdlAppProfileGenerateName(std::wstring profileHeader);
		static bool AdlAppExists(std::wstring fileName, std::wstring filePath, std::wstring driverArea);
		static bool AdlAppPropertyValid(std::wstring propertyName, std::wstring driverArea);
		static DATATYPES AdlAppPropertyDataTypeGet(std::wstring propertyName, std::wstring driverArea);
		static ADLProfilePropertyType AdlAppConvertDataTypeToPropertyType(DATATYPES dataType);
		static std::vector<ADLPropertyRecordCreate> AdlAppPropertyRecordCreateGet(std::vector<AdlAppProperty> adlAppProperties);
		static std::optional<AdlAppProperty> AdlAppPropertyGet(AdlApplication adlApp, std::wstring propertyName);
		static bool AdlAppPropertySave(AdlApplication& adlApp);
		static bool AdlAppPropertyLoad(AdlApplication& adlApplication);
		static bool AdlAppPropertyUpdate(AdlApplication& adlApp, std::vector<AdlAppProperty> properties, bool addOnly);
		static bool AdlAppPropertyUpdate(AdlApplication& adlApp, std::wstring propertyGpuId, std::wstring propertyName, std::wstring propertyValue);

		static bool Adl_Overdrive8_Values_Reset(int gpuAdapterIndex);
		static bool Adl_Overdrive8_Values_Set(int gpuAdapterIndex, std::vector<std::tuple<ADLOD8SettingId, int, bool>> saveSettings);
		static std::optional<int> Adl_Overdrive8_Load_Value(int gpuAdapterIndex, ADLOD8SettingId settingId);
		static std::optional<ADLOD8SingleInitSettingWrap> Adl_Overdrive8_Load_Default(int gpuAdapterIndex, ADLOD8SettingId settingId);
		static bool Adl_Overdrive8_Feature_Supported(int gpuAdapterIndex, ADLOD8FeatureControl featureId);

		bool Adl_Eyefinity_Create_Custom(int displayAdapterIndex);
		static bool Adl_Eyefinity_Delete_All(int displayAdapterIndex);
		static bool Adl_Eyefinity_IsEnabled(int displayAdapterIndex);
		static AdlCustomResult Adl_Eyefinity_Toggle(int displayAdapterIndex, bool setEnabled);
		static bool Adl_Eyefinity_Automatic_IsEnabled();

		void button_Overlay_CustomResolution_Close_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void button_Overlay_CustomResolution_Create_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_CustomResolution_ShowHide_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_CustomResolution_Remove_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void DisplayModeInfo_Calculate_Timings();
		void DisplayModeInfo_ToUI(ADLDisplayModeInfoX2 modeInfoX2, bool actualOnly);
		std::optional<ADLDisplayModeInfoX2> DisplayModeInfo_FromADL(int adapterIndex, int displayIndex, int presentationMode, int timingStandard, int pixelWidth, int pixelHeight, int refreshRate);
		winrt::IAsyncOperation<bool> CustomResolution_Create(int adapterIndex, int displayIndex, ADLDisplayModeInfoX2 modeInfoX2);
		winrt::IAsyncOperation<bool> CustomResolution_Remove(int adapterIndex, int displayIndex, ADLDisplayModeInfoX2 modeInfoX2);

		static DWORD AdlGammaRampClamp(float clampValue);
		static bool AdlGammaRampMatch(const AdlGammaRamp& gammaRampBegin, const AdlGammaRamp& gammaRampEnd);
		static AdlGammaRamp AdlGammaRampBuild(float baseGamma, float redGamma, float greenGamma, float blueGamma);
		static bool AdlGammaApply(int gpuAdapterIndex, float baseGamma, float redGamma, float greenGamma, float blueGamma);

		static bool AdlRegistrySettingSet(int gpuAdapterIndex, std::string subKey, std::string keyName, std::wstring keyValue);
		static bool AdlRegistrySettingSet(int gpuAdapterIndex, std::string subKey, std::string keyName, int keyValue);
		static std::optional<std::wstring> AdlRegistrySettingGetString(int gpuAdapterIndex, std::string subKey, std::string keyName, bool decodeBinary);
		static std::optional<INT> AdlRegistrySettingGetInt(int gpuAdapterIndex, std::string subKey, std::string keyName);

		static std::wstring AdlxGetGpuIdentifier(int adapterIndex);
		static std::wstring AdlxGetDisplayIdentifier(int adapterIndex, int displayIndex);

		bool GraphicsSettings_Convert_ToUI_Adl(GraphicsSettings graphicsSettings, AdlSettingGet settingType);
		bool GraphicsSettings_Convert_ToUI_Profile(GraphicsSettings graphicsSettings, AdlSettingGet settingType);
		std::optional<GraphicsSettings> GraphicsSettings_Generate_FromADLApp(int gpuAdapterIndex, AdlApplication& adlApplication, bool loadDefault);
		std::optional<GraphicsSettings> GraphicsSettings_Generate_FromADLRegistry(int gpuAdapterIndex, std::wstring application, bool loadDefault);
		GraphicsSettings GraphicsSettingsGetSupport(int gpuAdapterIndex);
		static std::vector<GraphicsStatus> GraphicsStatus_Get();
		static void GraphicsStatus_Update(uint64_t expectedSession);

		static bool GraphicsSettings_Profiles_SaveToFile();
		static bool GraphicsSettings_Profiles_LoadFromFile();
		static bool GraphicsSettings_Profile_SaveToFile(GraphicsSettings graphicsSettings, std::wstring savePath);
		static std::optional<GraphicsSettings> GraphicsSettings_Profile_LoadFromFile(std::wstring loadPath);
		static bool GraphicsSettings_Profile_Add(GraphicsSettings graphicsSettingsAdd);
		static bool GraphicsSettings_Profile_Replace(GraphicsSettings graphicsSettingsReplace);
		static bool GraphicsSettings_Profile_Remove(std::wstring deviceId, std::wstring application);
		static std::optional<std::reference_wrapper<GraphicsSettings>> GraphicsSettings_Profile_Get(std::wstring deviceId, std::wstring application);
		static std::vector<std::wstring> GraphicsSettings_Profile_GetAllApps(std::wstring deviceId);
		bool GraphicsSettings_Match(GraphicsSettings settingsProfile, GraphicsSettings settingsAdl);

		bool DisplaySettings_Convert_ToUI_Adl(DisplaySettings displaySettings, AdlSettingGet settingType);
		bool DisplaySettings_Convert_ToUI_Profile(DisplaySettings displaySettings, AdlSettingGet settingType);
		static std::optional<DisplaySettings> DisplaySettings_Generate_FromADL(int adapterIndex, int displayIndex, std::wstring application, bool loadDefault);

		static bool DisplaySettings_Profiles_SaveToFile();
		static bool DisplaySettings_Profiles_LoadFromFile();
		static bool DisplaySettings_Profile_SaveToFile(DisplaySettings displaySettings, std::wstring savePath);
		static std::optional<DisplaySettings> DisplaySettings_Profile_LoadFromFile(std::wstring loadPath);
		static bool DisplaySettings_Profile_Add(DisplaySettings displaySettingsAdd);
		static bool DisplaySettings_Profile_Replace(DisplaySettings displaySettingsReplace);
		static bool DisplaySettings_Profile_Remove(std::wstring deviceId, std::wstring application);
		static std::optional<std::reference_wrapper<DisplaySettings>> DisplaySettings_Profile_Get(std::wstring deviceId, std::wstring application);
		static std::vector<std::wstring> DisplaySettings_Profile_GetAllApps(std::wstring deviceId);
		static bool DisplaySettings_Profile_Set_UsingGlobal();
		static bool DisplaySettings_Profile_Set_Using(std::wstring deviceId, std::wstring application);
		static bool DisplaySettings_Profile_Any_Using(std::wstring deviceId);
		static bool DisplaySettings_Match(DisplaySettings settingsProfile, DisplaySettings settingsAdl, bool appProfileOnly);

		bool MultimediaSettings_Convert_ToUI_Adl(MultimediaSettings multimediaSettings, AdlSettingGet settingType);
		bool MultimediaSettings_Convert_ToUI_Profile(MultimediaSettings multimediaSettings, AdlSettingGet settingType);
		std::optional<MultimediaSettings> MultimediaSettings_Generate_FromADL(int gpuAdapterIndex, std::wstring application, bool loadDefault);

		static bool MultimediaSettings_Profiles_SaveToFile();
		static bool MultimediaSettings_Profiles_LoadFromFile();
		static bool MultimediaSettings_Profile_SaveToFile(MultimediaSettings multimediaSettings, std::wstring savePath);
		static std::optional<MultimediaSettings> MultimediaSettings_Profile_LoadFromFile(std::wstring loadPath);
		static bool MultimediaSettings_Profile_Add(MultimediaSettings multimediaSettingsAdd);
		static bool MultimediaSettings_Profile_Replace(MultimediaSettings multimediaSettingsReplace);
		static bool MultimediaSettings_Profile_Remove(std::wstring deviceId, std::wstring application);
		static std::optional<std::reference_wrapper<MultimediaSettings>> MultimediaSettings_Profile_Get(std::wstring deviceId, std::wstring application);
		static std::vector<std::wstring> MultimediaSettings_Profile_GetAllApps(std::wstring deviceId);
		static bool MultimediaSettings_Profile_Set_UsingGlobal();
		static bool MultimediaSettings_Profile_Set_Using(std::wstring deviceId, std::wstring application);
		static bool MultimediaSettings_Profile_Any_Using(std::wstring deviceId);
		bool MultimediaSettings_Match(MultimediaSettings settingsProfile, MultimediaSettings settingsAdl);

		bool TuningFanSettings_Convert_ToUI_Adl(TuningFanSettings tuningFanSettings, AdlSettingGet settingType);
		bool TuningFanSettings_Convert_ToUI_Profile(TuningFanSettings tuningFanSettings, AdlSettingGet settingType);
		static std::optional<TuningFanSettings> TuningFanSettings_Generate_FromADL(int gpuAdapterIndex, std::wstring application, bool loadDefault);
		static bool TuningFanSettings_Match(TuningFanSettings settingsProfile, TuningFanSettings settingsAdl);
		static void TuningMetrics_Update(uint64_t expectedSession);

		static bool TuningFanSettings_Profiles_SaveToFile();
		static bool TuningFanSettings_Profiles_LoadFromFile();
		static bool TuningFanSettings_Profile_SaveToFile(TuningFanSettings tuningFanSettings, std::wstring savePath);
		static std::optional<TuningFanSettings> TuningFanSettings_Profile_LoadFromFile(std::wstring loadPath);
		static bool TuningFanSettings_Profile_Add(TuningFanSettings tuningFanSettingsAdd);
		static bool TuningFanSettings_Profile_Replace(TuningFanSettings tuningFanSettingsReplace);
		static bool TuningFanSettings_Profile_Remove(std::wstring deviceId, std::wstring application);
		static std::optional<std::reference_wrapper<TuningFanSettings>> TuningFanSettings_Profile_Get(std::wstring deviceId, std::wstring application);
		static std::vector<std::wstring> TuningFanSettings_Profile_GetAllApps(std::wstring deviceId);
		static bool TuningFanSettings_Profile_Set_UsingGlobal();
		static bool TuningFanSettings_Profile_Set_Using(std::wstring deviceId, std::wstring application);
		static bool TuningFanSettings_Profile_Any_Using(std::wstring deviceId);

		void AdlxValuesExportDisplay();
		winrt::IAsyncAction AdlxValuesImportDisplay();
		void AdlxValuesExportGraphics();
		void AdlxValuesImportGraphics();
		void AdlxValuesExportTuning();
		winrt::IAsyncAction AdlxValuesImportTuning();
		bool AdlxResetShaderCache();

		static bool AdlGraphicsSettingsApply(int gpuAdapterIndex, std::wstring gpuUniqueIdentifierHex, AdlApplication& adlApp, GraphicsSettings graphicsSettings, AdlSettingGet settingGet);
		bool AdlMultimediaSettingsApply(int gpuAdapterIndex, MultimediaSettings multimediaSettings, AdlSettingGet settingGet);
		static bool AdlTuningFanSettingsApply(int gpuAdapterIndex, TuningFanSettings tuningFanSettings, AdlSettingGet settingGet);
		static bool AdlDisplaySettingsApply(int displayAdapterIndex, int displayDisplayIndex, DisplaySettings targetSettings, DisplaySettings adlSettings, AdlSettingGet settingGet, bool appProfileOnly);

		void AdlxValuesResetSelectGpu();
		void AdlxValuesResetSelectDisplay();
		winrt::IAsyncAction AdlxValuesLoadSelectTuningApp(int gpuAdapterIndex, std::wstring application);
		winrt::IAsyncAction AdlxValuesLoadSelectDisplayApp(int dispAdapterIndex, int dispDisplayIndex, std::wstring application);
		winrt::IAsyncAction AdlxValuesLoadSelectGraphicsApp(int gpuAdapterIndex, std::wstring application);
		winrt::IAsyncAction AdlxValuesLoadSelectMultimediaApp(int gpuAdapterIndex, std::wstring application);
		winrt::IAsyncAction AdlxValuesLoadSelectGpu(AdapterInfo adapterInfo);
		winrt::IAsyncAction AdlxValuesLoadSelectDisplay(ADLDisplayInfo displayInfo);
		void AdlxValuesLoadEyefinityDisplays();
		void AdlxValuesPrepare();
		void AdlxInfoLoad();
		std::wstring AdlxInfoGpu();
		std::wstring AdlxInfoDisplay();
		std::wstring AdlxInfoApplication();
		void UpdateFanGraphGpu(TuningFanSettings tuningFanSettings);
		void UpdateFanGraphProfile();
		void UpdateFanChartAxisLayout();
		void viewbox_Fan_Plot_SizeChanged(IInspectable const& sender, SizeChangedEventArgs const& e);
		void UpdateFanControlModeInterface(TuningFanSettings tuningFanSettings, bool animate = false);
		void ValidateFanSettings();
		void UpdateFanInputFields();
		void ApplyFanInputValue(TextBox const& textBox);
		Slider FanInputAdapter(TextBox const& textBox);
		void UpdateDependentSettingsInterface(bool animate);
		void ApplyDependentRow(FrameworkElement const& row, bool visible, bool animate);
		void FinalizeDependentRow(FrameworkElement const& row, bool visible);
		void InitializeValueHints();
		void toggle_Loaded(IInspectable const& sender, RoutedEventArgs const& e);
		void InitializeComboBoxVisuals();
		void PatchComboBoxVisuals(ComboBox const& comboBox);
		winrt::IAsyncAction SettingLoad();
		void SettingAdmin();
		void SelectDefaultIndexes();
		void ShowExperimentalSettings(BOOL silent);
		void ShowNotification(std::wstring text);

		void ThemeAdjustStyle();
		std::wstring ThemeGetImagePath(std::wstring imageName, bool includeAppx);

		static void AdlxLoopMetrics();
		static void AdlxLoopProfile();
		static void AdlxLoopSelectedDeviceAccess();
		static void AdlxLoopDisplayEyefinityAutomatic(const std::vector<std::wstring>& processExeRunning);
		static void AdlxLoopDisplayApplyGamma(int adapterIndex, DisplaySettings displaySettingsRunning);
		static void AdlxLoopDisplayApplyProfile(int adapterIndex, int displayIndex, DisplaySettings displaySettingsRunning);
		static void AdlxLoopDisplayApplicationProfile(const std::vector<std::wstring>& processExeRunning);
		static void AdlxLoopTuningApplyProfile(int adapterIndex, TuningFanSettings tuningFanSettingsRunning);
		static void AdlxLoopTuningApplicationProfile(const std::vector<std::wstring>& processExeRunning);

		winrt::IAsyncAction button_Eyefinity_Overlay_Remove_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Eyefinity_Overlay_Create_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Eyefinity_Disable_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Eyefinity_Enable_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Eyefinity_Automatic_Toggled(IInspectable const& sender, RoutedEventArgs const& e);

		winrt::IAsyncAction button_Tuning_Apply_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Tuning_Reset_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Tuning_Import_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void button_Tuning_Export_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_AppSelect_Tuning_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_AppAdd_Tuning_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_AppRemove_Tuning_Click(IInspectable const& sender, RoutedEventArgs const& e);

		winrt::IAsyncAction button_AppSelect_Graphics_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Graphics_Clear_ShaderCache_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Graphics_Reset_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_AppAdd_Graphics_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_AppRemove_Graphics_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void button_Graphics_Import_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void button_Graphics_Export_Click(IInspectable const& sender, RoutedEventArgs const& e);

		void combobox_VerticalSync_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void toggleswitch_RadeonChill_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void slider_RadeonChill_Min_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void slider_RadeonChill_Max_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void button_RadeonChill_Link_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_RadeonEnhancedSync_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void combobox_Display_ColorDepth_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void combobox_Display_PixelFormat_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void slider_FanCurve_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void textbox_FanInput_KeyDown(IInspectable const& sender, KeyRoutedEventArgs const& e);
		void textbox_FanInput_LostFocus(IInspectable const& sender, RoutedEventArgs const& e);
		void slider_Display_Contrast_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void slider_Display_Saturation_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void toggleswitch_FsrLatencyReduction_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void slider_RadeonBoost_MinResolution_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void toggleswitch_RadeonImageSharpening1_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void slider_RadeonImageSharpening1_Sharpening_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void toggleswitch_RadeonImageSharpening2_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_RadeonImageSharpening2_Desktop_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void slider_RadeonImageSharpening2_Sharpening_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void combobox_AntiAliasingMethod_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void combobox_AntiAliasingLevel_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void toggleswitch_MorphologicalAntiAliasing_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void combobox_AnisotropicTextureFiltering_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void combobox_Tessellation_Mode_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void combobox_Tessellation_Level_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void toggleswitch_Display_VSR_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Display_GpuScaling_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Display_IntegerScaling_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void combobox_Display_ScalingMode_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void toggleswitch_Display_HDCPSupport_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Display_VariBright_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void combobox_Display_VariBright_Level_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void slider_Display_ColorTemperature_Kelvin_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void slider_Display_Brightness_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void slider_Display_Hue_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void toggleswitch_Display_GammaControl_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void slider_Display_GammaRGB_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void slider_Video_Sharpening_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void toggleswitch_Video_Upscaling_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Window_Top_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Update_Check_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Update_Launch_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void button_Fps_Overlayer_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Fan_Zero_Rpm_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void slider_Core_Min_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void slider_Core_Max_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void toggleswitch_OpenGLTripleBuffering_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Close_Tray_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Hide_Tray_Icon_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void combobox_TextureFilteringQuality_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void toggleswitch_SurfaceFormatOptimization_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Shortcut_Startup_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Shortcut_StartMenu_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Shortcut_ContextMenu_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_StartWindowVisible_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_StartCheckUpdate_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_FsrOverrideUpscaling_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_FsrOverrideFrameGeneration_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void combobox_Display_DisplayColorEnhancement_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void slider_Display_Protanopia_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void slider_Display_Deuteranopia_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void slider_Display_Tritanopia_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		winrt::IAsyncAction button_Display_Import_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void button_Display_Export_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_AntiAliasingEnhancedQuality_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_AntiAliasingOverride_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Display_Reset_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Display_ColorTemperature_Control_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Display_CVDC_Control_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_FsrOverrideMultiFrameGeneration_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_FsrOverrideRayRegeneration_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_FsrOverrideNeuralRadianceCaching_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void combobox_FsrMultiFrameGenerationRatio_SelectionChanged(IInspectable const& sender, Controls::SelectionChangedEventArgs const& e);
		void toggleswitch_ShowExperimental_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_OpenGL10BitPixelFormat_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_Frtc_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void slider_Frtc_FrameRateTarget_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);

		void button_FsrDllLoadPath_Set_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void button_FsrDllLoadPath_Default_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void FsrOverrideDllUpdateTextPathInfo(std::wstring dllPath);
		void FsrOverrideDllUpdateTextVersion(std::wstring dllPath);
		std::wstring FsrOverrideDllGetPathDefault();
		std::wstring FsrOverrideDllGetPathSet(bool globalPath);

		static bool EnvironmentSettingGet(std::wstring settingName);
		static bool EnvironmentSettingRemove(std::wstring settingName);
		static bool EnvironmentSettingSet(std::wstring settingName, bool enabled);

		void DisplaySettings_Resolution_Revert();
		void DisplaySettings_Confirm_Resolution_Start();
		winrt::IAsyncAction DisplaySettings_Confirm_Resolution_Stop(bool revertResolution);
		void button_Overlay_ConfirmResolution_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void button_Overlay_RevertResolution_Click(IInspectable const& sender, RoutedEventArgs const& e);

		void DisplaySettings_Confirm_CustomResolution_Start();
		winrt::IAsyncAction DisplaySettings_Confirm_CustomResolution_Stop(bool revertResolution);
		winrt::IAsyncAction combobox_CustomResolution_TimingStandard_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		winrt::IAsyncAction textbox_CustomResolution_Resolution_TextChanged(IInspectable const& sender, TextChangedEventArgs const& e);
		winrt::IAsyncAction textbox_CustomResolution_RefreshRate_TextChanged(IInspectable const& sender, TextChangedEventArgs const& e);
		void combobox_CustomResolution_Presentation_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void textbox_CustomResolution_PixelClock_TextChanged(IInspectable const& sender, TextChangedEventArgs const& e);
		void textbox_TimingTotal_TextChanged(IInspectable const& sender, TextChangedEventArgs const& e);
		void textbox_TimingFrontPorch_TextChanged(IInspectable const& sender, TextChangedEventArgs const& e);
		void textbox_TimingSyncWidth_TextChanged(IInspectable const& sender, TextChangedEventArgs const& e);
		void combobox_TimingPolarity_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);


		void LaunchDriverCleanup();
		winrt::IAsyncAction DisplayList_Combined(bool waitUpdate);
		winrt::IAsyncAction DisplayList_SelectCurrent_Values(bool waitUpdate);
		winrt::IAsyncAction DisplayList_Resolution(bool waitUpdate);
		winrt::IAsyncAction DisplayList_RefreshRate(bool waitUpdate);
		winrt::IAsyncOperation<int> ShowMessageBox(std::wstring Question, std::wstring Description, std::vector<std::wstring> Answers);

		void toggleswitch_Display_HdrEnabled_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Multimedia_Reset_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_FrameGenEnabled_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void combobox_FrameGenAlgorithm_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void toggleswitch_KeepActive_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void combobox_Memory_Timing_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void slider_Memory_Max_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void slider_Power_Limit_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void slider_Power_Voltage_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void slider_Power_TDC_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void slider_Video_Brightness_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);
		void combobox_Display_Resolution_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void combobox_Display_RefreshRate_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void combobox_Display_Orientation_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void listview_Overlay_MessageBox_ItemClick(IInspectable const& sender, ItemClickEventArgs const& e);
		void button_Overlay_AppPicker_Confirm_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void listview_Overlay_AppPicker_ItemClick(IInspectable const& sender, ItemClickEventArgs const& e);
		void button_Overlay_AppPicker_Cancel_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_GpuSelect_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_DisplaySelect_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_AppSelect_Display_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_AppAdd_Display_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_AppRemove_Display_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Multimedia_Apply_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Display_Apply_Click(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction button_Graphics_Apply_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void button_Eyefinity_Manage_Click(IInspectable const& sender, RoutedEventArgs const& e);
		void combobox_RadeonBoost_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void combobox_Display_FreeSyncMode_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void toggleswitch_FsrShowInformation_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		winrt::IAsyncAction toggleswitch_SkipSupportCheckGraphics_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void combobox_Memory_Ecc_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void toggleswitch_ThemeIconMono_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void combobox_ThemeStyle_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void toggleswitch_UpdateExcludeDrivers_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void toggleswitch_FsrDisableMLSwapchain_Toggled(IInspectable const& sender, RoutedEventArgs const& e);
		void combobox_FanControl_Mode_SelectionChanged(IInspectable const& sender, SelectionChangedEventArgs const& e);
		void slider_FanRanged_ValueChanged(IInspectable const& sender, RangeBaseValueChangedEventArgs const& e);

#ifdef RADEONTUNER_UI_VALIDATION
		size_t UiValidationAnimationCount() const { return dependentAnimations.size(); }
		size_t UiValidationToggleCount() const { return toggleKnobInitialized.size(); }
		size_t UiValidationKnobAnimationCount() const { return toggleKnobAnimations.size(); }
		std::optional<double> UiValidationAnimationFrom(FrameworkElement const& row) const
		{
			auto it = dependentAnimations.find(static_cast<void*>(winrt::get_abi(row)));
			if (it == dependentAnimations.end()) { return std::nullopt; }
			return it->second.Children().GetAt(0).as<winrt::Windows::UI::Xaml::Media::Animation::DoubleAnimation>().From().Value();
		}
#endif
	private:
		void AnimateToggleKnob(ToggleSwitch const& toggle, winrt::Windows::UI::Xaml::Media::TranslateTransform const& knob);
		void AnimatePageEntrance(FrameworkElement const& page);
		void FinalizePageTransition();
		void AnimateComboBoxPopup(ComboBox const& comboBox, bool open);
		FrameworkElement FindComboBoxPopupBorder(ComboBox const& comboBox);

		struct DependentFoldAnimationGuard
		{
			explicit DependentFoldAnimationGuard(MainPage& page) : page(page), previous(page.suppressDependentFoldAnimation)
			{
				page.suppressDependentFoldAnimation = true;
			}

			~DependentFoldAnimationGuard()
			{
				page.suppressDependentFoldAnimation = previous;
			}

			DependentFoldAnimationGuard(DependentFoldAnimationGuard const&) = delete;
			DependentFoldAnimationGuard& operator=(DependentFoldAnimationGuard const&) = delete;

			MainPage& page;
			bool previous;
		};

		bool valueHintsInitialized = false;
		std::unordered_map<void*, bool> toggleKnobInitialized;
		bool comboVisualsInitialized = false;
		bool suppressDependentFoldAnimation = false;
		uint64_t pageTransitionGeneration = 0;
		FrameworkElement pageTransitionTarget{ nullptr };
		winrt::Windows::UI::Xaml::Media::Animation::Storyboard pageTransitionStoryboard{ nullptr };
		std::unordered_map<void*, winrt::Windows::UI::Xaml::Media::Animation::Storyboard> dependentAnimations;
		std::unordered_map<void*, winrt::Windows::UI::Xaml::Media::Animation::Storyboard> comboPopupAnimations;
		std::unordered_map<void*, winrt::Windows::UI::Xaml::Media::Animation::Storyboard> toggleKnobAnimations;

	};
}

namespace winrt::RadeonTuner::factory_implementation
{
	struct MainPage : MainPageT<MainPage, implementation::MainPage> {};
}
