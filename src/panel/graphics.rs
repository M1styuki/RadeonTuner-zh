use super::Panel;
use gpui::{Context, Div, div, prelude::*};
impl Panel {
    pub(super) fn render_graphics(&self, cx: &mut Context<Self>) -> Div {
        let mut section = div().flex().flex_col().gap_3();
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
                        }
                        cx.notify();
                    })),
            );
            section = section.child(row);
            if self.experimental {
                section = section.child(fsr.path().display().to_string());
            }
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
                            cx.notify();
                        }))
                } else {
                    button.opacity(0.45)
                });
            }
            section = section.child(row);
        }
        if self.graphics.is_empty() && self.fsr_library.is_none() {
            section = section.child("当前显卡没有可读取的图形设置");
        }
        if self.graphics.iter().any(|s| s.draft != s.current)
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
                        if let (Some(driver), Some(device)) = (&p.driver, p.devices.get(p.selected))
                        {
                            let result =
                                driver
                                    .apply_graphics(device, &p.graphics)
                                    .and_then(|()| match &p.fsr_library {
                                        Some(f) => driver.apply_fsr_override(device, f),
                                        None => Ok(()),
                                    });
                            p.fsr_library = driver.fsr_library(device, p.skip_graphics_check);
                            p.graphics = driver.graphics(device, p.skip_graphics_check);
                            p.graphics_status = match result {
                                Ok(()) => "图形设置已应用".into(),
                                Err(err) => err.to_string(),
                            };
                        }
                        cx.notify();
                    })),
            );
        }
        section.child(self.graphics_status.clone())
    }
    pub(super) fn read_device_details(&mut self) {
        self.read_video();
        self.graphics.clear();
        self.fsr_library = None;
        self.info.clear();
        self.graphics_status.clear();
        if let (Some(driver), Some(device)) = (&self.driver, self.devices.get(self.selected)) {
            self.graphics = driver.graphics(device, self.skip_graphics_check);
            self.fsr_library = driver.fsr_library(device, self.skip_graphics_check);
            self.info = driver.information(device);
        }
    }
}
