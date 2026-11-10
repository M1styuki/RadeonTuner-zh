param(
    [string]$Baseline = '32502495deb29defc93c6152c9adb0701ad5eac2'
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$xamlPath = Join-Path $repoRoot 'RadeonTuner/MainPage.xaml'
$appXamlPath = Join-Path $repoRoot 'RadeonTuner/App.xaml'
$projectPath = Join-Path $repoRoot 'RadeonTuner/RadeonTuner.vcxproj'
$xNamespace = 'http://schemas.microsoft.com/winfx/2006/xaml'
$failures = [System.Collections.Generic.List[string]]::new()
$checks = 0

function Add-Failure([string]$message) {
    $script:failures.Add($message)
}

function Read-GitText([string]$path) {
    $lines = & git -c "safe.directory=$repoRoot" show "${Baseline}:$path"
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to read baseline path from git: $path"
    }
    return ($lines -join "`n")
}

# Baseline paths with English on-disk line endings are written to a temporary file for XML parsing.
$baselineXamlTemp = Join-Path ([System.IO.Path]::GetTempPath()) "RadeonTuner-baseline-$PID.xaml"
try {
    Read-GitText 'RadeonTuner/MainPage.xaml' | Set-Content -LiteralPath $baselineXamlTemp -Encoding utf8
    [xml]$baselineXml = Get-Content -LiteralPath $baselineXamlTemp -Raw
}
finally {
    Remove-Item -LiteralPath $baselineXamlTemp -Force -ErrorAction SilentlyContinue
}

[xml]$currentXml = Get-Content -LiteralPath $xamlPath -Raw
[xml]$currentAppXml = Get-Content -LiteralPath $appXamlPath -Raw
$checks++
Write-Output 'PASS XML parse: MainPage.xaml and App.xaml'

function Get-NamedElements([xml]$document) {
    $result = @{}
    foreach ($node in $document.SelectNodes('//*')) {
        $name = $node.GetAttribute('Name', $xNamespace)
        if ($name) {
            $result[$name] = $node
        }
    }
    return $result
}

$baselineNames = Get-NamedElements $baselineXml
$currentNames = Get-NamedElements $currentXml
$intentionallyRemovedNames = @(
    'toggleswitch_Fan_Control',
    'textblock_Fan_Control_Value',
    'stackpanel_FsrOtaUpdates',
    'textblock_FsrOtaUpdates_Value',
    'combobox_FsrOtaUpdates',
    'textblock_FrameGenSearchMode_Value',
    'combobox_FrameGenSearchMode',
    'textblock_FrameGenPerfMode_Value',
    'combobox_FrameGenPerfMode',
    'textblock_FrameGenResponseMode_Value',
    'combobox_FrameGenResponseMode',
    'textbox_FsrDllLoadPath'
)
$missingNames = @($baselineNames.Keys | Where-Object { -not $currentNames.ContainsKey($_) -and $intentionallyRemovedNames -notcontains $_ } | Sort-Object)
$checks++
if ($missingNames.Count -eq 0) {
    Write-Output "PASS x:Name identifiers: all retained baseline identifiers and the new 0.70 controls are present"
}
else {
    Add-Failure "Missing x:Name identifiers: $($missingNames -join ', ')"
}

$requiredNewNames = @(
    'toggleswitch_Hide_Tray_Icon',
    'stackpanel_FsrDisableMLSwapchain',
    'toggleswitch_FsrDisableMLSwapchain',
    'combobox_FanControlMode',
    'combobox_FanRangedCurve',
    'button_DisableRadeonServices',
    'button_EnableRadeonServices',
    'button_DisableRadeonServices_AdminShield',
    'slider_Fan_Speed_Min',
    'slider_Fan_Speed_Max',
    'toggleswitch_Display_GammaControl',
    'slider_Display_GammaBase',
    'slider_Display_GammaRed',
    'slider_Display_GammaGreen',
    'slider_Display_GammaBlue',
    'stackpanel_Display_Gamma_Dependent',
    'stackpanel_Display_ColorTemperature_Kelvin_Dependent',
    'stackpanel_Display_CVDC_Dependent',
    'stackpanel_Display_VariBright_Level_Dependent',
    'toggleswitch_UpdateExcludeDrivers'
)
$missingNewNames = @($requiredNewNames | Where-Object { -not $currentNames.ContainsKey($_) })
if ($missingNewNames.Count -gt 0) {
    Add-Failure "Missing new 0.70/HideTrayIcon controls: $($missingNewNames -join ', ')"
}

$eventAttributes = 'Click', 'Toggled', 'TextChanged', 'ValueChanged', 'SelectionChanged', 'Loaded', 'PointerMoved', 'ItemClick'
$baselineEvents = [System.Collections.Generic.HashSet[string]]::new()
$currentEvents = [System.Collections.Generic.HashSet[string]]::new()
foreach ($name in $baselineNames.Keys) {
    foreach ($attribute in $eventAttributes) {
        $value = $baselineNames[$name].GetAttribute($attribute)
        if ($value) { [void]$baselineEvents.Add("$name|$attribute|$value") }
    }
}
foreach ($name in $currentNames.Keys) {
    foreach ($attribute in $eventAttributes) {
        $value = $currentNames[$name].GetAttribute($attribute)
        if ($value) { [void]$currentEvents.Add("$name|$attribute|$value") }
    }
}
$checks++
$intentionallyRenamedEvents = @(
    'slider_Fan_Speed_0|ValueChanged|slider_Fan_ValueChanged',
    'slider_Fan_Speed_1|ValueChanged|slider_Fan_ValueChanged',
    'slider_Fan_Speed_2|ValueChanged|slider_Fan_ValueChanged',
    'slider_Fan_Speed_3|ValueChanged|slider_Fan_ValueChanged',
    'slider_Fan_Speed_4|ValueChanged|slider_Fan_ValueChanged',
    'slider_Fan_Temp_0|ValueChanged|slider_Fan_ValueChanged',
    'slider_Fan_Temp_1|ValueChanged|slider_Fan_ValueChanged',
    'slider_Fan_Temp_2|ValueChanged|slider_Fan_ValueChanged',
    'slider_Fan_Temp_3|ValueChanged|slider_Fan_ValueChanged',
    'slider_Fan_Temp_4|ValueChanged|slider_Fan_ValueChanged'
)
$removedEvents = @($baselineEvents | Where-Object {
    $owner = ($_ -split '\|', 2)[0]
    $intentionallyRemovedNames -notcontains $owner -and $intentionallyRenamedEvents -notcontains $_ -and -not $currentEvents.Contains($_)
} | Sort-Object)
$addedEvents = @($currentEvents | Where-Object { -not $baselineEvents.Contains($_) } | Sort-Object)
if ($removedEvents.Count -eq 0) {
    Write-Output "PASS event hookups: all retained baseline handlers and the new 0.70 controls are hooked up; $($addedEvents.Count) new hookup(s) added"
}
else {
    Add-Failure "Missing baseline event hookups: $($removedEvents -join ', ')"
}
$requiredNewEvents = @(
    'toggleswitch_Hide_Tray_Icon|Toggled|toggleswitch_Hide_Tray_Icon_Toggled',
    'toggleswitch_FsrDisableMLSwapchain|Toggled|toggleswitch_FsrDisableMLSwapchain_Toggled',
    'combobox_FanControlMode|SelectionChanged|combobox_FanControlMode_SelectionChanged',
    'combobox_FanRangedCurve|SelectionChanged|combobox_FanRangedCurve_SelectionChanged',
    'button_DisableRadeonServices|Click|button_DisableRadeonServices_Click',
    'button_EnableRadeonServices|Click|button_EnableRadeonServices_Click',
    'slider_Fan_Speed_Min|ValueChanged|slider_FanRanged_ValueChanged',
    'slider_Fan_Speed_Max|ValueChanged|slider_FanRanged_ValueChanged',
    'toggleswitch_Display_GammaControl|Toggled|toggleswitch_Display_GammaControl_Toggled',
    'slider_Display_GammaBase|ValueChanged|slider_Display_GammaRGB_ValueChanged',
    'slider_Display_GammaRed|ValueChanged|slider_Display_GammaRGB_ValueChanged',
    'slider_Display_GammaGreen|ValueChanged|slider_Display_GammaRGB_ValueChanged',
    'slider_Display_GammaBlue|ValueChanged|slider_Display_GammaRGB_ValueChanged',
    'toggleswitch_UpdateExcludeDrivers|Toggled|toggleswitch_UpdateExcludeDrivers_Toggled'
)
$missingNewEvents = @($requiredNewEvents | Where-Object { -not $currentEvents.Contains($_) })
if ($missingNewEvents.Count -gt 0) {
    Add-Failure "Missing new 0.70 event hookups: $($missingNewEvents -join ', ')"
}

$settingAttributes = 'Minimum', 'Maximum', 'Value', 'TickFrequency', 'StepFrequency', 'SmallChange', 'LargeChange'
$baselineSettingValues = [System.Collections.Generic.HashSet[string]]::new()
$currentSettingValues = [System.Collections.Generic.HashSet[string]]::new()
foreach ($name in $baselineNames.Keys) {
    foreach ($attribute in $settingAttributes) {
        $value = $baselineNames[$name].GetAttribute($attribute)
        if ($value) { [void]$baselineSettingValues.Add("$name|$attribute|$value") }
    }
}
foreach ($name in $currentNames.Keys) {
    foreach ($attribute in $settingAttributes) {
        $value = $currentNames[$name].GetAttribute($attribute)
        if ($value) { [void]$currentSettingValues.Add("$name|$attribute|$value") }
    }
}
$checks++
if ($baselineSettingValues.SetEquals($currentSettingValues)) {
    Write-Output "PASS setting numeric attributes: all $($baselineSettingValues.Count) baseline values retained"
}
else {
    Add-Failure 'Setting range/default XAML attributes changed.'
}

$visibleAttributes = 'Text', 'Content', 'OffContent', 'OnContent', 'ToolTipService.ToolTip', 'PlaceholderText', 'Header'
$allowedLatinValues = @(
    'RadeonTuner <3', 'Radeon Boost 2', 'Radeon Chill', 'Vari-Bright',
    'Hz', 'MHz', '°C', '°F', 'W', 'H', 'Hor', 'Ver', 'GB', 'mV', 'RPM',
    '0°C', '50°C', '100°C', '0°C | 32°F', '50°C | 122°F', '100°C | 212°F'
)
$residualXaml = [System.Collections.Generic.HashSet[string]]::new()
foreach ($node in $currentXml.SelectNodes('//*')) {
    foreach ($attribute in $visibleAttributes) {
        if (-not $node.HasAttribute($attribute)) { continue }
        $value = $node.GetAttribute($attribute)
        if ($value -match '^\s*\{.*\}\s*$') { continue }
        if ($value -notmatch '[A-Za-z]' -or $value -match '[\u4e00-\u9fff]') { continue }
        if ($allowedLatinValues -notcontains $value) {
            [void]$residualXaml.Add($value)
        }
    }
}
$checks++
if ($residualXaml.Count -eq 0) {
    Write-Output 'PASS XAML residual English: only technical terms, units, product names and bindings remain'
}
else {
    Add-Failure "Unexpected XAML English values: $([string]::Join(' | ', $residualXaml))"
}

$uiCallPattern = 'ShowNotification\(|ShowMessageBox\(|\.Text\(|\.Content\(|filepicker_(open|save)\(|MessageBoxW\(|InsertMenuW\('
$allowedCppUiValues = @('MHz', 'MTs', 'GB', 'W', 'mV', 'RPM', '°C', 'Hz', 'FPS', 'K', 'FSR DLL', 'Global', 'amdxcff*.dll', '*.radd', '*.radg', '*.radt', '*.exe', '*.bin')
$residualCppUi = [System.Collections.Generic.HashSet[string]]::new()
$utf8 = [System.Text.UTF8Encoding]::new($false, $true)
Get-ChildItem -LiteralPath (Join-Path $repoRoot 'RadeonTuner') -File |
    Where-Object { $_.Extension -in '.h', '.cpp' } |
    ForEach-Object {
        $text = $utf8.GetString([System.IO.File]::ReadAllBytes($_.FullName))
        $lineNumber = 0
        foreach ($line in ($text -split "`r?`n")) {
            $lineNumber++
            $trimmed = $line.TrimStart()
            if ($trimmed.StartsWith('//') -or $line -match 'AVDebugWriteLine') { continue }
            if ($line -notmatch $uiCallPattern) { continue }
            foreach ($match in [regex]::Matches($line, 'L"((?:[^"\\]|\\.)*)"')) {
                $value = $match.Groups[1].Value
                if ($value -notmatch '[A-Za-z]' -or $value -match '[\u4e00-\u9fff]') { continue }
                if ($allowedCppUiValues -contains $value) { continue }
                [void]$residualCppUi.Add("$($_.Name):$lineNumber $value")
            }
        }
    }
$checks++
if ($residualCppUi.Count -eq 0) {
    Write-Output 'PASS C++ residual English: only approved technical terms, file filters and the internal Global key remain'
}
else {
    Add-Failure "Unexpected C++ UI English: $([string]::Join(' | ', $residualCppUi))"
}

$checks++
$selectSource = Get-Content -LiteralPath (Join-Path $repoRoot 'RadeonTuner/AdlxEventsSelect.h') -Raw
$globalMappingFiles = @('AdlxValuesLoadDisplay.h', 'AdlxValuesLoadGraphics.h', 'AdlxValuesLoadMultimedia.h', 'AdlxValuesLoadTuning.h')
$globalMappingPresent = $selectSource -match 'L"全局"' -and $selectSource -match 'messageResult == 0 \? L"Global"'
foreach ($file in $globalMappingFiles) {
    $source = Get-Content -LiteralPath (Join-Path $repoRoot "RadeonTuner/$file") -Raw
    $globalMappingPresent = $globalMappingPresent -and $source -match 'L"全局"' -and $source -match 'L"Global"'
}
if ($globalMappingPresent) {
    Write-Output 'PASS Global profile display mapping: internal value remains Global while UI shows 全局'
}
else {
    Add-Failure 'Global profile display mapping is incomplete.'
}

$checks++
$projectText = Get-Content -LiteralPath $projectPath -Raw
if ($projectText -match '/utf-8' -and $currentXml.DocumentElement.GetAttribute('Language') -eq 'zh-CN' -and $currentXml.DocumentElement.GetAttribute('FontFamily')) {
    Write-Output 'PASS UTF-8 compiler setting and Chinese page typography settings present'
}
else {
    Add-Failure 'Missing /utf-8, Page Language, or Page FontFamily setting.'
}

$checks++
$baselineEnumText = Read-GitText 'RadeonTuner/SettingEnumStrings.h'
$currentEnumText = Get-Content -LiteralPath (Join-Path $repoRoot 'RadeonTuner/SettingEnumStrings.h') -Raw
function Get-EnumArrayCounts([string]$source) {
    $counts = @{}
    foreach ($match in [regex]::Matches($source, '(?s)const std::vector<std::wstring>\s+(\w+)\s*=\s*\{(.*?)\};')) {
        $body = [regex]::Replace($match.Groups[2].Value, '(?m)//.*$', '')
        $counts[$match.Groups[1].Value] = ([regex]::Matches($body, 'L"(?:[^"\\]|\\.)*"')).Count
    }
    return $counts
}
$baselineEnumCounts = Get-EnumArrayCounts $baselineEnumText
$currentEnumCounts = Get-EnumArrayCounts $currentEnumText
$enumCountMismatch = @()
foreach ($key in $baselineEnumCounts.Keys) {
    if (-not $currentEnumCounts.ContainsKey($key) -or $currentEnumCounts[$key] -ne $baselineEnumCounts[$key]) {
        $enumCountMismatch += $key
    }
}
if ($enumCountMismatch.Count -eq 0) {
    Write-Output "PASS enum display arrays: all $($baselineEnumCounts.Count) array cardinalities preserved"
}
else {
    Add-Failure "Enum display array cardinality changed: $($enumCountMismatch -join ', ')"
}

$checks++
$contextMenuText = Get-Content -LiteralPath (Join-Path $repoRoot 'RadeonTuner/SettingSave.h') -Raw
$contextMenuLoadText = Get-Content -LiteralPath (Join-Path $repoRoot 'RadeonTuner/SettingLoad.h') -Raw
$identityText = Get-Content -LiteralPath (Join-Path $repoRoot 'RadeonTuner/SystemIntegrationIdentity.h') -Raw
$independentContextKey = $identityText -match 'ContextMenuKey\[\]\s*=\s*L"RadeonTunerZhCN"'
$pairedContextKey = $contextMenuText -match 'ContextMenuKey' -and $contextMenuLoadText -match 'ContextMenuKey'
$noOriginalContextKey = $contextMenuText -notmatch 'shell\\\\RadeonTuner"' -and $contextMenuLoadText -notmatch 'shell\\\\RadeonTuner"'
$quotedCommand = $contextMenuText -match 'ContextMenuCommand\(targetFilePath\)'
if ($independentContextKey -and $pairedContextKey -and $noOriginalContextKey -and $quotedCommand -and $contextMenuText -match 'L"打开 RadeonTuner"') {
    Write-Output 'PASS Explorer context menu uses the independent key and a quoted command path'
}
else {
    Add-Failure 'Explorer context menu identity or command quoting is inconsistent.'
}

$checks++
$mainPageText = Get-Content -LiteralPath (Join-Path $repoRoot 'RadeonTuner/MainPage.cpp') -Raw
$headerNamed = $currentNames.ContainsKey('border_Header') -and $currentNames.ContainsKey('border_ActionButtons')
$settingsRowsHandled = $mainPageText -match 'border_Header\(\)\.Visibility\(Visibility::Collapsed\)' -and
    $mainPageText -match 'border_ActionButtons\(\)\.Visibility\(Visibility::Collapsed\)' -and
    $mainPageText -match 'if \(selectedIndex != 5\)'
if ($headerNamed -and $settingsRowsHandled) {
    Write-Output 'PASS Settings page hides the empty selection/header and action-bar rows only there'
}
else {
    Add-Failure 'Settings page empty header/action rows are not handled consistently.'
}

$checks++
$sourceFiles = Get-ChildItem -LiteralPath (Join-Path $repoRoot 'RadeonTuner') -File |
    Where-Object { $_.Extension -in '.h', '.cpp', '.xaml', '.vcxproj' }
$invalidUtf8 = @()
foreach ($file in $sourceFiles) {
    try {
        [void]$utf8.GetString([System.IO.File]::ReadAllBytes($file.FullName))
    }
    catch {
        $invalidUtf8 += $file.Name
    }
}
if ($invalidUtf8.Count -eq 0) {
    Write-Output "PASS UTF-8 validity: $($sourceFiles.Count) source/project files decode as UTF-8"
}
else {
    Add-Failure "Invalid UTF-8 files: $($invalidUtf8 -join ', ')"
}

Write-Output ''
if ($failures.Count -eq 0) {
    Write-Output "STATIC CHECK PASS ($checks checks)"
    exit 0
}

Write-Output "STATIC CHECK FAIL ($($failures.Count) of $checks checks failed)"
$failures | ForEach-Object { Write-Output "- $_" }
exit 1
