use crate::panel::shape::ContinuousCorners;
use crate::{
    adl::{Device, Driver, Metric},
    settings::Settings,
};
use anyhow::Result;
use gpui::{
    App, Application, Bounds, Context, Div, Timer, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use std::time::Duration;

pub const TITLE: &str = "RadeonTuner · Rust";
mod controls;
mod curve;
mod display;
mod graphics;
mod layout;
mod multimedia;
mod numeric;
mod picker;
mod settings_page;
mod shape;
mod theme;
mod transfer;
mod updates;
const PAGES: [&str; 7] = ["调校", "风扇", "显卡", "显示", "多媒体", "设置", "信息"];
mod tuning;
struct Panel {
    picker: Option<picker::Picker>,
    dropdown: Option<controls::Dropdown>,
    updates: updates::Updates,
    start_check_update: bool,
    picker_generation: u64,
    pending_import: Option<transfer::Pending>,
    style: i64,
    palette: theme::Palette,
    mpo: bool,
    excludes_drivers: bool,
    admin: bool,
    edit: Option<numeric::Edit>,
    slider: Option<numeric::Field>,
    curve_drag: Option<usize>,
    page: usize,
    info: Vec<String>,
    games: graphics::apps::Games,
    graphics: Vec<crate::adl::graphics::Setting>,
    graphics_status: String,
    graphics_profile: serde_json::Value,
    graphics_dirty: bool,
    confirm_graphics_reset: bool,
    confirm_cache: bool,
    chill_linked: bool,
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
    video_app: String,
    video_apps: Vec<String>,
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
    fn nav_tile(&self, i: usize, label: &str, selected: bool) -> Div {
        let palette = self.palette;
        div()
            .relative()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .h(px(58.))
            .gap_1()
            .when(selected, |row| {
                row.smooth_background(palette.selected).child(
                    div()
                        .absolute()
                        .left_0()
                        .top(px(15.))
                        .w(px(2.))
                        .h(px(28.))
                        .smooth_background(palette.accent),
                )
            })
            .hover(move |row| row.bg(rgb(palette.hover)))
            .child(layout::nav_icon(i, palette.light))
            .child(label.to_string())
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
        let mut content = div().flex().flex_col().w_full().gap_3().px_4().pb_4();
        if self.page == 6 || self.updates.has_notice() {
            content = content.child(self.update_controls(cx));
        }
        if !self.message.is_empty() && self.page != 5 {
            content = content.child(self.message.clone());
        }
        if matches!(self.page, 0 | 1) && !self.controls.is_empty() {
            content = content
                .child(self.render_tuning_profiles(cx))
                .child(self.render_tuning_fields(cx));
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
            content = content.child(self.render_settings(cx));
        }
        let mut nav = div()
            .id("navigation")
            .w(px(82.))
            .flex_shrink_0()
            .overflow_y_scroll()
            .p_2()
            .flex()
            .flex_col()
            .gap_2()
            .border_r_1()
            .border_color(rgb(self.palette.border));
        for (i, label) in PAGES.iter().enumerate() {
            nav = nav.child(
                self.nav_tile(i, label, i == self.page)
                    .id(("nav", i))
                    .cursor_pointer()
                    .on_click(cx.listener(move |p, _, _, cx| {
                        if p.page == 3 && i != 3 {
                            p.finish_preview(false);
                        }
                        p.dropdown = None;
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
            self.nav_tile(7, "退出", false)
                .id("exit")
                .cursor_pointer()
                .on_click(cx.listener(|p, _, _, cx| {
                    if let Err(err) = crate::tray::request_exit() {
                        p.message = err.to_string();
                        cx.notify();
                    }
                })),
        );
        let mut window_content = div()
            .relative()
            .size_full()
            .text_size(px(13.))
            .font_family("Microsoft YaHei UI")
            .flex()
            .text_color(rgb(self.palette.text))
            .bg(rgb(self.palette.canvas))
            .child(nav)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(self.render_header(cx))
                    .child(
                        div()
                            .id(("content", self.page))
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .child(content),
                    ),
            );
        if self.picker.is_some() {
            window_content = window_content.child(self.modal(self.render_picker(cx)));
        } else if self.page == 3 {
            if let Some(dialog) = self.display_dialog(cx) {
                window_content = window_content.child(self.modal(dialog));
            }
        }
        window_content
    }
}

pub fn run() -> Result<()> {
    let settings = Settings::open()?;
    let start_visible = settings.flag("StartWindowVisible", false)?;
    let close_tray = settings.flag("CloseTray", true)?;
    let style = settings.integer("ThemeStyle", 0)?;
    let mut panel = Panel {
        picker: None,
        dropdown: None,
        updates: Default::default(),
        start_check_update: settings.flag("StartCheckUpdate", true)?,
        picker_generation: 0,
        pending_import: None,
        style,
        palette: theme::Palette::resolve(style, false),
        admin: unsafe { windows_sys::Win32::UI::Shell::IsUserAnAdmin() != 0 },
        mpo: crate::integration::mpo_disabled(),
        excludes_drivers: crate::integration::excludes_drivers(),
        edit: None,
        slider: None,
        curve_drag: None,
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
        games: Default::default(),
        graphics: Vec::new(),
        graphics_status: String::new(),
        graphics_profile: serde_json::json!({}),
        graphics_dirty: false,
        confirm_graphics_reset: false,
        confirm_cache: false,
        chill_linked: false,
        frames: Vec::new(),
        fsr_activity: Vec::new(),
        fsr_library: None,
        settings_message: String::new(),
        tuning: Default::default(),
        display: Default::default(),
        video: Vec::new(),
        video_app: "Global".into(),
        video_apps: Vec::new(),
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
        let bounds = Bounds::centered(None, size(px(860.), px(720.)), cx);
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
                    if panel.start_check_update && std::env::args_os().any(|a| a == "--first-panel")
                    {
                        panel.check_updates(false, cx);
                    }
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
