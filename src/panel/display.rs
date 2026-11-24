mod custom;
mod eyefinity;
use super::Panel;
use crate::adl::display::gamma::Gamma;
use crate::adl::display::settings::Feature;
use crate::{
    adl::display::{Display, Mode},
    mode_preview::{Preview, TIMEOUT},
};
use crate::{profile_store, profiles};
use anyhow::{Context as _, Result};
use gpui::{Context, Div, div, prelude::*};
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
    confirm_reset: bool,
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
        let old = self
            .display
            .displays
            .get(self.display.display_index)
            .map(|d| {
                (
                    d.adapter,
                    d.logical,
                    d.name.clone(),
                    self.display.app.clone(),
                )
            });
        self.display = DisplayPage::default();
        if let Some(driver) = &self.driver {
            for device in &self.devices {
                match driver.displays(device.index) {
                    Ok(mut displays) => self.display.displays.append(&mut displays),
                    Err(err) => self.display.status = err.to_string(),
                }
            }
            if let Some((adapter, logical, name, app)) = old {
                if let Some(i) =
                    self.display.displays.iter().position(|d| {
                        d.adapter == adapter && d.logical == logical && d.name == name
                    })
                {
                    self.display.display_index = i;
                    self.display.app = app;
                }
            }
            self.read_modes();
        }
    }
    fn read_modes(&mut self) {
        let page = &mut self.display;
        page.current = None;
        page.key = None;
        page.custom = None;
        page.confirm_reset = false;
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
            crate::profiles::sort_app_names(&mut page.apps);
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
        let _write = crate::driver_write::lock()?;
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
        let mut rows = profile_store::load(&path)?;
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
            profile_store::upsert(&mut rows, key, "Global", &baseline, false)?;
        }
        let record = self.display_record()?;
        profile_store::upsert(&mut rows, key, &page.app, &record, false)?;
        let staged = profile_store::staged_json(&path, &json!(&rows))?;
        let candidates = profiles::display_profiles(&rows, key);
        let running = profiles::process_names()?;
        let active = profiles::active_display(&candidates, &running);
        if page.app.eq_ignore_ascii_case("Global")
            || active.is_some_and(|p| {
                p["Application"]
                    .as_str()
                    .is_some_and(|n| n.eq_ignore_ascii_case(&page.app))
            })
        {
            let mut settings = driver.display_settings(display);
            profiles::display_live_draft(&mut settings, &candidates, &running)?;
            driver.apply_display(display, &settings)?;
        }
        let device = self
            .devices
            .iter()
            .find(|d| d.index == display.adapter)
            .context("显示器所属的显卡已断开")?;
        profiles::sync_display_policy(
            driver,
            device,
            &rows,
            &running,
            &self.settings.gamma_recovery_path()?,
        )?;
        staged
            .persist(&path)
            .context("驱动已更新，显示配置保存失败")?;
        crate::tray::settings_changed()
    }
    fn reset_display_profile(&mut self) -> Result<()> {
        let _write = crate::driver_write::lock()?;
        let page = &self.display;
        let d = page
            .displays
            .get(page.display_index)
            .context("未选择显示器")?;
        let driver = self.driver.as_ref().context("AMD 驱动不可用")?;
        let mut settings = driver.display_settings(d);
        for setting in settings
            .iter_mut()
            .filter(|s| page.app == "Global" || s.feature.per_app())
        {
            setting.draft = setting.default.clamp(setting.min, setting.max);
        }
        if let Some(key) = &page.key {
            let path = self.settings.profiles_path("DisplayProfiles.json")?;
            let mut rows = profile_store::load(&path)?;
            let candidates = profiles::display_profiles(&rows, key);
            let running = profiles::process_names()?;
            let active = page.app.eq_ignore_ascii_case("Global")
                || profiles::active_display(&candidates, &running).is_some_and(|r| {
                    r["Application"]
                        .as_str()
                        .is_some_and(|n| n.eq_ignore_ascii_case(&page.app))
                });
            let legacy = key.rsplit_once(' ').map(|(s, _)| s);
            rows.retain(|r| !profile_store::matches(r, key, &page.app));
            // A serial-specific reset must supersede an inherited legacy row.
            if rows
                .iter()
                .any(|r| legacy.is_some_and(|s| profile_store::matches(r, s, &page.app)))
            {
                let mut defaults =
                    json!({"DeviceId":key,"Application":page.app,"EyefinityAutomatic.Current":0});
                Gamma::default().record(&mut defaults);
                for s in settings
                    .iter()
                    .filter(|s| page.app == "Global" || s.feature.per_app())
                {
                    defaults[format!("{}.Current", s.feature.profile_name())] =
                        json!(s.default.clamp(s.min, s.max));
                }
                rows.push(defaults);
            }
            let staged = profile_store::staged_json(&path, &json!(&rows))?;
            if active {
                let new_candidates = profiles::display_profiles(&rows, key);
                profiles::display_live_draft(&mut settings, &new_candidates, &running)?;
                driver.apply_display(d, &settings)?;
            }
            let device = self
                .devices
                .iter()
                .find(|device| device.index == d.adapter)
                .context("显示器所属的显卡已断开")?;
            profiles::sync_display_policy(
                driver,
                device,
                &rows,
                &running,
                &self.settings.gamma_recovery_path()?,
            )?;
            staged
                .persist(&path)
                .context("驱动已更新，显示配置保存失败")?;
            crate::tray::settings_changed()?;
        } else {
            driver.apply_display(d, &settings)?;
        }
        self.read_modes();
        Ok(())
    }
    pub(super) fn display_app_names(&self) -> Vec<String> {
        self.display.apps.clone()
    }
    pub(super) fn add_display_app(&mut self, name: &str) -> Result<()> {
        let _write = crate::driver_write::lock()?;
        let (key, _) = self.display_target().context("未选择显示器")?;
        let path = self.settings.profiles_path("DisplayProfiles.json")?;
        let rows = profile_store::load(&path)?;
        let profiles = profiles::display_profiles(&rows, &key);
        if !profiles
            .iter()
            .any(|r| r["Application"].as_str() == Some("Global"))
        {
            self.display.app = "Global".into();
            self.read_modes();
            profile_store::save(&path, &key, "Global", &self.display_record()?)?;
        }
        self.display.app = name.into();
        self.read_modes();
        if !profiles.iter().any(|r| {
            r["Application"]
                .as_str()
                .is_some_and(|n| n.eq_ignore_ascii_case(name))
        }) {
            profile_store::save(&path, &key, name, &self.display_record()?)?;
        }
        crate::tray::settings_changed()
    }
    pub(super) fn remove_display_apps(&mut self, names: &[String]) -> Result<()> {
        let _write = crate::driver_write::lock()?;
        let (key, _) = self.display_target().context("未选择显示器")?;
        let path = self.settings.profiles_path("DisplayProfiles.json")?;
        let mut rows = profile_store::load(&path)?;
        let legacy = key.rsplit_once(' ').map(|(s, _)| s);
        rows.retain(|r| !names.iter().any(|n| profile_store::matches(r, &key, n)));
        if let Some(legacy) = legacy {
            for row in rows
                .iter_mut()
                .filter(|r| names.iter().any(|n| profile_store::matches(r, legacy, n)))
            {
                let mut excluded = row["ExcludedDisplayIds"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                if !excluded.iter().any(|v| v.as_str() == Some(key.as_str())) {
                    excluded.push(json!(key));
                }
                row["ExcludedDisplayIds"] = Value::Array(excluded);
            }
        }
        let driver = self.driver.as_ref().context("AMD 驱动不可用")?;
        let display = self
            .display
            .displays
            .get(self.display.display_index)
            .context("未选择显示器")?;
        let device = self
            .devices
            .iter()
            .find(|d| d.index == display.adapter)
            .context("显卡已断开")?;
        let running = profiles::process_names()?;
        let staged = profile_store::staged_json(&path, &json!(&rows))?;
        let mut settings = driver.display_settings(display);
        profiles::display_live_draft(
            &mut settings,
            &profiles::display_profiles(&rows, &key),
            &running,
        )?;
        driver.apply_display(display, &settings)?;
        profiles::sync_display_policy(
            driver,
            device,
            &rows,
            &running,
            &self.settings.gamma_recovery_path()?,
        )?;
        staged
            .persist(&path)
            .context("驱动已更新，显示配置保存失败")?;
        self.display.app = "Global".into();
        self.read_modes();
        crate::tray::settings_changed()
    }
    pub(super) fn display_header(&self, cx: &mut Context<Self>) -> Div {
        let page = &self.display;
        if page.preview.is_some() || page.custom.is_some() || page.eyefinity.is_some() {
            return div();
        }
        let labels = page.displays.iter().map(|d| d.name.clone()).collect();
        let mut row = div()
            .flex()
            .items_center()
            .flex_wrap()
            .gap_2()
            .child(self.select(
                "display",
                labels,
                Some(page.display_index),
                "选择显示器",
                152.,
                |p, i, _| {
                    p.display.display_index = i;
                    p.display.app = "Global".into();
                    p.display.status.clear();
                    p.read_modes();
                },
                cx,
            ));
        if page.key.is_some() {
            let apps = page.apps.clone();
            let selected = apps.iter().position(|a| a.eq_ignore_ascii_case(&page.app));
            let labels = apps
                .iter()
                .map(|s| {
                    if s == "Global" {
                        "全局".into()
                    } else {
                        s.clone()
                    }
                })
                .collect();
            row = row
                .child(self.select(
                    "display-app",
                    labels,
                    selected,
                    "选择应用程序",
                    crate::panel::controls::SELECT_MAX_WIDTH,
                    move |p, i, _| {
                        p.display.app = apps[i].clone();
                        p.read_modes();
                    },
                    cx,
                ))
                .child(self.app_picker_buttons(super::picker::Target::Display, cx));
        }
        row
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

    pub(super) fn display_dialog(&self, cx: &mut Context<Self>) -> Option<Div> {
        let page = &self.display;
        let mut section = div().flex().flex_col().gap_2();
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
                section = section.child(
                    self.button_style(true)
                        .id(id)
                        .cursor_pointer()
                        .child(text)
                        .on_click(cx.listener(move |panel, _, _, cx| {
                            panel.finish_preview(keep);
                            cx.notify();
                        })),
                );
            }
            return Some(section);
        }
        if let Some(editor) = &page.eyefinity {
            return Some(self.render_eyefinity(editor, cx));
        }
        if let Some(editor) = &page.custom {
            return Some(self.render_custom(editor, cx));
        }
        None
    }
    pub(super) fn render_displays(&self, cx: &mut Context<Self>) -> Div {
        let page = &self.display;
        let mut categories = Vec::new();
        let mut section = div().flex().flex_col();
        if let (Some(mode), Some(current), true) = (
            page.modes.get(page.choice),
            page.current,
            page.app == "Global",
        ) {
            let mut sizes = Vec::new();
            let mut same = Vec::new();
            for (i, m) in page.modes.iter().enumerate() {
                if !sizes
                    .iter()
                    .any(|(_, width, height)| (*width, *height) == (m.width, m.height))
                {
                    sizes.push((i, m.width, m.height));
                }
                if (m.width, m.height) == (mode.width, mode.height) {
                    same.push(i);
                }
            }
            let size_pos = sizes
                .iter()
                .position(|(_, w, h)| (*w, *h) == (mode.width, mode.height))
                .unwrap_or(0);
            let refresh_pos = same.iter().position(|i| *i == page.choice).unwrap_or(0);
            let preferred = sizes
                .iter()
                .map(|(i, w, h)| {
                    page.modes
                        .iter()
                        .enumerate()
                        .filter(|(_, m)| m.width == *w && m.height == *h)
                        .min_by(|(_, a), (_, b)| {
                            (a.refresh - mode.refresh)
                                .abs()
                                .total_cmp(&(b.refresh - mode.refresh).abs())
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(*i)
                })
                .collect::<Vec<_>>();
            section = section.child(self.setting_row(
                "显示分辨率",
                self.select(
                    "resolution",
                    sizes.iter().map(|(_, w, h)| format!("{w} × {h}")).collect(),
                    Some(size_pos),
                    "显示分辨率",
                    crate::panel::controls::SELECT_MAX_WIDTH,
                    move |p, i, _| p.display.choice = preferred[i],
                    cx,
                ),
            ));
            section = section.child(
                self.setting_row(
                    "显示刷新率",
                    self.select(
                        "refresh",
                        same.iter()
                            .map(|&i| format!("{:.2} Hz", page.modes[i].refresh))
                            .collect(),
                        Some(refresh_pos),
                        "显示刷新率",
                        crate::panel::controls::SELECT_MAX_WIDTH,
                        move |p, i, _| p.display.choice = same[i],
                        cx,
                    ),
                ),
            );
            let angles = [0, 90, 180, 270];
            section = section.child(self.setting_row(
                "显示方向",
                self.select(
                    "orientation",
                    angles.map(|a| format!("{a}°")).into(),
                    angles.iter().position(|&a| a == page.angle),
                    "显示方向",
                    crate::panel::controls::SELECT_MAX_WIDTH,
                    move |p, i, _| p.display.angle = angles[i],
                    cx,
                ),
            ));
            if current.width != mode.width
                || current.height != mode.height
                || current.refresh != mode.refresh
                || current.orientation != page.angle
            {
                section = section.child(
                    self.inline_action()
                        .id("preview-mode")
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
            categories.push(("显示模式", section));
        }
        let correction_on = page
            .settings
            .iter()
            .find(|s| s.feature == Feature::Correction(0))
            .is_some_and(|s| s.draft != 0);
        let temperature_on = page
            .settings
            .iter()
            .find(|s| s.feature == Feature::TemperatureSource)
            .is_some_and(|s| s.draft != 0);
        let vari_bright_on = page
            .settings
            .iter()
            .find(|s| s.feature == Feature::VariBright)
            .is_some_and(|s| s.draft != 0);
        const GROUPS: &[(&str, &[Option<Feature>])] = &[
            (
                "显示输出与缩放",
                &[
                    Some(Feature::Hdr),
                    Some(Feature::FreeSync),
                    Some(Feature::Vsr),
                    Some(Feature::GpuScaling),
                    Some(Feature::IntegerScaling),
                    Some(Feature::Scaling),
                    Some(Feature::Depth),
                    Some(Feature::Pixel),
                    Some(Feature::Hdcp),
                ],
            ),
            (
                "亮度与色彩",
                &[
                    Some(Feature::VariBright),
                    Some(Feature::VariBrightLevel),
                    None,
                    Some(Feature::Color(1)),
                    Some(Feature::Color(2)),
                    Some(Feature::Color(4)),
                    Some(Feature::Color(8)),
                    Some(Feature::Enhancement),
                    Some(Feature::TemperatureSource),
                    Some(Feature::Color(16)),
                ],
            ),
            (
                "色觉辅助",
                &[
                    Some(Feature::Correction(0)),
                    Some(Feature::Correction(1)),
                    Some(Feature::Correction(2)),
                    Some(Feature::Correction(3)),
                ],
            ),
        ];
        for &(name, features) in GROUPS {
            let mut rows = div().flex().flex_col();
            let mut has_rows = false;
            for &feature in features {
                let Some(feature) = feature else {
                    if page.gamma_supported {
                        has_rows = true;
                        rows = rows.child(self.toggle_row(
                            "gamma-control",
                            self.hint_label("显示伽马控制"),
                            page.gamma.enabled,
                            true,
                            |p, _| p.display.gamma.enabled = !p.display.gamma.enabled,
                            cx,
                        ));
                        if page.gamma.enabled {
                            for (i, label) in [
                                "显示伽马基准",
                                "显示伽马红色",
                                "显示伽马绿色",
                                "显示伽马蓝色",
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                rows = rows.child(self.setting_row(
                                    self.hint_label(label),
                                    self.number(
                                        super::numeric::Field::Gamma(i),
                                        page.gamma.values[i],
                                        50,
                                        300,
                                        1,
                                        "",
                                        cx,
                                    ),
                                ));
                            }
                        }
                    }
                    continue;
                };
                let Some((i, setting)) = page
                    .settings
                    .iter()
                    .enumerate()
                    .find(|(_, s)| s.feature == feature)
                else {
                    continue;
                };

                if page.app != "Global" && !setting.feature.per_app() {
                    continue;
                }
                if matches!(setting.feature, Feature::Correction(1..=3)) && !correction_on {
                    continue;
                }
                if setting.feature == Feature::Color(16) && !temperature_on {
                    continue;
                }
                if setting.feature == Feature::VariBrightLevel && !vari_bright_on {
                    continue;
                }
                has_rows = true;
                if setting.min == 0 && setting.max == 1 {
                    rows = rows.child(self.toggle_row(
                        ("display-toggle", i),
                        self.hint_label(setting.label),
                        setting.draft != 0,
                        true,
                        move |p, _| {
                            p.display.settings[i].draft =
                                i32::from(p.display.settings[i].draft == 0)
                        },
                        cx,
                    ));
                } else if matches!(
                    setting.feature,
                    Feature::Color(_) | Feature::Correction(1..=3)
                ) {
                    rows = rows.child(self.setting_row(
                        self.hint_label(setting.label),
                        self.number(
                            super::numeric::Field::Display(i),
                            setting.draft,
                            setting.min,
                            setting.max,
                            setting.step,
                            "",
                            cx,
                        ),
                    ));
                } else {
                    let labels = if setting.choices().is_empty() {
                        (setting.min..=setting.max)
                            .step_by(setting.step as usize)
                            .map(|n| n.to_string())
                            .collect()
                    } else {
                        setting.choices().iter().map(|s| s.to_string()).collect()
                    };
                    let min = setting.min;
                    let step = setting.step;
                    rows = rows.child(self.setting_row(
                        self.hint_label(setting.label),
                        self.select(
                            ("display-choice", i),
                            labels,
                            Some(((setting.draft - min) / step) as usize),
                            setting.label,
                            crate::panel::controls::SELECT_MAX_WIDTH,
                            move |p, n, _| p.display.settings[i].draft = min + n as i32 * step,
                            cx,
                        ),
                    ));
                }
            }
            if has_rows {
                categories.push((name, rows));
            }
        }
        let mut extras = div().flex().flex_col();
        let mut has_extras = false;
        if page.app == "Global" && page.current.is_some() {
            has_extras = true;
            extras = extras.child(
                self.setting_row(
                    self.hint_label("自定义分辨率"),
                    div().child(
                        self.button_style(true)
                            .id("custom-editor")
                            .cursor_pointer()
                            .child("管理自定义分辨率…")
                            .on_click(cx.listener(|p, _, _, cx| {
                                if let Err(err) = p.open_custom() {
                                    p.display.status = err.to_string();
                                }
                                cx.notify();
                            })),
                    ),
                ),
            );
        }
        if page.app == "Global" && !page.displays.is_empty() {
            has_extras = true;
            extras = extras.child(
                self.setting_row(
                    self.hint_label("Eyefinity 操作"),
                    div().child(
                        self.button_style(true)
                            .id("manage-eye")
                            .cursor_pointer()
                            .child("管理 Eyefinity")
                            .on_click(cx.listener(|p, _, _, cx| {
                                if let Err(e) = p.open_eyefinity() {
                                    p.display.status = e.to_string();
                                }
                                cx.notify();
                            })),
                    ),
                ),
            );
        } else if page.key.is_some() && page.app != "Global" {
            has_extras = true;
            extras = extras.child(self.toggle_row(
                "auto-eye",
                self.hint_label("自动 Eyefinity"),
                page.auto_eye,
                true,
                |p, _| p.display.auto_eye = !p.display.auto_eye,
                cx,
            ));
        }
        if has_extras {
            categories.push(("自定义分辨率与多屏", extras));
        }
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(self.display_actions(cx))
            .child(self.setting_sections(categories))
            .when(page.displays.is_empty() && page.status.is_empty(), |view| {
                view.child("未检测到可用的 AMD 显示器")
            })
            .child(page.status.clone())
    }

    fn display_actions(&self, cx: &mut Context<Self>) -> Div {
        let page = &self.display;
        let mut actions = div().flex().flex_wrap().items_center().gap_2();
        if page.displays.is_empty() {
            return actions;
        }
        let changed = page.gamma != page.saved_gamma
            || page.auto_eye != page.saved_eye
            || page.imported.is_some()
            || page
                .settings
                .iter()
                .zip(&page.saved)
                .any(|(s, saved)| s.draft != *saved)
            || (page.app == "Global" && page.settings.iter().any(|s| s.draft != s.current));
        let apply = self
            .button_style(changed)
            .id("display-apply")
            .tooltip(self.tooltip("应用显示设置。"))
            .child("应用");
        actions = actions.child(if changed {
            apply.cursor_pointer().on_click(cx.listener(|p, _, _, cx| {
                let result = p.save_display_profile();
                p.read_modes();
                p.display.status = result
                    .map(|()| "显示设置已保存".into())
                    .unwrap_or_else(|e| e.to_string());
                cx.notify();
            }))
        } else {
            apply
        });
        if !page.settings.is_empty() || page.gamma_supported {
            actions = actions.child(
                self.button_style(true)
                    .id("display-defaults")
                    .tooltip(self.tooltip("将显示设置恢复为默认值。"))
                    .cursor_pointer()
                    .child(if page.confirm_reset {
                        "确认重置所选显示配置"
                    } else {
                        "重置"
                    })
                    .on_click(cx.listener(|p, _, _, cx| {
                        if p.display.confirm_reset {
                            p.display.confirm_reset = false;
                            p.display.status = p
                                .reset_display_profile()
                                .map(|()| "显示设置已重置".to_owned())
                                .unwrap_or_else(|e| e.to_string());
                        } else {
                            p.display.confirm_reset = true;
                        }
                        cx.notify();
                    })),
            );
            if page.confirm_reset {
                actions = actions.child(
                    self.button_style(true)
                        .id("cancel-display-reset")
                        .cursor_pointer()
                        .child("取消")
                        .on_click(cx.listener(|p, _, _, cx| {
                            p.display.confirm_reset = false;
                            cx.notify();
                        })),
                );
            }
        }
        if page.key.is_some() {
            actions = actions.child(self.transfer_buttons(super::transfer::Kind::Display, cx));
        }
        actions
    }
}
