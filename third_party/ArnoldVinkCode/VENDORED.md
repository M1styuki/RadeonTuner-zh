# Vendored ArnoldVinkCode (C++)

Source: public snapshot `AeonCatalyst/ArnoldVinkCode`, commit
`59135adaeddc337d2c20a292b2a8effb011e270f` (2026-07-16), `CPP` directory.

Vendored locally so the project owns the source it builds against. The upstream
snapshot's `License.md` is copied verbatim next to this file; it states that the
work has no license. Do not redistribute this directory or its contents without
permission from the original author.

Local compatibility notes:

- `CPP/Api/ApiTokens.h` was added because the snapshot references it but does
  not contain it. `ApiTokens::GitHub` is an intentionally empty, optional token:
  update checks continue as public unauthenticated GitHub API requests.
- `CPP/AVString.h`, `CPP/AVDebug.h`, `CPP/AVFiles.h`, `CPP/AVSettingsJson.h`,
  `CPP/AVProcesses/ProcessDetail.h`, `CPP/AVProcesses/Defines/AVProcess.h`,
  `CPP/AVProcesses/Defines/ProcessDeclarationsClass.h`, and
  `CPP/AVProcesses/ProcessGetAll.h` contain small compatibility additions for
  wide/Unicode paths, UTF-8 log output, wide process accessors, and the
  wide-name process lookup already used by RadeonTuner. Existing narrow APIs
  are retained.
