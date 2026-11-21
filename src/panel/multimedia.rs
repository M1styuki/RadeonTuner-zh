use super::Panel;
use crate::{
    adl::multimedia::{Feature, VideoControl},
    profile_store,
};
use anyhow::{Context as _, Result, ensure};
use serde_json::{Value, json};
use std::path::Path;

fn video_record(device: &str, controls: &[VideoControl]) -> Value {
    let mut row = json!({"DeviceId":device,"Application":"Global"});
    for c in controls {
        row[c.feature.profile_key()] = json!(c.draft);
    }
    row
}
fn read_profile(path: &Path, device: &str, controls: &mut [VideoControl]) -> Result<()> {
    if device.is_empty() || controls.is_empty() {
        return Ok(());
    }
    let rows = profile_store::load(path)?;
    let Some(row) = rows
        .iter()
        .find(|row| profile_store::matches(row, device, "Global"))
    else {
        return profile_store::save(path, device, "Global", &video_record(device, controls));
    };
    for c in controls {
        let value = &row[c.feature.profile_key()];
        if value.is_null() {
            continue;
        }
        let integer = value
            .as_i64()
            .or_else(|| value.as_bool().map(i64::from))
            .context("视频配置数值无效")?;
        let (min, max, _) = c.feature.limits();
        ensure!(
            (i64::from(min)..=i64::from(max)).contains(&integer),
            "{} 超出范围",
            c.feature.label()
        );
        c.saved = integer as i32;
        c.draft = c.saved;
    }
    Ok(())
}

use gpui::{Context, Div, div, prelude::*, rgb};

impl Panel {
    pub(super) fn read_video(&mut self) {
        self.confirm_video_reset = false;
        self.video.clear();
        self.video_status.clear();
        if let (Some(driver), Some(device)) = (&self.driver, self.devices.get(self.selected)) {
            match driver.video_controls(device.index) {
                Ok(mut controls) => {
                    let result = self
                        .settings
                        .profiles_path("MultimediaProfiles.json")
                        .and_then(|path| read_profile(&path, &device.profile_key, &mut controls));
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
        let driver = self.driver.as_ref().context("AMD 驱动不可用")?;
        let device = self.devices.get(self.selected).context("未选择显卡")?;
        if reset {
            self.video = driver.video_controls(device.index)?;
            for c in &mut self.video {
                c.draft = c.feature.limits().2;
            }
        }
        if !device.profile_key.is_empty() {
            let path = self.settings.profiles_path("MultimediaProfiles.json")?;
            profile_store::save(
                &path,
                &device.profile_key,
                "Global",
                &video_record(&device.profile_key, &self.video),
            )?;
        }
        let result = driver.apply_video(device.index, &self.video);
        self.read_video();
        result?;
        if self.video_status.is_empty() {
            self.video_status = if reset {
                "多媒体设置已重置"
            } else {
                "多媒体设置已应用"
            }
            .into();
        }
        Ok(())
    }
    pub(super) fn render_video(&self, cx: &mut Context<Self>) -> Div {
        let mut section = div().flex().flex_col().gap_2().child("全局配置");
        if self.video.is_empty() {
            return section.child(if self.video_status.is_empty() {
                "当前设备无可用的视频设置".to_string()
            } else {
                self.video_status.clone()
            });
        }
        let upscale = self
            .video
            .iter()
            .any(|control| control.feature == Feature::Upscale && control.draft != 0);
        for (i, control) in self.video.iter().enumerate() {
            let mut row = div()
                .flex()
                .items_center()
                .gap_3()
                .child(div().flex_1().child(control.feature.label()));
            if control.feature == Feature::Upscale {
                row = row.child(
                    div()
                        .id("video-upscale")
                        .cursor_pointer()
                        .px_3()
                        .bg(rgb(self.palette.control))
                        .child(if control.draft == 0 { "关" } else { "开" })
                        .on_click(cx.listener(move |panel, _, _, cx| {
                            panel.video[i].draft = i32::from(panel.video[i].draft == 0);
                            cx.notify();
                        })),
                );
            } else {
                let (min, max, _) = control.feature.limits();
                if control.feature != Feature::Sharpness || upscale {
                    row = row.child(self.number(
                        super::numeric::Field::Video(i),
                        control.draft,
                        min,
                        max,
                        1,
                        "",
                        cx,
                    ));
                } else {
                    row = row.child(control.draft.to_string());
                }
                for (key, label, delta) in [("video-down", "−", -1), ("video-up", "+", 1)] {
                    let enabled = (control.feature != Feature::Sharpness || upscale)
                        && (min..=max).contains(&(control.draft.saturating_add(delta)));
                    let button = div()
                        .id((key, i))
                        .px_3()
                        .bg(rgb(self.palette.control))
                        .child(label);
                    row = row.child(if enabled {
                        button
                            .cursor_pointer()
                            .on_click(cx.listener(move |panel, _, _, cx| {
                                panel.video[i].draft =
                                    panel.video[i].draft.saturating_add(delta).clamp(min, max);
                                cx.notify();
                            }))
                    } else {
                        button.opacity(0.45)
                    });
                }
            }
            section = section.child(row);
        }
        if self.confirm_video_reset {
            section = section.child(
                div()
                    .flex()
                    .gap_2()
                    .child("将这张显卡的多媒体设置恢复为默认值？")
                    .child(
                        div()
                            .id("video-reset-confirm")
                            .px_2()
                            .cursor_pointer()
                            .child("重置")
                            .on_click(cx.listener(|p, _, _, cx| {
                                p.confirm_video_reset = false;
                                if let Err(err) = p.apply_video_profile(true) {
                                    p.video_status = err.to_string();
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .id("video-reset-cancel")
                            .px_2()
                            .cursor_pointer()
                            .child("取消")
                            .on_click(cx.listener(|p, _, _, cx| {
                                p.confirm_video_reset = false;
                                cx.notify();
                            })),
                    ),
            );
        } else {
            section = section.child(
                div()
                    .id("video-reset")
                    .cursor_pointer()
                    .child("重置多媒体设置")
                    .on_click(cx.listener(|p, _, _, cx| {
                        p.confirm_video_reset = true;
                        cx.notify();
                    })),
            );
        }
        if self
            .video
            .iter()
            .any(|control| control.current != control.draft || control.saved != control.draft)
        {
            section = section.child(
                div()
                    .id("video-apply")
                    .cursor_pointer()
                    .child("应用多媒体设置")
                    .on_click(cx.listener(|panel, _, _, cx| {
                        if let Err(err) = panel.apply_video_profile(false) {
                            panel.video_status = err.to_string();
                        }
                        cx.notify();
                    })),
            );
        }
        section.child(self.video_status.clone())
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
        read_profile(&path, "A", &mut controls).unwrap();
        let mut row = video_record("A", &controls);
        row["VideoBrightness.Current"] = json!(12);
        profile_store::save(&path, "A", "Global", &row).unwrap();
        profile_store::save(
            &path,
            "A",
            "player.exe",
            &json!({"DeviceId":"A","Application":"player.exe","VideoBrightness.Current":20}),
        )
        .unwrap();
        read_profile(&path, "B", &mut controls).unwrap();
        read_profile(&path, "A", &mut controls).unwrap();
        assert_eq!(
            (controls[0].current, controls[0].saved, controls[0].draft),
            (-5, 12, 12)
        );
        assert_eq!(profile_store::load(&path).unwrap().len(), 3);
    }
}
