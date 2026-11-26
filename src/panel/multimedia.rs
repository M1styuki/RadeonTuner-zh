use super::Panel;
use crate::{
    adl::multimedia::{Feature, VideoControl, video_draft},
    profile_store, profiles,
};
use anyhow::{Context as _, Result};
use gpui_kit::component::h_flex;
use gpui_kit::{Context, Div, ParentElement as _, Styled as _, div, prelude::FluentBuilder as _};
use serde_json::{Value, json};
use std::path::Path;

fn video_record(device: &str, app: &str, controls: &[VideoControl]) -> Value {
    let mut row = json!({"DeviceId":device,"Application":app});
    for c in controls {
        row[c.feature.profile_key()] = json!(c.draft);
    }
    row
}
fn read_profile(path: &Path, device: &str, app: &str, controls: &mut [VideoControl]) -> Result<()> {
    if device.is_empty() || controls.is_empty() {
        return Ok(());
    }
    let rows = profile_store::load(path)?;
    let Some(row) = rows
        .iter()
        .find(|row| profile_store::matches(row, device, app))
        .or_else(|| {
            rows.iter()
                .find(|row| profile_store::matches(row, device, "Global"))
        })
    else {
        return Ok(());
    };
    video_draft(controls, row)?;
    for c in controls {
        c.saved = c.draft;
    }
    Ok(())
}

impl Panel {
    pub(super) fn read_video(&mut self) {
        self.confirm_video_reset = false;
        self.video.clear();
        self.video_apps = vec!["Global".into()];
        self.video_status.clear();
        if let (Some(driver), Some(device)) = (&self.driver, self.devices.get(self.selected)) {
            match driver.video_controls(device.index) {
                Ok(mut controls) => {
                    let result = self
                        .settings
                        .profiles_path("MultimediaProfiles.json")
                        .and_then(|path| {
                            for row in profile_store::load(&path)? {
                                if row["DeviceId"].as_str() == Some(&device.profile_key) {
                                    if let Some(app) =
                                        row["Application"].as_str().filter(|a| !a.is_empty())
                                    {
                                        if !self
                                            .video_apps
                                            .iter()
                                            .any(|name| name.eq_ignore_ascii_case(app))
                                        {
                                            self.video_apps.push(app.into());
                                        }
                                    }
                                }
                            }
                            crate::profiles::sort_app_names(&mut self.video_apps);
                            if !self
                                .video_apps
                                .iter()
                                .any(|a| a.eq_ignore_ascii_case(&self.video_app))
                            {
                                self.video_app = "Global".into();
                            }
                            read_profile(&path, &device.profile_key, &self.video_app, &mut controls)
                        });
                    match result {
                        Ok(()) => self.video = controls,
                        Err(err) => self.video_status = err.to_string(),
                    }
                }
                Err(err) => self.video_status = format!("无法读取视频设置：{err}"),
            }
        }
    }

    fn apply_video_profile(&mut self, reset: bool) -> Result<()> {
        let _write = crate::driver_write::lock()?;
        let driver = self.driver.as_ref().context("AMD 驱动不可用")?;
        let device = self.devices.get(self.selected).context("未选择显卡")?;
        let mut live = driver.video_controls(device.index)?;
        let mut row = video_record(&device.profile_key, &self.video_app, &self.video);
        if reset {
            for c in &live {
                row[c.feature.profile_key()] = json!(c.feature.limits().2);
            }
        }
        let path = self.settings.profiles_path("MultimediaProfiles.json")?;
        let mut rows = profile_store::load(&path)?;
        profile_store::upsert(&mut rows, &device.profile_key, &self.video_app, &row, false)?;
        let staged = profile_store::staged_json(&path, &json!(&rows))?;
        let running = profiles::process_names()?;
        let active = profiles::select_profile(&rows, &device.profile_key, &running);
        let apply = active
            .is_some_and(|row| profile_store::matches(row, &device.profile_key, &self.video_app));
        if apply {
            video_draft(&mut live, &row)?;
            driver.apply_video(device.index, &live)?;
        }
        staged.persist(&path).context("多媒体配置保存失败")?;
        crate::tray::settings_changed()?;
        self.read_video();
        if self.video_status.is_empty() {
            self.video_status = if !apply {
                "应用配置已保存"
            } else if reset {
                "多媒体设置已重置"
            } else {
                "多媒体设置已应用"
            }
            .into();
        }
        Ok(())
    }
    pub(super) fn add_video_app(&mut self, app: &str) -> Result<()> {
        let _write = crate::driver_write::lock()?;
        let driver = self.driver.as_ref().context("AMD 驱动不可用")?;
        let device = self.devices.get(self.selected).context("未选择显卡")?;
        let path = self.settings.profiles_path("MultimediaProfiles.json")?;
        let mut rows = profile_store::load(&path)?;
        let mut controls = driver.video_controls(device.index)?;
        if let Some(global) = rows
            .iter()
            .find(|row| profile_store::matches(row, &device.profile_key, "Global"))
        {
            video_draft(&mut controls, global)?;
        } else {
            let global = video_record(&device.profile_key, "Global", &controls);
            profile_store::upsert(&mut rows, &device.profile_key, "Global", &global, false)?;
        }
        if !rows
            .iter()
            .any(|row| profile_store::matches(row, &device.profile_key, app))
        {
            let row = video_record(&device.profile_key, app, &controls);
            profile_store::upsert(&mut rows, &device.profile_key, app, &row, false)?;
            profile_store::write_json(&path, &json!(rows))?;
        }
        self.video_app = app.into();
        self.read_video();
        crate::tray::settings_changed()
    }
    pub(super) fn remove_video_apps(&mut self, names: &[String]) -> Result<()> {
        let _write = crate::driver_write::lock()?;
        let driver = self.driver.as_ref().context("AMD 驱动不可用")?;
        let device = self.devices.get(self.selected).context("未选择显卡")?;
        let path = self.settings.profiles_path("MultimediaProfiles.json")?;
        let mut rows = profile_store::load(&path)?;
        let running = profiles::process_names()?;
        let active_removed = profiles::select_profile(&rows, &device.profile_key, &running)
            .is_some_and(|row| {
                names
                    .iter()
                    .any(|name| profile_store::matches(row, &device.profile_key, name))
            });
        rows.retain(|row| {
            !names
                .iter()
                .any(|name| profile_store::matches(row, &device.profile_key, name))
        });
        let staged = profile_store::staged_json(&path, &json!(&rows))?;
        if active_removed {
            let mut controls = driver.video_controls(device.index)?;
            if let Some(row) = profiles::select_profile(&rows, &device.profile_key, &running) {
                video_draft(&mut controls, row)?;
            } else {
                for control in &mut controls {
                    control.draft = control.feature.limits().2;
                }
            }
            driver.apply_video(device.index, &controls)?;
        }
        staged.persist(&path).context("多媒体配置保存失败")?;
        self.video_app = "Global".into();
        self.read_video();
        crate::tray::settings_changed()
    }
    pub(super) fn video_profiles(&self, cx: &mut Context<Self>) -> Div {
        let mut row = h_flex().gap_2();
        if self.video.is_empty() {
            return row;
        }
        let apps = self.video_apps.clone();
        let selected = apps.iter().position(|app| *app == self.video_app);
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
        row = row.child(self.select(
            "video-app",
            labels,
            selected,
            "选择应用程序",
            crate::panel::controls::PROFILE_SELECT_WIDTH,
            move |p, i, _| {
                p.video_app = apps[i].clone();
                p.read_video();
            },
            cx,
        ));
        row.child(self.app_picker_buttons(super::picker::Target::Multimedia, cx))
    }
    pub(super) fn render_video(&self, cx: &mut Context<Self>) -> Div {
        let section = div().flex().flex_col().gap_2();
        if self.video.is_empty() {
            return section.when(self.video_status.is_empty(), |view| {
                view.child("当前设备无可用的视频设置")
            });
        }
        let upscale = self
            .video
            .iter()
            .any(|control| control.feature == Feature::Upscale && control.draft != 0);
        let find = |feature| {
            self.video
                .iter()
                .position(|control| control.feature == feature)
        };
        let mut sections = Vec::new();
        let mut picture = div().flex().flex_col();
        let upscale_index = find(Feature::Upscale);
        if let Some(i) = upscale_index {
            picture = picture.child(self.toggle_row(
                "video-upscale",
                self.hint_label(Feature::Upscale.label()),
                upscale,
                true,
                move |p, _| p.video[i].draft = i32::from(p.video[i].draft == 0),
                cx,
            ));
        }
        if upscale {
            if let Some(i) = find(Feature::Sharpness) {
                picture = picture.child(self.video_number_row(i, cx));
            }
        }
        sections.extend(upscale_index.map(|_| picture));
        sections.extend(find(Feature::Brightness).map(|i| self.video_number_row(i, cx)));
        self.setting_sections(sections, cx)
    }

    fn video_number_row(&self, index: usize, cx: &mut Context<Self>) -> Div {
        let control = &self.video[index];
        let (min, max, _) = control.feature.limits();
        self.setting_row(
            self.hint_label(control.feature.label()),
            self.number(
                super::numeric::Field::Video(index),
                control.draft,
                min,
                max,
                1,
                "",
                cx,
            ),
        )
    }

    pub(super) fn video_actions(&self, cx: &mut Context<Self>) -> Div {
        let mut actions = h_flex().flex_wrap().gap_2();
        let changed = self
            .video
            .iter()
            .any(|control| control.current != control.draft || control.saved != control.draft);
        actions = actions.child(self.primary_button(
            "video-apply",
            "应用",
            changed,
            Some("应用多媒体设置。"),
            |p, _, _| {
                if let Err(e) = p.apply_video_profile(false) {
                    p.video_status = e.to_string();
                }
            },
            cx,
        ));
        if self.confirm_video_reset {
            actions = actions.child(
                h_flex()
                    .gap_2()
                    .child("将这张显卡的多媒体设置恢复为默认值？")
                    .child(self.danger_button(
                        "video-reset-confirm",
                        "确认重置",
                        None,
                        |p, _, _| {
                            p.confirm_video_reset = false;
                            if let Err(err) = p.apply_video_profile(true) {
                                p.video_status = err.to_string();
                            }
                        },
                        cx,
                    ))
                    .child(self.contour_button(
                        "video-reset-cancel",
                        "取消",
                        true,
                        None,
                        |p, _, _| p.confirm_video_reset = false,
                        cx,
                    )),
            );
        } else {
            actions = actions.child(self.contour_button(
                "video-reset",
                "重置",
                true,
                Some("将多媒体设置恢复为默认值。"),
                |p, _, _| p.confirm_video_reset = true,
                cx,
            ));
        }
        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_video_controls_are_per_adapter_and_keep_driver_readings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("MultimediaProfiles.json");
        let mut controls = [VideoControl {
            feature: Feature::Brightness,
            current: -5,
            saved: -5,
            draft: -5,
        }];
        let mut row = video_record("A", "Global", &controls);
        row["VideoBrightness.Current"] = json!(12);
        profile_store::save(&path, "A", "Global", &row).unwrap();
        profile_store::save(
            &path,
            "A",
            "player.exe",
            &json!({"DeviceId":"A","Application":"player.exe","VideoBrightness.Current":20}),
        )
        .unwrap();
        read_profile(&path, "A", "player.exe", &mut controls).unwrap();
        assert_eq!(controls[0].draft, 20);
        read_profile(&path, "A", "Global", &mut controls).unwrap();
        assert_eq!(
            (controls[0].current, controls[0].saved, controls[0].draft),
            (-5, 12, 12)
        );
        assert_eq!(profile_store::load(&path).unwrap().len(), 2);
    }
}
