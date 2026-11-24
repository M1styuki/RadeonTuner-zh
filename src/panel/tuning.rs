use super::Panel;
use crate::panel::shape::ContinuousCorners;
use crate::{adl::overdrive::ECC, profile_store, profiles};
use anyhow::{Context as _, Result, ensure};
use gpui::{Context, Div, div, prelude::*, px};
use serde_json::{Value, json};

pub(super) struct Editor {
    app: String,
    apps: Vec<String>,
    mode: usize,
    quiet: bool,
    keep: bool,
    dirty: bool,
    confirm_reset: bool,
    source: Value,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            app: "Global".into(),
            apps: vec!["Global".into()],
            mode: 0,
            quiet: false,
            keep: false,
            dirty: false,
            confirm_reset: false,
            source: json!({}),
        }
    }
}

fn key(id: usize) -> Option<String> {
    let field: String = match id {
        0 => "CoreMax".into(),
        1 => "CoreMin".into(),
        8 => "MemoryMax".into(),
        9 => "PowerLimit".into(),
        14 => "MemoryTiming".into(),
        15 => "FanZeroRpm".into(),
        37 => "PowerVoltage".into(),
        47 => "PowerTDC".into(),
        ECC => "MemoryEcc".into(),
        19..=28 => format!(
            "Fan{}{}",
            if id % 2 == 1 { "Temp" } else { "Speed" },
            (id - 19) / 2
        ),
        _ => return None,
    };
    Some(field + ".Current")
}
impl Panel {
    pub(super) fn curve_points(&self) -> Vec<(i32, i32)> {
        let get = |id| self.controls.iter().find(|c| c.id == id);
        let speeds = get(20)
            .zip(get(28))
            .map(|(first, last)| profiles::curve(first.draft, last.draft, self.tuning.quiet));
        (0..5)
            .filter_map(|i| {
                let temp = get(19 + i * 2)?;
                let speed = get(20 + i * 2)?;
                Some(match self.tuning.mode {
                    3 => (temp.draft, speed.draft),
                    2 => (temp.default, speeds?[i]),
                    _ => (temp.default, speed.default),
                })
            })
            .collect()
    }

    pub(super) fn editable_curve(&self) -> bool {
        self.tuning.mode == 3 && self.curve_points().len() == 5
    }

    pub(super) fn move_curve_point(&mut self, point: usize, temperature: i32, speed: i32) {
        if !self.editable_curve() || point >= 5 {
            return;
        }
        let points = self.curve_points();
        for (axis, desired) in [temperature, speed].into_iter().enumerate() {
            let gap = i32::from(axis == 0);
            let axis_value = |(temp, speed)| if axis == 0 { temp } else { speed };
            let previous = point.checked_sub(1).map(|i| axis_value(points[i]) + gap);
            let next = points
                .get(point + 1)
                .copied()
                .map(|value| axis_value(value) - gap);
            if let Some(control) = self
                .controls
                .iter_mut()
                .find(|c| c.id == 19 + point * 2 + axis)
            {
                let min = previous.unwrap_or(control.min).max(control.min);
                let max = next.unwrap_or(control.max).min(control.max);
                if min <= max {
                    let draft = desired.clamp(min, max);
                    self.tuning.dirty |= draft != control.draft;
                    control.draft = draft;
                }
            }
        }
    }
    pub(super) fn load_tuning(&mut self) -> Result<()> {
        let device = self.devices.get(self.selected).context("未选择显卡")?;
        let rows = profile_store::load(&self.settings.profiles_path("TuningProfiles.json")?)?;
        let app = self.tuning.app.clone();
        self.tuning = Editor::default();
        self.tuning.app = app;
        for row in &rows {
            if row["DeviceId"].as_str() == Some(&device.profile_key) {
                if let Some(name) = row["Application"].as_str() {
                    if !self
                        .tuning
                        .apps
                        .iter()
                        .any(|a| a.eq_ignore_ascii_case(name))
                    {
                        self.tuning.apps.push(name.to_owned());
                    }
                }
            }
        }
        if !self
            .tuning
            .apps
            .iter()
            .any(|a| a.eq_ignore_ascii_case(&self.tuning.app))
        {
            self.tuning.apps.push(self.tuning.app.clone());
        }
        crate::profiles::sort_app_names(&mut self.tuning.apps);
        let Some(row) = rows
            .iter()
            .find(|row| profile_store::matches(row, &device.profile_key, &self.tuning.app))
        else {
            return Ok(());
        };
        self.stage_tuning(row)?;
        self.tuning.dirty = false;
        Ok(())
    }

    pub(super) fn tuning_target(&self) -> Option<(String, String)> {
        Some((
            self.devices.get(self.selected)?.profile_key.clone(),
            self.tuning.app.clone(),
        ))
    }
    pub(super) fn stage_tuning(&mut self, row: &Value) -> Result<()> {
        let mode = row["FanControlMode.Current"]
            .as_i64()
            .unwrap_or(self.tuning.mode as i64);
        ensure!((0..=3).contains(&mode), "风扇模式无效");
        let flag = |key: &str, prev: bool| -> Result<bool> {
            Ok(match &row[key] {
                Value::Null => prev,
                Value::Bool(b) => *b,
                v => match v.as_i64() {
                    Some(0) => false,
                    Some(1) => true,
                    _ => anyhow::bail!("开关数值无效"),
                },
            })
        };
        let quiet = flag("FanRangedCurve.Current", self.tuning.quiet)?;
        let keep = flag("KeepActive.Current", self.tuning.keep)?;
        let mut drafts = Vec::new();
        for c in &self.controls {
            let name = match (mode, c.id) {
                (2, 20) => "FanSpeedMin.Current".into(),
                (2, 28) => "FanSpeedMax.Current".into(),
                _ => key(c.id).context("调校项目未知")?,
            };
            if !row[&name].is_null() {
                let value = row[&name]
                    .as_i64()
                    .or_else(|| row[&name].as_bool().map(i64::from))
                    .context("调校数值无效")?;
                drafts.push(Some(value.clamp(i64::from(c.min), i64::from(c.max)) as i32));
            } else {
                drafts.push(None);
            }
        }
        for (control, value) in self.controls.iter_mut().zip(drafts) {
            if let Some(value) = value {
                control.draft = value;
            }
        }
        self.tuning
            .source
            .as_object_mut()
            .context("配置格式无效")?
            .extend(row.as_object().context("配置格式无效")?.clone());
        self.tuning.mode = mode as usize;
        self.tuning.quiet = quiet;
        self.tuning.keep = keep;
        self.tuning.dirty = true;
        Ok(())
    }

    pub(super) fn show_control(&self, id: usize) -> bool {
        match id {
            15 => self.tuning.mode != 0,
            19..=28 => self.tuning.mode == 3 || (self.tuning.mode == 2 && matches!(id, 20 | 28)),
            _ => true,
        }
    }

    pub(super) fn tuning_record(&self) -> Result<Value> {
        let device = self.devices.get(self.selected).context("未选择显卡")?;
        let mut row = self.tuning.source.clone();
        row["DeviceId"] = json!(device.profile_key);
        row["Application"] = json!(self.tuning.app);
        row["KeepActive.Current"] = json!(i32::from(self.tuning.keep));
        row["FanControlMode.Current"] = json!(self.tuning.mode);
        row["FanRangedCurve.Current"] = json!(i32::from(self.tuning.quiet));
        for c in &self.controls {
            if self.show_control(c.id) {
                let name = match (self.tuning.mode, c.id) {
                    (2, 20) => "FanSpeedMin.Current".into(),
                    (2, 28) => "FanSpeedMax.Current".into(),
                    _ => key(c.id).context("调校项目未知")?,
                };
                row[name] = json!(c.draft);
            }
        }
        Ok(row)
    }
    pub(super) fn save_tuning(&mut self) -> Result<()> {
        let _write = crate::driver_write::lock()?;
        let device = &self.devices[self.selected];
        let driver = self.driver.as_ref().context("AMD 驱动不可用")?;
        let path = self.settings.profiles_path("TuningProfiles.json")?;
        let row = self.tuning_record()?;
        let mut controls = driver.controls(device.index)?;
        profiles::tuning_draft(&mut controls, &row)?;
        let mut rows = profile_store::load(&path)?;
        profile_store::upsert(
            &mut rows,
            &device.profile_key,
            &self.tuning.app,
            &row,
            false,
        )?;
        let staged = profile_store::staged_json(&path, &json!(&rows))?;
        let running = profiles::process_names()?;
        if profiles::select_profile(&rows, &device.profile_key, &running)
            .is_some_and(|p| profile_store::matches(p, &device.profile_key, &self.tuning.app))
        {
            if controls.iter().any(|c| c.current != c.draft || c.reset) {
                driver.apply(device.index, &controls)?;
            }
        }
        staged
            .persist(&path)
            .context("驱动已更新，调校配置保存失败")?;
        crate::tray::settings_changed()?;
        self.read_controls();
        Ok(())
    }

    pub(super) fn add_tuning_app(&mut self, name: &str) -> Result<()> {
        let _write = crate::driver_write::lock()?;
        let device = self.devices.get(self.selected).context("未选择显卡")?;
        let path = self.settings.profiles_path("TuningProfiles.json")?;
        let rows = profile_store::load(&path)?;
        let key = device.profile_key.clone();
        if !rows
            .iter()
            .any(|r| profile_store::matches(r, &key, "Global"))
        {
            self.tuning.app = "Global".into();
            self.read_controls();
            profile_store::save(&path, &key, "Global", &self.tuning_record()?)?;
        }
        self.tuning.app = name.into();
        self.read_controls();
        if !rows.iter().any(|r| profile_store::matches(r, &key, name)) {
            profile_store::save(&path, &key, name, &self.tuning_record()?)?;
        }
        crate::tray::settings_changed()
    }
    pub(super) fn tuning_app_names(&self) -> Vec<String> {
        self.tuning.apps.clone()
    }
    pub(super) fn remove_tuning_apps(&mut self, names: &[String]) -> Result<()> {
        let _write = crate::driver_write::lock()?;
        let (key, _) = self.tuning_target().context("未选择显卡")?;
        let path = self.settings.profiles_path("TuningProfiles.json")?;
        let mut rows = profile_store::load(&path)?;
        rows.retain(|r| !names.iter().any(|n| profile_store::matches(r, &key, n)));
        profile_store::write_json(&path, &json!(rows))?;
        self.tuning.app = "Global".into();
        self.read_controls();
        crate::tray::settings_changed()
    }
    fn reset_tuning_profile(&mut self) -> Result<()> {
        let _write = crate::driver_write::lock()?;
        let device = self.devices.get(self.selected).context("未选择显卡")?;
        let path = self.settings.profiles_path("TuningProfiles.json")?;
        let mut rows = profile_store::load(&path)?;
        let running = profiles::process_names()?;
        let active = profiles::select_profile(&rows, &device.profile_key, &running)
            .map(|row| profile_store::matches(row, &device.profile_key, &self.tuning.app))
            .unwrap_or_else(|| self.tuning.app == "Global");
        rows.retain(|row| !profile_store::matches(row, &device.profile_key, &self.tuning.app));
        let staged = profile_store::staged_json(&path, &json!(&rows))?;
        if active {
            self.driver
                .as_ref()
                .context("显卡驱动不可用")?
                .reset_tuning(device.index)?;
        }
        staged
            .persist(&path)
            .context("驱动已重置，调校配置保存失败")?;
        self.read_controls();
        crate::tray::settings_changed()
    }
    pub(super) fn tuning_profiles(&self, cx: &mut Context<Self>) -> Div {
        let apps = self.tuning.apps.clone();
        let selected = apps
            .iter()
            .position(|name| name.eq_ignore_ascii_case(&self.tuning.app));
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
        let mut row = div().flex().gap_2().items_center();
        if !self.controls.is_empty() {
            row = row
                .child(self.select(
                    "tuning-app",
                    labels,
                    selected,
                    "选择应用程序",
                    crate::panel::controls::FIELD_WIDTH,
                    move |p, i, _| {
                        p.tuning.app = apps[i].clone();
                        p.read_controls();
                    },
                    cx,
                ))
                .child(self.app_picker_buttons(super::picker::Target::Tuning, cx));
        }
        row
    }
    pub(super) fn render_tuning_profiles(&self, cx: &mut Context<Self>) -> Div {
        let mut section = div().flex().flex_col().gap_2();
        let changed = self.tuning_changed();
        let apply = self
            .button_style(changed)
            .id("apply-tuning")
            .tooltip(self.tooltip("应用调校与风扇设置。"))
            .child("应用");
        let apply = if changed {
            apply.cursor_pointer().on_click(cx.listener(|p, _, _, cx| {
                p.tuning_status = p
                    .save_tuning()
                    .map(|()| "已应用调校配置".to_owned())
                    .unwrap_or_else(|err| err.to_string());
                cx.notify();
            }))
        } else {
            apply
        };
        let mut actions = div().flex().flex_wrap().items_center().gap_2().child(apply);
        actions = actions.child(
            self.button_style(true)
                .id("reset-tuning-profile")
                .tooltip(self.tooltip("将调校与风扇设置恢复为默认值。"))
                .cursor_pointer()
                .child(if self.tuning.confirm_reset {
                    "确认重置"
                } else {
                    "重置"
                })
                .on_click(cx.listener(|p, _, _, cx| {
                    if p.tuning.confirm_reset {
                        p.tuning.confirm_reset = false;
                        p.tuning_status = p
                            .reset_tuning_profile()
                            .map(|()| "调校与风扇设置已重置".to_owned())
                            .unwrap_or_else(|e| e.to_string());
                    } else {
                        p.tuning.confirm_reset = true;
                    }
                    cx.notify();
                })),
        );
        actions = actions.child(self.transfer_buttons(super::transfer::Kind::Tuning, cx));
        if self.tuning.confirm_reset {
            actions = actions.child(
                self.button_style(true)
                    .id("cancel-tuning-reset")
                    .cursor_pointer()
                    .child("取消")
                    .on_click(cx.listener(|p, _, _, cx| {
                        p.tuning.confirm_reset = false;
                        cx.notify();
                    })),
            );
        }
        section = section.child(
            actions.child(
                self.button_style(true)
                    .id("reload-tuning")
                    .cursor_pointer()
                    .child("读取驱动")
                    .on_click(cx.listener(|p, _, _, cx| {
                        p.read_controls();
                        cx.notify();
                    })),
            ),
        );
        section = section.child(self.toggle_row(
            "keep-tuning",
            self.hint_label("保持生效"),
            self.tuning.keep,
            true,
            |p, _| {
                p.tuning.keep = !p.tuning.keep;
                p.tuning.dirty = true;
            },
            cx,
        ));
        if self.page != 1 {
            return section;
        }
        section = section.child(
            self.setting_row(
                self.hint_label("风扇控制模式"),
                self.select(
                    "fan-mode",
                    ["无风扇控制", "仅零 RPM", "范围风扇转速", "自定义风扇曲线"]
                        .map(str::to_owned)
                        .into(),
                    Some(self.tuning.mode),
                    "风扇控制模式",
                    crate::panel::controls::FIELD_WIDTH,
                    |p, i, _| {
                        p.tuning.mode = i;
                        p.tuning.dirty = true;
                    },
                    cx,
                ),
            ),
        );
        if self.tuning.mode == 2 {
            section = section.child(self.setting_row(
                self.hint_label("范围风扇曲线"),
                self.select(
                    "fan-shape",
                    ["线性", "静音"].map(str::to_owned).into(),
                    Some(usize::from(self.tuning.quiet)),
                    "范围风扇曲线",
                    crate::panel::controls::FIELD_WIDTH,
                    |p, i, _| {
                        p.tuning.quiet = i == 1;
                        p.tuning.dirty = true;
                    },
                    cx,
                ),
            ));
        }
        section
    }
    pub(super) fn render_tuning_fields(&self, cx: &mut Context<Self>) -> Div {
        let mut page = div().flex().flex_col().gap_3();
        if self.page == 1 {
            for (i, control) in self.controls.iter().enumerate() {
                if control.id == 15 && self.show_control(control.id) {
                    page = page.child(self.tuning_field(i, cx));
                }
            }
            page = page.child(self.render_curve(cx));
            if self.tuning.mode == 3 {
                page = page.child(div().text_lg().child("风扇曲线"));
                let mut curves = div().flex().flex_wrap().gap_3();
                for point in 0..5 {
                    let mut card = div()
                        .w(px(300.))
                        .flex_shrink_0()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .px_3()
                        .py_2()
                        .smooth_background(self.palette.sidebar)
                        .child(div().text_lg().child(format!("风扇曲线 {}", point + 1)));
                    for id in [19 + 2 * point, 20 + 2 * point] {
                        if let Some((i, _)) =
                            self.controls.iter().enumerate().find(|(_, c)| c.id == id)
                        {
                            card = card.child(self.tuning_control(i, cx));
                        }
                    }
                    curves = curves.child(card);
                }
                page = page.child(curves);
            } else if self.tuning.mode == 2 {
                page = page.child(div().text_lg().child("风扇转速范围"));
                let mut range = div().flex().flex_wrap().gap_3();
                let mut found = false;
                for (i, control) in self.controls.iter().enumerate() {
                    if matches!(control.id, 20 | 28) && self.show_control(control.id) {
                        range = range.child(self.tuning_field(i, cx));
                        found = true;
                    }
                }
                if found {
                    page = page.child(range);
                }
            }
            return page;
        }
        for (label, ids) in [
            ("核心调校", &[0, 1][..]),
            ("显存调校", &[8, 14, ECC][..]),
            ("功耗与电压", &[9, 37, 47][..]),
        ] {
            let mut group = div()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_lg().child(label));
            let mut found = false;
            for (i, control) in self.controls.iter().enumerate() {
                if ids.contains(&control.id) {
                    group = group.child(self.tuning_field(i, cx));
                    found = true;
                }
            }
            if found {
                page = page.child(group);
            }
        }
        page
    }

    fn tuning_label(&self, control: &crate::adl::overdrive::Control) -> &'static str {
        match (self.tuning.mode, control.id) {
            (2, 20) => "最低风扇转速",
            (2, 28) => "最高风扇转速",
            (3, 19..=28) if control.id % 2 == 1 => "温度",
            (3, 19..=28) => "转速",
            _ => control.label,
        }
    }

    fn tuning_control(&self, i: usize, cx: &mut Context<Self>) -> Div {
        let control = &self.controls[i];
        let input = self.number(
            super::numeric::Field::Tuning(i),
            control.draft,
            control.min,
            control.max,
            control.step,
            control.unit,
            cx,
        );
        self.setting_row(self.hint_label(self.tuning_label(control)), input)
    }

    fn tuning_field(&self, i: usize, cx: &mut Context<Self>) -> Div {
        let control = &self.controls[i];
        if control.id == 15 && control.min == 0 && control.max == 1 {
            return self.toggle_row(
                "fan-zero-rpm",
                self.hint_label("风扇零 RPM 模式"),
                control.draft != 0,
                true,
                move |p, _| p.controls[i].draft = i32::from(p.controls[i].draft == 0),
                cx,
            );
        }
        if control.id == 14 || control.id == crate::adl::overdrive::ECC {
            let is_ecc = control.id == crate::adl::overdrive::ECC;
            let description: &[&str] = if is_ecc {
                &["默认", "禁用", "启用"]
            } else {
                &["默认", "快速时序 1", "快速时序 2"]
            };
            let minimum = control.min;
            let labels = (minimum..=control.max)
                .map(|v| {
                    description
                        .get(v as usize)
                        .map_or_else(|| v.to_string(), |s| (*s).into())
                })
                .collect();
            return self.setting_row(
                self.hint_label(self.tuning_label(control)),
                self.select(
                    ("tuning-choice", i),
                    labels,
                    Some((control.draft - minimum) as usize),
                    "选择",
                    crate::panel::controls::FIELD_WIDTH,
                    move |p, at, _| p.controls[i].draft = minimum + at as i32,
                    cx,
                ),
            );
        }
        self.tuning_control(i, cx)
    }

    pub(super) fn tuning_changed(&self) -> bool {
        self.tuning.dirty || self.controls.iter().any(|c| c.draft != c.current)
    }
    pub(super) fn reset_tuning_editor(&mut self) {
        self.curve_drag = None;
        self.tuning = Editor::default();
    }
}
