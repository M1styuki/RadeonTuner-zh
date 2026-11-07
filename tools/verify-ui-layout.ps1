#Requires -Version 7
[CmdletBinding()]
param(
    [string]$XamlPath = (Join-Path $PSScriptRoot '..\RadeonTuner\MainPage.xaml'),
    [string]$AppXamlPath = (Join-Path $PSScriptRoot '..\RadeonTuner\App.xaml'),
    [string]$MainPageCppPath = (Join-Path $PSScriptRoot '..\RadeonTuner\MainPage.cpp'),
    [string]$AdlxEventsDisplayPath = (Join-Path $PSScriptRoot '..\RadeonTuner\AdlxEventsDisplay.h'),
    [string]$AdlxEventsFansPath = (Join-Path $PSScriptRoot '..\RadeonTuner\AdlxEventsFans.h'),
    [string]$EvidencePath = (Join-Path $PSScriptRoot '..\.build-work\evidence\flat-layout-validation.json')
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$XamlPath = [IO.Path]::GetFullPath($XamlPath)
$AppXamlPath = [IO.Path]::GetFullPath($AppXamlPath)
$MainPageCppPath = [IO.Path]::GetFullPath($MainPageCppPath)
$AdlxEventsDisplayPath = [IO.Path]::GetFullPath($AdlxEventsDisplayPath)
$AdlxEventsFansPath = [IO.Path]::GetFullPath($AdlxEventsFansPath)
$EvidencePath = [IO.Path]::GetFullPath($EvidencePath)
$Xaml = [xml](Get-Content -LiteralPath $XamlPath -Raw -Encoding utf8)
$AppXaml = [xml](Get-Content -LiteralPath $AppXamlPath -Raw -Encoding utf8)
$MainPageCpp = Get-Content -LiteralPath $MainPageCppPath -Raw -Encoding utf8
$AdlxEventsDisplay = Get-Content -LiteralPath $AdlxEventsDisplayPath -Raw -Encoding utf8
$AdlxEventsFans = Get-Content -LiteralPath $AdlxEventsFansPath -Raw -Encoding utf8
$Namespace = New-Object System.Xml.XmlNamespaceManager($Xaml.NameTable)
$Namespace.AddNamespace('x', 'http://schemas.microsoft.com/winfx/2006/xaml')
$AppNamespace = New-Object System.Xml.XmlNamespaceManager($AppXaml.NameTable)
$AppNamespace.AddNamespace('x', 'http://schemas.microsoft.com/winfx/2006/xaml')

$PageNames = @(
    'stackpanel_Tuning', 'stackpanel_Fans', 'stackpanel_Graphics', 'stackpanel_Display',
    'stackpanel_Multimedia', 'stackpanel_Settings', 'stackpanel_Information'
)
$Failures = [System.Collections.Generic.List[string]]::new()
$ToggleCount = 0
$ComboCount = 0
$SliderRowCount = 0
$ButtonRowCount = 0
$FanPairRowCount = 0
$FanPairInputCount = 0
$FanRangeRowCount = 0
$FanRangeSliderCount = 0
$FanRangeInputCount = 0
$HiddenFanAdapterCount = 0
$PageCount = 0
$RowCount = 0

function Add-Failure([string]$Message) {
    $script:Failures.Add($Message)
}

function Get-XName([System.Xml.XmlElement]$Element) {
    return $Element.GetAttribute('Name', 'http://schemas.microsoft.com/winfx/2006/xaml')
}

function Get-Column([System.Xml.XmlElement]$Element) {
    $Value = $Element.GetAttribute('Grid.Column')
    if ([string]::IsNullOrWhiteSpace($Value)) { return 0 }
    return [int]$Value
}

function Get-ElementChildren([System.Xml.XmlElement]$Element) {
    return @($Element.ChildNodes | Where-Object { $_.NodeType -eq 'Element' })
}

function Get-Style([string]$Key) {
    $Node = $script:Xaml.SelectSingleNode("//*[@x:Key='$Key' or @x:Name='$Key']", $script:Namespace)
    if ($null -eq $Node) { Add-Failure "Missing style: $Key" }
    return $Node
}

function Get-Setter([System.Xml.XmlElement]$Style, [string]$Property) {
    if ($null -eq $Style) { return $null }
    return $Style.SelectSingleNode("./*[local-name()='Setter' and @Property='$Property']")
}

function Get-SetterValue([System.Xml.XmlElement]$Style, [string]$Property) {
    $Setter = Get-Setter $Style $Property
    if ($null -eq $Setter) { return $null }
    return $Setter.GetAttribute('Value')
}

function Require-Setter([System.Xml.XmlElement]$Style, [string]$Property, [string]$Expected, [string]$Context) {
    $Actual = Get-SetterValue $Style $Property
    if ($Actual -ne $Expected) {
        Add-Failure "$Context $Property expected '$Expected' but found '$Actual'"
    }
}

$SettingRowStyle = Get-Style 'FlatSettingRow'
Require-Setter $SettingRowStyle 'Padding' '0,6' 'FlatSettingRow'
Require-Setter $SettingRowStyle 'MinHeight' '44' 'FlatSettingRow'
$SettingLabelStyle = Get-Style 'FlatSettingLabel'
Require-Setter $SettingLabelStyle 'FontSize' '13' 'FlatSettingLabel'
Require-Setter $SettingLabelStyle 'TextWrapping' 'Wrap' 'FlatSettingLabel'
$ValueLabelStyle = Get-Style 'FlatValueLabel'
Require-Setter $ValueLabelStyle 'FontSize' '12' 'FlatValueLabel'
$ValueHintStyle = Get-Style 'FlatValueHint'
if ($null -eq $ValueHintStyle) {
    Add-Failure 'Missing FlatValueHint style.'
}
else {
    Require-Setter $ValueHintStyle 'Visibility' 'Collapsed' 'FlatValueHint'
    if ($ValueHintStyle.GetAttribute('BasedOn') -ne '{StaticResource FlatValueLabel}') {
        Add-Failure 'FlatValueHint must derive from FlatValueLabel.'
    }
}
$HelpIconStyle = Get-Style 'FlatHelpIcon'
Require-Setter $HelpIconStyle 'Width' '16' 'FlatHelpIcon'
Require-Setter $HelpIconStyle 'Height' '16' 'FlatHelpIcon'
$ToggleStyle = Get-Style 'FlatToggleSwitch'
Require-Setter $ToggleStyle 'Width' '44' 'FlatToggleSwitch'
Require-Setter $ToggleStyle 'MinWidth' '0' 'FlatToggleSwitch'
Require-Setter $ToggleStyle 'HorizontalAlignment' 'Right' 'FlatToggleSwitch'
$MetricLabelStyle = Get-Style 'FlatMetricLabel'
Require-Setter $MetricLabelStyle 'FontSize' '12' 'FlatMetricLabel'
$MetricValueStyle = Get-Style 'FlatMetricValue'
Require-Setter $MetricValueStyle 'FontSize' '14' 'FlatMetricValue'
if ((Get-SetterValue $MetricValueStyle 'FontFamily') -or $MetricValueStyle.OuterXml -match 'Consolas') {
    Add-Failure 'Metric values must use the shared UI font, not Consolas.'
}
$MetricUnitStyle = Get-Style 'FlatMetricUnit'
Require-Setter $MetricUnitStyle 'FontSize' '14' 'FlatMetricUnit'
Require-Setter $MetricUnitStyle 'Margin' '0' 'FlatMetricUnit'

# The visible toggle track must itself be 44 DIP wide and use the native Thumb overlay, so its drawn right edge can coincide with the number box edge.
$ToggleTemplate = $ToggleStyle.SelectSingleNode("./*[local-name()='Setter' and @Property='Template']/*[local-name()='Setter.Value']/*[local-name()='ControlTemplate']")
if ($null -eq $ToggleTemplate) {
    Add-Failure 'FlatToggleSwitch has no control template.'
}
else {
    $OuterBorder = $ToggleTemplate.SelectSingleNode(".//*[local-name()='Rectangle' and @x:Name='OuterBorder']", $Namespace)
    $SwitchKnob = $ToggleTemplate.SelectSingleNode(".//*[local-name()='Grid' and @x:Name='SwitchKnob']", $Namespace)
    $SwitchThumb = $ToggleTemplate.SelectSingleNode(".//*[local-name()='Thumb' and @x:Name='SwitchThumb']", $Namespace)
    if ($null -eq $OuterBorder -or $OuterBorder.GetAttribute('Width') -ne '44' -or $OuterBorder.GetAttribute('Height') -ne '20') {
        Add-Failure 'FlatToggleSwitch template must draw a 44x20 track.'
    }
    if ($null -eq $SwitchKnob -or $null -eq $SwitchThumb) {
        Add-Failure 'FlatToggleSwitch template is missing its knob or Thumb overlay.'
    }
}

# Slider: custom template with named parts, native keyboard/pointer behavior and the 125% DPI workaround preserved.
$AppStyles = @{}
foreach ($StyleNode in $AppXaml.SelectNodes("//*[local-name()='Style']")) {
    $Key = $StyleNode.GetAttribute('Name', 'http://schemas.microsoft.com/winfx/2006/xaml')
    if (-not $Key) { $Key = $StyleNode.GetAttribute('Key', 'http://schemas.microsoft.com/winfx/2006/xaml') }
    if ($Key) { $AppStyles[$Key] = $StyleNode }
}
$SliderStyle = $AppStyles['ApplicationSlider']
if ($null -eq $SliderStyle) { Add-Failure 'Missing ApplicationSlider style.' }
else {
    Require-Setter $SliderStyle 'UseLayoutRounding' 'False' 'ApplicationSlider'
    $SliderTemplate = $SliderStyle.SelectSingleNode("./*[local-name()='Setter' and @Property='Template']/*[local-name()='Setter.Value']/*[local-name()='ControlTemplate']")
    if ($null -eq $SliderTemplate) {
        Add-Failure 'ApplicationSlider has no control template.'
    }
    else {
        foreach ($Part in @('SliderContainer', 'HorizontalTemplate', 'HorizontalTrackRect', 'HorizontalDecreaseRect', 'HorizontalThumb', 'VerticalTemplate', 'VerticalThumb')) {
            if ($null -eq $SliderTemplate.SelectSingleNode(".//*[@x:Name='$Part']", $Namespace)) {
                Add-Failure "ApplicationSlider template is missing $Part."
            }
        }
        $Track = $SliderTemplate.SelectSingleNode(".//*[@x:Name='HorizontalTrackRect']", $Namespace)
        $Thumb = $SliderTemplate.SelectSingleNode(".//*[@x:Name='HorizontalThumb']", $Namespace)
        if ($null -eq $Track -or $Track.GetAttribute('Height') -ne '4' -or $Track.GetAttribute('RadiusX') -ne '2') {
            Add-Failure 'ApplicationSlider track must be a thin rounded 4 DIP track.'
        }
        if ($null -eq $Thumb -or $Thumb.GetAttribute('Width') -ne '16' -or $Thumb.GetAttribute('Height') -ne '16') {
            Add-Failure 'ApplicationSlider thumb must be a small 16 DIP circle.'
        }
    }
}

foreach ($StyleKey in @('ApplicationTextBox', 'ApplicationTextBoxNoHit', 'ApplicationTextBoxReadOnly')) {
    $Style = $AppStyles[$StyleKey]
    if ($null -eq $Style) { Add-Failure "Missing $StyleKey style."; continue }
    Require-Setter $Style 'Height' '32' $StyleKey
    Require-Setter $Style 'BorderThickness' '1' $StyleKey
    Require-Setter $Style 'CornerRadius' '{StaticResource ApplicationControlCornerRadius}' $StyleKey
    Require-Setter $Style 'FontSize' '13' $StyleKey
    Require-Setter $Style 'TextAlignment' 'Center' $StyleKey
}
$NoHitStyle = $AppStyles['ApplicationTextBoxNoHit']
if ($null -eq $NoHitStyle) { Add-Failure 'Missing ApplicationTextBoxNoHit style.' }
else {
    $NoHitTemplate = $NoHitStyle.SelectSingleNode("./*[local-name()='Setter' and @Property='Template']/*[local-name()='Setter.Value']/*[local-name()='ControlTemplate']")
    $NoHitText = if ($null -ne $NoHitTemplate) { $NoHitTemplate.SelectSingleNode(".//*[local-name()='TextBlock']") } else { $null }
    if ($null -eq $NoHitText -or
        $NoHitText.GetAttribute('VerticalAlignment') -ne 'Center' -or
        $NoHitText.GetAttribute('TextAlignment') -ne '{TemplateBinding TextAlignment}' -or
        $NoHitText.GetAttribute('Text') -ne '{TemplateBinding Text}') {
        Add-Failure 'ApplicationTextBoxNoHit does not render centered text from its Text property.'
    }
}
$FanNumberStyle = $AppStyles['ApplicationFanNumberBox']
if ($null -eq $FanNumberStyle) { Add-Failure 'Missing ApplicationFanNumberBox style.' }
else {
    if ($FanNumberStyle.GetAttribute('BasedOn') -ne '{StaticResource ApplicationTextBox}') {
        Add-Failure 'ApplicationFanNumberBox must derive from ApplicationTextBox.'
    }
    $FanNumberTemplate = $FanNumberStyle.SelectSingleNode("./*[local-name()='Setter' and @Property='Template']/*[local-name()='Setter.Value']/*[local-name()='ControlTemplate']")
    $FanNumberContent = if ($null -ne $FanNumberTemplate) { $FanNumberTemplate.SelectSingleNode(".//*[@x:Name='ContentElement']", $Namespace) } else { $null }
    if ($null -eq $FanNumberContent -or $FanNumberContent.GetAttribute('VerticalContentAlignment') -ne 'Center') {
        Add-Failure 'ApplicationFanNumberBox does not vertically center its native ContentElement.'
    }
    if ($null -ne $FanNumberTemplate -and $null -ne $FanNumberTemplate.SelectSingleNode(".//*[@x:Name='DeleteButton']", $Namespace)) {
        Add-Failure 'ApplicationFanNumberBox still reserves a delete-button column.'
    }
}

$ComboStyle = $AppStyles['ApplicationComboBox']
if ($null -eq $ComboStyle) { Add-Failure 'Missing ApplicationComboBox style.' }
else {
    Require-Setter $ComboStyle 'Height' '32' 'ApplicationComboBox'
    Require-Setter $ComboStyle 'BorderThickness' '1' 'ApplicationComboBox'
    Require-Setter $ComboStyle 'CornerRadius' '{StaticResource ApplicationControlCornerRadius}' 'ApplicationComboBox'
    Require-Setter $ComboStyle 'FontSize' '13' 'ApplicationComboBox'
}
foreach ($StyleKey in @('ApplicationButton', 'ApplicationButtonImage')) {
    $Style = $AppStyles[$StyleKey]
    if ($null -eq $Style) { Add-Failure "Missing $StyleKey style."; continue }
    Require-Setter $Style 'Height' '32' $StyleKey
    Require-Setter $Style 'MinHeight' '32' $StyleKey
    Require-Setter $Style 'CornerRadius' '{StaticResource ApplicationControlCornerRadius}' $StyleKey
}
$ButtonStyle = $AppStyles['ApplicationButton']
Require-Setter $ButtonStyle 'FontSize' '13' 'ApplicationButton'
$ButtonImageStyle = $AppStyles['ApplicationButtonImage']
Require-Setter $ButtonImageStyle 'Width' '32' 'ApplicationButtonImage'
Require-Setter $ButtonImageStyle 'Padding' '0' 'ApplicationButtonImage'
Require-Setter $ButtonImageStyle 'BorderThickness' '0' 'ApplicationButtonImage'

# Compact native scrollbar: thin 6 DIP thumbs, 16 DIP hit area, transparent track, all native parts present.
$ScrollBarStyle = $AppXaml.SelectSingleNode("//*[local-name()='Style' and @TargetType='ScrollBar']", $AppNamespace)
if ($null -eq $ScrollBarStyle) { Add-Failure 'Missing implicit ScrollBar style.' }
else {
    $ScrollBarTemplate = $ScrollBarStyle.SelectSingleNode("./*[local-name()='Setter' and @Property='Template']/*[local-name()='Setter.Value']/*[local-name()='ControlTemplate']")
    if ($null -eq $ScrollBarTemplate) { Add-Failure 'ScrollBar style has no control template.' }
    else {
        foreach ($Part in @('HorizontalRoot', 'HorizontalThumb', 'VerticalRoot', 'VerticalThumb', 'HorizontalLargeDecrease', 'HorizontalLargeIncrease', 'VerticalLargeDecrease', 'VerticalLargeIncrease')) {
            if ($null -eq $ScrollBarTemplate.SelectSingleNode(".//*[@x:Name='$Part']", $AppNamespace)) { Add-Failure "ScrollBar template is missing $Part." }
        }
        $VerticalThumb = $ScrollBarTemplate.SelectSingleNode(".//*[@x:Name='VerticalThumb']", $AppNamespace)
        $HorizontalThumb = $ScrollBarTemplate.SelectSingleNode(".//*[@x:Name='HorizontalThumb']", $AppNamespace)
        foreach ($TypeCheck in @(@('HorizontalRoot','Grid'), @('VerticalRoot','Grid'), @('HorizontalPanningRoot','Grid'), @('VerticalPanningRoot','Grid'), @('HorizontalPanningThumb','Border'), @('VerticalPanningThumb','Border'), @('HorizontalTrackRect','Rectangle'), @('VerticalTrackRect','Rectangle'), @('HorizontalThumb','Thumb'), @('VerticalThumb','Thumb'), @('HorizontalLargeDecrease','RepeatButton'), @('VerticalLargeIncrease','RepeatButton'))) {
            $Node = $ScrollBarTemplate.SelectSingleNode(".//*[@x:Name='$($TypeCheck[0])']", $AppNamespace)
            if ($null -eq $Node -or $Node.LocalName -ne $TypeCheck[1]) { Add-Failure "ScrollBar $($TypeCheck[0]) must be a $($TypeCheck[1])." }
        }
        $IndicatorStates = $ScrollBarTemplate.SelectSingleNode(".//*[@x:Name='ScrollingIndicatorStates']", $AppNamespace)
        if ($null -eq $IndicatorStates) { Add-Failure 'ScrollBar is missing ScrollingIndicatorStates.' }
        else {
            foreach ($StateName in @('TouchIndicator', 'MouseIndicator', 'NoIndicator')) {
                if ($null -eq $IndicatorStates.SelectSingleNode("./*[local-name()='VisualState' and @x:Name='$StateName']", $AppNamespace)) { Add-Failure "ScrollingIndicatorStates is missing $StateName." }
            }
            $TouchState = $IndicatorStates.SelectSingleNode("./*[local-name()='VisualState' and @x:Name='TouchIndicator']", $AppNamespace)
            $MouseState = $IndicatorStates.SelectSingleNode("./*[local-name()='VisualState' and @x:Name='MouseIndicator']", $AppNamespace)
            $NoState = $IndicatorStates.SelectSingleNode("./*[local-name()='VisualState' and @x:Name='NoIndicator']", $AppNamespace)
            if ($null -eq $TouchState -or $null -eq $TouchState.SelectSingleNode(".//*[local-name()='Setter' and @Target='HorizontalPanningRoot.Visibility' and @Value='Visible']", $AppNamespace)) { Add-Failure 'TouchIndicator must show the horizontal panning root.' }
            if ($null -eq $TouchState -or $null -eq $TouchState.SelectSingleNode(".//*[local-name()='Setter' and @Target='VerticalPanningRoot.Visibility' and @Value='Visible']", $AppNamespace)) { Add-Failure 'TouchIndicator must show the vertical panning root.' }
            if ($null -eq $MouseState -or $null -eq $MouseState.SelectSingleNode(".//*[local-name()='Setter' and @Target='HorizontalRoot.Visibility' and @Value='Visible']", $AppNamespace)) { Add-Failure 'MouseIndicator must restore the horizontal root.' }
            if ($null -eq $MouseState -or $null -eq $MouseState.SelectSingleNode(".//*[local-name()='Setter' and @Target='VerticalRoot.Visibility' and @Value='Visible']", $AppNamespace)) { Add-Failure 'MouseIndicator must restore the vertical root.' }
            if ($null -eq $NoState -or $null -eq $NoState.SelectSingleNode(".//*[local-name()='ObjectAnimationUsingKeyFrames' and @Storyboard.TargetName='VerticalRoot']", $AppNamespace)) { Add-Failure 'NoIndicator must collapse the vertical root.' }
            if ($null -eq $NoState -or $null -eq $NoState.SelectSingleNode(".//*[local-name()='ObjectAnimationUsingKeyFrames' and @Storyboard.TargetName='HorizontalRoot']", $AppNamespace)) { Add-Failure 'NoIndicator must collapse the horizontal root.' }
            if ($null -eq $NoState -or $null -eq $NoState.SelectSingleNode(".//*[local-name()='DoubleAnimation' and @Storyboard.TargetName='VerticalThumb' and @To='0']", $AppNamespace)) { Add-Failure 'NoIndicator must fade the vertical thumb.' }
            if ($null -eq $NoState -or $null -eq $NoState.SelectSingleNode(".//*[local-name()='DoubleAnimation' and @Storyboard.TargetName='HorizontalThumb' and @To='0']", $AppNamespace)) { Add-Failure 'NoIndicator must fade the horizontal thumb.' }
        }
        $ConsciousStates = $ScrollBarTemplate.SelectSingleNode(".//*[@x:Name='ConsciousStates']", $AppNamespace)
        if ($null -eq $ConsciousStates) { Add-Failure 'ScrollBar is missing ConsciousStates.' }
        else {
            $Expanded = $ConsciousStates.SelectSingleNode("./*[local-name()='VisualState' and @x:Name='Expanded']", $AppNamespace)
            if ($null -eq $Expanded -or $null -eq $Expanded.SelectSingleNode(".//*[local-name()='Setter' and @Target='VerticalThumb.Width' and @Value='{StaticResource ApplicationScrollBarExpandedThumbSize}']", $AppNamespace)) { Add-Failure 'Expanded must use the 8 DIP vertical thumb token.' }
            if ($null -eq $Expanded -or $null -eq $Expanded.SelectSingleNode(".//*[local-name()='Setter' and @Target='HorizontalThumb.Height' and @Value='{StaticResource ApplicationScrollBarExpandedThumbSize}']", $AppNamespace)) { Add-Failure 'Expanded must use the 8 DIP horizontal thumb token.' }
            if ($null -eq $Expanded -or $null -eq $Expanded.SelectSingleNode(".//*[local-name()='Setter' and @Target='VerticalThumb.Background' and @Value='{ThemeResource ScrollBarThumbFillPointerOver}']", $AppNamespace)) { Add-Failure 'Expanded must use the hover thumb color resource.' }
        }
        foreach ($Transform in @('HorizontalThumbTransform', 'VerticalThumbTransform')) {
            if ($null -eq $ScrollBarTemplate.SelectSingleNode(".//*[@x:Name='$Transform']", $AppNamespace)) { Add-Failure "ScrollBar is missing $Transform." }
        }
        $ThumbTemplate = $AppXaml.SelectSingleNode("//*[@x:Key='ApplicationScrollBarThumb']", $AppNamespace)
        if ($null -eq $ThumbTemplate) { Add-Failure 'Missing scrollbar thumb template.' }
        else {
            if ($null -eq $ThumbTemplate.SelectSingleNode(".//*[local-name()='Setter' and @Target='ThumbVisual.Background' and @Value='{ThemeResource ScrollBarThumbFillPointerOver}']", $AppNamespace)) { Add-Failure 'Scrollbar thumb pointer-over must use the hover color resource.' }
            if ($null -eq $ThumbTemplate.SelectSingleNode(".//*[local-name()='Setter' and @Target='ThumbVisual.Background' and @Value='{ThemeResource ScrollBarThumbFillPressed}']", $AppNamespace)) { Add-Failure 'Scrollbar thumb pressed must use the pressed color resource.' }
        }
        $ThumbSize = $AppXaml.SelectSingleNode("//*[@x:Key='ApplicationScrollBarThumbSize']", $AppNamespace)
        $ExpandedThumbSize = $AppXaml.SelectSingleNode("//*[@x:Key='ApplicationScrollBarExpandedThumbSize']", $AppNamespace)
        if ($null -eq $ThumbSize -or $ThumbSize.InnerText.Trim() -ne '6') { Add-Failure 'Scrollbar thumb size token must be 6.' }
        if ($null -eq $ExpandedThumbSize -or $ExpandedThumbSize.InnerText.Trim() -ne '8') { Add-Failure 'Expanded scrollbar thumb size token must be 8.' }
        if ($null -eq $VerticalThumb -or $VerticalThumb.GetAttribute('Width') -ne '{StaticResource ApplicationScrollBarThumbSize}') { Add-Failure 'Vertical scrollbar thumb must use the 6 DIP size token.' }
        if ($null -eq $HorizontalThumb -or $HorizontalThumb.GetAttribute('Height') -ne '{StaticResource ApplicationScrollBarThumbSize}') { Add-Failure 'Horizontal scrollbar thumb must use the 6 DIP size token.' }
    }
    Require-Setter $ScrollBarStyle 'MinWidth' '16' 'ScrollBar'
    Require-Setter $ScrollBarStyle 'MinHeight' '16' 'ScrollBar'
}
foreach ($ScrollKey in @('ScrollBarThumbFill', 'ScrollBarTrackFill', 'ScrollBarPanningThumbBackground')) {
    if ($null -eq $AppXaml.SelectSingleNode("//*[@x:Key='$ScrollKey']", $AppNamespace)) { Add-Failure "Missing $ScrollKey resource." }
}

# Palette: the dark surface, control, border, hover, text, muted and accent values requested by design.
$DarkDictionary = $AppXaml.SelectSingleNode("//*[local-name()='ResourceDictionary' and @x:Key='Dark']", $AppNamespace)
$HighContrastDictionary = $AppXaml.SelectSingleNode("//*[local-name()='ResourceDictionary' and @x:Key='HighContrast']", $AppNamespace)
if ($null -eq $DarkDictionary) { Add-Failure 'Missing Dark theme dictionary.' }
if ($null -eq $HighContrastDictionary) { Add-Failure 'Missing HighContrast theme dictionary.' }
if ($null -ne $DarkDictionary) {
    $Palette = [ordered]@{
        ThemeBackgroundColor = '#15181D'
        ThemeControlColor = '#20252D'
        ThemeBorderColor = '#3A424E'
        ThemeHoverColor = '#2B323D'
        ThemeForegroundColor = '#E6EBF2'
        ApplicationDimGrayColor = '#9FAAB8'
        ThemeNavIndicatorColor = '#C85C63'
    }
    foreach ($Key in $Palette.Keys) {
        $Node = $DarkDictionary.SelectSingleNode("./*[@x:Key='$Key']", $AppNamespace)
        if ($null -eq $Node -or $Node.InnerText.Trim() -ne $Palette[$Key]) {
            Add-Failure "Dark palette $Key expected $($Palette[$Key])."
        }
    }
}
$TextThickness = $AppXaml.SelectSingleNode("//*[@x:Key='TextControlBorderThemeThickness']", $AppNamespace)
$ComboThickness = $AppXaml.SelectSingleNode("//*[@x:Key='ComboBoxBorderThemeThickness']", $AppNamespace)
if ($null -eq $TextThickness -or $TextThickness.InnerText.Trim() -ne '1') { Add-Failure 'TextBox border theme thickness is not 1.' }
if ($null -eq $ComboThickness -or $ComboThickness.InnerText.Trim() -ne '1') { Add-Failure 'ComboBox border theme thickness is not 1.' }

foreach ($PageName in $PageNames) {
    $Page = $Xaml.SelectSingleNode("//*[@x:Name='$PageName']", $Namespace)
    if ($null -eq $Page) {
        Add-Failure "Missing page: $PageName"
        continue
    }
    $PageCount++

    $HeadingNodes = $Page.SelectNodes(".//*[local-name()='TextBlock' and @Style='{ThemeResource ApplicationSectionHeader}']")
    foreach ($HeadingNode in $HeadingNodes) {
        Add-Failure "Group heading remains in ${PageName}: $($HeadingNode.GetAttribute('Text'))"
    }

    $CardNodes = $Page.SelectNodes(".//*[local-name()='Border']")
    foreach ($CardNode in $CardNodes) {
        $Name = Get-XName $CardNode
        if ($Name -like '*_AccessOverlay') { continue }
        $Background = $CardNode.GetAttribute('Background')
        $CornerRadius = $CardNode.GetAttribute('CornerRadius')
        if (-not [string]::IsNullOrWhiteSpace($Background) -and $Background -notin @('Transparent', '{x:Null}') -and
            -not [string]::IsNullOrWhiteSpace($CornerRadius) -and $CornerRadius -ne '0') {
            Add-Failure "Card-like border remains in ${PageName}: $Name / $Background / $CornerRadius"
        }
        if ($Background -eq '{ThemeResource ThemeBackgroundBrush}' -or $Background -eq '{ThemeResource ThemeBackgroundDarkBrush}') {
            Add-Failure "Filled group surface remains in ${PageName}: $Name / $Background"
        }
    }

    $Separators = $Page.SelectNodes(".//*[local-name()='Border' and @Height='1']")
    foreach ($Separator in $Separators) {
        Add-Failure "Decorative separator remains in ${PageName}"
    }

    $Rows = @($Page.SelectNodes(".//*[local-name()='Border' and @Style='{ThemeResource FlatSettingRow}']"))
    foreach ($Row in $Rows) {
        $RowName = Get-XName $Row
        if ($RowName -eq 'grid_Fan_Graph') { continue }
        $RowCount++
        if ($RowName -eq 'grid_Fan_Range_Row') {
            $FanRangeRowCount++
            $RangeRoot = $Row.SelectSingleNode("./*[local-name()='Grid']")
            if ($null -eq $RangeRoot -or (Get-XName $RangeRoot) -ne 'grid_Fan_Range_Inputs') {
                Add-Failure "Fan range row is not built on the named two-group grid: $PageName / $RowName"
                continue
            }
            $RangeColumns = @($RangeRoot.SelectNodes("./*[local-name()='Grid.ColumnDefinitions']/*"))
            if ($RangeColumns.Count -ne 2 -or
                $RangeColumns[0].GetAttribute('Width') -ne '*' -or
                $RangeColumns[1].GetAttribute('Width') -ne '*') {
                Add-Failure "Fan range row does not use two equal star groups: $PageName / $RowName"
            }
            if ($RangeRoot.GetAttribute('ColumnSpacing') -ne '16' -or
                $RangeRoot.GetAttribute('HorizontalAlignment') -ne 'Stretch') {
                Add-Failure "Fan range row does not use a stretching 16 DIP two-group layout: $PageName / $RowName"
            }
            if ($null -ne $Row.SelectSingleNode(".//*[@Text='风扇转速范围']")) {
                Add-Failure "Redundant outer fan range label remains: $PageName / $RowName"
            }
            $RangeGroups = @($RangeRoot.SelectNodes("./*[local-name()='Grid']"))
            if ($RangeGroups.Count -ne 2) {
                Add-Failure "Fan range row does not have exactly two paired groups: $PageName / $RowName"
                continue
            }
            $RangeSpecs = @(
                @{ Slider = 'slider_Fan_Speed_Min'; Label = '最低转速（%）'; TextBox = 'textbox_Fan_Speed_Min'; Hint = 'textblock_Fan_Speed_Min_Value' },
                @{ Slider = 'slider_Fan_Speed_Max'; Label = '最高转速（%）'; TextBox = 'textbox_Fan_Speed_Max'; Hint = 'textblock_Fan_Speed_Max_Value' }
            )
            for ($RangeIndex = 0; $RangeIndex -lt 2; $RangeIndex++) {
                $Group = $RangeGroups[$RangeIndex]
                $Spec = $RangeSpecs[$RangeIndex]
                if ((Get-Column $Group) -ne $RangeIndex -or $Group.GetAttribute('MinWidth') -ne '0') {
                    Add-Failure "Fan range group is not in its equal star column: $PageName / $RowName / $($Spec.Slider)"
                }
                $GroupColumns = @($Group.SelectNodes("./*[local-name()='Grid.ColumnDefinitions']/*"))
                if ($GroupColumns.Count -ne 4 -or
                    $GroupColumns[0].GetAttribute('Width') -ne 'Auto' -or
                    $GroupColumns[1].GetAttribute('Width') -ne '*' -or
                    $GroupColumns[2].GetAttribute('Width') -ne '8' -or
                    $GroupColumns[3].GetAttribute('Width') -ne '160') {
                    Add-Failure "Fan range group does not use Auto/star/8/160 columns: $PageName / $RowName / $($Spec.Slider)"
                }
                $RangeLabel = $Group.SelectSingleNode("./*[local-name()='TextBlock' and @Style='{ThemeResource FlatSettingLabel}']")
                if ($null -eq $RangeLabel -or
                    $RangeLabel.GetAttribute('Text') -ne $Spec.Label -or
                    $RangeLabel.GetAttribute('TextWrapping') -ne 'NoWrap' -or
                    $RangeLabel.GetAttribute('Margin') -ne '0,0,8,0') {
                    Add-Failure "Fan range label is not compact and correctly named: $PageName / $RowName / $($Spec.Slider)"
                }
                $RangeSlider = $Group.SelectSingleNode("./*[local-name()='Slider']")
                $RangeSliderMinWidth = 0
                if ($null -eq $RangeSlider -or
                    (Get-XName $RangeSlider) -ne $Spec.Slider -or
                    $RangeSlider.GetAttribute('Style') -ne '{ThemeResource ApplicationSlider}' -or
                    $RangeSlider.GetAttribute('ValueChanged') -ne 'slider_FanRanged_ValueChanged' -or
                    (Get-Column $RangeSlider) -ne 1 -or
                    $RangeSlider.GetAttribute('HorizontalAlignment') -ne 'Stretch' -or
                    -not [int]::TryParse($RangeSlider.GetAttribute('MinWidth'), [ref]$RangeSliderMinWidth) -or
                    $RangeSliderMinWidth -lt 56 -or
                    $RangeSlider.GetAttribute('Height') -ne '32' -or
                    $RangeSlider.GetAttribute('Visibility') -eq 'Collapsed') {
                    Add-Failure "Fan range slider is not a visible stretching >=56 DIP adapter: $PageName / $RowName / $($Spec.Slider)"
                }
                $RangeTextBox = $Group.SelectSingleNode("./*[local-name()='TextBox']")
                if ($null -eq $RangeTextBox -or
                    (Get-XName $RangeTextBox) -ne $Spec.TextBox -or
                    (Get-Column $RangeTextBox) -ne 3 -or
                    $RangeTextBox.GetAttribute('Style') -ne '{ThemeResource ApplicationFanNumberBox}' -or
                    $RangeTextBox.GetAttribute('Width') -ne '160' -or
                    $RangeTextBox.GetAttribute('MinWidth') -ne '160' -or
                    $RangeTextBox.GetAttribute('Height') -ne '32' -or
                    $RangeTextBox.GetAttribute('HorizontalAlignment') -ne 'Right' -or
                    $RangeTextBox.GetAttribute('VerticalAlignment') -ne 'Center' -or
                    $RangeTextBox.GetAttribute('TextAlignment') -ne 'Center' -or
                    $RangeTextBox.GetAttribute('InputScope') -ne 'Number' -or
                    $RangeTextBox.GetAttribute('KeyDown') -ne 'textbox_FanInput_KeyDown' -or
                    $RangeTextBox.GetAttribute('LostFocus') -ne 'textbox_FanInput_LostFocus' -or
                    $RangeTextBox.GetAttribute('IsEnabled') -ne "{Binding ElementName=$($Spec.Slider), Path=IsEnabled, Mode=OneWay}") {
                    Add-Failure "Fan range number box is not the editable 160x32 adapter input: $PageName / $RowName / $($Spec.Slider)"
                }
                $RangeHint = $Group.SelectSingleNode(".//*[@x:Name='$($Spec.Hint)']", $Namespace)
                $ExpectedHintGroup = 'stackpanel_' + $Spec.TextBox.Substring('textbox_'.Length) + '_Hint'
                if ($null -eq $RangeHint -or
                    $RangeHint.GetAttribute('Style') -ne '{ThemeResource FlatValueHint}' -or
                    $RangeHint.GetAttribute('TextWrapping') -ne 'Wrap' -or
                    $RangeHint.GetAttribute('MaxWidth') -ne '300' -or
                    (Get-XName $RangeHint.ParentNode) -ne $ExpectedHintGroup -or
                    $RangeHint.ParentNode.GetAttribute('Visibility') -ne 'Collapsed' -or
                    $RangeHint.ParentNode.GetAttribute('Grid.Row') -ne '1' -or
                    $RangeHint.ParentNode.GetAttribute('Grid.ColumnSpan') -ne '4') {
                    Add-Failure "Fan range driver hint is missing or not kept below the controls: $PageName / $RowName / $($Spec.Slider)"
                }
                $FanRangeSliderCount++
                $FanRangeInputCount++
            }
            continue
        }
        $Outer = $Row.SelectSingleNode("./*[local-name()='Grid']")
        if ($null -eq $Outer) {
            Add-Failure "Setting row is not built on a Grid: $PageName / $RowName"
            continue
        }
        $OuterColumns = @($Outer.SelectNodes("./*[local-name()='Grid.ColumnDefinitions']/*"))
        if ($OuterColumns.Count -ne 2 -or $OuterColumns[0].GetAttribute('Width') -ne '300' -or $OuterColumns[1].GetAttribute('Width') -ne '*') {
            Add-Failure "Setting row does not use the shared 300/* columns: $PageName / $RowName"
            continue
        }
        if ($Outer.GetAttribute('ColumnSpacing') -ne '16') {
            Add-Failure "Setting row does not use a 16 DIP gap between description and control: $PageName / $RowName"
        }

        $Left = $Outer.SelectSingleNode("./*[local-name()='StackPanel' and @Grid.Column='0']")
        if ($null -eq $Left) {
            Add-Failure "Setting row has no left description stack: $PageName / $RowName"
            continue
        }
        $LeftLine = $Left.SelectSingleNode("./*[local-name()='StackPanel' and @Orientation='Horizontal']")
        if ($null -eq $LeftLine) {
            Add-Failure "Left description is not a compact label/icon line: $PageName / $RowName"
        }
        else {
            $LeftChildren = @(Get-ElementChildren $LeftLine)
            $Label = $LeftChildren | Where-Object { $_.LocalName -eq 'TextBlock' } | Select-Object -First 1
            if ($null -eq $Label -or [string]::IsNullOrWhiteSpace($Label.GetAttribute('Text'))) {
                Add-Failure "Left description has no label: $PageName / $RowName"
            }
            else {
                if ($Label.GetAttribute('Style') -ne '{ThemeResource FlatSettingLabel}' -or $Label.GetAttribute('TextWrapping') -ne 'Wrap') {
                    Add-Failure "Row label is not wrapping body text: $PageName / $RowName"
                }
                $MaxWidth = 0
                if (-not [int]::TryParse($Label.GetAttribute('MaxWidth'), [ref]$MaxWidth) -or $MaxWidth -lt 120 -or $MaxWidth -gt 300) {
                    Add-Failure "Row label has no usable shared-width constraint: $PageName / $RowName"
                }
            }
            $IconStack = $LeftChildren | Where-Object { $_.LocalName -eq 'StackPanel' } | Select-Object -First 1
            if ($null -ne $IconStack) {
                $IconIndex = [Array]::IndexOf($LeftChildren, $IconStack)
                $LabelIndex = [Array]::IndexOf($LeftChildren, $Label)
                if ($LabelIndex -lt 0 -or $IconIndex -lt $LabelIndex) {
                    Add-Failure "Help/status icons do not follow the label: $PageName / $RowName"
                }
                $Images = @($IconStack.SelectNodes("./*[local-name()='Image']"))
                $SawWarning = $false
                foreach ($Image in $Images) {
                    $Tooltip = $Image.GetAttribute('ToolTipService.ToolTip')
                    if ([string]::IsNullOrWhiteSpace($Tooltip)) {
                        Add-Failure "Description icon has no tooltip: $PageName / $RowName"
                    }
                    elseif ($Tooltip.StartsWith('*')) {
                        $SawWarning = $true
                    }
                    elseif ($SawWarning) {
                        Add-Failure "Help icon appears after the warning icon: $PageName / $RowName"
                    }
                }
            }
        }
        $Hint = $Left.SelectSingleNode("./*[local-name()='TextBlock' and (@Style='{ThemeResource FlatValueLabel}' or @Style='{ThemeResource FlatValueHint}')]")
        if ($null -ne $Hint) {
            if ($Hint.GetAttribute('TextWrapping') -ne 'Wrap' -or $Hint.GetAttribute('MaxWidth') -ne '300') {
                Add-Failure "Driver value hint is not confined to the left description: $PageName / $RowName"
            }
        }

        $RightChildren = @(Get-ElementChildren $Outer | Where-Object { (Get-Column $_) -eq 1 })
        if ($RightChildren.Count -ne 1) {
            Add-Failure "Setting row must have exactly one right control: $PageName / $RowName"
            continue
        }
        $Control = $RightChildren[0]
        if ($Control.LocalName -eq 'ToggleSwitch') {
            $ToggleCount++
            if ($Control.GetAttribute('Style') -ne '{ThemeResource FlatToggleSwitch}' -or
                $Control.GetAttribute('Width') -ne '44' -or
                $Control.GetAttribute('MinWidth') -ne '0' -or
                $Control.GetAttribute('HorizontalAlignment') -ne 'Right' -or
                $Control.GetAttribute('OnContent') -ne '' -or
                $Control.GetAttribute('OffContent') -ne '') {
                Add-Failure "Toggle is not a compact 44 DIP right-aligned control: $PageName / $(Get-XName $Control)"
            }
            if ([string]::IsNullOrWhiteSpace($Control.GetAttribute('AutomationProperties.Name'))) {
                Add-Failure "Toggle has no automation label: $PageName / $(Get-XName $Control)"
            }
            if ($Control.SelectSingleNode("./*[local-name()='Image' or local-name()='TextBlock']")) {
                Add-Failure "Toggle row has description content in the right area: $PageName / $(Get-XName $Control)"
            }
        }
        elseif ($Control.LocalName -eq 'ComboBox') {
            $ComboCount++
            if ($Control.GetAttribute('Width') -ne '160' -or
                $Control.GetAttribute('MinWidth') -ne '160' -or
                $Control.GetAttribute('Height') -ne '32' -or
                $Control.GetAttribute('HorizontalAlignment') -ne 'Right') {
                Add-Failure "ComboBox is not a 160x32 right-aligned control: $PageName / $(Get-XName $Control)"
            }
            if ($Control.SelectSingleNode("./*[local-name()='Image' or local-name()='TextBlock']")) {
                Add-Failure "ComboBox row has description content in the right area: $PageName / $(Get-XName $Control)"
            }
        }
        elseif ($Control.LocalName -eq 'Grid' -and $Control.SelectSingleNode("./*[local-name()='Slider']")) {
            $SliderRowCount++
            $SliderColumns = @($Control.SelectNodes("./*[local-name()='Grid.ColumnDefinitions']/*"))
            if ($SliderColumns.Count -ne 3 -or
                $SliderColumns[0].GetAttribute('Width') -ne '*' -or
                $SliderColumns[1].GetAttribute('Width') -ne '12' -or
                $SliderColumns[2].GetAttribute('Width') -ne '160') {
                Add-Failure "Slider row does not use the shared star/12/160 columns: $PageName / $RowName"
            }
            $Slider = $Control.SelectSingleNode("./*[local-name()='Slider']")
            $NumberBox = $Control.SelectSingleNode("./*[local-name()='TextBox']")
            if ($null -eq $NumberBox) {
                Add-Failure "Slider row has no numeric TextBox: $PageName / $RowName"
                continue
            }
            if ((Get-Column $Slider) -ne 0 -or (Get-Column $NumberBox) -ne 2) {
                Add-Failure "Slider/number box are not in the shared star/160 columns: $PageName / $RowName"
            }
            if ($Slider.GetAttribute('HorizontalAlignment') -ne 'Stretch' -or
                $Slider.GetAttribute('Margin') -ne '0' -or
                [int]$Slider.GetAttribute('MinWidth') -lt 120 -or
                $Slider.GetAttribute('Height') -ne '32') {
                Add-Failure "Slider does not stretch from the shared start with a usable minimum: $PageName / $RowName"
            }
            if ($NumberBox.GetAttribute('Style') -ne '{ThemeResource ApplicationSliderNumberBox}' -or
                $NumberBox.GetAttribute('Width') -ne '160' -or
                $NumberBox.GetAttribute('MinWidth') -ne '160' -or
                $NumberBox.GetAttribute('Height') -ne '32' -or
                $NumberBox.GetAttribute('HorizontalAlignment') -ne 'Right' -or
                $NumberBox.GetAttribute('TextAlignment') -ne 'Center') {
                Add-Failure "Numeric box is not 160x32 right-aligned with centered text: $PageName / $(Get-XName $NumberBox)"
            }
            $LeftHint = $Control.SelectSingleNode(".//*[local-name()='TextBlock' and @Style='{ThemeResource FlatValueLabel}']")
            if ($null -ne $LeftHint) {
                Add-Failure "Driver value hint still sits in the slider region: $PageName / $RowName"
            }
        }
        elseif ($Control.LocalName -eq 'Grid' -and (Get-XName $Control) -like 'grid_Fan_*') {
            $FanPairRowCount++
            $PairName = Get-XName $Control
            $PairColumns = @($Control.SelectNodes("./*[local-name()='Grid.ColumnDefinitions']/*"))
            if ($PairColumns.Count -ne 5 -or
                $PairColumns[0].GetAttribute('Width') -ne 'Auto' -or
                $PairColumns[1].GetAttribute('Width') -ne '160' -or
                $PairColumns[2].GetAttribute('Width') -ne '32' -or
                $PairColumns[3].GetAttribute('Width') -ne 'Auto' -or
                $PairColumns[4].GetAttribute('Width') -ne '160') {
                Add-Failure "Fan paired inputs do not use the equal Auto/160/32/Auto/160 columns: $PageName / $RowName"
            }
            if ($Control.GetAttribute('HorizontalAlignment') -ne 'Right') {
                Add-Failure "Fan paired inputs are not right-aligned: $PageName / $RowName"
            }
            if ($Control.SelectSingleNode(".//*[local-name()='Slider']")) {
                Add-Failure "Fan paired inputs still contain a visible slider: $PageName / $RowName"
            }

            $PairLabels = @($Control.SelectNodes("./*[local-name()='TextBlock']"))
            $PairInputs = @($Control.SelectNodes("./*[local-name()='TextBox']"))
            if ($PairLabels.Count -ne 2 -or $PairInputs.Count -ne 2) {
                Add-Failure "Fan input-only pair row is not two labeled inputs: $PageName / $RowName"
                continue
            }
            foreach ($PairLabel in $PairLabels) {
                if ($PairLabel.GetAttribute('Style') -ne '{ThemeResource FlatSettingLabel}' -or
                    $PairLabel.GetAttribute('TextWrapping') -ne 'NoWrap' -or
                    $PairLabel.GetAttribute('Margin') -ne '0,0,8,0' -or
                    [string]::IsNullOrWhiteSpace($PairLabel.GetAttribute('Text'))) {
                    Add-Failure "Fan input label is not compact or unlabeled: $PageName / $RowName"
                }
            }
            foreach ($PairInput in $PairInputs) {
                $InputColumn = Get-Column $PairInput
                if ($InputColumn -ne 1 -and $InputColumn -ne 4) {
                    Add-Failure "Fan input is not in a 160 DIP control column: $PageName / $RowName"
                }
                if ($PairInput.GetAttribute('Style') -ne '{ThemeResource ApplicationFanNumberBox}' -or
                    $PairInput.GetAttribute('Width') -ne '160' -or
                    $PairInput.GetAttribute('MinWidth') -ne '160' -or
                    $PairInput.GetAttribute('Height') -ne '32' -or
                    $PairInput.GetAttribute('HorizontalAlignment') -ne 'Right' -or
                    $PairInput.GetAttribute('VerticalAlignment') -ne 'Center' -or
                    $PairInput.GetAttribute('TextAlignment') -ne 'Center' -or
                    $PairInput.GetAttribute('InputScope') -ne 'Number' -or
                    $PairInput.GetAttribute('KeyDown') -ne 'textbox_FanInput_KeyDown' -or
                    $PairInput.GetAttribute('LostFocus') -ne 'textbox_FanInput_LostFocus' -or
                    $PairInput.GetAttribute('IsEnabled') -notmatch '^\{Binding ElementName=slider_Fan_') {
                    Add-Failure "Fan input is not an editable centered 160x32 number box tied to its adapter: $PageName / $(Get-XName $PairInput)"
                }
            }

            $LeftLine = $Left.SelectSingleNode("./*[local-name()='StackPanel' and @Orientation='Horizontal']")
            $LeftLabelNode = if ($null -ne $LeftLine) { $LeftLine.SelectSingleNode("./*[local-name()='TextBlock']") } else { $null }
            $LeftText = if ($null -ne $LeftLabelNode) { $LeftLabelNode.GetAttribute('Text') } else { '' }
            if ($LeftText -match '^曲线点([1-5])$') {
                $CurveIndex = [int]$Matches[1] - 1
                if ($PairName -ne "grid_Fan_Curve_Inputs_$CurveIndex" -or
                    $PairLabels[0].GetAttribute('Text') -ne '温度（°C）' -or
                    $PairLabels[1].GetAttribute('Text') -ne '转速（%）' -or
                    (Get-XName $PairInputs[0]) -ne "textbox_Fan_Temp_$CurveIndex" -or
                    (Get-XName $PairInputs[1]) -ne "textbox_Fan_Speed_$CurveIndex" -or
                    (Get-Column $PairInputs[1]) -ne 4) {
                    Add-Failure "Fan curve point row does not pair temperature then speed: $PageName / $RowName"
                }
            }
            else {
                Add-Failure "Fan input-only pair row has an unexpected description: $PageName / $RowName / $LeftText"
            }
            $FanPairInputCount += $PairInputs.Count
        }
        elseif ($Control.LocalName -eq 'StackPanel' -and $Control.SelectSingleNode(".//*[local-name()='Button']")) {
            $ButtonRowCount++
            if ($Control.GetAttribute('HorizontalAlignment') -ne 'Right') {
                Add-Failure "Button row is not right aligned: $PageName / $RowName"
            }
        }
        else {
            Add-Failure "Unrecognized right-side control: $PageName / $RowName / $($Control.LocalName)"
        }
    }
}

$FanAdapterPanel = $Xaml.SelectSingleNode("//*[@x:Name='stackpanel_Fan_AdapterSliders']", $Namespace)
if ($null -eq $FanAdapterPanel -or $FanAdapterPanel.GetAttribute('Visibility') -ne 'Collapsed') {
    Add-Failure 'Missing collapsed fan adapter slider panel.'
}
else {
    $ExpectedFanAdapters = @(
        'slider_Fan_Temp_0', 'slider_Fan_Temp_1', 'slider_Fan_Temp_2', 'slider_Fan_Temp_3', 'slider_Fan_Temp_4',
        'slider_Fan_Speed_0', 'slider_Fan_Speed_1', 'slider_Fan_Speed_2', 'slider_Fan_Speed_3', 'slider_Fan_Speed_4'
    )
    foreach ($AdapterName in $ExpectedFanAdapters) {
        $Adapter = $FanAdapterPanel.SelectSingleNode(".//*[@x:Name='$AdapterName']", $Namespace)
        if ($null -eq $Adapter) {
            Add-Failure "Missing hidden fan adapter slider: $AdapterName"
            continue
        }
        $ExpectedHandler = 'slider_FanCurve_ValueChanged'
        if ($Adapter.LocalName -ne 'Slider' -or
            $Adapter.GetAttribute('Visibility') -ne 'Collapsed' -or
            $Adapter.GetAttribute('Style') -ne '{ThemeResource ApplicationSlider}' -or
            $Adapter.GetAttribute('ValueChanged') -ne $ExpectedHandler) {
            Add-Failure "Fan adapter slider is not a collapsed adapter to the existing handler: $AdapterName"
        }
        $HiddenFanAdapterCount++
    }
}

$FanGraph = $Xaml.SelectSingleNode("//*[@x:Name='grid_Fan_Graph']", $Namespace)
if ($null -eq $FanGraph) {
    Add-Failure 'Missing fan curve graph.'
}
else {
    if ($FanGraph.LocalName -ne 'Border' -or
        $FanGraph.GetAttribute('BorderThickness') -ne '1' -or
        $FanGraph.GetAttribute('Background') -ne 'Transparent' -or
        $FanGraph.GetAttribute('CornerRadius') -ne '6' -or
        $FanGraph.GetAttribute('Padding') -ne '12' -or
        $FanGraph.GetAttribute('HorizontalAlignment') -ne 'Stretch' -or
        $FanGraph.GetAttribute('BorderBrush') -ne '{ThemeResource ThemeBorderBrush}') {
        Add-Failure 'Fan graph is not a transparent, full-width, 6 DIP rounded frame with 12 DIP padding.'
    }
    if ($FanGraph.SelectSingleNode(".//*[local-name()='Slider']")) {
        Add-Failure 'Fan graph still contains a slider.'
    }
    $FanViewbox = $FanGraph.SelectSingleNode(".//*[@x:Name='viewbox_Fan_Plot']", $Namespace)
    if ($null -eq $FanViewbox -or $FanViewbox.GetAttribute('Stretch') -ne 'Uniform') {
        Add-Failure 'Fan graph has no uniformly scaled Viewbox for the existing 500x100 plot.'
    }
    else {
        $FanRoot = $FanViewbox.SelectSingleNode("./*[local-name()='Grid']", $Namespace)
        if ($null -eq $FanRoot -or $FanRoot.GetAttribute('Width') -ne '512' -or $FanRoot.GetAttribute('Height') -ne '112') {
            Add-Failure 'Fan Viewbox does not preserve the 512x112 root with a 6 DIP plot origin.'
        }
        $FanPlot = if ($null -ne $FanRoot) { $FanRoot.SelectSingleNode(".//*[local-name()='Canvas' and @Width='500' and @Height='100']", $Namespace) } else { $null }
        if ($null -eq $FanPlot -or $FanPlot.GetAttribute('Margin') -ne '6' -or
            $FanPlot.GetAttribute('HorizontalAlignment') -ne 'Left' -or $FanPlot.GetAttribute('VerticalAlignment') -ne 'Top') {
            Add-Failure 'Fan plot is not the logical 500x100 Canvas at the 6 DIP root origin.'
        }
        if ($FanViewbox.SelectNodes(".//*[@Style='{ThemeResource FanAxisTick}' or @Style='{ThemeResource FanAxisLabel}']", $Namespace).Count -gt 0) {
            Add-Failure 'Fan axis labels are inside the scaled plot instead of fixed-size outside labels.'
        }
        foreach ($PlotName in @('border_Fan_Zero_Rpm_Range', 'grid_Fan_Zero_Rpm_Line_Gpu', 'grid_Fan_Zero_Rpm_Line_Profile', 'polyline_Fan_Lines_Gpu', 'polyline_Fan_Lines_Profile')) {
            if ($null -eq $FanPlot.SelectSingleNode(".//*[@x:Name='$PlotName']", $Namespace)) {
                Add-Failure "Fan plot is missing $PlotName."
            }
        }
        foreach ($LineName in @('grid_Fan_Zero_Rpm_Line_Gpu', 'grid_Fan_Zero_Rpm_Line_Profile')) {
            $ZeroLine = $FanPlot.SelectSingleNode(".//*[@x:Name='$LineName']", $Namespace)
            if ($null -ne $ZeroLine -and ($ZeroLine.GetAttribute('X1') -ne '0' -or $ZeroLine.GetAttribute('X2') -ne '0' -or
                $ZeroLine.GetAttribute('Y1') -ne '0' -or $ZeroLine.GetAttribute('Y2') -ne '100')) {
                Add-Failure "Fan zero RPM line is not in the 500x100 plot coordinates: $LineName"
            }
        }
        for ($PointIndex = 0; $PointIndex -lt 5; $PointIndex++) {
            $PointNode = $FanRoot.SelectSingleNode(".//*[@x:Name='ellipse_Fan_Point_$PointIndex']", $Namespace)
            if ($null -eq $PointNode) {
                Add-Failure "Fan plot is missing ellipse_Fan_Point_$PointIndex."
            }
            elseif ($PointNode.GetAttribute('Width') -ne '6' -or $PointNode.GetAttribute('Height') -ne '6' -or
                $PointNode.GetAttribute('Fill') -ne '{ThemeResource ApplicationAccentDimBrush}') {
                Add-Failure "Fan point node is not a circular 6 DIP target marker: $PointIndex"
            }
        }
    }
    foreach ($AxisSpec in @(
        @('textblock_Fan_Axis_X_0', '0', '30'),
        @('textblock_Fan_Axis_X_20', '20', '30'),
        @('textblock_Fan_Axis_X_40', '40', '30'),
        @('textblock_Fan_Axis_X_60', '60', '30'),
        @('textblock_Fan_Axis_X_80', '80', '30'),
        @('textblock_Fan_Axis_X_100', '100', '30')
    )) {
        $AxisNode = $FanGraph.SelectSingleNode(".//*[@x:Name='$($AxisSpec[0])']", $Namespace)
        if ($null -eq $AxisNode -or $AxisNode.GetAttribute('Text') -ne $AxisSpec[1] -or
            $AxisNode.GetAttribute('Width') -ne $AxisSpec[2] -or
            $AxisNode.GetAttribute('Style') -ne '{ThemeResource FanAxisTick}') {
            Add-Failure "Fan x-axis tick is missing or malformed: $($AxisSpec[0])"
        }
    }
    foreach ($AxisSpec in @(
        @('textblock_Fan_Axis_Y_100', '100'),
        @('textblock_Fan_Axis_Y_80', '80'),
        @('textblock_Fan_Axis_Y_60', '60'),
        @('textblock_Fan_Axis_Y_40', '40'),
        @('textblock_Fan_Axis_Y_20', '20'),
        @('textblock_Fan_Axis_Y_0', '0')
    )) {
        $AxisNode = $FanGraph.SelectSingleNode(".//*[@x:Name='$($AxisSpec[0])']", $Namespace)
        if ($null -eq $AxisNode -or $AxisNode.GetAttribute('Text') -ne $AxisSpec[1] -or
            $AxisNode.GetAttribute('Width') -ne '26' -or $AxisNode.GetAttribute('TextAlignment') -ne 'Right' -or
            $AxisNode.GetAttribute('Style') -ne '{ThemeResource FanAxisTick}') {
            Add-Failure "Fan y-axis tick is missing or malformed: $($AxisSpec[0])"
        }
    }
    if ($null -eq $FanGraph.SelectSingleNode(".//*[@x:Name='textblock_Fan_Y_Title' and @Grid.Row='1' and @VerticalAlignment='Center']", $Namespace) -or
        $null -eq $FanGraph.SelectSingleNode(".//*[@Text='温度（°C）']", $Namespace)) {
        Add-Failure 'Fan chart axis units are missing.'
    }
    foreach ($LegendText in @('当前曲线', '目标曲线')) {
        if ($null -eq $FanGraph.SelectSingleNode(".//*[@Text='$LegendText']", $Namespace)) {
            Add-Failure "Fan chart legend text is missing: $LegendText"
        }
    }
    $CurrentCurve = $FanGraph.SelectSingleNode(".//*[@x:Name='polyline_Fan_Lines_Gpu']", $Namespace)
    $TargetCurve = $FanGraph.SelectSingleNode(".//*[@x:Name='polyline_Fan_Lines_Profile']", $Namespace)
    if ($null -eq $CurrentCurve -or $CurrentCurve.GetAttribute('Stroke') -ne '{ThemeResource ApplicationDimGrayBrush}' -or
        $null -eq $TargetCurve -or $TargetCurve.GetAttribute('Stroke') -ne '{ThemeResource ApplicationAccentDimBrush}') {
        Add-Failure 'Fan curves do not use the neutral current and muted target colors.'
    }
    foreach ($TooltipName in @('polyline_Fan_Lines_Gpu', 'polyline_Fan_Lines_Profile', 'border_Fan_Zero_Rpm_Range', 'grid_Fan_Zero_Rpm_Line_Gpu', 'grid_Fan_Zero_Rpm_Line_Profile', 'ellipse_Fan_Point_0', 'ellipse_Fan_Point_1', 'ellipse_Fan_Point_2', 'ellipse_Fan_Point_3', 'ellipse_Fan_Point_4')) {
        $TooltipNode = $FanGraph.SelectSingleNode(".//*[@x:Name='$TooltipName']", $Namespace)
        if ($null -ne $TooltipNode -and [string]::IsNullOrWhiteSpace($TooltipNode.GetAttribute('ToolTipService.ToolTip'))) {
            Add-Failure "Fan graph tooltip is missing on $TooltipName."
        }
    }
    for ($PointIndex = 0; $PointIndex -lt 5; $PointIndex++) {
        $ExpectedMargin = "ellipse_Fan_Point_$PointIndex().Margin(Thickness(fanTemp$PointIndex + 3, fanSpeed$PointIndex + 3, 0, 0));"
        if (-not $AdlxEventsFans.Contains($ExpectedMargin)) {
            Add-Failure "Fan point $PointIndex is not centered with the 6 DIP node offset."
        }
    }
    if (-not $AdlxEventsFans.Contains('Canvas::SetLeft(grid_Fan_Zero_Rpm_Line_Gpu(), fanZeroLinePosition)') -or
        -not $AdlxEventsFans.Contains('Canvas::SetLeft(grid_Fan_Zero_Rpm_Line_Profile(), fanZeroLinePosition)') -or
        -not $AdlxEventsFans.Contains('border_Fan_Zero_Rpm_Range().Width(fanZeroLinePosition);') -or
        $AdlxEventsFans.Contains('fanZeroLinePosition + 6')) {
        Add-Failure 'Fan zero RPM overlay is not aligned to the 500x100 plot coordinates.'
    }
    if (-not $MainPageCpp.Contains('UpdateFanChartAxisLayout') -or
        -not $MainPageCpp.Contains('viewbox_Fan_Plot_SizeChanged') -or
        -not $MainPageCpp.Contains('textblock_Fan_Axis_Y_100()') -or
        -not $MainPageCpp.Contains('plotOrigin + chartWidth * 0.20') -or
        -not $MainPageCpp.Contains('plotOrigin + chartWidth')) {
        Add-Failure 'Fan axis labels are not laid out against the 512x112 plot origin.'
    }
}

$FanCurvePanel = $Xaml.SelectSingleNode("//*[@x:Name='stackpanel_FanControl_Curve']", $Namespace)
$FanRangePanel = $Xaml.SelectSingleNode("//*[@x:Name='stackpanel_FanControl_RangedFanSpeed']", $Namespace)
if ($null -eq $FanCurvePanel -or $null -eq $FanRangePanel) {
    Add-Failure 'Missing fan curve or fan range section for final order validation.'
}
else {
    $FanSectionNames = @($FanCurvePanel.ParentNode.ChildNodes | Where-Object { $_.NodeType -eq 'Element' } | ForEach-Object { Get-XName $_ })
    $CurveSectionIndex = [Array]::IndexOf($FanSectionNames, 'stackpanel_FanControl_Curve')
    $RangeSectionIndex = [Array]::IndexOf($FanSectionNames, 'stackpanel_FanControl_RangedFanSpeed')
    if ($CurveSectionIndex -lt 0 -or $RangeSectionIndex -lt 0 -or $CurveSectionIndex -ge $RangeSectionIndex) {
        Add-Failure 'Fan range slider section is not after the fan curve section.'
    }
    $ExpectedCurveOrder = @('grid_Fan_Graph', 'grid_Fan_Curve_Row_0', 'grid_Fan_Curve_Row_1', 'grid_Fan_Curve_Row_2', 'grid_Fan_Curve_Row_3', 'grid_Fan_Curve_Row_4')
    $ActualCurveOrder = @(Get-ElementChildren $FanCurvePanel | ForEach-Object { Get-XName $_ })
    if (($ActualCurveOrder -join '|') -ne ($ExpectedCurveOrder -join '|')) {
        Add-Failure "Fan curve section is not the chart followed by five input-only rows: $($ActualCurveOrder -join ', ')"
    }
}

# Fan value hints are named but collapsed, and the editable boxes expose the same driver value as a tooltip.
for ($CurveIndex = 0; $CurveIndex -lt 5; $CurveIndex++) {
    $HintGroup = $Xaml.SelectSingleNode("//*[@x:Name='stackpanel_Fan_Curve_${CurveIndex}_Hint']", $Namespace)
    if ($null -eq $HintGroup -or $HintGroup.GetAttribute('Visibility') -ne 'Collapsed') {
        Add-Failure "Fan curve hint group is missing or visible: $CurveIndex"
        continue
    }
    foreach ($Kind in @('Temp', 'Speed')) {
        $HintName = "textblock_Fan_Curve_${CurveIndex}_${Kind}_Value"
        $HintNode = $HintGroup.SelectSingleNode(".//*[@x:Name='$HintName']", $Namespace)
        if ($null -eq $HintNode -or $HintNode.GetAttribute('Style') -ne '{ThemeResource FlatValueHint}') {
            Add-Failure "Fan curve hint is not a collapsed value hint: $HintName"
        }
        $InputName = if ($Kind -eq 'Temp') { "textbox_Fan_Temp_$CurveIndex" } else { "textbox_Fan_Speed_$CurveIndex" }
        $InputNode = $Xaml.SelectSingleNode("//*[@x:Name='$InputName']", $Namespace)
        if ($null -eq $InputNode -or $InputNode.GetAttribute('ToolTipService.ToolTip') -ne "{Binding ElementName=$HintName, Path=Text}") {
            Add-Failure "Fan curve input does not expose its driver value as a tooltip: $InputName"
        }
    }
}
if ($null -ne $Xaml.SelectSingleNode("//*[@Text='驱动：' or @Text=' / ' or @Text='驱动：/']", $Namespace)) {
    Add-Failure 'Fan driver placeholder prefix or slash remains in the page.'
}

# Dependent rows are named wrappers that start collapsed; the parent controls keep their existing names/events.
$DependentGroups = [ordered]@{
    'stackpanel_FrameGenAlgorithm_Dependent' = @('combobox_FrameGenAlgorithm')
    'stackpanel_RadeonChill_Fps' = @('slider_RadeonChill_Min', 'slider_RadeonChill_Max')
    'stackpanel_Frtc_Fps' = @('slider_Frtc_FrameRateTarget')
    'stackpanel_RadeonBoost_MinResolution' = @('slider_RadeonBoost_MinResolution')
    'stackpanel_RadeonImageSharpening1_Sharpening' = @('slider_RadeonImageSharpening1_Sharpening')
    'stackpanel_RadeonImageSharpening2_Dependent' = @('toggleswitch_RadeonImageSharpening2_Desktop', 'slider_RadeonImageSharpening2_Sharpening')
    'stackpanel_AntiAliasing_Dependent' = @('combobox_AntiAliasingMethod', 'combobox_AntiAliasingLevel', 'toggleswitch_AntiAliasingEnhancedQuality')
    'stackpanel_TessellationLevel_Dependent' = @('combobox_Tessellation_Level')
    'stackpanel_Display_Gamma_Dependent' = @('slider_Display_GammaBase', 'textbox_Display_GammaBase', 'slider_Display_GammaRed', 'textbox_Display_GammaRed', 'slider_Display_GammaGreen', 'textbox_Display_GammaGreen', 'slider_Display_GammaBlue', 'textbox_Display_GammaBlue')
    'stackpanel_Display_ColorTemperature_Kelvin_Dependent' = @('slider_Display_ColorTemperature_Kelvin', 'textbox_Display_ColorTemperature')
    'stackpanel_Display_CVDC_Dependent' = @('slider_Display_Protanopia', 'textbox_Display_Protanopia', 'slider_Display_Deuteranopia', 'textbox_Display_Deuteranopia', 'slider_Display_Tritanopia', 'textbox_Display_Tritanopia')
    'stackpanel_Display_VariBright_Level_Dependent' = @('combobox_Display_VariBright_Level')
}
foreach ($GroupName in $DependentGroups.Keys) {
    $GroupNode = $Xaml.SelectSingleNode("//*[@x:Name='$GroupName']", $Namespace)
    if ($null -eq $GroupNode) {
        Add-Failure "Missing dependent row wrapper: $GroupName"
        continue
    }
    if ($GroupNode.GetAttribute('Visibility') -ne 'Collapsed') {
        Add-Failure "Dependent row wrapper must start collapsed: $GroupName"
    }
    foreach ($ChildName in $DependentGroups[$GroupName]) {
        if ($null -eq $GroupNode.SelectSingleNode(".//*[@x:Name='$ChildName']", $Namespace)) {
            Add-Failure "Dependent wrapper $GroupName does not contain $ChildName"
        }
    }
}
$DisplayDependentWiring = [ordered]@{
    'stackpanel_Display_Gamma_Dependent' = 'toggleswitch_Display_GammaControl'
    'stackpanel_Display_ColorTemperature_Kelvin_Dependent' = 'toggleswitch_Display_ColorTemperature_Control'
    'stackpanel_Display_CVDC_Dependent' = 'toggleswitch_Display_CVDC_Control'
    'stackpanel_Display_VariBright_Level_Dependent' = 'toggleswitch_Display_VariBright'
}
foreach ($WrapperName in $DisplayDependentWiring.Keys) {
    $ParentToggle = $DisplayDependentWiring[$WrapperName]
    $ExpectedCall = "ApplyDependentRow($WrapperName(), $ParentToggle().IsOn(), animate)"
    if (-not $MainPageCpp.Contains($ExpectedCall)) {
        Add-Failure "Display dependent wrapper is not refreshed by UpdateDependentSettingsInterface: $WrapperName"
    }
    $HandlerPattern = 'void MainPage::' + [regex]::Escape($ParentToggle) + '_Toggled\([\s\S]*?UpdateDependentSettingsInterface\(!suppressDependentFoldAnimation\);\s*//Check if saving is disabled\s*if \(disable_saving\) \{ return; \}'
    if ($AdlxEventsDisplay -notmatch $HandlerPattern) {
        Add-Failure "Display dependent wrapper is not refreshed before the disable_saving return: $ParentToggle"
    }
}
# Interaction polish: page entrance, toggle transition, dropdown timing and fan editing viewport.
$FlatToggle = $Xaml.SelectSingleNode("//*[@x:Key='FlatToggleSwitch']", $Namespace)
if ($null -eq $FlatToggle) {
    Add-Failure 'Missing FlatToggleSwitch template.'
}
else {
    $SwitchKnob = $FlatToggle.SelectSingleNode(".//*[@x:Name='SwitchKnob']", $Namespace)
    if ($null -eq $SwitchKnob -or $SwitchKnob.GetAttribute('HorizontalAlignment') -ne 'Left') {
        Add-Failure 'Toggle switch knob no longer uses the stable left alignment.'
    }
    foreach ($StateName in @('Off', 'On')) {
        $Transition = $FlatToggle.SelectSingleNode(".//*[local-name()='VisualTransition' and @To='$StateName']", $Namespace)
        if ($null -eq $Transition -or $Transition.GetAttribute('GeneratedDuration') -ne '0:0:0.1') {
            Add-Failure "Toggle $StateName is missing its interruptible 100 ms transition."
        }
        $State = $FlatToggle.SelectSingleNode(".//*[@x:Name='$StateName']", $Namespace)
        $Setter = $State.SelectSingleNode(".//*[local-name()='DoubleAnimation' and @Storyboard.TargetName='KnobTranslateTransform' and @Storyboard.TargetProperty='X']")
        $Expected = if ($StateName -eq 'Off') { '0' } else { '20' }
        if ($null -eq $Setter -or $Setter.GetAttribute('To') -ne $Expected -or $Setter.GetAttribute('Duration') -ne '0') {
            Add-Failure "Toggle $StateName final position is incorrect."
        }
    }
    $AlignmentSetters = @($FlatToggle.SelectNodes(".//*[local-name()='Setter' and @Target='SwitchKnob.HorizontalAlignment']", $Namespace))
    if ($AlignmentSetters.Count -ne 0) {
        Add-Failure 'Toggle switch still snaps the knob with horizontal alignment setters.'
    }
}

$FanNumberBox = $AppXaml.SelectSingleNode("//*[@x:Key='ApplicationFanNumberBox']", $AppNamespace)
if ($null -eq $FanNumberBox) {
    Add-Failure 'Missing ApplicationFanNumberBox template.'
}
else {
    $FanContent = $FanNumberBox.SelectSingleNode(".//*[@x:Name='ContentElement']", $AppNamespace)
    $FanContentOk = $null -ne $FanContent -and
        $FanContent.GetAttribute('Height') -eq '18' -and
        $FanContent.GetAttribute('VerticalAlignment') -eq 'Center' -and
        $FanContent.GetAttribute('HorizontalAlignment') -eq 'Stretch' -and
        $FanContent.GetAttribute('Padding') -eq '0'
    if (-not $FanContentOk) {
        Add-Failure 'Fan number ContentElement is not an 18 DIP vertically centered full-width editing viewport.'
    }
}
$FanInputs = @($Xaml.SelectNodes("//*[@x:Name]", $Namespace) | Where-Object {
    $_.GetAttribute('Name', 'http://schemas.microsoft.com/winfx/2006/xaml') -like 'textbox_Fan_*' -and
    $_.GetAttribute('Style') -eq '{ThemeResource ApplicationFanNumberBox}'
})
if ($FanInputs.Count -ne 12) {
    Add-Failure "Expected 12 fan number boxes using ApplicationFanNumberBox, found $($FanInputs.Count)."
}

foreach ($RequiredCode in @(
    'AnimatePageEntrance',
    'FinalizePageTransition',
    'pageTransitionStoryboard',
    'pageTransitionGeneration',
    'if (!SystemAnimationsEnabled()) { return; }',
    'pageEntranceMilliseconds = 120',
    'pageEntranceOffset = 6',
    'AnimateComboBoxPopup',
    'popupOpenMilliseconds = 90',
    'popupCloseMilliseconds = 70',
    'ChildTransitions',
    'popupTransitions.Clear()',
    'childTransitions.Clear()',
    'AnimatePageEntrance(selectedPage)',
    'DropDownOpened',
    'DropDownClosed'
)) {
    if (-not $MainPageCpp.Contains($RequiredCode)) {
        Add-Failure "MainPage.cpp is missing interaction polish implementation: $RequiredCode"
    }
}

foreach ($RowsName in @('stackpanel_Graphics_Rows', 'stackpanel_Fans_Rows')) {
    $RowsNode = $Xaml.SelectSingleNode("//*[@x:Name='$RowsName']", $Namespace)
    $Transition = if ($null -ne $RowsNode) {
        $RowsNode.SelectSingleNode("./*[local-name()='StackPanel.ChildrenTransitions']/*[local-name()='TransitionCollection']/*[local-name()='RepositionThemeTransition']")
    } else { $null }
    if ($null -eq $RowsNode -or $null -eq $Transition) {
        Add-Failure "Missing native RepositionThemeTransition on $RowsName."
    }
}
foreach ($RequiredCode in @(
    'UpdateDependentSettingsInterface',
    'ApplyDependentRow',
    'EnableDependentAnimation(true)',
    'std::chrono::milliseconds(170)',
    'DurationType::TimeSpan',
    'AnimationsEnabled',
    'InitializeValueHints',
    'TextProperty()',
    'DropDownOpened'
)) {
    if (-not $MainPageCpp.Contains($RequiredCode)) {
        Add-Failure "MainPage.cpp is missing conditional UI implementation: $RequiredCode"
    }
}

if ($RowCount -ne 104) { Add-Failure "Expected 104 setting rows, found $RowCount." }
if ($ToggleCount -ne 46) { Add-Failure "Expected 46 toggle rows, found $ToggleCount." }
if ($ComboCount -ne 23) { Add-Failure "Expected 23 combo rows, found $ComboCount." }
if ($SliderRowCount -ne 26) { Add-Failure "Expected 26 ordinary visible slider rows, found $SliderRowCount." }
if ($FanPairRowCount -ne 5) { Add-Failure "Expected 5 input-only fan curve rows, found $FanPairRowCount." }
if ($FanPairInputCount -ne 10) { Add-Failure "Expected 10 input-only fan curve number boxes, found $FanPairInputCount." }
if ($FanRangeRowCount -ne 1) { Add-Failure "Expected 1 fan range slider pair row, found $FanRangeRowCount." }
if ($FanRangeSliderCount -ne 2) { Add-Failure "Expected 2 visible fan range sliders, found $FanRangeSliderCount." }
if ($FanRangeInputCount -ne 2) { Add-Failure "Expected 2 fan range number boxes, found $FanRangeInputCount." }
if ($HiddenFanAdapterCount -ne 10) { Add-Failure "Expected 10 hidden fan curve adapter sliders, found $HiddenFanAdapterCount." }
$AllHandledSliders = @($Xaml.SelectNodes("//*[local-name()='Slider' and @ValueChanged]", $Namespace))
if ($AllHandledSliders.Count -ne 38) { Add-Failure "Expected the previous 38 value-driven sliders, found $($AllHandledSliders.Count)." }
if (($SliderRowCount + $FanRangeSliderCount + $HiddenFanAdapterCount) -ne 38) { Add-Failure "Ordinary, visible range and hidden fan sliders do not add up to the 38 slider guarantee." }
if ($ButtonRowCount -ne 3) { Add-Failure "Expected 3 button rows, found $ButtonRowCount." }

$MetricOrder = @(
    'textblock_Current_Gpu_Activity', 'textblock_Current_Core_Speed', 'textblock_Current_Memory_Usage',
    'textblock_Current_Memory_Speed', 'textblock_Current_Memory_Activity', 'textblock_Current_Power_Wattage',
    'textblock_Current_Power_Voltage', 'textblock_Current_Temp_Core', 'textblock_Current_Temp_Hotspot',
    'textblock_Current_Temp_Memory', 'textblock_Current_Fan_Speed_RPM', 'textblock_Current_Fan_Speed_Percentage',
    'textblock_Current_Temp_Intake'
)
$MetricNodes = @($Xaml.SelectNodes("//*[local-name()='TextBlock' and starts-with(@x:Name,'textblock_Current_')]", $Namespace))
$ActualMetricOrder = @($MetricNodes | ForEach-Object { Get-XName $_ })
if (($ActualMetricOrder -join '|') -ne ($MetricOrder -join '|')) {
    Add-Failure "GPU metrics grid is not the approved 5x3/13-field order: $($ActualMetricOrder -join ', ')"
}
foreach ($MetricNode in $MetricNodes) {
    $MetricStyle = $MetricNode.GetAttribute('Style')
    if ($MetricStyle -ne '{ThemeResource FlatMetricValue}') {
        Add-Failure "Metric value does not use the shared 14 DIP style: $(Get-XName $MetricNode)"
    }
}

# Toolbar icon-only buttons must be 32x32 with padding 0 and 16-18 DIP icons; icon+text buttons need an 8 DIP gap.
foreach ($Name in @('button_AppAdd_Graphics', 'button_AppRemove_Graphics', 'button_AppAdd_Tuning', 'button_AppRemove_Tuning', 'button_AppAdd_Display', 'button_AppRemove_Display')) {
    $Button = $Xaml.SelectSingleNode("//*[@x:Name='$Name']", $Namespace)
    if ($null -eq $Button) { Add-Failure "Missing toolbar button: $Name"; continue }
    if ($Button.GetAttribute('Style') -ne '{ThemeResource ApplicationButtonImage}' -or
        $Button.GetAttribute('Width') -ne '32' -or
        $Button.GetAttribute('Height') -ne '32' -or
        $Button.GetAttribute('Padding') -ne '0') {
        Add-Failure "Toolbar icon-only button is not 32x32/padding 0: $Name"
    }
    $Image = $Button.SelectSingleNode(".//*[local-name()='Image']")
    if ($null -eq $Image) { Add-Failure "Toolbar icon-only button has no image: $Name"; continue }
    $ImageWidth = [int]$Image.GetAttribute('Width')
    $ImageHeight = [int]$Image.GetAttribute('Height')
    if ($ImageWidth -lt 16 -or $ImageWidth -gt 18 -or $ImageWidth -ne $ImageHeight) {
        Add-Failure "Toolbar icon is not 16-18 DIP: $Name"
    }
}
foreach ($Name in @('button_GpuSelect', 'button_DisplaySelect', 'button_AppSelect_Graphics', 'button_AppSelect_Tuning', 'button_AppSelect_Display', 'button_AppSelect_Multimedia')) {
    $Button = $Xaml.SelectSingleNode("//*[@x:Name='$Name']", $Namespace)
    if ($null -eq $Button) { Add-Failure "Missing toolbar text button: $Name"; continue }
    $Image = $Button.SelectSingleNode(".//*[local-name()='Image']")
    $Text = $Button.SelectSingleNode(".//*[local-name()='TextBlock']")
    if ($null -eq $Image -or [int]$Image.GetAttribute('Width') -ne 18 -or [int]$Image.GetAttribute('Height') -ne 18) {
        Add-Failure "Toolbar text button icon is not 18 DIP: $Name"
    }
    if ($null -eq $Text -or $Text.GetAttribute('Margin') -ne '8,0,0,0') {
        Add-Failure "Toolbar text button gap is not 8 DIP: $Name"
    }
}
$UpdateButtonImage = $Xaml.SelectSingleNode("//*[@x:Name='button_Update_Launch']//*[local-name()='Image']", $Namespace)
if ($null -eq $UpdateButtonImage -or [int]$UpdateButtonImage.GetAttribute('Width') -ne 16 -or [int]$UpdateButtonImage.GetAttribute('Height') -ne 16) {
    Add-Failure 'Header update icon is not 16 DIP.'
}

$Result = [ordered]@{
    xaml = $XamlPath
    app_xaml = $AppXamlPath
    pages = $PageCount
    setting_rows = $RowCount
    toggles_checked = $ToggleCount
    combo_rows_checked = $ComboCount
    slider_rows_checked = $SliderRowCount
    fan_pair_rows_checked = $FanPairRowCount
    fan_pair_inputs_checked = $FanPairInputCount
    fan_range_pair_rows_checked = $FanRangeRowCount
    fan_range_sliders_checked = $FanRangeSliderCount
    fan_range_inputs_checked = $FanRangeInputCount
    hidden_fan_adapters_checked = $HiddenFanAdapterCount
    button_rows_checked = $ButtonRowCount
    failures = $Failures
    status = if ($Failures.Count -eq 0) { 'passed' } else { 'failed' }
}
$EvidenceDirectory = Split-Path -Parent $EvidencePath
if (-not (Test-Path -LiteralPath $EvidenceDirectory)) {
    New-Item -ItemType Directory -Path $EvidenceDirectory -Force | Out-Null
}
[IO.File]::WriteAllText($EvidencePath, ($Result | ConvertTo-Json -Depth 4), [Text.UTF8Encoding]::new($false))

if ($Failures.Count -gt 0) {
    foreach ($Failure in $Failures) { Write-Error $Failure }
    exit 1
}

Write-Output "Flat layout validation passed: $PageCount pages, $RowCount setting rows, $ToggleCount toggles, $ComboCount combo rows, $SliderRowCount ordinary slider rows, $FanRangeSliderCount visible range sliders, $FanPairRowCount input-only fan rows, $ButtonRowCount button rows."
