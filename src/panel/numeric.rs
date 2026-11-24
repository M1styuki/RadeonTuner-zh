use super::Panel;
use crate::panel::shape::ContinuousCorners;
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
            self.set_number(edit.field, value);
        }
    }
    fn set_number(&mut self, field: Field, value: i32) {
        match field {
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
            Field::Frame(i) => self.change_frame(i, value),
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
    ) -> Div {
        let text = self.number_text(field, value, min, max, step, unit, cx);
        let mut input = div().flex().items_center().gap_2();
        if matches!(field, Field::Custom(_)) || max <= min {
            return input.child(text);
        }
        let position = move |fraction: f32| {
            let range = i64::from(max) - i64::from(min);
            let step = i64::from(step.max(1));
            (i64::from(min)
                + ((fraction.clamp(0., 1.) as f64 * range as f64 / step as f64).round() as i64)
                    * step)
                .clamp(i64::from(min), i64::from(max)) as i32
        };
        let slot = std::rc::Rc::new(std::cell::Cell::new(None::<gpui::Bounds<gpui::Pixels>>));
        let geometry = slot.clone();
        let palette = self.palette;
        let entity = cx.entity();
        let track = gpui::canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                geometry.set(Some(bounds));
                let ratio =
                    ((value as f64 - min as f64) / (max as f64 - min as f64)).clamp(0., 1.) as f32;
                let x = bounds.origin.x + bounds.size.width * ratio;
                let y = bounds.origin.y + bounds.size.height * 0.5;
                for (from, to, width, color) in [
                    (
                        gpui::point(bounds.origin.x, y),
                        gpui::point(bounds.right(), y),
                        3.,
                        palette.grid,
                    ),
                    (
                        gpui::point(bounds.origin.x, y),
                        gpui::point(x, y),
                        3.,
                        palette.accent,
                    ),
                    (
                        gpui::point(x, y - px(4.)),
                        gpui::point(x, y + px(4.)),
                        5.,
                        palette.accent,
                    ),
                ] {
                    let mut line = gpui::PathBuilder::stroke(px(width));
                    line.move_to(from);
                    line.line_to(to);
                    if let Ok(path) = line.build() {
                        window.paint_path(path, rgb(color));
                    }
                }
                let dragged = entity.clone();
                window.on_mouse_event(move |ev: &gpui::MouseMoveEvent, phase, _, cx| {
                    if !phase.bubble() || !ev.dragging() || dragged.read(cx).slider != Some(field) {
                        return;
                    }
                    let v = position((ev.position.x - bounds.origin.x) / bounds.size.width);
                    dragged.update(cx, |p, cx| {
                        p.set_number(field, v);
                        cx.notify();
                    });
                });
                window.on_mouse_event(move |ev: &gpui::MouseUpEvent, _, _, cx| {
                    if ev.button == gpui::MouseButton::Left && entity.read(cx).slider == Some(field)
                    {
                        entity.update(cx, |p, _| p.slider = None);
                    }
                });
            },
        )
        .size_full();
        input = input.child(
            div()
                .w(px(120.))
                .h(px(20.))
                .smooth_hover_background(None, palette.hover, super::shape::CONTROL_CORNER)
                .cursor_pointer()
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |p, ev: &gpui::MouseDownEvent, _, cx| {
                        if let Some(bounds) = slot.get() {
                            p.finish_edit(true);
                            p.slider = Some(field);
                            p.set_number(
                                field,
                                position((ev.position.x - bounds.origin.x) / bounds.size.width),
                            );
                            cx.notify();
                        }
                    }),
                )
                .child(track),
        );
        input.child(text)
    }
    fn number_text(
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
        let palette = self.palette;
        let edit = self.edit.as_ref().filter(|e| e.field == field);
        let base = div()
            .id(id)
            .w(px(160.))
            .flex_shrink_0()
            .h(px(32.))
            .flex()
            .justify_center()
            .items_center()
            .relative()
            .when(edit.is_none(), |input| {
                input.smooth_hover_background(
                    Some(palette.control),
                    palette.hover,
                    super::shape::CONTROL_CORNER,
                )
            })
            .when(edit.is_some(), |input| {
                input.child(super::shape::surface_radius(
                    palette.control,
                    super::shape::CONTROL_CORNER,
                ))
            })
            .child(super::shape::outline_radius(
                if edit.is_some() {
                    palette.accent
                } else {
                    palette.border
                },
                super::shape::CONTROL_CORNER,
            ));
        if let Some(edit) = edit {
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
