use super::{Panel, Preview};
use crate::adl::display::custom::{STANDARDS, Timing};
use anyhow::{Context as _, Result, ensure};
use gpui::{Context, Div, div, prelude::*};

pub(super) struct Editor {
    basic: [i32; 3],
    standard: usize,
    interlaced: bool,
    timing: Option<Timing>,
    existing: Vec<Timing>,
    remove: Option<usize>,
    status: String,
}
impl Panel {
    pub(super) fn open_custom(&mut self) -> Result<()> {
        self.dropdown = None;
        let page = &mut self.display;
        let d = page
            .displays
            .get(page.display_index)
            .context("未选择显示器")?;
        let driver = self.driver.as_ref().context("AMD 驱动不可用")?;
        let mode = driver.current_mode(d)?;
        let existing = driver.custom_timings(d)?;
        page.custom = Some(Editor {
            basic: [mode.width, mode.height, mode.refresh.round() as i32],
            standard: 2,
            interlaced: false,
            timing: None,
            existing,
            remove: None,
            status: String::new(),
        });
        self.calculate_custom();
        Ok(())
    }
    fn calculate_custom(&mut self) {
        let page = &mut self.display;
        if let (Some(editor), Some(driver), Some(d)) = (
            &mut page.custom,
            &self.driver,
            page.displays.get(page.display_index),
        ) {
            let [w, h, r] = editor.basic;
            match driver.calculate_timing(d, w, h, r, editor.standard, editor.interlaced) {
                Ok(t) => {
                    editor.timing = Some(t);
                    editor.status.clear();
                }
                Err(e) => {
                    editor.timing = None;
                    editor.status = e.to_string();
                }
            }
        }
    }
    pub(in crate::panel) fn custom_number(&mut self, index: usize, value: i32) {
        let Some(editor) = &mut self.display.custom else {
            return;
        };
        if let Some(basic) = editor.basic.get_mut(index) {
            *basic = value;
            self.calculate_custom();
        } else if let Some(t) = &mut editor.timing {
            let det = &mut t.detail;
            match index {
                3 => det.clock = value,
                4 => det.h.total = value,
                5 => det.v.total = value,
                6 => det.h.start = det.h.active.saturating_add(value),
                7 => det.v.start = det.v.active.saturating_add(value),
                8 => det.h.sync = value,
                9 => det.v.sync = value,
                _ => {}
            }
        }
    }
    fn remove_custom(&mut self, index: usize) -> Result<()> {
        let _write = crate::driver_write::lock()?;
        let page = &mut self.display;
        let driver = self.driver.as_ref().context("驱动不可用")?;
        let d = page
            .displays
            .get(page.display_index)
            .context("显示器不可用")?;
        let editor = page.custom.as_mut().context("自定义分辨率窗口已关闭")?;
        let item = *editor.existing.get(index).context("所选时序已改变")?;
        let connected = driver
            .displays(d.adapter)?
            .into_iter()
            .find(|v| v.logical == d.logical && v.name == d.name)
            .context("显示器已断开")?;
        if let Some(key) = &page.key {
            ensure!(
                driver.display_profile_key(&connected).as_ref() == Some(key),
                "显示器已更换"
            );
        }
        ensure!(
            driver.custom_timings(d)?.contains(&item),
            "所选时序已改变，请重新打开列表"
        );
        driver.delete_timing(d, item)?;
        editor.existing = driver.custom_timings(d)?;
        editor.remove = None;
        editor.status = "已删除自定义分辨率".into();
        Ok(())
    }
    pub(super) fn render_custom(&self, editor: &Editor, cx: &mut Context<Self>) -> Div {
        let mut section = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_xl().child("自定义分辨率"));
        section = section.child(self.setting_row(
            "时序标准",
            self.select(
                "timing-standard",
                STANDARDS.iter().map(|(name, _)| name.to_string()).collect(),
                Some(editor.standard),
                "时序标准",
                160.,
                |p, i, _| {
                    if let Some(e) = &mut p.display.custom {
                        e.standard = i;
                        p.calculate_custom();
                    }
                },
                cx,
            ),
        ));
        section = section.child(self.setting_row(
            "扫描方式",
            self.select(
                "scan-type",
                ["逐行", "隔行"].map(str::to_owned).into(),
                Some(usize::from(editor.interlaced)),
                "扫描方式",
                160.,
                |p, i, _| {
                    if let Some(e) = &mut p.display.custom {
                        e.interlaced = i == 1;
                        p.calculate_custom();
                    }
                },
                cx,
            ),
        ));
        let mut fields = Vec::from(editor.basic);
        if let Some(t) = editor.timing {
            let d = t.detail;
            fields.extend([
                d.clock,
                d.h.total,
                d.v.total,
                d.h.start - d.h.active,
                d.v.start - d.v.active,
                d.h.sync,
                d.v.sync,
            ]);
        }
        let names = [
            "宽度",
            "高度",
            "刷新率",
            "像素时钟",
            "水平总长",
            "垂直总长",
            "水平前沿",
            "垂直前沿",
            "水平同步宽度",
            "垂直同步宽度",
        ];
        for (i, val) in fields.into_iter().enumerate() {
            let unit = match i {
                2 => "Hz",
                3 => "×10 kHz",
                _ => "",
            };
            section = section.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(names[i])
                    .child(self.number(
                        super::super::numeric::Field::Custom(i),
                        val,
                        if matches!(i, 6 | 7) { 0 } else { 1 },
                        i32::MAX,
                        1,
                        unit,
                        cx,
                    )),
            );
        }
        if let Some(t) = editor.timing {
            for (i, (label, bit)) in [("水平同步极性", 4), ("垂直同步极性", 8)]
                .into_iter()
                .enumerate()
            {
                section = section.child(self.setting_row(
                    label,
                    self.select(
                        ("sync-polarity", i),
                        ["正极性", "负极性"].map(str::to_owned).into(),
                        Some(usize::from(t.detail.flags & bit != 0)),
                        label,
                        160.,
                        move |p, value, _| {
                            if let Some(t) =
                                p.display.custom.as_mut().and_then(|e| e.timing.as_mut())
                            {
                                t.detail.flags =
                                    (t.detail.flags & !bit) | if value == 1 { bit } else { 0 };
                            }
                        },
                        cx,
                    ),
                ));
            }
            if let Err(e) = t.validate() {
                section = section.child(e.to_string());
            } else {
                section = section
                    .child(format!("实际刷新率：{:.3} Hz", t.rate()))
                    .child(
                        div()
                            .id("create-custom-mode")
                            .cursor_pointer()
                            .child("创建并预览")
                            .on_click(cx.listener(move |p, _, _, cx| {
                                if let Some(d) = p.display.displays.get(p.display.display_index) {
                                    match Preview::custom(d, p.display.key.as_deref(), t) {
                                        Ok(preview) => p.display.preview = Some(preview),
                                        Err(e) => {
                                            if let Some(editor) = &mut p.display.custom {
                                                editor.status = e.to_string();
                                            }
                                        }
                                    }
                                }
                                cx.notify();
                            })),
                    );
            }
        }
        if !editor.existing.is_empty() {
            section = section.child("已保存的自定义分辨率");
        }
        for (i, t) in editor.existing.iter().enumerate() {
            let mut row = div().flex().gap_2().child(t.label());
            if editor.remove == Some(i) {
                row = row
                    .child(
                        div()
                            .id(("confirm-timing-remove", i))
                            .cursor_pointer()
                            .child("确认删除")
                            .on_click(cx.listener(move |p, _, _, cx| {
                                if let Err(e) = p.remove_custom(i) {
                                    if let Some(editor) = &mut p.display.custom {
                                        editor.status = e.to_string();
                                    }
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .id("cancel-timing-remove")
                            .cursor_pointer()
                            .child("取消")
                            .on_click(cx.listener(|p, _, _, cx| {
                                if let Some(e) = &mut p.display.custom {
                                    e.remove = None;
                                }
                                cx.notify();
                            })),
                    );
            } else {
                row = row.child(
                    div()
                        .id(("remove-timing", i))
                        .cursor_pointer()
                        .child("删除")
                        .on_click(cx.listener(move |p, _, _, cx| {
                            if let Some(e) = &mut p.display.custom {
                                e.remove = Some(i);
                            }
                            cx.notify();
                        })),
                );
            }
            section = section.child(row);
        }
        section.child(editor.status.clone()).child(
            div()
                .id("close-timing")
                .cursor_pointer()
                .child("关闭")
                .on_click(cx.listener(|p, _, _, cx| {
                    p.read_modes();
                    cx.notify();
                })),
        )
    }
}
