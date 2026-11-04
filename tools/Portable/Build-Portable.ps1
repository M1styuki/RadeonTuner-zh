[CmdletBinding()]
param(
    [switch]$UiValidation,
    [string]$PayloadSourceDir,
    [string]$OutFile,
    [string]$AppFolderName,
    [string]$WorkRoot,
    [string]$LogPath,
    [string]$RedistRoot
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$RepoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$TaskRoot = [IO.Path]::GetFullPath((Join-Path $RepoRoot '..\..'))

if ([string]::IsNullOrWhiteSpace($WorkRoot)) {
    $WorkRoot = if ($UiValidation) {
        Join-Path $RepoRoot '.build-work\portable-validation-work'
    } else {
        Join-Path $RepoRoot '.build-work\portable-production-work'
    }
}
$WorkRoot = [IO.Path]::GetFullPath($WorkRoot)

if ([string]::IsNullOrWhiteSpace($PayloadSourceDir)) {
    $PayloadSourceDir = if ($UiValidation) {
        Join-Path $RepoRoot 'bin\x64\Release-UiValidation'
    } else {
        Join-Path $RepoRoot 'bin\x64\Release'
    }
}
$PayloadSourceDir = [IO.Path]::GetFullPath($PayloadSourceDir)

if ([string]::IsNullOrWhiteSpace($AppFolderName)) {
    $AppFolderName = if ($UiValidation) { 'RadeonTuner-zh-CN-Validation' } else { 'RadeonTuner-zh-CN' }
}
if ($AppFolderName -notmatch '^[A-Za-z0-9._-]+$') {
    throw "AppFolderName contains unsupported characters: $AppFolderName"
}

if ([string]::IsNullOrWhiteSpace($OutFile)) {
    $OutFile = if ($UiValidation) {
        Join-Path $RepoRoot '.build-work\portable-validation\RadeonTuner-简体中文版-Validation.exe'
    } else {
        Join-Path $TaskRoot 'outputs\RadeonTuner-简体中文版.exe'
    }
}
$OutFile = [IO.Path]::GetFullPath($OutFile)
if ($UiValidation -and $OutFile.StartsWith((Join-Path $TaskRoot 'outputs'), [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Validation output must not be written into the deliverables outputs directory.'
}

if ([string]::IsNullOrWhiteSpace($LogPath)) {
    $LogPath = Join-Path $RepoRoot $(if ($UiValidation) { '.build-work\portable-build-validation.log' } else { '.build-work\portable-build-production.log' })
}
$LogPath = [IO.Path]::GetFullPath($LogPath)
$LogDirectory = Split-Path $LogPath -Parent
if (-not (Test-Path -LiteralPath $LogDirectory)) {
    New-Item -ItemType Directory -Path $LogDirectory -Force | Out-Null
}
Set-Content -LiteralPath $LogPath -Value "RadeonTuner portable build started $(Get-Date -Format o)" -Encoding utf8

function Write-Log {
    param([string]$Message)
    $line = "$(Get-Date -Format 'HH:mm:ss') $Message"
    Write-Host $line
    Add-Content -LiteralPath $LogPath -Value $line -Encoding utf8
}

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [string[]]$Arguments = @(),
        [string]$WorkingDirectory
    )
    Write-Log "RUN $FilePath $($Arguments -join ' ')"
    $oldLocation = Get-Location
    try {
        if ($WorkingDirectory) {
            Set-Location -LiteralPath $WorkingDirectory
        }
        $output = & $FilePath @Arguments 2>&1
        $exitCode = $LASTEXITCODE
    } finally {
        Set-Location -LiteralPath $oldLocation
    }
    foreach ($line in @($output)) {
        if ($null -ne $line) {
            Write-Log "  $line"
        }
    }
    if ($exitCode -ne 0) {
        throw "Command failed with exit code ${exitCode}: $FilePath"
    }
}

function Copy-PayloadFile {
    param([string]$SourceRoot, [string]$DestinationRoot, [string]$RelativePath)
    $source = Join-Path $SourceRoot $RelativePath
    if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
        throw "Required payload file is missing: $source"
    }
    $destination = Join-Path $DestinationRoot $RelativePath
    $parent = Split-Path $destination -Parent
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
    Copy-Item -LiteralPath $source -Destination $destination -Force
}

if (-not (Test-Path -LiteralPath $PayloadSourceDir -PathType Container)) {
    throw "Payload source directory does not exist: $PayloadSourceDir"
}

$vsWhere = 'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe'
$vsRoot = $null
if (Test-Path -LiteralPath $vsWhere) {
    $candidates = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($LASTEXITCODE -eq 0 -and $candidates) {
        $vsRoot = @($candidates)[0].Trim()
    }
}
if (-not $vsRoot -and (Test-Path -LiteralPath 'C:\Program Files\Microsoft Visual Studio\18\BuildTools')) {
    $vsRoot = 'C:\Program Files\Microsoft Visual Studio\18\BuildTools'
}
if (-not $vsRoot) {
    throw 'Visual Studio with the MSVC x64 toolchain was not found.'
}
$vcvars = Join-Path $vsRoot 'VC\Auxiliary\Build\vcvars64.bat'
if (-not (Test-Path -LiteralPath $vcvars)) {
    throw "vcvars64.bat was not found: $vcvars"
}

$msvcTools = Get-ChildItem -LiteralPath (Join-Path $vsRoot 'VC\Tools\MSVC') -Directory | Sort-Object Name -Descending | Select-Object -First 1
if (-not $msvcTools) {
    throw 'No MSVC tools directory was found.'
}
$dumpbin = Join-Path $msvcTools.FullName 'bin\Hostx64\x64\dumpbin.exe'
if (-not (Test-Path -LiteralPath $dumpbin)) {
    throw "dumpbin.exe was not found: $dumpbin"
}

if ([string]::IsNullOrWhiteSpace($RedistRoot)) {
    $redistVersions = Get-ChildItem -LiteralPath (Join-Path $vsRoot 'VC\Redist\MSVC') -Directory | Sort-Object Name -Descending
    foreach ($versionDirectory in $redistVersions) {
        $x64 = Join-Path $versionDirectory.FullName 'x64'
        if (-not (Test-Path -LiteralPath $x64)) { continue }
        $crt = Get-ChildItem -LiteralPath $x64 -Directory -Filter 'Microsoft.VC*.CRT' | Select-Object -First 1
        if ($crt) {
            $RedistRoot = $crt.FullName
            break
        }
    }
}
if ([string]::IsNullOrWhiteSpace($RedistRoot) -or -not (Test-Path -LiteralPath $RedistRoot -PathType Container)) {
    throw 'The MSVC x64 CRT redistributable directory was not found.'
}

Write-Log "RepoRoot: $RepoRoot"
Write-Log "PayloadSourceDir: $PayloadSourceDir"
Write-Log "AppFolderName: $AppFolderName"
Write-Log "OutFile: $OutFile"
Write-Log "MSVC: $($msvcTools.FullName)"
Write-Log "Redist: $RedistRoot"

if (Test-Path -LiteralPath $WorkRoot) {
    Remove-Item -LiteralPath $WorkRoot -Recurse -Force
}
$Stage = Join-Path $WorkRoot 'payload'
$CabDirectory = Join-Path $WorkRoot 'cab'
$LauncherDirectory = Join-Path $WorkRoot 'launcher'
New-Item -ItemType Directory -Path $Stage -Force | Out-Null
New-Item -ItemType Directory -Path $CabDirectory -Force | Out-Null
New-Item -ItemType Directory -Path $LauncherDirectory -Force | Out-Null

$fixedFiles = @(
    'RadeonTuner.exe',
    'Resources.pri',
    'RadeonTuner.winmd',
    'Microsoft.UI.Xaml.pri',
    'Microsoft.UI.Xaml.winmd',
    'Microsoft.UI.Xaml.xml',
    'Microsoft.Web.WebView2.Core.dll',
    'Microsoft.Web.WebView2.Core.winmd'
)
foreach ($relative in $fixedFiles) {
    Copy-PayloadFile -SourceRoot $PayloadSourceDir -DestinationRoot $Stage -RelativePath $relative
}

# The official updater is a fixed, unmodified third-party binary and must ship with the
# matching Settings\Updater.json update flow.
$officialUpdaterSource = Join-Path $RepoRoot 'third_party\OfficialUpdater\Updater.exe'
$officialUpdaterSha256 = 'D71662119832709B367545171BDD9F9E5FA551B443C2EDF21B7F5F8F20B91385'
$officialUpdaterSize = 2034688
if (-not (Test-Path -LiteralPath $officialUpdaterSource -PathType Leaf)) {
    throw "Official updater is missing: $officialUpdaterSource"
}
$officialUpdater = Get-Item -LiteralPath $officialUpdaterSource
$officialUpdaterHash = (Get-FileHash -LiteralPath $officialUpdaterSource -Algorithm SHA256).Hash
if ($officialUpdaterHash -ne $officialUpdaterSha256 -or $officialUpdater.Length -ne $officialUpdaterSize) {
    throw "Official updater hash/size mismatch: $officialUpdaterHash / $($officialUpdater.Length)"
}
Copy-Item -LiteralPath $officialUpdaterSource -Destination (Join-Path $Stage 'Updater.exe') -Force

$assetSource = Join-Path $PayloadSourceDir 'Assets'
if (-not (Test-Path -LiteralPath $assetSource -PathType Container)) {
    throw "Assets directory is missing: $assetSource"
}
$assetFiles = Get-ChildItem -LiteralPath $assetSource -File | Sort-Object Name
if ($assetFiles.Count -eq 0) {
    throw "Assets directory is empty: $assetSource"
}
foreach ($asset in $assetFiles) {
    Copy-PayloadFile -SourceRoot $PayloadSourceDir -DestinationRoot $Stage -RelativePath (Join-Path 'Assets' $asset.Name)
}
Copy-PayloadFile -SourceRoot $PayloadSourceDir -DestinationRoot $Stage -RelativePath 'Settings\Updater.json'

foreach ($runtimeDll in @('msvcp140.dll', 'vcruntime140.dll', 'vcruntime140_1.dll')) {
    $source = Join-Path $RedistRoot $runtimeDll
    if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
        throw "Required VC runtime DLL is missing from the redist directory: $source"
    }
    Copy-Item -LiteralPath $source -Destination (Join-Path $Stage $runtimeDll) -Force
}

# Only the explicit application build output, assets, default updater settings and
# VC runtime files above are staged. PDBs, logs, validation output, user Settings
# and Profiles are deliberately not copied.

$manifestFiles = Get-ChildItem -LiteralPath $Stage -Recurse -File | Sort-Object FullName
$manifestLines = [System.Collections.Generic.List[string]]::new()
$manifestLines.Add('RTPAYLOAD 1')
$hashRecords = [System.Collections.Generic.List[object]]::new()
foreach ($file in $manifestFiles) {
    $relative = $file.FullName.Substring($Stage.Length + 1)
    $relative = $relative.Replace('/', '\')
    if ($relative -match '[^\x20-\x7E]' -or $relative -match '(^|\\)\.\.?(\\|$)') {
        throw "Unsupported payload path: $relative"
    }
    $hash = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    $manifestLines.Add("$hash`t$($file.Length)`t$relative")
    $hashRecords.Add([pscustomobject]@{
        path = $relative
        size = [int64]$file.Length
        sha256 = $hash
        source = $file.FullName
    })
}
$manifestPath = Join-Path $Stage '.radeontuner-payload.txt'
[IO.File]::WriteAllText($manifestPath, (($manifestLines -join "`n") + "`n"), [Text.UTF8Encoding]::new($false))
$payloadId = (Get-FileHash -LiteralPath $manifestPath -Algorithm SHA256).Hash.ToLowerInvariant()
Write-Log "payload content ID (manifest SHA256): $payloadId"

$cabDirective = Join-Path $WorkRoot 'payload.ddf'
$ddf = [System.Collections.Generic.List[string]]::new()
$ddf.Add('.Set CabinetNameTemplate=payload.cab')
$ddf.Add(".Set DiskDirectoryTemplate=$CabDirectory")
$ddf.Add('.Set CompressionType=MSZIP')
$ddf.Add('.Set CompressionMemory=21')
$ddf.Add('.Set Cabinet=on')
$ddf.Add('.Set Compress=on')
$ddf.Add('.Set MaxDiskSize=0')
$ddf.Add('.Set RptFileName=NUL')
$cabFiles = Get-ChildItem -LiteralPath $Stage -Recurse -File | Sort-Object FullName
foreach ($file in $cabFiles) {
    $relative = $file.FullName.Substring($Stage.Length + 1)
    $ddf.Add(('"{0}" "{1}"' -f $file.FullName, $relative))
}
[IO.File]::WriteAllText($cabDirective, (($ddf -join "`r`n") + "`r`n"), [Text.UTF8Encoding]::new($false))

$makeCab = Join-Path $env:SystemRoot 'System32\makecab.exe'
if (-not (Test-Path -LiteralPath $makeCab)) {
    throw "makecab.exe was not found: $makeCab"
}
Invoke-Checked -FilePath $makeCab -Arguments @('/F', $cabDirective) -WorkingDirectory $CabDirectory
$stableCab = Join-Path $CabDirectory 'payload.cab'
if (-not (Test-Path -LiteralPath $stableCab -PathType Leaf)) {
    throw "makecab did not create payload.cab in $CabDirectory"
}

Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'RadeonTunerLauncher.cpp') -Destination $LauncherDirectory -Force
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'RadeonTunerLauncher.rc') -Destination $LauncherDirectory -Force
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'launcher.manifest') -Destination $LauncherDirectory -Force
Copy-Item -LiteralPath (Join-Path $RepoRoot 'RadeonTuner\AppIcon.ico') -Destination (Join-Path $LauncherDirectory 'launcher.ico') -Force
Copy-Item -LiteralPath $stableCab -Destination (Join-Path $LauncherDirectory 'payload.cab') -Force

$buildConfig = @"
#pragma once
#define RT_APP_FOLDER L"$AppFolderName"
#define RT_PAYLOAD_ID L"$payloadId"
"@
[IO.File]::WriteAllText((Join-Path $LauncherDirectory 'buildconfig.h'), $buildConfig, [Text.UTF8Encoding]::new($false))

$buildCmd = Join-Path $LauncherDirectory 'build-launcher.cmd'
$cmdText = @"
@echo off
setlocal
call "$vcvars" >nul
if errorlevel 1 exit /b 10
cd /d "$LauncherDirectory"
rc.exe /nologo /fo launcher.res RadeonTunerLauncher.rc
if errorlevel 1 exit /b 11
cl.exe /nologo /c /MT /O2 /GS /Gy /EHsc /utf-8 /std:c++20 /DUNICODE /D_UNICODE /DWIN32_LEAN_AND_MEAN /I"$LauncherDirectory" RadeonTunerLauncher.cpp
if errorlevel 1 exit /b 12
link.exe /nologo /SUBSYSTEM:WINDOWS /MACHINE:X64 /MANIFEST:NO /INCREMENTAL:NO /OPT:REF /OPT:ICF /OUT:RadeonTunerLauncher.exe RadeonTunerLauncher.obj launcher.res cabinet.lib shell32.lib bcrypt.lib user32.lib ole32.lib
if errorlevel 1 exit /b 13
exit /b 0
"@
[IO.File]::WriteAllText($buildCmd, $cmdText, [Text.UTF8Encoding]::new($false))
Invoke-Checked -FilePath 'C:\Windows\System32\cmd.exe' -Arguments @('/d', '/s', '/c', $buildCmd) -WorkingDirectory $LauncherDirectory

$builtExe = Join-Path $LauncherDirectory 'RadeonTunerLauncher.exe'
if (-not (Test-Path -LiteralPath $builtExe -PathType Leaf)) {
    throw "Launcher was not built: $builtExe"
}

$dependencyOutput = & $dumpbin /nologo /dependents $builtExe 2>&1
$dependencyText = ($dependencyOutput -join "`n")
Write-Log 'Launcher imports:'
foreach ($line in $dependencyOutput) { Write-Log "  $line" }
if ($dependencyText -match 'MSVCP|VCRUNTIME|ucrtbase|api-ms-win-crt') {
    throw 'Launcher still imports a VC/UCRT runtime DLL; /MT static linking failed.'
}

$builtVersion = (Get-Item -LiteralPath $builtExe).VersionInfo
Write-Log "Launcher version: $($builtVersion.FileVersion) / $($builtVersion.CompanyName)"
if ($builtVersion.CompanyName -ne 'Arnold Vink') {
    throw 'Launcher version/author resource was not embedded.'
}

$cabHash = (Get-FileHash -LiteralPath $stableCab -Algorithm SHA256).Hash.ToLowerInvariant()
$builtHash = (Get-FileHash -LiteralPath $builtExe -Algorithm SHA256).Hash.ToLowerInvariant()
$builtSize = (Get-Item -LiteralPath $builtExe).Length
Write-Log "payload.cab SHA256: $cabHash"
Write-Log "launcher SHA256: $builtHash"
Write-Log "launcher size: $builtSize"

$outputDirectory = Split-Path $OutFile -Parent
if (-not (Test-Path -LiteralPath $outputDirectory)) {
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
}
Copy-Item -LiteralPath $builtExe -Destination $OutFile -Force
$outputHash = (Get-FileHash -LiteralPath $OutFile -Algorithm SHA256).Hash.ToLowerInvariant()
$outputSize = (Get-Item -LiteralPath $OutFile).Length
if ($outputHash -ne $builtHash -or $outputSize -ne $builtSize) {
    throw 'Copied output does not match the built launcher.'
}

$report = [ordered]@{
    built_utc = (Get-Date).ToUniversalTime().ToString('o')
    configuration = if ($UiValidation) { 'UiValidation' } else { 'Release' }
    payload_source = $PayloadSourceDir
    app_folder_name = $AppFolderName
    out_file = $OutFile
    out_sha256 = $outputHash
    out_size = $outputSize
    payload_id_sha256 = $payloadId
    payload_cab_sha256 = $cabHash
    payload_cab_size = (Get-Item -LiteralPath $stableCab).Length
    official_updater_sha256 = $officialUpdaterHash
    official_updater_size = $officialUpdater.Length
    launcher_imports_checked = $true
    launcher_has_static_crt = $true
    version_company = $builtVersion.CompanyName
    version_file = $builtVersion.FileVersion
    file_count = $hashRecords.Count + 1
    files = $hashRecords
}
$reportPath = if ($UiValidation) {
    Join-Path $RepoRoot '.build-work\portable-build-report-validation.json'
} else {
    Join-Path $RepoRoot '.build-work\portable-build-report-production.json'
}
[IO.File]::WriteAllText($reportPath, ($report | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))
Write-Log "Build report: $reportPath"
Write-Log "Output: $OutFile ($outputSize bytes, SHA256 $outputHash)"
Write-Log 'Build completed successfully.'
