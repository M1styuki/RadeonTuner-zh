use super::Panel;
use crate::integration::{self, Link, services::State};
use gpui::{Context, Div, div, prelude::*, rgb};

impl Panel {
    pub(super) fn render_settings(&self, cx: &mut Context<Self>) -> Div {
        let link_toggle = |i: usize, link: Link, label, cx: &mut Context<Self>| {
            self.toggle_row(
                ("link", i),
                self.hint_label(label),
                self.links[i],
                link != Link::ContextMenu || self.admin,
                move |p, _| match link.set(!p.links[i]) {
                    Ok(()) => p.links[i] = link.enabled(),
                    Err(e) => p.settings_message = e.to_string(),
                },
                cx,
            )
        };
        let startup = div()
            .flex()
            .flex_col()
            .child(link_toggle(0, Link::Startup, "Windows 启动时自动运行", cx))
            .child(self.preference(
                "StartWindowVisible",
                "启动时显示窗口",
                self.start_visible,
                cx,
            ))
            .child(self.preference(
                "StartCheckUpdate",
                "启动时检查更新",
                self.start_check_update,
                cx,
            ));
        let shortcuts = div()
            .flex()
            .flex_col()
            .child(link_toggle(1, Link::StartMenu, "添加到开始菜单", cx))
            .child(link_toggle(2, Link::ContextMenu, "添加到桌面右键菜单", cx));
        let appearance = div()
            .flex()
            .flex_col()
            .child(
                self.setting_row(
                    self.hint_label("主题颜色样式"),
                    self.select(
                        "theme",
                        ["灰色", "黑色", "白色", "跟随系统"]
                            .map(str::to_owned)
                            .into(),
                        Some(self.style as usize),
                        "主题颜色样式",
                        crate::panel::controls::SELECT_MAX_WIDTH,
                        |p, i, _| match p.settings.set("ThemeStyle", serde_json::json!(i)) {
                            Ok(()) => p.style = i as i64,
                            Err(e) => p.settings_message = e.to_string(),
                        },
                        cx,
                    ),
                ),
            )
            .child(self.preference("SetTopMost", "窗口置顶", self.topmost, cx));
        let tray = div()
            .flex()
            .flex_col()
            .child(self.preference("ThemeIconMono", "显示单色应用程序图标", self.mono, cx))
            .child(self.preference("HideTrayIcon", "隐藏托盘图标", self.hide_tray, cx))
            .child(self.preference("CloseTray", "关闭到系统托盘", self.close_tray, cx));
        let advanced = div()
            .flex()
            .flex_col()
            .child(self.preference("ShowExperimental", "显示实验性设置", self.experimental, cx))
            .child(self.preference(
                "SkipSupportCheckGraphics",
                "跳过显卡功能支持检查",
                self.skip_graphics_check,
                cx,
            ));
        let service_on = self.radeon_services == Some(State::Enabled);
        let service_available = matches!(
            self.radeon_services,
            Some(State::Enabled | State::Disabled | State::Mixed)
        );
        let service_hint = match self.radeon_services {
            None => Some("无法读取状态"),
            Some(State::Missing) => Some("未安装"),
            Some(State::Mixed) if !self.admin => Some("部分启用 · 需要管理员权限"),
            Some(State::Mixed) => Some("部分启用"),
            _ if !self.admin => Some("需要管理员权限"),
            _ => None,
        };
        let service_label = div()
            .flex()
            .items_center()
            .gap_2()
            .child("Radeon 服务")
            .children(
                service_hint.map(|hint| div().text_color(rgb(self.palette.muted_text)).child(hint)),
            );
        let service_label = self.with_native_hints(service_label, "Radeon 服务");
        let advanced = advanced
            .child(self.toggle_row(
                "services",
                service_label,
                service_on,
                service_available && self.admin,
                move |p, _| {
                    let result = integration::services::set_enabled(!service_on);
                    p.radeon_services = integration::services::state().ok();
                    p.settings_message = result
                        .map(|()| {
                            if service_on {
                                "已禁用 Radeon 服务".into()
                            } else {
                                "已启用 Radeon 服务".into()
                            }
                        })
                        .unwrap_or_else(|e| e.to_string());
                },
                cx,
            ))
            .child(self.toggle_row(
                "mpo",
                self.hint_label("禁用多平面叠加（MPO）"),
                self.mpo,
                self.admin,
                |p, _| match integration::set_mpo_disabled(!p.mpo) {
                    Ok(()) => {
                        p.mpo = !p.mpo;
                        p.settings_message = "MPO 设置已保存，重启电脑后生效".into();
                    }
                    Err(e) => p.settings_message = e.to_string(),
                },
                cx,
            ))
            .child(self.toggle_row(
                "exclude-drivers",
                self.hint_label("从 Windows 更新中排除驱动程序"),
                self.excludes_drivers,
                self.admin,
                |p, _| match integration::set_excludes_drivers(!p.excludes_drivers) {
                    Ok(()) => {
                        p.excludes_drivers = integration::excludes_drivers();
                        p.settings_message = "Windows 更新设置已保存".into();
                    }
                    Err(e) => p.settings_message = e.to_string(),
                },
                cx,
            ));
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(self.setting_sections([startup, appearance, shortcuts, advanced, tray]))
            .when(!self.settings_message.is_empty(), |page| {
                page.child(self.settings_message.clone())
            })
    }

    fn preference(
        &self,
        key: &'static str,
        label: &'static str,
        on: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        self.toggle_row(
            key,
            self.hint_label(label),
            on,
            true,
            move |p, _| {
                let new = !on;
                if let Err(e) = p.settings.set_flag(key, new) {
                    p.settings_message = e.to_string();
                    return;
                }
                let result = match key {
                    "SetTopMost" => {
                        p.topmost = new;
                        crate::tray::topmost(new)
                    }
                    "HideTrayIcon" => {
                        p.hide_tray = new;
                        crate::tray::settings_changed()
                    }
                    "ThemeIconMono" => {
                        p.mono = new;
                        crate::tray::settings_changed()
                    }
                    "ShowExperimental" => {
                        p.experimental = new;
                        Ok(())
                    }
                    "SkipSupportCheckGraphics" => {
                        p.skip_graphics_check = new;
                        p.read_device_details();
                        Ok(())
                    }
                    "StartCheckUpdate" => {
                        p.start_check_update = new;
                        Ok(())
                    }
                    "CloseTray" => {
                        p.close_tray = new;
                        Ok(())
                    }
                    _ => {
                        p.start_visible = new;
                        Ok(())
                    }
                };
                if let Err(e) = result {
                    p.settings_message = e.to_string();
                }
            },
            cx,
        )
    }
}
