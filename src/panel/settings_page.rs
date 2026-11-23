use super::Panel;
use crate::integration::{self, Link};
use gpui::{Context, Div, div, prelude::*};

impl Panel {
    pub(super) fn render_settings(&self, cx: &mut Context<Self>) -> Div {
        let mut page = div().flex().flex_col();
        for (i, link, label) in [
            (0, Link::Startup, "Windows 启动时自动运行"),
            (1, Link::StartMenu, "添加到开始菜单"),
            (2, Link::ContextMenu, "添加到桌面右键菜单"),
        ] {
            let enabled = link != Link::ContextMenu || self.admin;
            page = page.child(self.toggle_row(
                ("link", i),
                label,
                self.links[i],
                enabled,
                move |p, _| match link.set(!p.links[i]) {
                    Ok(()) => p.links[i] = link.enabled(),
                    Err(e) => p.settings_message = e.to_string(),
                },
                cx,
            ));
        }
        for (key, label, on) in [
            ("StartWindowVisible", "启动时显示窗口", self.start_visible),
            ("SetTopMost", "窗口置顶", self.topmost),
            ("CloseTray", "关闭到系统托盘", self.close_tray),
            ("HideTrayIcon", "隐藏托盘图标", self.hide_tray),
        ] {
            page = page.child(self.preference(key, label, on, cx));
        }
        page = page.child(
            self.setting_row(
                "主题颜色样式",
                self.select(
                    "theme",
                    ["灰色", "黑色", "白色", "跟随系统"]
                        .map(str::to_owned)
                        .into(),
                    Some(self.style as usize),
                    "主题颜色样式",
                    160.,
                    |p, i, _| match p.settings.set("ThemeStyle", serde_json::json!(i)) {
                        Ok(()) => p.style = i as i64,
                        Err(e) => p.settings_message = e.to_string(),
                    },
                    cx,
                ),
            ),
        );
        for (key, label, on) in [
            ("ThemeIconMono", "显示单色应用程序图标", self.mono),
            ("ShowExperimental", "显示实验性设置", self.experimental),
            (
                "SkipSupportCheckGraphics",
                "跳过显卡功能支持检查",
                self.skip_graphics_check,
            ),
        ] {
            page = page.child(self.preference(key, label, on, cx));
        }
        let mut services = div().flex().gap_2();
        for (enable, label) in [(false, "禁用 Radeon 服务"), (true, "启用 Radeon 服务")] {
            let button = self
                .button_style(self.admin)
                .id(("services", usize::from(enable)))
                .child(label);
            services = services.child(if self.admin {
                button
                    .cursor_pointer()
                    .on_click(cx.listener(move |p, _, _, cx| {
                        p.settings_message = integration::services::set_enabled(enable)
                            .map(|()| {
                                if enable {
                                    "已启用 Radeon 服务".into()
                                } else {
                                    "已禁用 Radeon 服务".into()
                                }
                            })
                            .unwrap_or_else(|e| e.to_string());
                        cx.notify();
                    }))
            } else {
                button
            });
        }
        page.child(self.setting_row("Radeon 服务", services))
            .child(self.preference(
                "StartCheckUpdate",
                "启动时检查更新",
                self.start_check_update,
                cx,
            ))
            .child(self.toggle_row(
                "mpo",
                "禁用多平面叠加（MPO）",
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
                "从 Windows 更新中排除驱动程序",
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
            ))
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
            label,
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
