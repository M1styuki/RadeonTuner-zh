use super::Panel;
use crate::{
    adl::display::{Display, Mode},
    mode_preview::{Preview, TIMEOUT},
};
use gpui::{Context, Div, div, prelude::*, rgb};

#[derive(Default)]
pub(super) struct DisplayPage {
    settings: Vec<crate::adl::display::settings::Setting>,
    displays: Vec<Display>,
    display_index: usize,
    modes: Vec<Mode>,
    choice: usize,
    current: Option<Mode>,
    angle: i32,
    preview: Option<Preview>,
    status: String,
}

impl Panel {
    pub(super) fn read_displays(&mut self) {
        self.display = DisplayPage::default();
        if let (Some(driver), Some(device)) = (&self.driver, self.devices.get(self.selected)) {
            match driver.displays(device.index) {
                Ok(displays) => {
                    self.display.displays = displays;
                    self.read_modes();
                }
                Err(err) => self.display.status = err.to_string(),
            }
        }
    }
    fn read_modes(&mut self) {
        let page = &mut self.display;
        page.current = None;
        page.modes.clear();
        page.settings.clear();
        if let (Some(driver), Some(display)) = (&self.driver, page.displays.get(page.display_index))
        {
            page.settings = driver.display_settings(display);
            match driver
                .current_mode(display)
                .and_then(|current| Ok((current, driver.possible_modes(display)?)))
            {
                Ok((current, mut modes)) => {
                    page.choice = modes
                        .iter()
                        .position(|m| {
                            m.width == current.width
                                && m.height == current.height
                                && m.refresh == current.refresh
                        })
                        .unwrap_or_else(|| {
                            modes.push(current);
                            modes.len() - 1
                        });
                    page.modes = modes;
                    page.angle = current.orientation;
                    page.current = Some(current);
                }
                Err(err) => page.status = err.to_string(),
            }
        }
    }

    pub(super) fn finish_preview(&mut self, keep: bool) {
        if let Some(mut preview) = self.display.preview.take() {
            self.display.status = match preview.finish(keep) {
                Ok(true) => "已保留显示模式".into(),
                Ok(false) => "已还原显示模式".into(),
                Err(err) => err.to_string(),
            };
            self.read_modes();
        }
    }

    pub(super) fn expire_preview(&mut self) {
        if self
            .display
            .preview
            .as_ref()
            .is_some_and(|p| p.started.elapsed() >= TIMEOUT)
        {
            self.finish_preview(false);
        }
    }

    pub(super) fn render_displays(&self, cx: &mut Context<Self>) -> Div {
        let page = &self.display;
        let mut section = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_xl().child("显示器"));
        if let Some(preview) = &page.preview {
            section = section.child(format!(
                "将在 {} 秒后还原显示模式",
                TIMEOUT
                    .as_secs()
                    .saturating_sub(preview.started.elapsed().as_secs())
            ));
            for (id, text, keep) in [("keep-mode", "保留", true), ("revert-mode", "还原", false)]
            {
                section = section.child(div().id(id).cursor_pointer().child(text).on_click(
                    cx.listener(move |panel, _, _, cx| {
                        panel.finish_preview(keep);
                        cx.notify();
                    }),
                ));
            }
            return section;
        }
        for (index, display) in page.displays.iter().enumerate() {
            section = section.child(
                div()
                    .id(("display", index))
                    .cursor_pointer()
                    .px_3()
                    .bg(rgb(if index == page.display_index {
                        0x713e47
                    } else {
                        0x292d34
                    }))
                    .child(format!("{} · {}", display.name, display.logical + 1))
                    .on_click(cx.listener(move |panel, _, _, cx| {
                        panel.display.display_index = index;
                        panel.display.status.clear();
                        panel.read_modes();
                        cx.notify();
                    })),
            );
        }
        if let (Some(mode), Some(current)) = (page.modes.get(page.choice), page.current) {
            let mut row = div().flex().gap_3().items_center().child(format!(
                "{} × {} · {:.2} Hz",
                mode.width, mode.height, mode.refresh
            ));
            for (id, label, dir) in [("mode-prev", "上一项", -1isize), ("mode-next", "下一项", 1)]
            {
                let next = page
                    .choice
                    .checked_add_signed(dir)
                    .filter(|i| *i < page.modes.len());
                let button = div().id(id).px_2().child(label);
                row = row.child(if let Some(index) = next {
                    button
                        .cursor_pointer()
                        .on_click(cx.listener(move |panel, _, _, cx| {
                            panel.display.choice = index;
                            cx.notify();
                        }))
                } else {
                    button.opacity(0.45)
                });
            }
            section = section.child(row);
            let mut row = div().flex().gap_3().child("显示方向");
            for (i, angle) in [0, 90, 180, 270].into_iter().enumerate() {
                row = row.child(
                    div()
                        .id(("orientation", i))
                        .cursor_pointer()
                        .px_2()
                        .bg(rgb(if page.angle == angle {
                            0x713e47
                        } else {
                            0x292d34
                        }))
                        .child(format!("{angle}°"))
                        .on_click(cx.listener(move |panel, _, _, cx| {
                            panel.display.angle = angle;
                            cx.notify();
                        })),
                );
            }
            section = section.child(row);
            if current.width != mode.width
                || current.height != mode.height
                || current.refresh != mode.refresh
                || current.orientation != page.angle
            {
                section = section.child(
                    div()
                        .id("preview-mode")
                        .cursor_pointer()
                        .child("应用显示模式")
                        .on_click(cx.listener(|panel, _, _, cx| {
                            let page = &mut panel.display;
                            match Preview::start(
                                &page.displays[page.display_index],
                                page.modes[page.choice],
                                page.angle,
                            ) {
                                Ok(preview) => page.preview = Some(preview),
                                Err(err) => page.status = err.to_string(),
                            }
                            cx.notify();
                        })),
                );
            }
        }
        for (i, setting) in page.settings.iter().enumerate() {
            let mut row = div()
                .flex()
                .gap_3()
                .items_center()
                .child(div().flex_1().child(setting.label))
                .child(setting.text());
            for (id, label, step) in [
                ("display-down", "−", -setting.step),
                ("display-up", "+", setting.step),
            ] {
                let next = setting
                    .draft
                    .saturating_add(step)
                    .clamp(setting.min, setting.max);
                let button = div().id((id, i)).px_2().child(label);
                row = row.child(if next != setting.draft {
                    button
                        .cursor_pointer()
                        .on_click(cx.listener(move |p, _, _, cx| {
                            p.display.settings[i].draft = next;
                            cx.notify();
                        }))
                } else {
                    button.opacity(0.45)
                });
            }
            section = section.child(row);
        }
        if !page.settings.is_empty() {
            section = section.child(
                div()
                    .id("display-defaults")
                    .cursor_pointer()
                    .child("载入显示默认值")
                    .on_click(cx.listener(|p, _, _, cx| {
                        for s in &mut p.display.settings {
                            s.draft = s.default.clamp(s.min, s.max);
                        }
                        cx.notify();
                    })),
            );
        }
        if page.settings.iter().any(|s| s.current != s.draft) {
            section = section.child(
                div()
                    .id("display-apply")
                    .cursor_pointer()
                    .child("应用显示设置")
                    .on_click(cx.listener(|p, _, _, cx| {
                        if let (Some(driver), Some(display)) =
                            (&p.driver, p.display.displays.get(p.display.display_index))
                        {
                            let result = driver.apply_display(display, &p.display.settings);
                            p.read_modes();
                            p.display.status = match result {
                                Ok(()) => "显示设置已应用".into(),
                                Err(err) => err.to_string(),
                            };
                        }
                        cx.notify();
                    })),
            );
        }
        if page.displays.is_empty() && page.status.is_empty() {
            section = section.child("未检测到可用的 AMD 显示器");
        }
        section.child(page.status.clone())
    }
}
