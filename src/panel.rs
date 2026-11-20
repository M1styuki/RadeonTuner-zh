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
mod display;
mod multimedia;
struct Panel {
    display: display::DisplayPage,
    video: Vec<crate::adl::multimedia::VideoControl>,
    video_status: String,
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
        self.read_video();
        self.controls.clear();
        self.tuning_status.clear();
        if let (Some(driver), Some(device)) = (&self.driver, self.devices.get(self.selected)) {
            match driver.controls(device.index) {
                Ok(values) => self.controls = values,
                Err(err) => self.tuning_status = err.to_string(),
            }
        }
    }
    fn refresh(&mut self) {
        self.expire_preview();
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut content = div()
            .id("content")
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_3()
            .p_5()
            .bg(rgb(0x171a20))
            .text_color(rgb(0xe4e6eb))
            .child(div().text_xl().child("显卡监控"));
        for (i, device) in self.devices.iter().enumerate() {
            content = content.child(
                div()
                    .id(("gpu", i))
                    .cursor_pointer()
                    .px_3()
                    .py_1()
                    .bg(rgb(if self.selected == i {
                        0x713e47
                    } else {
                        0x292d34
                    }))
                    .child(device.name.clone())
                    .on_click(cx.listener(move |panel, _, _, cx| {
                        panel.finish_preview(false);
                        panel.selected = i;
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
        for item in &self.metrics {
            let value = item
                .value
                .map(|n| format!("{n} {}", item.unit))
                .unwrap_or_else(|| "—".into());
            content = content.child(
                div()
                    .flex()
                    .justify_between()
                    .child(item.label)
                    .child(value),
            );
        }
        if !self.controls.is_empty() {
            content = content.child(div().text_xl().child("调校与风扇曲线"));
            for (i, control) in self.controls.iter().enumerate() {
                let mut row = div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(control.label)
                    .child(format!("{} {}", control.draft, control.unit));
                for (key, label, step) in [("down", "−", -control.step), ("up", "+", control.step)]
                {
                    row = row.child(
                        div()
                            .id((key, i))
                            .cursor_pointer()
                            .px_3()
                            .bg(rgb(0x292d34))
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
            if self.controls.iter().any(|c| c.draft != c.current) {
                content = content.child(
                    div()
                        .id("apply")
                        .cursor_pointer()
                        .p_2()
                        .child("应用更改")
                        .on_click(cx.listener(|panel, _, _, cx| {
                            if let (Some(driver), Some(device)) =
                                (&panel.driver, panel.devices.get(panel.selected))
                            {
                                match driver.apply(device.index, &panel.controls) {
                                    Ok(()) => panel.read_controls(),
                                    Err(err) => panel.tuning_status = err.to_string(),
                                }
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
        if !self.tuning_status.is_empty() {
            content = content.child(self.tuning_status.clone());
        }
        content = content.child(self.render_displays(cx));
        content = content.child(self.render_video(cx));
        content = content.child(div().text_xl().child("窗口设置"));
        for (key, label, on) in [
            ("StartWindowVisible", "启动时打开面板", self.start_visible),
            ("CloseTray", "关闭面板后驻留托盘", self.close_tray),
        ] {
            content = content.child(
                div()
                    .id(key)
                    .cursor_pointer()
                    .p_2()
                    .bg(rgb(0x292d34))
                    .flex()
                    .justify_between()
                    .child(label)
                    .child(if on { "开" } else { "关" })
                    .on_click(cx.listener(move |panel, _, _, cx| {
                        match panel.settings.set_flag(key, !on) {
                            Ok(()) if key == "CloseTray" => panel.close_tray = !on,
                            Ok(()) => panel.start_visible = !on,
                            Err(err) => panel.message = format!("保存失败：{err}"),
                        }
                        cx.notify();
                    })),
            );
        }
        content
    }
}

pub fn run() -> Result<()> {
    let settings = Settings::open()?;
    let start_visible = settings.flag("StartWindowVisible", false)?;
    let close_tray = settings.flag("CloseTray", true)?;
    let mut panel = Panel {
        display: Default::default(),
        video: Vec::new(),
        video_status: String::new(),
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
