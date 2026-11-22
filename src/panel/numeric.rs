use super::Panel;
use gpui::{
    Context, Div, FocusHandle, KeyDownEvent, Stateful, Subscription, div, prelude::*, px, rgb,
};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Field {
    Tuning(usize),
    Video(usize),
    Display(usize),
    Custom(usize),
    Gamma(usize),
    Frame(usize),
}

pub(super) struct Edit {
    field: Field,
    text: String,
    anchor: usize,
    cursor: usize,
    min: i32,
    max: i32,
    step: i32,
    focus: FocusHandle,
    _blur: Subscription,
}
impl Edit {
    fn replace(&mut self, mut insert: &str) {
        insert = insert.trim();
        let a = self.anchor.min(self.cursor);
        let b = self.anchor.max(self.cursor);
        let mut text = self.text.clone();
        text.replace_range(a..b, insert);
        if text.len() > 12
            || !(matches!(self.field, Field::Gamma(_))
                && text.bytes().all(|c| c.is_ascii_digit() || c == b'.')
                && text.matches('.').count() <= 1)
                && !text
                    .strip_prefix('-')
                    .unwrap_or(&text)
                    .bytes()
                    .all(|c| c.is_ascii_digit())
        {
            return;
        }
        self.cursor = a + insert.len();
        self.anchor = self.cursor;
        self.text = text;
    }
    fn value(&self) -> Option<i32> {
        if matches!(self.field, Field::Gamma(_)) {
            return self
                .text
                .parse::<f64>()
                .ok()
                .map(|v| (v * 100.0).round().clamp(self.min as f64, self.max as f64) as i32);
        }
        let value = self
            .text
            .parse::<i64>()
            .ok()?
            .clamp(self.min as i64, self.max as i64);
        let step = self.step.max(1) as i64;
        let snapped = self.min as i64 + ((value - self.min as i64 + step / 2) / step) * step;
        Some(snapped.clamp(self.min as i64, self.max as i64) as i32)
    }
}
impl Panel {
    fn finish_edit(&mut self, commit: bool) {
        let Some(edit) = self.edit.take() else {
            return;
        };
        if !commit {
            return;
        }
        if let Some(value) = edit.value() {
            match edit.field {
                Field::Tuning(i) => {
                    if let Some(c) = self.controls.get_mut(i) {
                        c.draft = value;
                    }
                }
                Field::Video(i) => {
                    if let Some(c) = self.video.get_mut(i) {
                        c.draft = value;
                    }
                }
                Field::Gamma(i) => self.set_gamma(i, value),
                Field::Custom(i) => self.custom_number(i, value),
                Field::Display(i) => self.set_display_draft(i, value),
                Field::Frame(i) => {
                    if let Some(c) = self.frames.get_mut(i) {
                        c.draft = value;
                        self.graphics_dirty = true;
                    }
                }
            }
        }
    }
    pub(super) fn number(
        &self,
        field: Field,
        value: i32,
        min: i32,
        max: i32,
        step: i32,
        unit: &'static str,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let id = match field {
            Field::Tuning(i) => ("tuning-number", i),
            Field::Video(i) => ("video-number", i),
            Field::Display(i) => ("display-number", i),
            Field::Custom(i) => ("custom-number", i),
            Field::Gamma(i) => ("gamma-number", i),
            Field::Frame(i) => ("frame-number", i),
        };
        let base = div()
            .id(id)
            .w(px(160.))
            .h(px(32.))
            .flex()
            .justify_center()
            .items_center()
            .bg(rgb(self.palette.control))
            .rounded_md();
        if let Some(edit) = self.edit.as_ref().filter(|e| e.field == field) {
            let (l, r) = (edit.anchor.min(edit.cursor), edit.anchor.max(edit.cursor));
            return base
                .track_focus(&edit.focus)
                .child(edit.text[..l].to_owned())
                .child(
                    div()
                        .bg(rgb(self.palette.selection))
                        .child(edit.text[l..r].to_owned()),
                )
                .when(l == r, |d| {
                    d.child(div().w(px(1.)).h(px(17.)).bg(rgb(self.palette.text)))
                })
                .child(edit.text[r..].to_owned())
                .on_mouse_down_out(cx.listener(|p, _, _, cx| {
                    p.finish_edit(true);
                    cx.notify();
                }))
                .on_key_down(cx.listener(|p, e: &KeyDownEvent, _, cx| {
                    let Some(edit) = p.edit.as_mut() else {
                        return;
                    };
                    let k = &e.keystroke;
                    match k.key.as_str() {
                        "enter" => p.finish_edit(true),
                        "escape" => p.finish_edit(false),
                        "a" if k.modifiers.control => {
                            edit.anchor = 0;
                            edit.cursor = edit.text.len();
                        }
                        "v" if k.modifiers.control => {
                            if let Some(text) = cx.read_from_clipboard().and_then(|c| c.text()) {
                                edit.replace(&text);
                            }
                        }
                        "backspace" | "delete" => {
                            if edit.anchor == edit.cursor {
                                edit.anchor = if k.key == "backspace" {
                                    edit.cursor.saturating_sub(1)
                                } else {
                                    (edit.cursor + 1).min(edit.text.len())
                                };
                            }
                            edit.replace("");
                        }
                        "left" | "right" | "home" | "end" => {
                            edit.cursor = match k.key.as_str() {
                                "left" => edit.cursor.saturating_sub(1),
                                "right" => (edit.cursor + 1).min(edit.text.len()),
                                "home" => 0,
                                _ => edit.text.len(),
                            };
                            if !k.modifiers.shift {
                                edit.anchor = edit.cursor;
                            }
                        }
                        _ => {
                            if k.modifiers.control {
                                return;
                            }
                            let text = k.key_char.as_deref().unwrap_or(&k.key);
                            if text.len() != 1 {
                                return;
                            }
                            edit.replace(text);
                        }
                    }
                    cx.stop_propagation();
                    cx.notify();
                }));
        }
        let formatted = if matches!(field, Field::Gamma(_)) {
            format!("{:.2}", value as f64 / 100.)
        } else {
            value.to_string()
        };
        base.cursor_text()
            .child(format!("{formatted} {unit}").trim_end().to_owned())
            .on_click(cx.listener(move |p, _, window, cx| {
                let focus = cx.focus_handle();
                let blur = cx.on_blur(&focus, window, move |p, _, cx| {
                    if p.edit.as_ref().is_some_and(|e| e.field == field) {
                        p.finish_edit(true);
                        cx.notify();
                    }
                });
                focus.focus(window);
                let text = formatted.clone();
                let cursor = text.len();
                p.edit = Some(Edit {
                    field,
                    text,
                    cursor,
                    anchor: 0,
                    min,
                    max,
                    step,
                    focus,
                    _blur: blur,
                });
                cx.notify();
            }))
    }
}
