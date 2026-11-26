use crate::{
    adl::{Device, Driver, Metric},
    integration::services,
    settings::Settings,
};
use anyhow::Result;
use gpui_kit::component::{
    ActiveTheme as _, Root, Sizable as _, TitleBar, alert::Alert, h_flex, scroll::Scrollbar, v_flex,
};
use gpui_kit::{
    App, AppContext as _, Bounds, Context, Div, FontWeight, InteractiveElement as _, IntoElement,
    KeyDownEvent, ParentElement as _, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled as _, TitlebarOptions, Window,
    WindowBackgroundAppearance, WindowBounds, WindowDecorations, WindowOptions, canvas, div, point,
    px, rems, size,
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{cell::RefCell, collections::HashMap, time::Duration};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    IsIconic, SW_SHOW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    SetForegroundWindow, SetWindowPos, ShowWindow,
};

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

const PAGES: [&str; 7] = ["调校", "风扇", "显卡", "显示", "多媒体", "设置", "信息"];
mod tuning;

struct Panel {
    picker: Option<picker::Picker>,
    dropdown: Option<controls::Dropdown>,
    control_glyph_widths: RefCell<HashMap<char, f32>>,
    content_scroll: ScrollHandle,
    content_smooth_scroll: motion::SmoothScroll,
    applied_theme: Option<(i64, bool, bool)>,
    animations: bool,
    updates: updates::Updates,
    start_check_update: bool,
    picker_generation: u64,
    pending_import: Option<transfer::Pending>,
    style: i64,
    mica: bool,
    frame_dark: Option<bool>,
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

    fn render_info_page(&self, cx: &mut Context<Self>) -> Div {
        let mut links = h_flex().flex_wrap().gap_2().px_4().py_3();
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
            links = links.child(self.contour_button(
                ("info-link", i),
                label,
                true,
                None,
                move |p, _, _| {
                    if let Err(err) = crate::integration::open(url) {
                        p.message = err.to_string();
                    }
                },
                cx,
            ));
        }
        let version = v_flex()
            .gap_2()
            .px_4()
            .py_3()
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                    .child(format!(
                        "RadeonTuner · 原作者 Arnold Vink · Rust 版 {}",
                        env!("CARGO_PKG_VERSION")
                    )),
            )
            .child(self.update_controls(cx));

        let mut sections: Vec<(&'static str, Div)> =
            vec![("版本与更新", controls::card_rows([version], cx))];
        if !self.info.is_empty() {
            let device = controls::card_rows(
                self.info.iter().cloned().map(|line| {
                    div()
                        .px_4()
                        .py_2()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(line)
                }),
                cx,
            );
            sections.push(("设备与驱动信息", device));
        }
        sections.push(("相关链接", controls::card_rows([links], cx)));
        self.setting_sections_titled(sections, cx)
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
        let system_light = theme::is_system_light(window.appearance());
        let theme_key = (self.style, system_light, self.mica);
        if self.applied_theme != Some(theme_key) {
            self.applied_theme = Some(theme_key);
            theme::sync_theme(Some(window), self.style, self.mica, cx);
        }
        let dark = !theme::is_light(self.style, system_light);
        if self.frame_dark != Some(dark) {
            theme::set_caption_dark(window, dark);
            self.frame_dark = Some(dark);
        }

        let mut content = v_flex().w_full().gap_3().px_6().pt_1().pb_6();
        if self.page != 6 && self.updates.has_notice() {
            content = content.child(self.update_controls(cx));
        }
        if !self.message.is_empty() && self.page != 5 {
            content = content.child(Alert::warning("panel-msg", self.message.clone()).small());
        }
        if matches!(self.page, 0 | 1) && !self.controls.is_empty() {
            content = content.child(self.render_tuning_fields(window, cx));
        }
        match self.page {
            2 => content = content.child(self.render_graphics(window, cx)),
            3 => content = content.child(self.render_displays(window, cx)),
            4 => content = content.child(self.render_video(cx)),
            5 => content = content.child(self.render_settings(cx)),
            6 => content = content.child(self.render_info_page(cx)),
            _ => {}
        }

        let viewport = window.viewport_size();
        let nav_w = controls::sidebar_width(viewport.width);
        let compact = nav_w < px(80.);
        let pill_color = cx.theme().sidebar_accent;
        let pill_radius = f32::from(cx.theme().radius);
        let active_index = self.page.min(PAGES.len() - 1) as f32;

        let mut nav_list = v_flex().relative().gap_1().child(
            canvas(
                move |_, window, cx| {
                    let step = rems(2.5).to_pixels(window.rem_size());
                    motion::smooth_spring("rt-nav-pill", step * active_index, window, cx)
                },
                move |bounds, y, window, _| {
                    let pill_h = rems(2.25).to_pixels(window.rem_size());
                    let pill = Bounds::new(
                        point(bounds.origin.x, bounds.origin.y + y),
                        size(bounds.size.width, pill_h),
                    );
                    shape::paint_fill(pill, pill_color, pill_radius, window);
                },
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full(),
        );
        for (i, label) in PAGES.iter().enumerate() {
            nav_list = nav_list.child(self.nav_item(
                ("nav", i),
                label,
                i,
                compact,
                i == self.page,
                move |p, _, _| {
                    if p.page == 3 && i == 3 && p.display_subpage().is_some() {
                        p.exit_display_subpage();
                    } else if p.page == 3 && i != 3 {
                        p.finish_preview(false);
                        p.close_display_subpages();
                    }
                    p.dropdown = None;
                    let was_settings = p.page == 5;
                    p.page = i;
                    p.content_scroll.set_offset(point(px(0.), px(0.)));
                    p.content_smooth_scroll.reset();
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
                },
                cx,
            ));
        }

        let nav = v_flex()
            .id("navigation")
            .w(nav_w)
            .h_full()
            .flex_shrink_0()
            .bg(cx.theme().sidebar)
            .px_2()
            .pt_2()
            .pb_3()
            .child(
                v_flex()
                    .id("nav-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(nav_list),
            );

        let content_area = v_flex()
            .relative()
            .flex_1()
            .h_full()
            .min_h_0()
            .child(
                v_flex()
                    .id(("content", self.page))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.content_scroll)
                    .child(
                        self.content_smooth_scroll
                            .wrap(self.content_scroll.clone(), content.flex_shrink_0()),
                    ),
            )
            .child(Scrollbar::vertical(&self.content_scroll));

        let is_display_subpage = self.page == 3 && self.display_subpage().is_some();
        let entrance_key = if is_display_subpage {
            format!(
                "page-entrance-3-{}",
                self.display_subpage().unwrap_or_default()
            )
        } else {
            format!("page-entrance-{}", self.page)
        };
        let entered_content = if is_display_subpage {
            motion::slide_entrance(SharedString::from(entrance_key), -24., content_area)
                .into_any_element()
        } else {
            motion::entrance(SharedString::from(entrance_key), content_area).into_any_element()
        };

        let mut window_content = v_flex()
            .relative()
            .size_full()
            .text_sm()
            .text_color(cx.theme().foreground)
            .font_family("Microsoft YaHei UI")
            .on_key_down(cx.listener(|p, e: &KeyDownEvent, _, cx| {
                if e.keystroke.key == "escape"
                    && p.picker.is_none()
                    && p.dropdown.is_none()
                    && p.edit.is_none()
                    && p.page == 3
                    && p.display_subpage().is_some()
                    && !p.has_display_preview()
                {
                    p.exit_display_subpage();
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .child(
                TitleBar::new()
                    .border_b_0()
                    .bg(cx.theme().transparent)
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(cx.theme().muted_foreground)
                            .child("RadeonTuner"),
                    ),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .items_stretch()
                    .child(nav)
                    .child(
                        v_flex()
                            .flex_1()
                            .h_full()
                            .min_w_0()
                            .child(self.render_header(viewport.width, cx))
                            .child(v_flex().flex_1().min_h_0().w_full().child(entered_content)),
                    ),
            );
        if self.picker.is_some() {
            window_content = window_content.child(self.modal(self.render_picker(cx), cx));
        } else if self.page == 3 {
            if let Some(dialog) = self.display_dialog(cx) {
                window_content = window_content.child(self.modal(dialog, cx));
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

fn build_panel(settings: Settings, initialize: impl FnOnce(&mut Panel)) -> Result<Panel> {
    let start_visible = settings.flag("StartWindowVisible", false)?;
    let close_tray = settings.flag("CloseTray", true)?;
    let style = settings.integer("ThemeStyle", 0)?;
    let mut panel = Panel {
        picker: None,
        dropdown: None,
        control_glyph_widths: Default::default(),
        content_scroll: ScrollHandle::new(),
        content_smooth_scroll: motion::SmoothScroll::new(),
        applied_theme: None,
        animations: motion::enabled(),
        updates: Default::default(),
        start_check_update: settings.flag("StartCheckUpdate", true)?,
        picker_generation: 0,
        pending_import: None,
        style,
        mica: false,
        frame_dark: None,
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
    Ok(panel)
}

fn open_panel(poll_hardware: bool, initialize: impl FnOnce(&mut Panel)) -> Result<()> {
    let settings = Settings::open()?;
    let mut panel = build_panel(settings, initialize)?;
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(move |cx: &mut App| {
            gpui_kit::init(cx);
            gpui_kit::base::set_reduce_motion_preference(
                if panel.animations {
                    gpui_kit::base::ReduceMotionPreference::Allow
                } else {
                    gpui_kit::base::ReduceMotionPreference::Reduce
                },
                cx,
            );
            theme::sync_theme(None, panel.style, false, cx);

            let bounds = Bounds::centered(None, size(px(860.), px(720.)), cx);
            let mut hwnd_cell = None;
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some(TITLE.into()),
                        appears_transparent: true,
                        ..Default::default()
                    }),
                    window_background: WindowBackgroundAppearance::MicaBackdrop,
                    window_decorations: Some(WindowDecorations::Client),
                    app_owns_titlebar_drag: true,
                    show: false,
                    focus: false,
                    ..Default::default()
                },
                |window, cx| {
                    set_window_icon(window);
                    let system_light = theme::is_system_light(window.appearance());
                    let dark = !theme::is_light(panel.style, system_light);
                    theme::set_caption_dark(window, dark);
                    panel.frame_dark = Some(dark);
                    panel.mica = theme::enable_mica(window);
                    theme::sync_theme(Some(window), panel.style, panel.mica, cx);
                    panel.applied_theme = Some((
                        panel.style,
                        theme::is_system_light(window.appearance()),
                        panel.mica,
                    ));
                    let hwnd = HasWindowHandle::window_handle(window)
                        .ok()
                        .and_then(|handle| match handle.as_raw() {
                            RawWindowHandle::Win32(handle) => Some(handle.hwnd.get()),
                            _ => None,
                        });
                    hwnd_cell = hwnd;
                    if panel.topmost {
                        if let Err(err) = crate::tray::topmost(true) {
                            panel.message = err.to_string();
                        }
                    }
                    let view = cx.new(|cx: &mut Context<Panel>| {
                        if panel.start_check_update
                            && std::env::args_os().any(|a| a == "--first-panel")
                        {
                            panel.check_updates(false, cx);
                        }
                        if poll_hardware {
                            let clock = cx.background_executor().clone();
                            cx.spawn(async move |view, cx| {
                                loop {
                                    clock.timer(Duration::from_secs(2)).await;
                                    if view
                                        .update(cx, |panel, cx| {
                                            let preview_active = panel.poll_preview();
                                            let visible = hwnd.is_none_or(|hwnd| unsafe {
                                                IsIconic(hwnd as _) == 0
                                            });
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
                    });
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .expect("无法创建 GPUI 面板");
            if let Some(hwnd) = hwnd_cell {
                unsafe {
                    SetWindowPos(
                        hwnd as _,
                        std::ptr::null_mut(),
                        0,
                        0,
                        0,
                        0,
                        SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                    ShowWindow(hwnd as _, SW_SHOW);
                    SetForegroundWindow(hwnd as _);
                }
            }
            cx.on_window_closed(|cx, _| {
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
fn populate_preview_fixture(panel: &mut Panel) {
    use crate::adl::graphics::frames::{Control, Field};
    use crate::adl::multimedia::{Feature as VideoFeature, VideoControl};

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
}

#[cfg(test)]
#[test]
#[ignore = "opens a window for an interactive fan-layout check"]
fn preview_fan_page() {
    open_panel(false, populate_preview_fixture).expect("fan page preview");
}

#[cfg(test)]
#[test]
fn panel_renders_all_pages_and_themes_without_panicking() {
    use gpui_kit::{TestAppContext, VisualTestContext};

    let jobs = TestAppContext::single();
    let temp = tempfile::tempdir().unwrap();
    let settings = Settings::at_path(temp.path().join("Settings.json"));
    let mut panel = build_panel(settings, populate_preview_fixture).unwrap();
    panel.mica = true;

    let (handle, entity) = jobs.update(|cx| {
        gpui_kit::init(cx);
        theme::sync_theme(None, panel.style, panel.mica, cx);
        let mut cell = Some(panel);
        let (handle, entity) = gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                    point(px(0.), px(0.)),
                    size(px(860.), px(720.)),
                ))),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| cell.take().unwrap()),
        )
        .unwrap();
        (handle, entity)
    });

    let mut window = VisualTestContext::from_window(handle, &jobs);
    for style in 0..=3 {
        for page in 0..PAGES.len() {
            entity.update(&mut window, |panel, cx| {
                panel.style = style;
                panel.page = page;
                cx.notify();
            });
            window.simulate_resize(size(px(860.), px(720.)));
            window.update(|w, cx| w.draw(cx).clear(cx));
        }
    }
    window.simulate_resize(size(px(560.), px(480.)));
    window.update(|w, cx| w.draw(cx).clear(cx));
}
