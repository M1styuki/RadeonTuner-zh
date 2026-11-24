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
        let mut section = div().flex().flex_col().gap_0();
        for game in &self.fsr_activity {
            section = section.child(game.clone());
        }
        use crate::adl::graphics::frames::Field;
        const GROUPS: &[&[&str]] = &[
            &[
                "DeLagEnabled.Current",
                "MfgRatio.Current",
                "FsrShowInformation.Current",
                "FsrDisableMLSwapchain.Current",
                "FsrOverride.Current",
                "MlfiOverride.Current",
                "MfgOverride.Current",
                "MldOverride.Current",
                "NrcOverride.Current",
                "FsrOvrDLLPath.Current",
            ],
            &[
                "FrameGenEnabled.Current",
                "FrameGenAlgorithm.Current",
                "ChillEnabled.Current",
                "ChillMinFps.Current",
                "ChillMaxFps.Current",
                "FrtcEnabled.Current",
                "FrtcFrameRateTarget.Current",
            ],
            &[
                "BoostMode.Current",
                "BoostMinResolution.Current",
                "RisEnabled.Current",
                "RisDesktopEnabled.Current",
                "RisSharpeningDegree.Current",
            ],
            &[
                "VerticalSync.Current",
                "EnhancedSync.Current",
                "OpenGLTripleBuffering.Current",
            ],
            &[
                "AntiAliasingOverride.Current",
                "AntiAliasingMethod.Current",
                "AntiAliasingLevel.Current",
                "AntiAliasingEnhancedQuality.Current",
                "AntiAliasingMorphological.Current",
            ],
            &[
                "AnisotropicOverride.Current",
                "TextureFilteringQuality.Current",
                "SurfaceFormatOptimization.Current",
                "OpenGL10BitPixelFormat.Current",
                "TessellationMode.Current",
                "TessellationLevel.Current",
            ],
        ];
        let mut categories = Vec::new();
        for &keys in GROUPS {
            let mut rows = div().flex().flex_col();
            let mut has_rows = false;
            for &key in keys {
                if key == "FsrOvrDLLPath.Current" {
                    if let Some(fsr) = self.fsr_controls(cx) {
                        rows = rows.child(fsr);
                        has_rows = true;
                    }
                    continue;
                }
                if let Some((i, control)) = self
                    .frames
                    .iter()
                    .enumerate()
                    .find(|(_, c)| c.field.name() == key)
                {
                    if !control.field.enabled(&self.frames)
                        || (control.field.experimental() && !self.experimental)
                    {
                        continue;
                    }
                    has_rows = true;
                    let label = control.field.label();
                    let mut control_view = if control.min == 0 && control.max == 1 {
                        rows = rows.child(self.toggle_row(
                            ("frame-toggle", i),
                            self.hint_label(label),
                            control.draft != 0,
                            true,
                            move |p, _| p.change_frame(i, i32::from(p.frames[i].draft == 0)),
                            cx,
                        ));
                        continue;
                    } else if !control.field.entries().is_empty() {
                        self.select(
                            ("frame-choice", i),
                            control
                                .field
                                .entries()
                                .iter()
                                .map(|s| s.to_string())
                                .collect(),
                            Some(control.draft as usize),
                            label,
                            crate::panel::controls::SELECT_MAX_WIDTH,
                            move |p, value, _| p.change_frame(i, value as i32),
                            cx,
                        )
                    } else if self.chill_linked && control.field == Field::Low {
                        div().child(format!("{} FPS", control.draft)).opacity(0.5)
                    } else {
                        self.number(
                            super::numeric::Field::Frame(i),
                            control.draft,
                            control.min,
                            control.max,
                            control.step,
                            control.field.unit(),
                            cx,
                        )
                    };
                    if control.field == Field::High {
                        control_view = div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                self.button_style(true)
                                    .min_w(gpui::px(44.))
                                    .id("chill-link")
                                    .cursor_pointer()
                                    .child(if self.chill_linked {
                                        "取消联动"
                                    } else {
                                        "联动"
                                    })
                                    .on_click(cx.listener(|p, _, _, cx| {
                                        p.chill_linked = !p.chill_linked;
                                        cx.notify();
                                    })),
                            )
                            .child(control_view);
                    }
                    rows = rows.child(self.setting_row(self.hint_label(label), control_view));
                }
                if let Some((i, s)) = self
                    .graphics
                    .iter()
                    .enumerate()
                    .find(|(_, s)| s.choice.profile_key == key)
                {
                    if s.choice.experimental && !self.experimental {
                        continue;
                    }
                    has_rows = true;
                    let label = s.choice.label;
                    let set = move |p: &mut Panel, index, _: &mut Context<Panel>| {
                        p.graphics[i].draft = index;
                        p.graphics_dirty = true;
                    };
                    if matches!(
                        key,
                        "FsrOverride.Current"
                            | "MlfiOverride.Current"
                            | "MfgOverride.Current"
                            | "MldOverride.Current"
                            | "NrcOverride.Current"
                            | "EnhancedSync.Current"
                            | "OpenGLTripleBuffering.Current"
                            | "AntiAliasingOverride.Current"
                            | "AntiAliasingEnhancedQuality.Current"
                            | "AntiAliasingMorphological.Current"
                            | "SurfaceFormatOptimization.Current"
                    ) {
                        rows = rows.child(self.toggle_row(
                            ("graphics-toggle", i),
                            self.hint_label(label),
                            s.draft != 0,
                            true,
                            move |p, cx| set(p, usize::from(p.graphics[i].draft == 0), cx),
                            cx,
                        ));
                    } else {
                        rows = rows.child(
                            self.setting_row(
                                self.hint_label(label),
                                self.select(
                                    ("graphics-choice", i),
                                    s.choice
                                        .entries
                                        .iter()
                                        .map(|(label, _)| label.to_string())
                                        .collect(),
                                    Some(s.draft),
                                    label,
                                    crate::panel::controls::SELECT_MAX_WIDTH,
                                    set,
                                    cx,
                                ),
                            ),
                        );
                    }
                }
            }
            if has_rows {
                categories.push(rows);
            }
        }
        if self.graphics.is_empty() && self.fsr_library.is_none() && self.frames.is_empty() {
            section = section.child("当前显卡没有可读取的图形设置");
        }
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(section)
            .child(self.setting_sections(categories))
    }

    fn fsr_controls(&self, cx: &mut Context<Self>) -> Option<Div> {
        let fsr = self.fsr_library.as_ref()?;
        let mut section = div().flex().flex_col();
        let mut row = div().flex().items_center().gap_2();
        row = row.child(
            self.button_style(true)
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
                                        if path.file_name().and_then(|s| s.to_str()).is_some_and(
                                            |s| {
                                                let name = s.to_ascii_lowercase();
                                                name.starts_with("amdxcff")
                                                    && name.ends_with(".dll")
                                            },
                                        ) {
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
            self.button_style(true)
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
        section = section.child(
            self.setting_row(
                div()
                    .flex()
                    .min_w_0()
                    .items_center()
                    .gap_2()
                    .child(
                        self.hint_label("FSR 覆盖库")
                            .whitespace_nowrap()
                            .flex_shrink_0(),
                    )
                    .child(
                        div()
                            .id("fsr-library-description")
                            .flex_1()
                            .min_w_0()
                            .text_xs()
                            .truncate()
                            .text_color(gpui::rgb(self.palette.muted_text))
                            .tooltip(self.tooltip_owned(fsr.description.clone().into()))
                            .child(fsr.description.clone()),
                    ),
                row,
            ),
        );
        if self.experimental {
            let path = fsr.path().display().to_string();
            section = section.child(
                div()
                    .id("fsr-library-path")
                    .w_full()
                    .text_xs()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_color(gpui::rgb(self.palette.muted_text))
                    .tooltip(self.tooltip_owned(path.clone().into()))
                    .child(path),
            );
        }
        Some(section)
    }

    pub(super) fn graphics_actions(&self, cx: &mut Context<Self>) -> Div {
        let mut actions = div().flex().flex_wrap().items_center().gap_2();
        if self.graphics.is_empty() && self.fsr_library.is_none() && self.frames.is_empty() {
            return actions;
        }
        let changed = self.graphics_dirty
            || self
                .frames
                .iter()
                .any(|c| c.field.enabled(&self.frames) && c.current != c.draft)
            || self.graphics.iter().any(|s| s.draft != s.current)
            || self
                .fsr_library
                .as_ref()
                .is_some_and(|f| f.current != f.draft);
        let apply = self
            .button_style(changed)
            .id("graphics-apply")
            .tooltip(self.tooltip("应用显卡设置。"))
            .child("应用");
        actions = actions.child(if changed {
            apply.cursor_pointer().on_click(cx.listener(|p, _, _, cx| {
                p.graphics_status = p
                    .save_graphics(false)
                    .map(|()| "图形设置已应用".into())
                    .unwrap_or_else(|e| e.to_string());
                cx.notify();
            }))
        } else {
            apply
        });
        if !self.graphics.is_empty() || !self.frames.is_empty() || self.fsr_library.is_some() {
            if self.confirm_graphics_reset {
                actions = actions.child(
                    div()
                        .flex()
                        .gap_2()
                        .child(if self.games.selected.is_some() {
                            "恢复所选游戏的图形默认设置？"
                        } else {
                            "恢复这张显卡的全局图形默认设置？"
                        })
                        .child(
                            self.inline_action()
                                .id("confirm-graphics-reset")
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
                            self.inline_action()
                                .id("cancel-graphics-reset")
                                .child("取消")
                                .on_click(cx.listener(|p, _, _, cx| {
                                    p.confirm_graphics_reset = false;
                                    cx.notify();
                                })),
                        ),
                );
            } else {
                actions = actions.child(
                    self.button_style(true)
                        .id("graphics-reset")
                        .tooltip(self.tooltip("将显卡设置恢复为默认值。"))
                        .cursor_pointer()
                        .child("重置")
                        .on_click(cx.listener(|p, _, _, cx| {
                            p.confirm_graphics_reset = true;
                            cx.notify();
                        })),
                );
            }
        }
        actions = actions.child(self.transfer_buttons(super::transfer::Kind::Graphics, cx));
        if self.devices.get(self.selected).is_some() {
            actions = actions.child(
                self.button_style(true)
                    .id("clear-shaders")
                    .tooltip(self.tooltip("清除基于磁盘的着色器缓存内容。"))
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
                actions = actions.child(
                    self.inline_action()
                        .id("cancel-clear-shaders")
                        .child("取消")
                        .on_click(cx.listener(|p, _, _, cx| {
                            p.confirm_cache = false;
                            cx.notify();
                        })),
                );
            }
        }
        actions
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
