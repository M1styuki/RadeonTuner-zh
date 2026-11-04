# RadeonTuner single-file launcher

`Build-Portable.ps1` builds the small native Win32 launcher that embeds the
RadeonTuner application payload as an MSZIP CAB resource. The launcher itself is
compiled with the MSVC static CRT (`/MT`) and only imports Windows system DLLs.

## Build

Production, using the already-built `bin\x64\Release` output:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File tools\Portable\Build-Portable.ps1
```

The default production output is:

```text
D:\Documents\codex\2026-11-02\jia\outputs\RadeonTuner-简体中文版.exe
```

Validation build (uses `bin\x64\Release-UiValidation`, never the production core):

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File tools\Portable\Build-Portable.ps1 -UiValidation
```

The build script copies only the explicit core binaries, `Assets`, the default
`Settings/Updater.json`, the unmodified official `third_party/OfficialUpdater/Updater.exe`,
and the required x64 VC runtime DLLs from the installed Visual Studio redistributable
directory. It does not copy PDBs, logs, validation output, or user Settings/Profiles.
The fixed official updater hash and size are checked during packaging.

## Runtime layout

- Production app directory: `%LOCALAPPDATA%\RadeonTuner-zh-CN\App`
- Validation app directory: `%LOCALAPPDATA%\RadeonTuner-zh-CN-Validation\App`
- Core executable inside the app directory: `RadeonTuner.exe`
- Cache state: `App\.radeontuner-payload.state`
- Incomplete-apply journal: `App\.radeontuner-payload.pending`

The launcher starts the core in the app directory and exits immediately. It does
not wait for the core and does not leave a resident parent process.

The cache ID is the SHA-256 of the extracted payload manifest. A named mutex based
on the final resolved app directory serializes the state re-read, extraction,
replacement, completion marker and launch decision between concurrent launchers.
On a matching cache it launches the core directly; if a matching core is already
running it recalls only that core's `RadeonTunerMainWindow` and never starts a
second core while the first is still starting. A successful extraction/apply is
recorded only after the complete payload has been hash-validated. If the existing
core has been replaced by RadeonTuner's own updater, the launcher preserves it
instead of overwriting it merely because this outer EXE has a different payload.

`Settings\Updater.json` is seeded only when it does not exist. Existing
`Settings.json`, `Profiles` and additional user files are left untouched, and the
post-apply program-file check does not require the user-editable updater config to
equal the embedded default. Manifest paths reject non-canonical/reserved aliases
and reparse-point traversal below the app root; legitimate system LocalAppData
redirection is resolved rather than rejected.

If a new payload must overwrite program files while the existing core is
running, the launcher asks the user to exit RadeonTuner first. It never force
closes the application.

## Extract-only acceptance mode

```powershell
RadeonTuner-简体中文版.exe --extract-only <empty-directory> --extract-report <report.json>
```

This mode extracts and verifies the embedded payload into the requested
directory, writes a report containing `"launched": false` and
`"no_launch": true`, and never starts the core.
