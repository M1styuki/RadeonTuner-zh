use super::{Panel, controls::card_rows, shape::ContinuousCorners as _};
use crate::{adl::overdrive::ECC, profile_store, profiles};
use anyhow::{Context as _, Result, ensure};
use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::{
    Context, Div, InteractiveElement as _, ParentElement as _, StatefulInteractiveElement as _,
    Styled as _, div, px,
};
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

pub(super) type FanCurve = [(i32, i32); 5];

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
    pub(super) fn fan_curves(&self) -> Option<(FanCurve, FanCurve)> {
        let get = |id| self.controls.iter().find(|c| c.id == id);
        let range = if self.tuning.mode == 2 {
            profiles::curve(get(20)?.draft, get(28)?.draft, self.tuning.quiet)
        } else {
            [0; 5]
        };
        let mut current = [(0, 0); 5];
        let mut target = [(0, 0); 5];
        for i in 0..5 {
            let temp = get(19 + i * 2)?;
            let speed = get(20 + i * 2)?;
            current[i] = (temp.current, speed.current);
            target[i] = match self.tuning.mode {
                3 => (temp.draft, speed.draft),
                2 => (temp.default, range[i]),
                _ => (temp.default, speed.default),
            };
        }
        Some((current, target))
    }

    pub(super) fn is_fan_point(&self, index: usize) -> bool {
        self.tuning.mode == 3
            && self
                .controls
                .get(index)
                .is_some_and(|c| (19..=28).contains(&c.id))
    }

    pub(super) fn set_tuning_number(&mut self, index: usize, mut value: i32) {
        if self.is_fan_point(index) {
            let control = &self.controls[index];
            let get = |id| self.controls.iter().find(|c| c.id == id).map(|c| c.draft);
            let gap = i32::from(control.id % 2 == 1);
            let min = if control.id >= 21 {
                get(control.id - 2).map_or(control.min, |prev| {
                    prev.saturating_add(gap).max(control.min)
                })
            } else {
                control.min
            };
            let max = if control.id <= 26 {
                get(control.id + 2).map_or(control.max, |next| {
                    next.saturating_sub(gap).min(control.max)
                })
            } else {
                control.max
            };
            if min > max {
                return;
            }
            value = value.clamp(min, max);
        }
        if let Some(control) = self.controls.get_mut(index) {
            control.draft = value;
        }
    }

    pub(super) fn move_curve_point(&mut self, point: usize, temperature: i32, speed: i32) {
        if self.tuning.mode != 3 || point >= 5 {
            return;
        }
        for (axis, desired) in [temperature, speed].into_iter().enumerate() {
            if let Some(index) = self
                .controls
                .iter()
                .position(|c| c.id == 19 + point * 2 + axis)
            {
                self.set_tuning_number(index, desired);
            }
        }
    }

    pub(super) fn fan_curve_editable(&self) -> bool {
        self.tuning.mode == 3
    }

    #[cfg(test)]
    pub(super) fn override_fan_mode_for_preview(&mut self) {
        self.tuning.mode = 3;
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
                    crate::panel::controls::PROFILE_SELECT_WIDTH,
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
    pub(super) fn tuning_actions(&self, cx: &mut Context<Self>) -> Div {
        let changed = self.tuning_changed();
        let apply = self.primary_button(
            "apply-tuning",
            "应用",
            changed,
            Some("应用调校与风扇设置。"),
            |p, _, _| {
                p.tuning_status = p
                    .save_tuning()
                    .map(|()| "已应用调校配置".to_owned())
                    .unwrap_or_else(|err| err.to_string());
            },
            cx,
        );
        let mut actions = h_flex().flex_wrap().gap_2().child(apply);
        actions = actions.child(if self.tuning.confirm_reset {
            self.danger_button(
                "reset-tuning-profile",
                "确认重置",
                Some("将调校与风扇设置恢复为默认值。"),
                |p, _, _| {
                    p.tuning.confirm_reset = false;
                    p.tuning_status = p
                        .reset_tuning_profile()
                        .map(|()| "调校与风扇设置已重置".to_owned())
                        .unwrap_or_else(|e| e.to_string());
                },
                cx,
            )
        } else {
            self.contour_button(
                "reset-tuning-profile",
                "重置",
                true,
                Some("将调校与风扇设置恢复为默认值。"),
                |p, _, _| p.tuning.confirm_reset = true,
                cx,
            )
        });
        actions = actions.child(self.transfer_buttons(super::transfer::Kind::Tuning, cx));
        if self.tuning.confirm_reset {
            actions = actions.child(self.contour_button(
                "cancel-tuning-reset",
                "取消",
                true,
                None,
                |p, _, _| p.tuning.confirm_reset = false,
                cx,
            ));
        }
        actions.child(self.contour_button(
            "reload-tuning",
            "读取驱动",
            true,
            None,
            |p, _, _| p.read_controls(),
            cx,
        ))
    }

    fn tuning_persistence(&self, cx: &mut Context<Self>) -> Div {
        self.toggle_row(
            "keep-tuning",
            self.hint_label("保持生效"),
            self.tuning.keep,
            true,
            |p, _| {
                p.tuning.keep = !p.tuning.keep;
                p.tuning.dirty = true;
            },
            cx,
        )
    }

    fn fan_mode_settings(&self, cx: &mut Context<Self>) -> Vec<Div> {
        let mut rows = vec![
            self.setting_row(
                self.hint_label("风扇控制模式"),
                self.select(
                    "fan-mode",
                    ["无风扇控制", "仅零 RPM", "范围风扇转速", "自定义风扇曲线"]
                        .map(str::to_owned)
                        .into(),
                    Some(self.tuning.mode),
                    "风扇控制模式",
                    crate::panel::controls::SELECT_MAX_WIDTH,
                    |p, i, _| {
                        p.tuning.mode = i;
                        p.tuning.dirty = true;
                    },
                    cx,
                ),
            ),
        ];
        if self.tuning.mode == 2 {
            rows.push(self.setting_row(
                self.hint_label("范围风扇曲线"),
                self.select(
                    "fan-shape",
                    ["线性", "静音"].map(str::to_owned).into(),
                    Some(usize::from(self.tuning.quiet)),
                    "范围风扇曲线",
                    crate::panel::controls::SELECT_MAX_WIDTH,
                    |p, i, _| {
                        p.tuning.quiet = i == 1;
                        p.tuning.dirty = true;
                    },
                    cx,
                ),
            ));
        }
        rows
    }

    pub(super) fn render_tuning_fields(&self, cx: &mut Context<Self>) -> Div {
        let mut groups = Vec::new();
        if self.page == 1 {
            let mut mode_rows = self.fan_mode_settings(cx);
            mode_rows.extend(self.ordered_tuning_rows(&[15], cx));
            groups.push(card_rows(mode_rows, cx));

            let mut blocks = Vec::new();
            if let Some(curve) = self.render_curve(cx) {
                blocks.push(div().px_4().py_3().child(curve));
            }
            if self.tuning.mode == 2 {
                blocks.extend(self.ordered_tuning_rows(&[20, 28], cx));
            }
            if self.tuning.mode == 3
                && self
                    .controls
                    .iter()
                    .any(|control| (19..=28).contains(&control.id))
            {
                blocks.push(div().px_4().py_3().child(self.fan_point_card(cx)));
            }
            if !blocks.is_empty() {
                groups.push(card_rows(blocks, cx));
            }
        } else {
            for ids in [&[1, 0, 37][..], &[8, 14, ECC][..], &[9, 47][..]] {
                let rows = self.ordered_tuning_rows(ids, cx);
                if !rows.is_empty() {
                    groups.push(card_rows(rows, cx));
                }
            }
        }
        groups.push(self.tuning_persistence(cx));
        self.setting_sections(groups, cx)
    }

    fn ordered_tuning_rows(&self, ids: &[usize], cx: &mut Context<Self>) -> Vec<Div> {
        ids.iter()
            .filter_map(|&id| {
                if !self.show_control(id) {
                    return None;
                }
                self.controls
                    .iter()
                    .position(|control| control.id == id)
                    .map(|index| self.tuning_field(index, cx))
            })
            .collect()
    }

    fn fan_point_card(&self, cx: &mut Context<Self>) -> impl gpui_kit::IntoElement {
        const CARD_PADDING: f32 = 12.;
        const SCROLL_EDGE_GUTTER: f32 = 2.;
        let muted = cx.theme().muted_foreground;
        let point_column = || div().w(px(40.)).flex_shrink_0();
        let value_column = || div().min_w_0().flex_1();
        let row = || div().flex().items_center().gap_4();
        let mut rows = v_flex()
            .w_full()
            .min_w(px(320. + SCROLL_EDGE_GUTTER))
            .pr(px(SCROLL_EDGE_GUTTER))
            .gap_2()
            .child(
                row()
                    .text_xs()
                    .text_color(muted)
                    .child(point_column())
                    .child(value_column().child("温度（°C）"))
                    .child(value_column().child("转速（%）")),
            );
        for i in 0..5 {
            let cells = [19 + 2 * i, 20 + 2 * i].map(|id| {
                let value = self
                    .controls
                    .iter()
                    .position(|control| control.id == id)
                    .map(|index| self.tuning_number_input(index, cx))
                    .unwrap_or_else(|| div().text_color(muted).child("—"));
                value_column().child(value)
            });
            rows = rows.child(
                row()
                    .child(point_column().child(format!("点{}", i + 1)))
                    .children(cells),
            );
        }
        div()
            .relative()
            .w_full()
            .p(px(CARD_PADDING))
            // Keep part of the right padding inside the scroll viewport so the
            // last input outline never sits directly on the clipping edge.
            .pr(px(CARD_PADDING - SCROLL_EDGE_GUTTER))
            .rounded(cx.theme().radius)
            .contour(
                Some(cx.theme().input_background()),
                Some(cx.theme().border),
                f32::from(cx.theme().radius),
            )
            .child(
                div()
                    .id("fan-point-editor")
                    .w_full()
                    .min_w_0()
                    .overflow_x_scroll()
                    .child(rows),
            )
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
        self.setting_row(
            self.hint_label(self.tuning_label(&self.controls[i])),
            self.tuning_number_input(i, cx),
        )
    }

    fn tuning_number_input(&self, i: usize, cx: &mut Context<Self>) -> Div {
        let control = &self.controls[i];
        self.number(
            super::numeric::Field::Tuning(i),
            control.draft,
            control.min,
            control.max,
            control.step,
            control.unit,
            cx,
        )
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
                    crate::panel::controls::SELECT_MAX_WIDTH,
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
