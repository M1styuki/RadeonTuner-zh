pub(super) mod apps;
use super::Panel;
use crate::profile_store;
use anyhow::{Context as _, Result, ensure};
use serde_json::{Value, json};

use gpui::{Context, Div, div, prelude::*};
impl Panel {
    pub(super) fn graphics_target(&self) -> Option<(String, String)> {
        Some((
            self.devices.get(self.selected)?.profile_key.clone(),
            self.game_name(),
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
        let (indices, frames) =
            crate::adl::graphics::profile_drafts(&self.graphics, &self.frames, row)?;
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
        self.confirm_graphics_reset = false;
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

    fn reset_graphics(&mut self) -> Result<()> {
        if self.games.selected.is_some() {
            self.confirm_graphics_reset = false;
            crate::adl::graphics::apps::settings::defaults(&mut self.graphics, &mut self.frames);
            if let Some(f) = &mut self.fsr_library {
                f.select(String::new());
            }
            return self.save_game(true);
        }
        use crate::adl::graphics::frames::Field::{DeepColor, Desktop};
        self.confirm_graphics_reset = false;
        let driver = self.driver.as_ref().context("AMD 驱动不可用")?;
        let d = self.devices.get(self.selected).context("未选择显卡")?;
        self.graphics = driver.graphics(d, self.skip_graphics_check);
        self.frames = driver.frame_controls(d, self.skip_graphics_check);
        self.fsr_library = driver.fsr_library(d, self.skip_graphics_check);
        let ris2 = self.frames.iter().any(|c| c.field == Desktop);
        let deep = if self.frames.iter().any(|c| c.field == DeepColor) {
            driver.deep_color(d).context("无法读取驱动的默认色深")?.0
        } else {
            0
        };
        for c in &mut self.frames {
            c.draft = c.field.default_value(ris2, deep).clamp(c.min, c.max);
        }
        for s in &mut self.graphics {
            s.draft = match s.choice.profile_key {
                "VerticalSync.Current" | "TextureFilteringQuality.Current" => 1,
                "TessellationLevel.Current" => 7,
                _ => 0,
            };
        }
        if let Some(fsr) = &mut self.fsr_library {
            fsr.select(String::new());
        }
        self.graphics_profile = json!({});
        self.save_graphics(true)
    }

    fn save_graphics(&mut self, reset: bool) -> Result<()> {
        crate::adl::graphics::frames::validate(&self.frames)?;
        if self.games.selected.is_some() {
            return self.save_game(reset);
        }
        let _write = crate::driver_write::lock()?;
        let (key, app) = self.graphics_target().context("未选择显卡")?;
        let row = self.graphics_record()?;
        let path = self.settings.profiles_path("GraphicsProfiles.json")?;
        let mut rows = profile_store::load(&path)?;
        let staged = if key.is_empty() {
            None
        } else {
            profile_store::upsert(&mut rows, &key, &app, &row, reset)?;
            Some(profile_store::staged_json(&path, &json!(&rows))?)
        };
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
        if result.is_ok() {
            if let Some(staged) = staged {
                staged
                    .persist(&path)
                    .context("已应用图形设置，但保存配置失败")?;
            }
        }
        self.read_graphics_profile()?;
        result
    }

    pub(super) fn change_frame(&mut self, index: usize, val: i32) {
        use crate::adl::graphics::frames::Field::{High, Low};
        let Some(control) = self.frames.get_mut(index) else {
            return;
        };
        let field = control.field;
        control.draft = val;
        for c in &mut self.frames {
            if (field == High && c.field == Low && (self.chill_linked || c.draft > val))
                || (field == Low && c.field == High && c.draft < val)
            {
                c.draft = val.clamp(c.min, c.max);
            }
        }
        self.graphics_dirty = true;
    }
    pub(super) fn render_graphics(&self, cx: &mut Context<Self>) -> Div {
        let mut section = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(self.game_controls(cx));
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
                        let game = p.games.selected.clone();
                        let picker = cx.prompt_for_paths(gpui::PathPromptOptions {
                            files: true,
                            directories: false,
                            multiple: false,
                            prompt: Some("选择 AMD FSR 库".into()),
                        });
                        cx.spawn(async move |view, cx| {
                            let selected = async { Ok::<_, anyhow::Error>(picker.await??) }.await;
                            let _ = view.update(cx, |p, cx| {
                                if p.games.selected != game
                                    || p.devices.get(p.selected).is_none_or(|d| d.index != device)
                                {
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
        use crate::adl::graphics::frames::Field::{Chill, Low};
        if self.frames.iter().any(|c| c.field == Chill && c.draft != 0) {
            section = section.child(
                div()
                    .id("chill-link")
                    .cursor_pointer()
                    .child(if self.chill_linked {
                        "Chill 最低与最高帧率　已联动"
                    } else {
                        "联动 Chill 最低与最高帧率"
                    })
                    .on_click(cx.listener(|p, _, _, cx| {
                        p.chill_linked = !p.chill_linked;
                        cx.notify();
                    })),
            );
        }
        for (i, control) in self.frames.iter().enumerate() {
            if self.chill_linked && control.field == Low {
                if control.field.enabled(&self.frames) {
                    section = section.child(format!(
                        "{}　{} FPS（联动）",
                        control.field.label(),
                        control.draft
                    ));
                }
                continue;
            }
            if !control.field.enabled(&self.frames)
                || (control.field.experimental() && !self.experimental)
            {
                continue;
            }
            let mut controls = div().flex().items_center().gap_2();
            if control.min == 0 && control.max == 1 && control.step == 1 {
                let on = control.draft != 0;
                controls = controls.child(
                    self.toggle_switch(on)
                        .id(("frame-toggle", i))
                        .cursor_pointer()
                        .on_click(cx.listener(move |p, _, _, cx| {
                            p.change_frame(i, i32::from(!on));
                            cx.notify();
                        })),
                );
                section = section.child(self.setting_row(control.field.label(), controls));
                continue;
            }
            let entries = control.field.entries();
            if entries.is_empty() {
                controls = controls.child(self.number(
                    super::numeric::Field::Frame(i),
                    control.draft,
                    control.min,
                    control.max,
                    control.step,
                    control.field.unit(),
                    cx,
                ));
            } else {
                controls = controls.child(entries[control.draft as usize]);
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
                controls = controls.child(if next != control.draft {
                    button
                        .cursor_pointer()
                        .on_click(cx.listener(move |p, _, _, cx| {
                            p.change_frame(i, next);
                            cx.notify();
                        }))
                } else {
                    button.opacity(0.45)
                });
            }
            section = section.child(self.setting_row(control.field.label(), controls));
        }
        for (i, s) in self.graphics.iter().enumerate() {
            if s.choice.experimental && !self.experimental {
                continue;
            }
            let labels: Vec<&str> = s.choice.entries.iter().map(|(label, _)| *label).collect();
            let selected = s.draft;
            let current = s.choice.entries[selected].0;
            let controls = div().flex().items_center().gap_2().child(
                self.button_style(true)
                    .id(("graphics-choice", i))
                    .cursor_pointer()
                    .child(format!("{current} ▾"))
                    .on_click(cx.listener(move |p, _, _, cx| {
                        if let Some(index) = Panel::select_menu(&labels, selected) {
                            p.graphics[i].draft = index;
                            p.graphics_dirty = true;
                            cx.notify();
                        }
                    })),
            );
            section = section.child(self.setting_row(s.choice.label, controls));
        }
        if self.graphics.is_empty() && self.fsr_library.is_none() && self.frames.is_empty() {
            section = section.child("当前显卡没有可读取的图形设置");
        }
        if !self.graphics.is_empty() || !self.frames.is_empty() || self.fsr_library.is_some() {
            if self.confirm_graphics_reset {
                section = section.child(
                    div()
                        .flex()
                        .gap_2()
                        .child(if self.games.selected.is_some() {
                            "恢复所选游戏的图形默认设置？"
                        } else {
                            "恢复这张显卡的全局图形默认设置？"
                        })
                        .child(
                            div()
                                .id("confirm-graphics-reset")
                                .cursor_pointer()
                                .child("重置")
                                .on_click(cx.listener(|p, _, _, cx| {
                                    p.graphics_status = match p.reset_graphics() {
                                        Ok(()) => "图形设置已重置".into(),
                                        Err(err) => err.to_string(),
                                    };
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .id("cancel-graphics-reset")
                                .cursor_pointer()
                                .child("取消")
                                .on_click(cx.listener(|p, _, _, cx| {
                                    p.confirm_graphics_reset = false;
                                    cx.notify();
                                })),
                        ),
                );
            } else {
                section = section.child(
                    div()
                        .id("graphics-reset")
                        .cursor_pointer()
                        .child(if self.games.selected.is_some() {
                            "重置游戏图形设置"
                        } else {
                            "重置全局图形设置"
                        })
                        .on_click(cx.listener(|p, _, _, cx| {
                            p.confirm_graphics_reset = true;
                            cx.notify();
                        })),
                );
            }
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
                        p.graphics_status = match p.save_graphics(false) {
                            Ok(()) => "图形设置已应用".into(),
                            Err(err) => err.to_string(),
                        };
                        cx.notify();
                    })),
            );
        }
        if self.devices.get(self.selected).is_some() {
            section = section.child(
                div()
                    .id("clear-shaders")
                    .cursor_pointer()
                    .child(if self.confirm_cache {
                        "确认清除着色器缓存"
                    } else {
                        "清除着色器缓存"
                    })
                    .on_click(cx.listener(|p, _, _, cx| {
                        if p.confirm_cache {
                            p.confirm_cache = false;
                            p.graphics_status = crate::integration::cache::clear_shader_cache()
                                .map(|()| "已清除着色器缓存".to_owned())
                                .unwrap_or_else(|e| e.to_string());
                        } else {
                            p.confirm_cache = true;
                        }
                        cx.notify();
                    })),
            );
            if self.confirm_cache {
                section = section.child(
                    div()
                        .id("cancel-clear-shaders")
                        .cursor_pointer()
                        .child("取消")
                        .on_click(cx.listener(|p, _, _, cx| {
                            p.confirm_cache = false;
                            cx.notify();
                        })),
                );
            }
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
        self.refresh_games();
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
