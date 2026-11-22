mod custom;
mod eyefinity;
use super::Panel;
use crate::adl::display::gamma::Gamma;
use crate::{
    adl::display::{Display, Mode},
    mode_preview::{Preview, TIMEOUT},
};
use crate::{profile_store, profiles};
use anyhow::{Context as _, Result};
use gpui::{Context, Div, div, prelude::*, rgb};
use serde_json::{Value, json};

#[derive(Default)]
pub(super) struct DisplayPage {
    custom: Option<custom::Editor>,
    eyefinity: Option<eyefinity::Editor>,
    auto_eye: bool,
    saved_eye: bool,
    gamma: Gamma,
    saved_gamma: Gamma,
    gamma_supported: bool,
    settings: Vec<crate::adl::display::settings::Setting>,
    displays: Vec<Display>,
    display_index: usize,
    key: Option<String>,
    app: String,
    apps: Vec<String>,
    saved: Vec<i32>,
    imported: Option<Value>,
    modes: Vec<Mode>,
    choice: usize,
    current: Option<Mode>,
    angle: i32,
    preview: Option<Preview>,
    status: String,
}

impl DisplayPage {
    pub(super) fn set_status(&mut self, status: String) {
        self.status = status;
    }
}

impl Panel {
    pub(super) fn set_gamma(&mut self, i: usize, value: i32) {
        self.display.gamma.values[i] = value;
    }
    pub(super) fn set_display_draft(&mut self, i: usize, value: i32) {
        if let Some(s) = self.display.settings.get_mut(i) {
            s.draft = value;
        }
    }
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
        page.custom = None;
        page.eyefinity = None;
        page.auto_eye = false;
        page.gamma = Gamma::default();
        page.gamma_supported = false;
        page.imported = None;
        page.modes.clear();
        page.settings.clear();
        if let (Some(driver), Some(display)) = (&self.driver, page.displays.get(page.display_index))
        {
            page.key = driver.display_profile_key(display);
            page.gamma_supported = page.key.is_some() && driver.supports_gamma(display.adapter);
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
        if let Err(err) = self.load_display_profile() {
            self.display.status = err.to_string();
        }
    }

    fn load_display_profile(&mut self) -> Result<()> {
        let page = &mut self.display;
        if page.app.is_empty() {
            page.app = "Global".into();
        }
        page.apps = vec!["Global".into()];
        if let Some(key) = &page.key {
            let path = self.settings.profiles_path("DisplayProfiles.json")?;
            let rows = profile_store::load(&path)?;
            let candidates = profiles::display_profiles(&rows, key);
            for app in candidates.iter().filter_map(|p| p["Application"].as_str()) {
                if !page.apps.iter().any(|name| name.eq_ignore_ascii_case(app)) {
                    page.apps.push(app.to_string());
                }
            }
            if !page
                .apps
                .iter()
                .any(|app| app.eq_ignore_ascii_case(&page.app))
            {
                page.apps.push(page.app.clone());
            }
            if let Some(row) = candidates.into_iter().find(|p| {
                p["Application"]
                    .as_str()
                    .is_some_and(|app| app.eq_ignore_ascii_case(&page.app))
            }) {
                profiles::display_draft(&mut page.settings, row, page.app != "Global")?;
                page.auto_eye = profiles::automatic_eye(row);
                page.gamma = Gamma::read(row)?;
            }
        }
        page.saved_eye = page.auto_eye;
        page.saved_gamma = page.gamma;
        page.saved = page.settings.iter().map(|s| s.draft).collect();
        Ok(())
    }
    pub(super) fn display_target(&self) -> Option<(String, String)> {
        Some((self.display.key.clone()?, self.display.app.clone()))
    }
    pub(super) fn stage_display(&mut self, row: &Value) -> Result<()> {
        let page = &mut self.display;
        let mut settings = page.settings.clone();
        profiles::display_draft(&mut settings, row, page.app != "Global")?;
        if row
            .as_object()
            .is_some_and(|o| o.keys().any(|k| k.starts_with("Gamma")))
        {
            let mut values = json!({});
            page.gamma.record(&mut values);
            for (k, v) in row.as_object().context("配置格式无效")? {
                values[k] = v.clone();
            }
            page.gamma = Gamma::read(&values)?;
        }
        page.settings = settings;
        if row.get("EyefinityAutomatic.Current").is_some() {
            page.auto_eye = profiles::automatic_eye(row);
        }
        page.imported = Some(row.clone());
        Ok(())
    }
    pub(super) fn display_record(&self) -> Result<Value> {
        let page = &self.display;
        let key = page.key.as_ref().context("未选择显示器")?;
        let rows = profile_store::load(&self.settings.profiles_path("DisplayProfiles.json")?)?;
        let profiles = profiles::display_profiles(&rows, key);
        let mut record = json!({"DeviceId":key,"Application":page.app});
        // Carry unknown fields forward when converting a profile without a serial number.
        if let Some(old) = profiles.iter().find(|p| {
            p["Application"]
                .as_str()
                .is_some_and(|n| n.eq_ignore_ascii_case(&page.app))
        }) {
            record = (*old).clone();
            record["DeviceId"] = json!(key);
        }
        if let Some(imported) = &page.imported {
            record
                .as_object_mut()
                .context("配置格式无效")?
                .extend(imported.as_object().context("配置格式无效")?.clone());
        }
        record["DeviceId"] = json!(key);
        record["Application"] = json!(page.app);
        for s in page
            .settings
            .iter()
            .filter(|s| page.app == "Global" || s.feature.per_app())
        {
            record[format!("{}.Current", s.feature.profile_name())] = Value::from(s.draft);
        }
        if page.app != "Global" {
            record["EyefinityAutomatic.Current"] = json!(i32::from(page.auto_eye));
        }
        if page.gamma_supported {
            page.gamma.record(&mut record);
        }
        Ok(record)
    }
    fn save_display_profile(&mut self) -> Result<()> {
        let driver = self.driver.as_ref().context("AMD 驱动不可用")?;
        let page = &self.display;
        let display = page
            .displays
            .get(page.display_index)
            .context("未选择显示器")?;
        let Some(key) = &page.key else {
            return driver.apply_display(display, &page.settings);
        };
        let path = self.settings.profiles_path("DisplayProfiles.json")?;
        let rows = profile_store::load(&path)?;
        let profiles = profiles::display_profiles(&rows, key);
        if page.app != "Global"
            && !profiles
                .iter()
                .any(|p| p["Application"].as_str() == Some("Global"))
        {
            let mut baseline = json!({"DeviceId":key,"Application":"Global"});
            for s in &page.settings {
                baseline[format!("{}.Current", s.feature.profile_name())] = Value::from(s.current);
            }
            profile_store::save(&path, key, "Global", &baseline)?;
        }
        let record = self.display_record()?;
        profile_store::save(&path, key, &page.app, &record)?;
        let rows = profile_store::load(&path)?;
        let candidates = profiles::display_profiles(&rows, key);
        let active = profiles::active_display(&candidates, &profiles::process_names()?);
        if active.is_some_and(|p| {
            p["Application"]
                .as_str()
                .is_some_and(|n| n.eq_ignore_ascii_case(&page.app))
        }) {
            driver.apply_display(display, &page.settings)?;
        }
        crate::tray::settings_changed()
    }
    fn render_display_profiles(&self, cx: &mut Context<Self>) -> Div {
        let page = &self.display;
        let mut row = div().flex().flex_wrap().gap_2();
        let Some(key) = page.key.clone() else {
            return row;
        };
        for (i, app) in page.apps.iter().enumerate() {
            let name = app.clone();
            row = row.child(
                div()
                    .id(("display-app", i))
                    .px_2()
                    .cursor_pointer()
                    .bg(rgb(if page.app.eq_ignore_ascii_case(app) {
                        self.palette.selected
                    } else {
                        self.palette.control
                    }))
                    .child(if app == "Global" {
                        "全局".to_owned()
                    } else {
                        app.clone()
                    })
                    .on_click(cx.listener(move |p, _, _, cx| {
                        p.display.app = name.clone();
                        p.read_modes();
                        cx.notify();
                    })),
            );
        }
        row.child(
            div()
                .id("pick-display-app")
                .cursor_pointer()
                .child("选择应用…")
                .on_click(cx.listener(move |_, _, _, cx| {
                    let key = key.clone();
                    Self::pick_profile_app(cx, move |p, result| {
                        if p.display.key.as_ref() == Some(&key) {
                            match result {
                                Ok(Some(app)) => {
                                    p.display.app = app;
                                    p.read_modes();
                                }
                                Err(err) => p.display.status = err.to_string(),
                                Ok(None) => {}
                            }
                        }
                    });
                })),
        )
    }

    pub(super) fn finish_preview(&mut self, keep: bool) {
        if let Some(mut preview) = self.display.preview.take() {
            self.display.status = match preview.finish(keep) {
                Ok(true) if preview.custom => "已保留自定义分辨率".into(),
                Ok(false) if preview.custom => "已撤销自定义时序".into(),
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
                "将在 {} 秒后{}",
                TIMEOUT
                    .as_secs()
                    .saturating_sub(preview.started.elapsed().as_secs()),
                if preview.custom {
                    "撤销自定义时序"
                } else {
                    "还原显示模式"
                }
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
        if let Some(editor) = &page.eyefinity {
            return self.render_eyefinity(editor, cx);
        }
        if let Some(editor) = &page.custom {
            return self.render_custom(editor, cx);
        }
        for (index, display) in page.displays.iter().enumerate() {
            section = section.child(
                div()
                    .id(("display", index))
                    .cursor_pointer()
                    .px_3()
                    .bg(rgb(if index == page.display_index {
                        self.palette.selected
                    } else {
                        self.palette.control
                    }))
                    .child(format!(
                        "{} ({}/{})",
                        display.name, display.adapter, display.logical
                    ))
                    .on_click(cx.listener(move |panel, _, _, cx| {
                        panel.display.display_index = index;
                        panel.display.app = "Global".into();
                        panel.display.status.clear();
                        panel.read_modes();
                        cx.notify();
                    })),
            );
        }
        section = section.child(self.render_display_profiles(cx));
        if page.app == "Global" && !page.displays.is_empty() {
            section = section.child(
                div()
                    .id("manage-eye")
                    .cursor_pointer()
                    .child("管理 Eyefinity 多屏拼接")
                    .on_click(cx.listener(|p, _, _, cx| {
                        if let Err(e) = p.open_eyefinity() {
                            p.display.status = e.to_string();
                        }
                        cx.notify();
                    })),
            );
        } else if page.key.is_some() && page.app != "Global" {
            section = section.child(
                div()
                    .id("auto-eye")
                    .cursor_pointer()
                    .child(if page.auto_eye {
                        "自动 Eyefinity　已开启"
                    } else {
                        "自动 Eyefinity　已关闭"
                    })
                    .on_click(cx.listener(|p, _, _, cx| {
                        p.display.auto_eye = !p.display.auto_eye;
                        cx.notify();
                    })),
            );
        }
        if page.key.is_some() {
            section = section.child(self.transfer_buttons(super::transfer::Kind::Display, cx));
        }
        if let (Some(mode), Some(current), true) = (
            page.modes.get(page.choice),
            page.current,
            page.app == "Global",
        ) {
            section = section.child(
                div()
                    .id("custom-editor")
                    .cursor_pointer()
                    .child("管理自定义分辨率…")
                    .on_click(cx.listener(|p, _, _, cx| {
                        if let Err(err) = p.open_custom() {
                            p.display.status = err.to_string();
                        }
                        cx.notify();
                    })),
            );
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
                            self.palette.selected
                        } else {
                            self.palette.control
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
        if page.gamma_supported {
            section = section.child(
                div()
                    .id("gamma-control")
                    .cursor_pointer()
                    .child(if page.gamma.enabled {
                        "伽马调整　已开启"
                    } else {
                        "伽马调整　已关闭"
                    })
                    .on_click(cx.listener(|p, _, _, cx| {
                        p.display.gamma.enabled = !p.display.gamma.enabled;
                        cx.notify();
                    })),
            );
            if page.gamma.enabled {
                for (i, label) in ["基础伽马", "红色", "绿色", "蓝色"].into_iter().enumerate()
                {
                    let val = page.gamma.values[i];
                    let mut row = div()
                        .flex()
                        .gap_3()
                        .items_center()
                        .child(div().flex_1().child(label))
                        .child(self.number(
                            super::numeric::Field::Gamma(i),
                            val,
                            50,
                            300,
                            1,
                            "",
                            cx,
                        ));
                    for (id, text, delta) in [("gamma-down", "−", -1), ("gamma-up", "+", 1)] {
                        row = row.child(
                            div()
                                .id((id, i))
                                .px_2()
                                .cursor_pointer()
                                .child(text)
                                .on_click(cx.listener(move |p, _, _, cx| {
                                    p.display.gamma.values[i] =
                                        (p.display.gamma.values[i] + delta).clamp(50, 300);
                                    cx.notify();
                                })),
                        );
                    }
                    section = section.child(row);
                }
            }
        }
        for (i, setting) in page.settings.iter().enumerate() {
            if page.app != "Global" && !setting.feature.per_app() {
                continue;
            }
            let mut row = div()
                .flex()
                .gap_3()
                .items_center()
                .child(div().flex_1().child(setting.label));
            if matches!(
                setting.feature,
                crate::adl::display::settings::Feature::Color(_)
                    | crate::adl::display::settings::Feature::Correction(1..=3)
            ) {
                row = row.child(self.number(
                    super::numeric::Field::Display(i),
                    setting.draft,
                    setting.min,
                    setting.max,
                    setting.step,
                    "",
                    cx,
                ));
            } else {
                row = row.child(setting.text());
            }
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
                        p.display.auto_eye = false;
                        p.display.gamma = Gamma::default();
                        for s in p
                            .display
                            .settings
                            .iter_mut()
                            .filter(|s| p.display.app == "Global" || s.feature.per_app())
                        {
                            s.draft = s.default.clamp(s.min, s.max);
                        }
                        cx.notify();
                    })),
            );
        }
        if page.gamma != page.saved_gamma
            || page.auto_eye != page.saved_eye
            || page.imported.is_some()
            || page
                .settings
                .iter()
                .zip(&page.saved)
                .any(|(s, saved)| s.draft != *saved)
            || (page.app == "Global" && page.settings.iter().any(|s| s.draft != s.current))
        {
            section = section.child(
                div()
                    .id("display-apply")
                    .cursor_pointer()
                    .child("应用显示设置")
                    .on_click(cx.listener(|p, _, _, cx| {
                        let result = p.save_display_profile();
                        p.read_modes();
                        p.display.status = match result {
                            Ok(()) => "显示设置已保存".into(),
                            Err(err) => err.to_string(),
                        };
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
