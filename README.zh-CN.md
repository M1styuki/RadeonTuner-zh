# RadeonTuner 简体中文版

基于 [dumbie/RadeonTuner](https://github.com/dumbie/RadeonTuner) 0.70 的个人定制版本。保留上游功能，提供简体中文、深色界面、整齐的设置布局、条件设置折叠与动画，并新增隐藏系统托盘图标选项。原项目及第三方许可文件保持原样。

## 运行

交付文件为 `RadeonTuner-简体中文版.exe`。首次运行会将必要文件释放到 `%LOCALAPPDATA%\RadeonTuner-zh-CN\App`；主设置位于其中的 `Settings\Settings.json`。隐藏托盘后，再次运行 EXE 可唤回窗口。

## 构建

使用 Windows x64、Visual Studio 2026 Build Tools（MSVC v145）、Windows SDK 10.0.26100.0 和 C++ UWP 构建支持。NuGet 包版本由 `RadeonTuner\packages.config` 固定，第三方 C++ 源码已保存在 `third_party`。

在开发者 PowerShell 中执行：

```powershell
nuget restore RadeonTuner.sln -PackagesDirectory packages
MSBuild RadeonTuner.sln /m:2 /p:Configuration=Release /p:Platform=x64 /p:SignOutput=false
pwsh -File tools\Portable\Build-Portable.ps1 -OutFile .build-work\delivery\RadeonTuner-简体中文版.exe
```

打包说明见 [tools/Portable/README.md](tools/Portable/README.md)。构建缓存、开发工具、用户设置和测试输出均不作为源码提交。

## 界面验证

`tools\verify-localization.ps1` 与 `tools\verify-ui-layout.ps1` 检查汉化及布局。添加 `/p:UiValidation=true` 可构建独立隔离版，其 `-ui-animation-check` 验证开关连点、反向动画及条件行折叠；`-ui-lifecycle-check` 验证关闭到托盘后旧页面释放、计时器停止、草稿恢复、快速重开以及托盘/重复 EXE 唤回，并记录同进程 WorkingSet/PrivateBytes 样本。该版本阻断 ADL 初始化及用户设置保存，仅用于界面检查。

尚未取得相同实机条件下的完整程序内存前后对比，不宣称具体内存降幅。