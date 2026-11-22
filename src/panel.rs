use crate::{
    adl::{Device, Driver, Metric},
    settings::Settings,
};
use anyhow::Result;
use gpui::{
    App, Application, Bounds, Context, Timer, Window, WindowBounds, WindowOptions, div, prelude::*,
    px, rgb, size,
};
use std::time::Duration;

pub const TITLE: &str = "RadeonTuner · Rust";
mod curve;
mod display;
mod graphics;
mod multimedia;
mod numeric;
mod theme;
mod transfer;
const PAGES: [&str; 7] = ["调校", "风扇", "图形", "显示器", "多媒体", "设置", "信息"];
mod tuning;
struct Panel {
    pending_import: Option<transfer::Pending>,
    style: i64,
    palette: theme::Palette,
    mpo: bool,
    admin: bool,
    edit: Option<numeric::Edit>,
    page: usize,
    info: Vec<String>,
    graphics: Vec<crate::adl::graphics::Setting>,
    graphics_status: String,
    graphics_profile: serde_json::Value,
    graphics_dirty: bool,
    confirm_graphics_reset: bool,
    frames: Vec<crate::adl::graphics::frames::Control>,
    fsr_activity: Vec<String>,
    fsr_library: Option<crate::adl::graphics::fsr::Library>,
    settings_message: String,
    experimental: bool,
    skip_graphics_check: bool,
    topmost: bool,
    hide_tray: bool,
    mono: bool,
    links: [bool; 3],
    tuning: tuning::Editor,
    display: display::DisplayPage,
    video: Vec<crate::adl::multimedia::VideoControl>,
    video_status: String,
    confirm_video_reset: bool,
    driver: Option<Driver>,
    devices: Vec<Device>,
    selected: usize,
    metrics: Vec<Metric>,
    controls: Vec<crate::adl::overdrive::Control>,
    tuning_status: String,
    message: String,
    settings: Settings,
    start_visible: bool,
    close_tray: bool,
}

impl Panel {
    fn read_controls(&mut self) {
        self.controls.clear();
        self.tuning_status.clear();
        if let (Some(driver), Some(device)) = (&self.driver, self.devices.get(self.selected)) {
            match driver.controls(device.index) {
                Ok(values) => {
                    self.controls = values;
                    if let Err(err) = self.load_tuning() {
                        self.tuning_status = err.to_string();
                    }
                }
                Err(err) => self.tuning_status = err.to_string(),
            }
        }
    }
    fn refresh(&mut self) {
        self.expire_preview();
        if self.page == 2 {
            self.fsr_activity = crate::adl::graphics::activity::active_games();
        }
        self.metrics.clear();
        if let (Some(driver), Some(device)) = (&self.driver, self.devices.get(self.selected)) {
            match driver.metrics(device.index) {
                Ok(values) => {
                    self.metrics = values;
                    self.message.clear();
                }
                Err(err) => self.message = err.to_string(),
            }
        }
    }
}

impl Render for Panel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.palette = theme::Palette::resolve(
            self.style,
            matches!(
                window.appearance(),
                gpui::WindowAppearance::Light | gpui::WindowAppearance::VibrantLight
            ),
        );
        let mut content = div()
            .id(("content", self.page))
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_3()
            .p_5()
            .bg(rgb(self.palette.canvas))
            .text_color(rgb(self.palette.text))
            .child(div().text_xl().child(PAGES[self.page]));
        if self.page != 5 {
            for (i, device) in self.devices.iter().enumerate() {
                content = content.child(
                    div()
                        .id(("gpu", i))
                        .cursor_pointer()
                        .px_3()
                        .py_1()
                        .bg(rgb(if self.selected == i {
                            self.palette.selected
                        } else {
                            self.palette.control
                        }))
                        .child(format!("{} ({})", device.name, device.index))
                        .on_click(cx.listener(move |panel, _, _, cx| {
                            panel.finish_preview(false);
                            panel.selected = i;
                            panel.reset_tuning_editor();
                            panel.read_device_details();
                            panel.read_displays();
                            panel.read_controls();
                            panel.refresh();
                            cx.notify();
                        })),
                );
            }
            if !self.message.is_empty() {
                content = content.child(self.message.clone());
            }
        }
        if matches!(self.page, 0 | 1 | 2 | 6) {
            let mut grid = div().flex().flex_wrap();
            for item in &self.metrics {
                let value = item
                    .value
                    .map(|n| format!("{n} {}", item.unit))
                    .unwrap_or_else(|| "—".into());
                grid = grid.child(
                    div()
                        .w(gpui::relative(0.2))
                        .flex()
                        .flex_col()
                        .py_1()
                        .child(item.label)
                        .child(value),
                );
            }
            content = content.child(grid);
        }
        if matches!(self.page, 0 | 1) && !self.controls.is_empty() {
            content = content.child(self.render_tuning_profiles(cx));
            if self.page == 1 {
                content = content.child(self.render_curve(cx));
            }
            for (i, control) in self.controls.iter().enumerate() {
                if ((control.id == 15 || (19..=28).contains(&control.id)) != (self.page == 1))
                    || !self.show_control(control.id)
                {
                    continue;
                }
                let mut row = div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(control.label)
                    .child(self.number(
                        numeric::Field::Tuning(i),
                        control.draft,
                        control.min,
                        control.max,
                        control.step,
                        control.unit,
                        cx,
                    ));
                for (key, label, step) in [("down", "−", -control.step), ("up", "+", control.step)]
                {
                    row = row.child(
                        div()
                            .id((key, i))
                            .cursor_pointer()
                            .px_3()
                            .bg(rgb(self.palette.control))
                            .child(label)
                            .on_click(cx.listener(move |panel, _, _, cx| {
                                let c = &mut panel.controls[i];
                                c.draft = c.draft.saturating_add(step).clamp(c.min, c.max);
                                cx.notify();
                            })),
                    );
                }
                content = content.child(row);
            }
            if self.tuning_changed() {
                content = content.child(
                    div()
                        .id("apply")
                        .cursor_pointer()
                        .p_2()
                        .child("应用更改")
                        .on_click(cx.listener(|panel, _, _, cx| {
                            if let Err(err) = panel.save_tuning() {
                                panel.tuning_status = err.to_string();
                            }
                            cx.notify();
                        })),
                );
            }
            content = content.child(
                div()
                    .id("reload")
                    .cursor_pointer()
                    .p_2()
                    .child("重新读取驱动设置")
                    .on_click(cx.listener(|panel, _, _, cx| {
                        panel.read_controls();
                        cx.notify();
                    })),
            );
        }
        if matches!(self.page, 0 | 1) && !self.tuning_status.is_empty() {
            content = content.child(self.tuning_status.clone());
        }
        match self.page {
            2 => content = content.child(self.render_graphics(cx)),
            3 => content = content.child(self.render_displays(cx)),
            4 => content = content.child(self.render_video(cx)),
            6 => {
                let mut links = div().flex().flex_wrap().gap_3();
                for (i, (label, url)) in [
                    ("访问项目网站", "https://projects.arnoldvink.com"),
                    ("打开捐赠页面", "https://donation.arnoldvink.com"),
                    (
                        "获取 FpsOverlayer",
                        "https://github.com/dumbie/FpsOverlayer",
                    ),
                ]
                .into_iter()
                .enumerate()
                {
                    links = links.child(
                        div()
                            .id(("info-link", i))
                            .px_2()
                            .cursor_pointer()
                            .child(label)
                            .on_click(cx.listener(move |p, _, _, cx| {
                                if let Err(err) = crate::integration::open(url) {
                                    p.message = err.to_string();
                                    cx.notify();
                                }
                            })),
                    );
                }
                content = content.child(links).child(format!(
                    "RadeonTuner · 原作者 Arnold Vink · Rust 版 {}",
                    env!("CARGO_PKG_VERSION")
                ));
                for line in &self.info {
                    content = content.child(line.clone());
                }
            }
            _ => {}
        }
        if self.page == 5 {
            let mut choices = div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().flex_1().child("应用主题"));
            for (i, name) in ["灰色", "黑色", "白色", "跟随系统"].into_iter().enumerate()
            {
                choices = choices.child(
                    div()
                        .id(("theme", i))
                        .cursor_pointer()
                        .px_2()
                        .bg(rgb(if self.style == i as i64 {
                            self.palette.selected
                        } else {
                            self.palette.control
                        }))
                        .child(name)
                        .on_click(cx.listener(move |p, _, _, cx| {
                            match p.settings.set("ThemeStyle", serde_json::json!(i)) {
                                Ok(()) => p.style = i as i64,
                                Err(e) => p.settings_message = e.to_string(),
                            }
                            cx.notify();
                        })),
                );
            }
            content = content.child(choices);
            if !self.settings_message.is_empty() {
                content = content.child(self.settings_message.clone());
            }
            let mpo = div()
                .id("mpo")
                .p_2()
                .flex()
                .justify_between()
                .child("禁用多平面叠加（MPO）")
                .child(if self.mpo { "开" } else { "关" });
            content = content
                .child(if self.admin {
                    mpo.cursor_pointer().on_click(cx.listener(|p, _, _, cx| {
                        match crate::integration::set_mpo_disabled(!p.mpo) {
                            Ok(()) => {
                                p.mpo = !p.mpo;
                                p.settings_message = "MPO 设置已保存，重启电脑后生效".into();
                            }
                            Err(err) => p.settings_message = err.to_string(),
                        }
                        cx.notify();
                    }))
                } else {
                    mpo.opacity(0.5)
                })
                .child(if self.admin {
                    "更改 MPO 后需重启电脑"
                } else {
                    "更改 MPO 需要管理员权限，重启电脑后生效"
                });
            for (key, label, on) in [
                ("StartWindowVisible", "启动时打开面板", self.start_visible),
                ("CloseTray", "关闭面板后驻留托盘", self.close_tray),
                ("SetTopMost", "窗口置顶", self.topmost),
                ("HideTrayIcon", "隐藏托盘图标", self.hide_tray),
                ("ThemeIconMono", "单色托盘图标", self.mono),
                ("ShowExperimental", "显示实验性设置", self.experimental),
                (
                    "SkipSupportCheckGraphics",
                    "跳过图形功能支持检查",
                    self.skip_graphics_check,
                ),
            ] {
                content = content.child(
                    div()
                        .id(key)
                        .cursor_pointer()
                        .p_2()
                        .bg(rgb(self.palette.control))
                        .flex()
                        .justify_between()
                        .child(label)
                        .child(if on { "开" } else { "关" })
                        .on_click(cx.listener(move |panel, _, _, cx| {
                            match panel.settings.set_flag(key, !on) {
                                Ok(()) if key == "SetTopMost" => {
                                    panel.topmost = !on;
                                    if let Err(err) = crate::tray::topmost(!on) {
                                        panel.message = err.to_string();
                                    }
                                }
                                Ok(()) if key == "HideTrayIcon" || key == "ThemeIconMono" => {
                                    if key == "HideTrayIcon" {
                                        panel.hide_tray = !on;
                                    } else {
                                        panel.mono = !on;
                                    }
                                    if let Err(err) = crate::tray::settings_changed() {
                                        panel.message = err.to_string();
                                    }
                                }
                                Ok(()) if key == "ShowExperimental" => panel.experimental = !on,
                                Ok(()) if key == "SkipSupportCheckGraphics" => {
                                    panel.skip_graphics_check = !on;
                                    panel.read_device_details();
                                }
                                Ok(()) if key == "CloseTray" => panel.close_tray = !on,
                                Ok(()) => panel.start_visible = !on,
                                Err(err) => panel.message = format!("保存失败：{err}"),
                            }
                            cx.notify();
                        })),
                );
            }
            use crate::integration::Link;
            for (i, (link, label)) in [
                (Link::Startup, "随 Windows 启动"),
                (Link::StartMenu, "开始菜单快捷方式"),
                (Link::ContextMenu, "桌面右键菜单"),
            ]
            .into_iter()
            .enumerate()
            {
                let on = self.links[i];
                content = content.child(
                    div()
                        .id(("link", i))
                        .p_2()
                        .flex()
                        .justify_between()
                        .cursor_pointer()
                        .child(label)
                        .child(if on { "开" } else { "关" })
                        .on_click(cx.listener(move |p, _, _, cx| {
                            match link.set(!on) {
                                Ok(()) => p.links[i] = link.enabled(),
                                Err(err) => p.message = err.to_string(),
                            }
                            cx.notify();
                        })),
                );
            }
            if !self.message.is_empty() {
                content = content.child(self.message.clone());
            }
        }
        let mut nav = div()
            .w(px(110.))
            .flex_shrink_0()
            .p_2()
            .flex()
            .flex_col()
            .gap_1()
            .bg(rgb(self.palette.sidebar));
        for (i, label) in PAGES.iter().enumerate() {
            nav = nav.child(
                div()
                    .id(("nav", i))
                    .px_3()
                    .py_2()
                    .cursor_pointer()
                    .bg(rgb(if i == self.page {
                        self.palette.selected
                    } else {
                        self.palette.sidebar
                    }))
                    .child(*label)
                    .on_click(cx.listener(move |p, _, _, cx| {
                        if p.page == 3 && i != 3 {
                            p.finish_preview(false);
                        }
                        p.page = i;
                        if i == 2 {
                            p.fsr_activity = crate::adl::graphics::activity::active_games();
                        }
                        if let Err(err) = p.settings.set("MenuIndex", serde_json::json!(i)) {
                            p.message = err.to_string();
                        }
                        cx.notify();
                    })),
            );
        }
        nav = nav.child(
            div()
                .id("exit")
                .px_3()
                .py_2()
                .cursor_pointer()
                .child("退出")
                .on_click(cx.listener(|p, _, _, cx| {
                    if let Err(err) = crate::tray::request_exit() {
                        p.message = err.to_string();
                        cx.notify();
                    }
                })),
        );
        div()
            .size_full()
            .flex()
            .text_color(rgb(self.palette.text))
            .child(nav)
            .child(content)
    }
}

pub fn run() -> Result<()> {
    let settings = Settings::open()?;
    let start_visible = settings.flag("StartWindowVisible", false)?;
    let close_tray = settings.flag("CloseTray", true)?;
    let style = settings.integer("ThemeStyle", 0)?;
    let mut panel = Panel {
        pending_import: None,
        style,
        palette: theme::Palette::resolve(style, false),
        admin: unsafe { windows_sys::Win32::UI::Shell::IsUserAnAdmin() != 0 },
        mpo: crate::integration::mpo_disabled(),
        edit: None,
        page: usize::try_from(settings.integer("MenuIndex", 0)?)
            .ok()
            .filter(|i| *i < PAGES.len())
            .unwrap_or(0),
        topmost: settings.flag("SetTopMost", false)?,
        hide_tray: settings.flag("HideTrayIcon", false)?,
        mono: settings.flag("ThemeIconMono", false)?,
        links: [
            crate::integration::Link::Startup,
            crate::integration::Link::StartMenu,
            crate::integration::Link::ContextMenu,
        ]
        .map(|link| link.enabled()),
        experimental: settings.flag("ShowExperimental", false)?,
        skip_graphics_check: settings.flag("SkipSupportCheckGraphics", false)?,
        info: Vec::new(),
        graphics: Vec::new(),
        graphics_status: String::new(),
        graphics_profile: serde_json::json!({}),
        graphics_dirty: false,
        confirm_graphics_reset: false,
        frames: Vec::new(),
        fsr_activity: Vec::new(),
        fsr_library: None,
        settings_message: String::new(),
        tuning: Default::default(),
        display: Default::default(),
        video: Vec::new(),
        video_status: String::new(),
        confirm_video_reset: false,
        driver: None,
        devices: vec![],
        selected: 0,
        metrics: vec![],
        controls: vec![],
        tuning_status: String::new(),
        message: String::new(),
        settings,
        start_visible,
        close_tray,
    };
    match Driver::open().and_then(|driver| Ok((driver.devices()?, driver))) {
        Ok((devices, driver)) => {
            if devices.is_empty() {
                panel.message = "未检测到可访问的 AMD 显卡".into();
            }
            panel.devices = devices;
            panel.driver = Some(driver);
            panel.read_device_details();
            panel.read_displays();
            panel.read_controls();
            panel.refresh();
        }
        Err(err) => panel.message = format!("{err:#}"),
    }
    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(670.), px(740.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                window.set_window_title(TITLE);
                if panel.topmost {
                    if let Err(err) = crate::tray::topmost(true) {
                        panel.message = err.to_string();
                    }
                }
                cx.new(|cx: &mut Context<Panel>| {
                    cx.spawn(async move |view, cx| {
                        loop {
                            Timer::after(Duration::from_secs(2)).await;
                            if view
                                .update(cx, |panel, cx| {
                                    panel.refresh();
                                    cx.notify();
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                    })
                    .detach();
                    if panel.settings.flag("FirstLaunch", true).unwrap_or(false) {
                        if let Err(err) = panel.settings.set_flag("FirstLaunch", false) {
                            panel.message = format!("保存首次启动状态失败：{err}");
                        }
                    }
                    panel
                })
            },
        )
        .expect("无法创建 GPUI 面板");
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        cx.activate(true);
    });
    Ok(())
}
