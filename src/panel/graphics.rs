use super::Panel;
use crate::adl::graphics::Setting;
use crate::profile_store;
use anyhow::{Context as _, Result, ensure};
use serde_json::{Value, json};

fn setting_indices(settings: &[Setting], row: &Value) -> Result<Vec<usize>> {
    settings
        .iter()
        .map(|s| {
            let v = &row[s.choice.profile_key];
            if v.is_null() {
                return Ok(s.draft);
            }
            let index = v
                .as_u64()
                .or_else(|| v.as_bool().map(u64::from))
                .and_then(|n| usize::try_from(n).ok())
                .context("图形配置的选项无效")?;
            ensure!(
                index < s.choice.entries.len(),
                "{} 超出可选范围",
                s.choice.label
            );
            Ok(index)
        })
        .collect()
}
use gpui::{Context, Div, div, prelude::*};
impl Panel {
    pub(super) fn graphics_target(&self) -> Option<(String, String)> {
        Some((
            self.devices.get(self.selected)?.profile_key.clone(),
            "Global".into(),
        ))
    }

    pub(super) fn graphics_record(&self) -> Result<Value> {
        let (device, app) = self.graphics_target().context("未选择显卡")?;
        let mut row = self.graphics_profile.clone();
        row["DeviceId"] = json!(device);
        row["Application"] = json!(app);
        for s in &self.graphics {
            row[s.choice.profile_key] = json!(s.draft);
        }
        if self
            .frames
            .iter()
            .any(|c| c.field == crate::adl::graphics::frames::Field::Sharpen)
        {
            row["RisVersion.Version"] = json!(if self
                .frames
                .iter()
                .any(|c| c.field == crate::adl::graphics::frames::Field::Desktop)
            {
                2
            } else {
                1
            });
        }
        for c in &self.frames {
            row[c.field.name()] = json!(c.draft);
        }
        if let Some(fsr) = &self.fsr_library {
            row["FsrOvrDLLPath.Current"] = json!(if fsr.draft.trim().is_empty() {
                " "
            } else {
                &fsr.draft
            });
        }
        Ok(row)
    }

    pub(super) fn stage_graphics(&mut self, row: &Value) -> Result<()> {
        let indices = setting_indices(&self.graphics, row)?;
        let mut frames = self.frames.clone();
        for c in &mut frames {
            let v = &row[c.field.name()];
            if !v.is_null() {
                let value = v
                    .as_i64()
                    .or_else(|| v.as_bool().map(i64::from))
                    .context("帧率配置数值无效")?;
                if c.field == crate::adl::graphics::frames::Field::Target && value == 0 {
                    continue;
                }
                ensure!(
                    (i64::from(c.min)..=i64::from(c.max)).contains(&value),
                    "{} 超出驱动范围",
                    c.field.label()
                );
                c.draft = value as i32;
            }
        }
        crate::adl::graphics::frames::validate(&frames)?;
        let path = &row["FsrOvrDLLPath.Current"];
        ensure!(path.is_null() || path.is_string(), "FSR 库路径无效");
        let fields = row.as_object().context("图形配置格式无效")?;
        self.graphics_profile
            .as_object_mut()
            .context("图形配置格式无效")?
            .extend(fields.clone());
        for (setting, index) in self.graphics.iter_mut().zip(indices) {
            setting.draft = index;
        }
        if let (Some(path), Some(fsr)) = (path.as_str(), &mut self.fsr_library) {
            fsr.select(path.trim().to_string());
        }
        self.frames = frames;
        self.graphics_dirty = true;
        Ok(())
    }

    fn read_graphics_profile(&mut self) -> Result<()> {
        self.graphics_profile = json!({});
        self.graphics_dirty = false;
        let Some((key, app)) = self.graphics_target() else {
            return Ok(());
        };
        if key.is_empty()
            || (self.graphics.is_empty() && self.fsr_library.is_none() && self.frames.is_empty())
        {
            return Ok(());
        }
        let path = self.settings.profiles_path("GraphicsProfiles.json")?;
        let rows = profile_store::load(&path)?;
        if let Some(row) = rows
            .iter()
            .find(|row| profile_store::matches(row, &key, &app))
        {
            self.stage_graphics(row)?;
            self.graphics_dirty = false;
        } else {
            profile_store::save(&path, &key, &app, &self.graphics_record()?)?;
        }
        Ok(())
    }

    fn save_graphics(&mut self) -> Result<()> {
        crate::adl::graphics::frames::validate(&self.frames)?;
        let (key, app) = self.graphics_target().context("未选择显卡")?;
        let row = self.graphics_record()?;
        if !key.is_empty() {
            profile_store::save(
                &self.settings.profiles_path("GraphicsProfiles.json")?,
                &key,
                &app,
                &row,
            )?;
        }
        let driver = self.driver.as_ref().context("AMD 驱动不可用")?;
        let device = &self.devices[self.selected];
        let result = driver
            .apply_graphics(device, &self.graphics)
            .and_then(|()| driver.apply_frames(device, &self.frames))
            .and_then(|()| match &self.fsr_library {
                Some(f) => driver.apply_fsr_override(device, f),
                None => Ok(()),
            });
        self.graphics = driver.graphics(device, self.skip_graphics_check);
        self.frames = driver.frame_controls(device, self.skip_graphics_check);
        self.fsr_library = driver.fsr_library(device, self.skip_graphics_check);
        self.read_graphics_profile()?;
        result
    }

    pub(super) fn render_graphics(&self, cx: &mut Context<Self>) -> Div {
        let mut section = div().flex().flex_col().gap_3();
        if !self.graphics.is_empty() || self.fsr_library.is_some() || !self.frames.is_empty() {
            section = section.child(self.transfer_buttons(super::transfer::Kind::Graphics, cx));
        }
        for game in &self.fsr_activity {
            section = section.child(game.clone());
        }
        if let Some(fsr) = &self.fsr_library {
            let mut row = div().flex().items_center().gap_2().child(
                div()
                    .flex_1()
                    .child(format!("FSR 库 · {}", fsr.description)),
            );
            row = row.child(
                div()
                    .id("fsr-pick")
                    .cursor_pointer()
                    .px_2()
                    .child("选择 DLL…")
                    .on_click(cx.listener(|p, _, _, cx| {
                        let device = p.devices[p.selected].index;
                        let picker = cx.prompt_for_paths(gpui::PathPromptOptions {
                            files: true,
                            directories: false,
                            multiple: false,
                            prompt: Some("选择 AMD FSR 库".into()),
                        });
                        cx.spawn(async move |view, cx| {
                            let selected = async { Ok::<_, anyhow::Error>(picker.await??) }.await;
                            let _ = view.update(cx, |p, cx| {
                                if p.devices.get(p.selected).is_none_or(|d| d.index != device) {
                                    return;
                                }
                                match selected {
                                    Ok(Some(paths)) => {
                                        if let (Some(path), Some(fsr)) =
                                            (paths.first(), &mut p.fsr_library)
                                        {
                                            if path
                                                .file_name()
                                                .and_then(|s| s.to_str())
                                                .is_some_and(|s| {
                                                    let name = s.to_ascii_lowercase();
                                                    name.starts_with("amdxcff")
                                                        && name.ends_with(".dll")
                                                })
                                            {
                                                fsr.select(path.to_string_lossy().into_owned());
                                                p.graphics_dirty = true;
                                            } else {
                                                p.graphics_status =
                                                    "请选择 amdxcff 开头的 AMD FSR DLL 文件".into();
                                            }
                                        }
                                    }
                                    Err(err) => p.graphics_status = err.to_string(),
                                    Ok(None) => {}
                                }
                                cx.notify();
                            });
                        })
                        .detach();
                    })),
            );
            row = row.child(
                div()
                    .id("fsr-default")
                    .cursor_pointer()
                    .px_2()
                    .child("驱动默认库")
                    .on_click(cx.listener(|p, _, _, cx| {
                        if let Some(fsr) = &mut p.fsr_library {
                            fsr.select(String::new());
                            p.graphics_dirty = true;
                        }
                        cx.notify();
                    })),
            );
            section = section.child(row);
            if self.experimental {
                section = section.child(fsr.path().display().to_string());
            }
        }
        for (i, control) in self.frames.iter().enumerate() {
            if !control.field.enabled(&self.frames)
                || (control.field.experimental() && !self.experimental)
            {
                continue;
            }
            let mut row = div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().flex_1().child(control.field.label()));
            let entries = control.field.entries();
            if entries.is_empty() {
                row = row.child(self.number(
                    super::numeric::Field::Frame(i),
                    control.draft,
                    control.min,
                    control.max,
                    control.step,
                    control.field.unit(),
                    cx,
                ));
            } else {
                row = row.child(entries[control.draft as usize]);
            }
            for (id, label, delta) in [
                ("frame-prev", "−", -control.step),
                ("frame-next", "+", control.step),
            ] {
                let next = control
                    .draft
                    .saturating_add(delta)
                    .clamp(control.min, control.max);
                let button = div().id((id, i)).px_2().child(label);
                row = row.child(if next != control.draft {
                    button
                        .cursor_pointer()
                        .on_click(cx.listener(move |p, _, _, cx| {
                            p.frames[i].draft = next;
                            p.graphics_dirty = true;
                            cx.notify();
                        }))
                } else {
                    button.opacity(0.45)
                });
            }
            section = section.child(row);
        }
        for (i, s) in self.graphics.iter().enumerate() {
            if s.choice.experimental && !self.experimental {
                continue;
            }
            let mut row = div()
                .flex()
                .gap_2()
                .items_center()
                .child(div().flex_1().child(s.choice.label))
                .child(s.choice.entries[s.draft].0);
            for (id, label, delta) in [("graphics-prev", "−", -1), ("graphics-next", "+", 1)] {
                let next = s
                    .draft
                    .checked_add_signed(delta)
                    .filter(|i| *i < s.choice.entries.len());
                let button = div().id((id, i)).px_3().child(label);
                row = row.child(if let Some(index) = next {
                    button
                        .cursor_pointer()
                        .on_click(cx.listener(move |p, _, _, cx| {
                            p.graphics[i].draft = index;
                            p.graphics_dirty = true;
                            cx.notify();
                        }))
                } else {
                    button.opacity(0.45)
                });
            }
            section = section.child(row);
        }
        if self.graphics.is_empty() && self.fsr_library.is_none() && self.frames.is_empty() {
            section = section.child("当前显卡没有可读取的图形设置");
        }
        if self.graphics_dirty
            || self
                .frames
                .iter()
                .any(|c| c.field.enabled(&self.frames) && c.current != c.draft)
            || self.graphics.iter().any(|s| s.draft != s.current)
            || self
                .fsr_library
                .as_ref()
                .is_some_and(|f| f.current != f.draft)
        {
            section = section.child(
                div()
                    .id("graphics-apply")
                    .cursor_pointer()
                    .child("应用图形设置")
                    .on_click(cx.listener(|p, _, _, cx| {
                        p.graphics_status = match p.save_graphics() {
                            Ok(()) => "图形设置已应用".into(),
                            Err(err) => err.to_string(),
                        };
                        cx.notify();
                    })),
            );
        }
        section.child(self.graphics_status.clone())
    }
    pub(super) fn read_device_details(&mut self) {
        self.read_video();
        self.graphics.clear();
        self.frames.clear();
        self.fsr_library = None;
        self.info.clear();
        self.graphics_status.clear();
        if let (Some(driver), Some(device)) = (&self.driver, self.devices.get(self.selected)) {
            self.graphics = driver.graphics(device, self.skip_graphics_check);
            self.frames = driver.frame_controls(device, self.skip_graphics_check);
            self.fsr_library = driver.fsr_library(device, self.skip_graphics_check);
            self.info = driver.information(device);
        }
        if let Err(err) = self.read_graphics_profile() {
            self.graphics_status = err.to_string();
        }
    }
}
