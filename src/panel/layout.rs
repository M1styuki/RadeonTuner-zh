use super::{PAGES, Panel, shape::ContinuousCorners};
use gpui::{Context, Div, div, prelude::*, rgb};
use std::collections::HashSet;

impl Panel {
    pub(super) fn render_header(&self, cx: &mut Context<Self>) -> Div {
        let mut heading = div().flex().flex_wrap().items_center().gap_3().child(
            div().text_xl().child(PAGES[self.page]),
        );
        if self.page != 5 {
            let mut locations = HashSet::new();
            let visible: Vec<_> = self
                .devices
                .iter()
                .enumerate()
                .filter(|(_, device)| {
                    device
                        .pci_location
                        .is_none_or(|location| locations.insert(location))
                })
                .collect();
            let mut devices = div().flex().flex_wrap().items_center().gap_2();
            for &(i, device) in &visible {
                let selected = self.selected == i;
                let name = if visible.iter().filter(|(_, d)| d.name == device.name).count() > 1 {
                    format!("{} ({})", device.name, device.index)
                } else {
                    device.name.clone()
                };
                let chip = div()
                    .id(("gpu", i))
                    .px_3()
                    .py_1()
                    .smooth_background(if selected {
                        self.palette.selected
                    } else {
                        self.palette.control
                    })
                    .child(name);
                devices = devices.child(if selected {
                    chip
                } else {
                    chip.cursor_pointer().on_click(cx.listener(move |panel, _, _, cx| {
                        panel.finish_preview(false);
                        panel.selected = i;
                        panel.reset_tuning_editor();
                        panel.video_app = "Global".into();
                        panel.read_device_details();
                        panel.read_displays();
                        panel.read_controls();
                        panel.refresh();
                        cx.notify();
                    }))
                });
            }
            heading = heading.child(devices);
        }
        let mut header = div().flex_shrink_0().flex().flex_col().gap_2().p_4().child(heading);
        if matches!(self.page, 0 | 1 | 2 | 6) && !self.metrics.is_empty() {
            let mut stats = div()
                .flex()
                .flex_wrap()
                .gap_x_3()
                .gap_y_1()
                .px_3()
                .py_2()
                .text_sm()
                .smooth_background(self.palette.sidebar);
            for item in &self.metrics {
                if let Some(value) = item.value {
                    stats = stats.child(
                        div().flex_shrink_0().child(format!("{} {value} {}", item.label, item.unit)),
                    );
                }
            }
            header = header.child(stats);
        }
        header.text_color(rgb(self.palette.text))
    }
}
