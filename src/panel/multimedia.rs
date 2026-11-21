use super::Panel;
use crate::adl::multimedia::Feature;
use gpui::{Context, Div, div, prelude::*, rgb};

impl Panel {
    pub(super) fn read_video(&mut self) {
        self.video.clear();
        self.video_status.clear();
        if let (Some(driver), Some(device)) = (&self.driver, self.devices.get(self.selected)) {
            match driver.video_controls(device.index) {
                Ok(controls) => self.video = controls,
                Err(err) => self.video_status = format!("无法读取视频设置：{err}"),
            }
        }
    }

    pub(super) fn render_video(&self, cx: &mut Context<Self>) -> Div {
        let mut section = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_xl().child("多媒体"));
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
        section = section.child(
            div()
                .id("video-default")
                .cursor_pointer()
                .child("载入默认值")
                .on_click(cx.listener(|panel, _, _, cx| {
                    for control in &mut panel.video {
                        control.draft = control.feature.limits().2;
                    }
                    cx.notify();
                })),
        );
        if self
            .video
            .iter()
            .any(|control| control.current != control.draft)
        {
            section = section.child(
                div()
                    .id("video-apply")
                    .cursor_pointer()
                    .child("应用多媒体设置")
                    .on_click(cx.listener(|panel, _, _, cx| {
                        if let (Some(driver), Some(device)) =
                            (&panel.driver, panel.devices.get(panel.selected))
                        {
                            match driver.apply_video(device.index, &panel.video) {
                                Ok(()) => panel.read_video(),
                                Err(err) => panel.video_status = err.to_string(),
                            }
                        }
                        cx.notify();
                    })),
            );
        }
        section.child(self.video_status.clone())
    }
}
