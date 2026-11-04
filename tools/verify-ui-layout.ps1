#Requires -Version 7
[CmdletBinding()]
param(
    [string]$XamlPath = (Join-Path $PSScriptRoot '..\RadeonTuner\MainPage.xaml'),
    [string]$EvidencePath = (Join-Path $PSScriptRoot '..\.build-work\evidence\flat-layout-validation.json')
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$XamlPath = [IO.Path]::GetFullPath($XamlPath)
$EvidencePath = [IO.Path]::GetFullPath($EvidencePath)
$Xaml = [xml](Get-Content -LiteralPath $XamlPath -Raw -Encoding utf8)
$Namespace = New-Object System.Xml.XmlNamespaceManager($Xaml.NameTable)
$Namespace.AddNamespace('x', 'http://schemas.microsoft.com/winfx/2006/xaml')

$PageNames = @(
    'stackpanel_Tuning', 'stackpanel_Fans', 'stackpanel_Graphics', 'stackpanel_Display',
    'stackpanel_Multimedia', 'stackpanel_Settings', 'stackpanel_Information'
)
$Failures = [System.Collections.Generic.List[string]]::new()
$ToggleCount = 0
$PageCount = 0

function Get-Column([System.Xml.XmlElement]$Element) {
    $Value = $Element.GetAttribute('Grid.Column')
    if ([string]::IsNullOrWhiteSpace($Value)) { return 0 }
    return [int]$Value
}

foreach ($PageName in $PageNames) {
    $Page = $Xaml.SelectSingleNode("//*[@x:Name='$PageName']", $Namespace)
    if ($null -eq $Page) {
        $Failures.Add("Missing page: $PageName")
        continue
    }
    $PageCount++

    $HeadingNodes = $Page.SelectNodes(".//*[local-name()='TextBlock' and @Style='{ThemeResource ApplicationSectionHeader}']")
    foreach ($HeadingNode in $HeadingNodes) {
        $Failures.Add("Group heading remains in ${PageName}: $($HeadingNode.GetAttribute('Text'))")
    }

    $CardNodes = $Page.SelectNodes(".//*[local-name()='Border']")
    foreach ($CardNode in $CardNodes) {
        $Name = $CardNode.GetAttribute('Name', 'http://schemas.microsoft.com/winfx/2006/xaml')
        if ($Name -like '*_AccessOverlay') { continue }
        $Background = $CardNode.GetAttribute('Background')
        $CornerRadius = $CardNode.GetAttribute('CornerRadius')
        if (-not [string]::IsNullOrWhiteSpace($Background) -and $Background -notin @('Transparent', '{x:Null}') -and
            -not [string]::IsNullOrWhiteSpace($CornerRadius) -and $CornerRadius -ne '0') {
            $Failures.Add("Card-like border remains in ${PageName}: $Name / $Background / $CornerRadius")
        }
        if ($Background -eq '{ThemeResource ThemeBackgroundBrush}' -or $Background -eq '{ThemeResource ThemeBackgroundDarkBrush}') {
            $Failures.Add("Filled group surface remains in ${PageName}: $Name / $Background")
        }
    }

    $Toggles = $Page.SelectNodes(".//*[local-name()='ToggleSwitch']")
    foreach ($Toggle in $Toggles) {
        $ToggleCount++
        $ToggleName = $Toggle.GetAttribute('Name', 'http://schemas.microsoft.com/winfx/2006/xaml')
        $AutomationName = $Toggle.GetAttribute('AutomationProperties.Name')
        if ([string]::IsNullOrWhiteSpace($AutomationName)) {
            $Failures.Add("Toggle has no automation label: $PageName / $ToggleName")
        }
        if (-not [string]::IsNullOrEmpty($Toggle.GetAttribute('OnContent')) -or -not [string]::IsNullOrEmpty($Toggle.GetAttribute('OffContent'))) {
            $Failures.Add("Toggle still has On/OffContent text: $PageName / $ToggleName")
        }
        $HorizontalAlignment = $Toggle.GetAttribute('HorizontalAlignment')
        $ToggleStyle = $Toggle.GetAttribute('Style')
        if ($HorizontalAlignment -ne 'Right' -and $ToggleStyle -ne '{ThemeResource FlatToggleSwitch}') {
            $Failures.Add("Toggle is not right aligned: $PageName / $ToggleName")
        }

        $Parent = $Toggle.ParentNode
        if ($null -eq $Parent -or $Parent.LocalName -ne 'Grid') {
            $Failures.Add("Toggle row is not a Grid: $PageName / $ToggleName")
            continue
        }
        $ToggleColumn = Get-Column $Toggle
        $LabelColumns = [System.Collections.Generic.List[int]]::new()
        $HelpColumns = [System.Collections.Generic.List[int]]::new()
        $WarningColumns = [System.Collections.Generic.List[int]]::new()
        foreach ($Sibling in $Parent.ChildNodes) {
            if ($Sibling.NodeType -ne 'Element') { continue }
            $SiblingColumn = Get-Column $Sibling
            if ($Sibling.LocalName -eq 'TextBlock') {
                $Text = $Sibling.GetAttribute('Text')
                if (-not [string]::IsNullOrWhiteSpace($Text) -and $SiblingColumn -lt $ToggleColumn) {
                    $LabelColumns.Add($SiblingColumn)
                }
            }
            if ($Sibling.LocalName -eq 'Image') {
                $Tooltip = $Sibling.GetAttribute('ToolTipService.ToolTip')
                if (-not [string]::IsNullOrWhiteSpace($Tooltip) -and $SiblingColumn -lt $ToggleColumn) {
                    if ($Tooltip.StartsWith('*')) { $WarningColumns.Add($SiblingColumn) } else { $HelpColumns.Add($SiblingColumn) }
                }
            }
        }
        if ($LabelColumns.Count -eq 0) {
            $Failures.Add("Toggle row has no leading label: $PageName / $ToggleName")
            continue
        }
        if ($HelpColumns.Count -gt 0 -and ($HelpColumns | Measure-Object -Minimum).Minimum -le ($LabelColumns | Measure-Object -Minimum).Minimum) {
            $Failures.Add("Help icon does not follow the label: $PageName / $ToggleName")
        }
        if ($WarningColumns.Count -gt 0) {
            $ReferenceColumns = if ($HelpColumns.Count -gt 0) { $HelpColumns } else { $LabelColumns }
            if (($WarningColumns | Measure-Object -Minimum).Minimum -le ($ReferenceColumns | Measure-Object -Minimum).Minimum) {
                $Failures.Add("Warning icon does not follow the help icon/label: $PageName / $ToggleName")
            }
        }
    }
}

$Result = [ordered]@{
    xaml = $XamlPath
    pages = $PageCount
    toggles_checked = $ToggleCount
    failures = $Failures
    status = if ($Failures.Count -eq 0) { 'passed' } else { 'failed' }
}
$EvidenceDirectory = Split-Path -Parent $EvidencePath
if (-not (Test-Path -LiteralPath $EvidenceDirectory)) {
    New-Item -ItemType Directory -Path $EvidenceDirectory -Force | Out-Null
}
[IO.File]::WriteAllText($EvidencePath, ($Result | ConvertTo-Json -Depth 4), [Text.UTF8Encoding]::new($false))

if ($Failures.Count -gt 0) {
    $Failures | ForEach-Object { Write-Error $_ }
    exit 1
}

Write-Output "Flat layout validation passed: $PageCount pages, $ToggleCount toggles."
