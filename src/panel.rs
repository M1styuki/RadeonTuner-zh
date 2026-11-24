use crate::panel::shape::ContinuousCorners;
use crate::{
    adl::{Device, Driver, Metric},
    integration::services,
    settings::Settings,
};
use anyhow::Result;
use gpui::{
    App, Application, Bounds, Context, Div, Timer, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{cell::RefCell, collections::HashMap, time::Duration};
use windows_sys::Win32::UI::WindowsAndMessaging::IsIconic;

pub const TITLE: &str = "RadeonTuner · Rust";
mod controls;
mod curve;
mod display;
mod graphics;
mod layout;
mod motion;
mod multimedia;
mod numeric;
mod picker;
mod settings_page;
mod shape;
mod theme;
mod tooltips;
mod transfer;
mod updates;
const NAV_WIDTH: f32 = 82.;
const PAGES: [&str; 7] = ["调校", "风扇", "显卡", "显示", "多媒体", "设置", "信息"];
mod tuning;
struct Panel {
    picker: Option<picker::Picker>,
    dropdown: Option<controls::Dropdown>,
    control_glyph_widths: RefCell<HashMap<char, f32>>,
    animations: bool,
    updates: updates::Updates,
    start_check_update: bool,
    picker_generation: u64,
    pending_import: Option<transfer::Pending>,
    style: i64,
    palette: theme::Palette,
    mpo: bool,
    excludes_drivers: bool,
    admin: bool,
    radeon_services: Option<services::State>,
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
            .smooth_hover_background(
                selected.then_some(palette.selected),
                palette.hover,
                shape::NAV_CORNER,
            )
            .child(motion::transition(
                "nav-marker",
                if selected { 1. } else { 0. },
                self.animations,
                move |t| {
                    div()
                        .absolute()
                        .left_0()
                        .top(px(29. - 14. * t))
                        .w(px(2.))
                        .h(px(28. * t))
                        .opacity(t)
                        .when(t > 0., |marker| {
                            marker.child(shape::surface_radius(palette.accent, 1.))
                        })
                },
            ))
            .child(layout::nav_icon(i, palette.text))
            .child(label.to_string())
    }
    fn refresh(&mut self) -> bool {
        let mut changed = false;
        if self.page == 2 {
            changed |= replace_changed(
                &mut self.fsr_activity,
                crate::adl::graphics::activity::active_games(),
            );
        }
        if self.page == 5 {
            return changed;
        }
        let (Some(driver), Some(device)) = (&self.driver, self.devices.get(self.selected)) else {
            return changed | replace_changed(&mut self.metrics, Vec::new());
        };
        let (metrics, message) = match driver.metrics(device.index) {
            Ok(values) => (values, String::new()),
            Err(err) => (Vec::new(), err.to_string()),
        };
        changed
            | replace_changed(&mut self.metrics, metrics)
            | replace_changed(&mut self.message, message)
    }
}

fn replace_changed<T: PartialEq>(current: &mut T, next: T) -> bool {
    if *current == next {
        return false;
    }
    *current = next;
    true
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
            .flex()
            .flex_col()
            .w_full()
            .gap_3()
            .px_4()
            .pb_4()
            .when(self.page != 5, |content| content.pt_3());
        if self.page != 6 && self.updates.has_notice() {
            content = content.child(self.update_controls(cx));
        }
        if !self.message.is_empty() && self.page != 5 {
            content = content.child(self.message.clone());
        }
        if matches!(self.page, 0 | 1) && !self.controls.is_empty() {
            content = content.child(self.render_tuning_fields(cx));
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
                let version = div().flex().flex_col().gap_2().child(format!(
                    "RadeonTuner · 原作者 Arnold Vink · Rust 版 {}",
                    env!("CARGO_PKG_VERSION")
                ));
                let device = div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(self.info.iter().cloned());
                let groups = std::iter::once(version.child(self.update_controls(cx)))
                    .chain((!self.info.is_empty()).then_some(device))
                    .chain(std::iter::once(links));
                content = content.child(self.setting_sections(groups));
            }
            _ => {}
        }
        if self.page == 5 {
            content = content.child(self.render_settings(cx));
        }
        let mut nav = div()
            .id("navigation")
            .w(px(NAV_WIDTH))
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
                        let was_settings = p.page == 5;
                        p.page = i;
                        if was_settings && i != 5 {
                            p.refresh();
                        } else if i == 2 {
                            p.fsr_activity = crate::adl::graphics::activity::active_games();
                        }
                        if i == 5 {
                            p.radeon_services = services::state().ok();
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
                    .child(self.render_header(window.viewport_size().width, cx))
                    .child(
                        div()
                            .id(("content", self.page))
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .child(motion::reveal(
                                content,
                                ("page-entrance", self.page),
                                self.animations,
                            )),
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

fn set_window_icon(window: &Window) {
    use windows_sys::Win32::{
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{ICON_BIG, ICON_SMALL, LoadIconW, SendMessageW, WM_SETICON},
    };

    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    // Resource ID 1 is the application icon in app-icon.rc.
    unsafe {
        let icon = LoadIconW(GetModuleHandleW(std::ptr::null()), 1 as *const u16);
        if !icon.is_null() {
            let hwnd = handle.hwnd.get() as _;
            SendMessageW(hwnd, WM_SETICON, ICON_BIG as usize, icon as isize);
            SendMessageW(hwnd, WM_SETICON, ICON_SMALL as usize, icon as isize);
        }
    }
}

pub fn run() -> Result<()> {
    open_panel(true, |panel| {
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
    })
}

fn open_panel(poll_hardware: bool, initialize: impl FnOnce(&mut Panel)) -> Result<()> {
    let settings = Settings::open()?;
    let start_visible = settings.flag("StartWindowVisible", false)?;
    let close_tray = settings.flag("CloseTray", true)?;
    let style = settings.integer("ThemeStyle", 0)?;
    let mut panel = Panel {
        picker: None,
        dropdown: None,
        control_glyph_widths: Default::default(),
        animations: motion::enabled(),
        updates: Default::default(),
        start_check_update: settings.flag("StartCheckUpdate", true)?,
        picker_generation: 0,
        pending_import: None,
        style,
        palette: theme::Palette::resolve(style, false),
        admin: unsafe { windows_sys::Win32::UI::Shell::IsUserAnAdmin() != 0 },
        radeon_services: services::state().ok(),
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
    initialize(&mut panel);
    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(860.), px(720.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                window.set_window_title(TITLE);
                set_window_icon(window);
                let hwnd = window
                    .window_handle()
                    .ok()
                    .and_then(|handle| match handle.as_raw() {
                        RawWindowHandle::Win32(handle) => Some(handle.hwnd.get()),
                        _ => None,
                    });
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
                    if poll_hardware {
                        cx.spawn(async move |view, cx| {
                            loop {
                                Timer::after(Duration::from_secs(2)).await;
                                if view
                                    .update(cx, |panel, cx| {
                                        let preview_active = panel.poll_preview();
                                        let visible = hwnd
                                            .is_none_or(|hwnd| unsafe { IsIconic(hwnd as _) == 0 });
                                        let readings_changed = visible && panel.refresh();
                                        if preview_active || readings_changed {
                                            cx.notify();
                                        }
                                    })
                                    .is_err()
                                {
                                    break;
                                }
                            }
                        })
                        .detach();
                    }
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

#[cfg(test)]
#[test]
#[ignore = "opens a window for an interactive fan-layout check"]
fn preview_fan_page() {
    open_panel(false, |panel| {
        use crate::adl::graphics::frames::{Control, Field};
        panel.page = 1;
        panel.message = "界面预览（示例数据）".into();
        panel.experimental = true;
        panel.devices.push(Device {
            gamma_key: String::new(),
            registry_path: String::new(),
            profile_key: String::new(),
            pci_location: None,
            index: 0,
            name: "AMD Radeon RX 9070 XT".into(),
        });
        panel.games = graphics::apps::Games::preview_fixture();
        panel.fsr_library = Some(crate::adl::graphics::fsr::Library::preview_fixture());
        panel.display = display::DisplayPage::preview_fixture();
        use crate::adl::multimedia::{Feature as VideoFeature, VideoControl};
        panel.video = [
            (VideoFeature::Brightness, 0),
            (VideoFeature::Sharpness, 50),
            (VideoFeature::Upscale, 1),
        ]
        .into_iter()
        .map(|(feature, value)| VideoControl {
            feature,
            current: value,
            saved: value,
            draft: value,
        })
        .collect();
        panel.controls.extend(
            [
                (9, "功耗限制", "%", -10, 15, 0),
                (0, "最高核心频率", "MHz", 2300, 3200, 2900),
                (8, "最高显存频率", "MHz", 2000, 2700, 2518),
                (37, "电压偏移", "mV", -60, 60, 0),
                (1, "最低核心频率", "MHz", 500, 2200, 2000),
                (15, "零转速", "", 0, 1, 1),
            ]
            .into_iter()
            .map(
                |(id, label, unit, min, max, value)| crate::adl::overdrive::Control {
                    id,
                    label,
                    unit,
                    min,
                    max,
                    step: 1,
                    current: value,
                    draft: value,
                    default: value,
                    reset: false,
                },
            ),
        );
        panel.info.push("驱动／适配器信息（示例）".into());
        panel.frames = [
            (Field::AntiLag, 0, 1),
            (Field::Generate, 1, 1),
            (Field::Algorithm, 1, 2),
            (Field::Chill, 1, 1),
            (Field::Low, 75, 300),
            (Field::High, 140, 300),
            (Field::Cap, 1, 1),
            (Field::Target, 120, 300),
            (Field::Boost, 2, 3),
            (Field::MinResolution, 84, 100),
            (Field::Sharpen, 1, 1),
            (Field::Sharpness, 65, 100),
            (Field::Desktop, 0, 1),
            (Field::AntiAliasing, 1, 2),
            (Field::DeepColor, 0, 1),
        ]
        .into_iter()
        .map(|(field, draft, max)| Control {
            field,
            current: draft,
            draft,
            min: 0,
            max,
            step: 1,
        })
        .collect();
        panel.metrics = [
            ("核心占用", Some(51), "%"),
            ("显存占用率", Some(32), "%"),
            ("已用显存", Some(5241), "MB"),
            ("核心频率", Some(2312), "MHz"),
            ("显存频率", Some(2518), "MHz"),
            ("功耗", Some(198), "W"),
            ("核心电压", Some(966), "mV"),
            ("风扇转速", Some(1024), "RPM"),
            ("风扇转速比", Some(52), "%"),
            ("核心温度", Some(61), "°C"),
            ("显存温度", Some(64), "°C"),
            ("热点温度", Some(72), "°C"),
        ]
        .into_iter()
        .map(|(label, value, unit)| Metric { label, value, unit })
        .collect();
        panel.override_fan_mode_for_preview();
        let temperatures = [30, 45, 58, 73, 88];
        let speeds = [24, 34, 47, 65, 83];
        for (point, (&temp, &speed)) in temperatures.iter().zip(&speeds).enumerate() {
            for (axis, value) in [temp, speed].into_iter().enumerate() {
                let temperature = axis == 0;
                panel.controls.push(crate::adl::overdrive::Control {
                    id: 19 + point * 2 + axis,
                    label: if temperature { "温度" } else { "转速" },
                    unit: if temperature { "°C" } else { "%" },
                    min: if temperature { 20 } else { 10 },
                    max: 100,
                    step: 1,
                    current: if temperature { value } else { value - 4 },
                    draft: value,
                    default: value,
                    reset: false,
                });
            }
        }
    })
    .expect("fan page preview");
}
