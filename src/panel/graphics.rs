use super::Panel;
use gpui::{Context, Div, div, prelude::*};
impl Panel {
    pub(super) fn render_graphics(&self, cx: &mut Context<Self>) -> Div {
        let mut section = div().flex().flex_col().gap_3();
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
        if self.graphics.is_empty() {
            section = section.child("当前显卡没有可读取的图形设置");
        }
        if self.graphics.iter().any(|s| s.draft != s.current) {
            section = section.child(
                div()
                    .id("graphics-apply")
                    .cursor_pointer()
                    .child("应用图形设置")
                    .on_click(cx.listener(|p, _, _, cx| {
                        if let (Some(driver), Some(device)) = (&p.driver, p.devices.get(p.selected))
                        {
                            let result = driver.apply_graphics(device, &p.graphics);
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
        self.info.clear();
        self.graphics_status.clear();
        if let (Some(driver), Some(device)) = (&self.driver, self.devices.get(self.selected)) {
            self.graphics = driver.graphics(device, self.skip_graphics_check);
            self.info = driver.information(device);
        }
    }
}
