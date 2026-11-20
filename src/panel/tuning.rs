use super::Panel;
use crate::{adl::overdrive::ECC, profile_store, profiles};
use anyhow::{Context as _, Result, ensure};
use gpui::{Context, Div, PathPromptOptions, div, prelude::*, rgb};
use serde_json::{Value, json};

pub(super) struct Editor {
    app: String,
    apps: Vec<String>,
    mode: usize,
    quiet: bool,
    keep: bool,
    dirty: bool,
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
        let Some(row) = rows
            .iter()
            .find(|row| profile_store::matches(row, &device.profile_key, &self.tuning.app))
        else {
            return Ok(());
        };
        self.tuning.mode = row["FanControlMode.Current"].as_u64().unwrap_or(0).min(3) as usize;
        self.tuning.quiet = row["FanRangedCurve.Current"].as_i64() == Some(1);
        self.tuning.keep = row["KeepActive.Current"].as_i64() == Some(1)
            || row["KeepActive.Current"].as_bool() == Some(true);
        for c in &mut self.controls {
            let k = match (self.tuning.mode, c.id) {
                (2, 20) => "FanSpeedMin.Current".into(),
                (2, 28) => "FanSpeedMax.Current".into(),
                _ => key(c.id).context("调校项目未知")?,
            };
            if let Some(value) = row[k].as_i64().and_then(|n| i32::try_from(n).ok()) {
                c.draft = value.clamp(c.min, c.max);
            }
        }
        Ok(())
    }

    pub(super) fn show_control(&self, id: usize) -> bool {
        match id {
            15 => self.tuning.mode != 0,
            19..=28 => self.tuning.mode == 3 || (self.tuning.mode == 2 && matches!(id, 20 | 28)),
            _ => true,
        }
    }

    pub(super) fn save_tuning(&mut self) -> Result<()> {
        let device = &self.devices[self.selected];
        let driver = self.driver.as_ref().context("AMD 驱动不可用")?;
        let path = self.settings.profiles_path("TuningProfiles.json")?;
        let mut row = json!({"DeviceId":device.profile_key,"Application":self.tuning.app, "KeepActive.Current":i32::from(self.tuning.keep), "FanControlMode.Current":self.tuning.mode,"FanRangedCurve.Current":i32::from(self.tuning.quiet)});
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
        let mut controls = driver.controls(device.index)?;
        profiles::tuning_draft(&mut controls, &row)?;
        profile_store::save(&path, &device.profile_key, &self.tuning.app, &row)?;
        let rows = profile_store::load(&path)?;
        let running = profiles::process_names()?;
        if profiles::select_profile(&rows, &device.profile_key, &running)
            .is_some_and(|p| profile_store::matches(p, &device.profile_key, &self.tuning.app))
        {
            if controls.iter().any(|c| c.current != c.draft || c.reset) {
                driver.apply(device.index, &controls)?;
            }
        }
        self.read_controls();
        Ok(())
    }

    pub(super) fn render_tuning_profiles(&self, cx: &mut Context<Self>) -> Div {
        let mut section = div().flex().flex_col().gap_2();
        let mut choices = div().flex().flex_wrap().gap_2();
        for (i, app) in self.tuning.apps.iter().enumerate() {
            let name = app.clone();
            choices = choices.child(
                div()
                    .id(("tuning-app", i))
                    .cursor_pointer()
                    .px_2()
                    .bg(rgb(if app == &self.tuning.app {
                        0x713e47
                    } else {
                        0x292d34
                    }))
                    .child(if app == "Global" {
                        "全局".to_string()
                    } else {
                        app.clone()
                    })
                    .on_click(cx.listener(move |panel, _, _, cx| {
                        panel.tuning.app = name.clone();
                        panel.read_controls();
                        cx.notify();
                    })),
            );
        }
        choices = choices.child(
            div()
                .id("pick-tuning-app")
                .cursor_pointer()
                .child("选择应用…")
                .on_click(cx.listener(|panel, _, _, cx| {
                    let device = panel.devices[panel.selected].profile_key.clone();
                    let pick = cx.prompt_for_paths(PathPromptOptions {
                        files: true,
                        directories: false,
                        multiple: false,
                        prompt: Some("选择应用程序".into()),
                    });
                    cx.spawn(async move |view, cx| {
                        let result: Result<Option<String>> = async {
                            let Some(paths) = pick.await?? else {
                                return Ok(None);
                            };
                            let path = paths.first().context("未选择文件")?;
                            ensure!(
                                path.extension()
                                    .is_some_and(|s| s.eq_ignore_ascii_case("exe")),
                                "请选择 EXE 程序"
                            );
                            Ok(Some(
                                path.file_name()
                                    .context("文件名无效")?
                                    .to_string_lossy()
                                    .into_owned(),
                            ))
                        }
                        .await;
                        let _ = view.update(cx, |panel, cx| {
                            if panel
                                .devices
                                .get(panel.selected)
                                .is_some_and(|d| d.profile_key == device)
                            {
                                match result {
                                    Ok(Some(name)) => {
                                        panel.tuning.app = name;
                                        panel.read_controls();
                                    }
                                    Ok(None) => {}
                                    Err(err) => panel.tuning_status = err.to_string(),
                                }
                                cx.notify();
                            }
                        });
                    })
                    .detach();
                })),
        );
        section = section.child(choices);
        section = section.child(
            div()
                .id("keep-tuning")
                .cursor_pointer()
                .child(if self.tuning.keep {
                    "持续应用配置：开"
                } else {
                    "持续应用配置：关"
                })
                .on_click(cx.listener(|p, _, _, cx| {
                    p.tuning.keep = !p.tuning.keep;
                    p.tuning.dirty = true;
                    cx.notify();
                })),
        );
        if self.page != 1 {
            return section;
        }
        let mut modes = div().flex().gap_2();
        for (i, label) in ["驱动默认", "仅零转速", "范围转速", "自定义曲线"]
            .into_iter()
            .enumerate()
        {
            modes = modes.child(
                div()
                    .id(("fan-mode", i))
                    .px_2()
                    .cursor_pointer()
                    .bg(rgb(if self.tuning.mode == i {
                        0x713e47
                    } else {
                        0x292d34
                    }))
                    .child(label)
                    .on_click(cx.listener(move |p, _, _, cx| {
                        p.tuning.mode = i;
                        p.tuning.dirty = true;
                        cx.notify();
                    })),
            );
        }
        section = section.child(modes);
        if self.tuning.mode == 2 {
            section = section.child(
                div()
                    .id("fan-shape")
                    .cursor_pointer()
                    .child(if self.tuning.quiet {
                        "曲线：静音"
                    } else {
                        "曲线：线性"
                    })
                    .on_click(cx.listener(|p, _, _, cx| {
                        p.tuning.quiet = !p.tuning.quiet;
                        p.tuning.dirty = true;
                        cx.notify();
                    })),
            );
        }
        section
    }
    pub(super) fn tuning_changed(&self) -> bool {
        self.tuning.dirty || self.controls.iter().any(|c| c.draft != c.current)
    }
    pub(super) fn reset_tuning_editor(&mut self) {
        self.tuning = Editor::default();
    }
}
