# RadeonTuner 简体中文版

# Rust + GPUI 0.75.1

此分支的程序入口是 `Cargo.toml` 和 `src/main.rs`。下载 [GitHub Releases](https://github.com/M1styuki/RadeonTuner-zh/releases) 中 Rust 标签下的 `RadeonTuner-Rust-x64.exe`，在 Windows x64 上直接运行。首次切换版本前请退出 WinUI 版，两版使用相同的本地设置及配置目录。

- 调校、图形和显示支持原版 `.radt`、`.radg` 和 `.radd`。多媒体应用配置从原有 `MultimediaProfiles.json` 读取。
- Gamma 采用适配器级 LUT，关闭时恢复本程序首次更改前读取的 LUT。恢复记录位于 App 的 `Settings\RustGammaRecovery.json`。
- 私有仓库用户可登录浏览器手动下载安装包。内置更新可使用仅授予此仓库 Contents 读取权限的 GitHub 细粒度令牌，从启动进程的 `RADEONTUNER_GITHUB_TOKEN` 环境变量读取，也兼容已登录的 GitHub CLI。程序不保存令牌。
- GitHub Actions 的 **Rust Windows** 工作流运行 Rust 测试并输出 EXE 和 SHA-256。推送与 Cargo 版本一致的 `rust-vX.Y.Z` 标签后生成待审核的草稿发布。

```powershell
cargo fmt --check
cargo test --locked --release
cargo build --locked --release
```

## C++ / WinUI 参考源码

本仓库还保留基于 [dumbie/RadeonTuner](https://github.com/dumbie/RadeonTuner) 的 WinUI 源码，用于对照原有设置和行为。原项目及第三方许可文件保持原样。以下内容仅适用于 C++ 版。

范围风扇模式支持“线性”和“静音”曲线。伽马关闭时，仅在本程序实际修改过对应适配器后恢复默认值，避免无故覆盖 ICC 校准。Radeon 服务启停需要管理员权限且仅由手动按钮触发；禁用相关服务会影响 AFMF 和 Radeon Boost。

### 运行

交付文件为 `RadeonTuner-简体中文版.exe`。首次运行会将必要文件释放到 `%LOCALAPPDATA%\RadeonTuner-zh-CN\App`；主设置位于其中的 `Settings\Settings.json`。隐藏托盘后，再次运行 EXE 可唤回窗口。

正式版使用原生后台与按需启动的独立设置面板。启用“关闭到系统托盘”后，关闭面板会结束界面进程；后台保留配置保持和应用配置切换等原功能。再次打开时恢复页面、设备选择和未应用输入。托盘图标按任务栏 DPI 选择原生尺寸，150% 缩放对应 24×24 像素。

### 构建

使用 Windows x64、Visual Studio 2026 Build Tools（MSVC v145）、Windows SDK 10.0.26100.0 和 C++ UWP 构建支持。NuGet 包版本由 `RadeonTuner\packages.config` 固定，第三方 C++ 源码已保存在 `third_party`。

在开发者 PowerShell 中执行：

```powershell
nuget restore RadeonTuner.sln -PackagesDirectory packages
MSBuild RadeonTuner.sln /m:2 /p:Configuration=Release /p:Platform=x64 /p:SignOutput=false
pwsh -File tools\Portable\Build-Portable.ps1 -OutFile .build-work\delivery\RadeonTuner-简体中文版.exe
```

打包说明见 [tools/Portable/README.md](tools/Portable/README.md)。构建缓存、开发工具、用户设置和测试输出均不作为源码提交。

### 界面验证

`tools\verify-localization.ps1` 与 `tools\verify-ui-layout.ps1` 检查汉化及布局。添加 `/p:UiValidation=true` 可构建独立隔离版，其 `-ui-animation-check` 验证开关连点、反向动画及条件行折叠；`-ui-lifecycle-check` 验证关闭到托盘后旧页面释放、计时器停止、草稿恢复、快速重开以及托盘/重复 EXE 唤回，并记录同进程 WorkingSet/PrivateBytes 样本。该版本阻断 ADL 初始化及用户设置保存，仅用于界面检查。

尚未取得相同实机条件下的完整程序内存前后对比，不宣称具体内存降幅。

隔离生命周期检查仍针对同进程界面会话，不覆盖正式版的跨进程交接。本轮按用户要求仅进行代码检查及云端编译，不自动运行程序或采集内存。
