# RadeonTuner

面向 AMD Radeon 显卡的轻量简体中文桌面控制面板，专为安装「仅驱动程序（Driver Only）」但需要核心调校与显示设置的用户设计。支持 Windows 11 原生 Mica 外观与浅/深色切换，单文件直接运行，无需安装额外运行时。

[下载最新版](https://github.com/M1styuki/RadeonTuner-zh/releases/latest)

## 主要功能

- **调校与风扇控制**：GPU 频率、电压、显存时序、功耗与零转速/曲线/范围风扇调校，支持驱动程序重置后自动重新应用及随应用动态提频（Power Boost）。
- **游戏图形设置**：支持全局及单应用/游戏配置文件（`.radg`），覆盖 Anti-Lag、Radeon Boost、AFMF、FSR / FSR 4、Chill、Image Sharpening 等选项。
- **显示设置与扩展**：刷新率与色彩格式、FreeSync、HDR、CVD 色觉辅助、显示颜色增强、适配器级 LUT 伽马校准（关闭时安全恢复）、自定义分辨率管理及 Eyefinity 多屏拼接。
- **多媒体与配置管理**：多媒体增强及应用规则配置，支持 `.radt` / `.radg` / `.radd` 配置导入导出，兼容原有配置格式。
- **系统集成与托盘**：开机自启、关闭至系统托盘、单色托盘图标、Windows 更新驱动排除、MPO 开关，以及基于 SHA-256 校验的应用内更新。

## 系统要求

- Windows 10（18362）64 位或 Windows 11 64 位
- AMD Radeon 驱动程序 25.5.1 或更高版本（建议使用「仅驱动程序 / Driver Only」安装）
- AMD Radeon RDNA 或更新架构显卡

## 快速开始

1. 前往 [GitHub Releases](https://github.com/M1styuki/RadeonTuner-zh/releases/latest) 下载 `RadeonTuner.exe`。
2. 放在任意目录直接运行即可；涉及驱动服务、桌面右键菜单、MPO 或排错等高级项时按界面提示以管理员身份运行。

## 从源码构建

需要 Rust 工具链（`Cargo.toml` 与 `Cargo.lock` 已锁定依赖）及 Windows x64 MSVC 构建环境：

```powershell
cargo fmt --check
cargo test --locked
cargo build --release --locked
```

构建产物位于 `target\release\radeon-tuner.exe`。

## 致谢与许可

本项目基于 [dumbie/RadeonTuner](https://github.com/dumbie/RadeonTuner) 进行原生化重构与简体中文适配。相关许可声明请见 [License.md](License.md)。
