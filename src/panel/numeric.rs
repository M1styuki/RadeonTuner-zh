use super::{
    Panel,
    motion::{pointer_feedback, smooth_value},
    shape::{ContinuousCorners as _, paint_outline},
};
use gpui_kit::component::{ActiveTheme as _, h_flex};
use gpui_kit::{
    Bounds, Context, Div, ElementId, FocusHandle, InteractiveElement as _, KeyDownEvent,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement as _, Pixels, Size,
    Stateful, StatefulInteractiveElement as _, Styled as _, Subscription, canvas, div, fill,
    point, prelude::FluentBuilder as _, px, size,
};
use std::{cell::Cell, rc::Rc, time::Duration};

const THUMB_RADIUS: f32 = 8.;
const SLIDER_HEIGHT: f32 = 28.;

fn slider_fraction(bounds: Bounds<Pixels>, x: Pixels) -> f32 {
    let inset = px(THUMB_RADIUS);
    let travel = (bounds.size.width - inset * 2.).max(px(1.));
    (x - bounds.origin.x - inset) / travel
}

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
            Field::Tuning(i) => self.set_tuning_number(i, value),
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
        let compact = matches!(field, Field::Tuning(i) if self.is_fan_point(i));
        let has_slider = !matches!(field, Field::Custom(_)) && max > min;
        let width = match (field, unit) {
            (Field::Custom(_), "×10 kHz") => 130.,
            (Field::Custom(_), _) => 98.,
            _ => {
                let widest = [min, max, value]
                    .into_iter()
                    .map(|n| {
                        let reading = format!("{} {unit}", format_number(field, n));
                        self.control_text_width(reading.trim_end(), cx)
                    })
                    .fold(0., f32::max);
                let minimum_width = match field {
                    Field::Tuning(_) if self.page == 0 => 76.,
                    Field::Tuning(_) => 56.,
                    Field::Frame(_) => 64.,
                    Field::Display(_) | Field::Gamma(_) => 52.,
                    _ => 48.,
                };
                (((widest + 12.) / 4.).ceil() * 4.).max(minimum_width)
            }
        };
        let height = if has_slider { SLIDER_HEIGHT } else { 32. };
        let text = self.number_text(
            field,
            value,
            min,
            max,
            step,
            unit,
            size(px(width), px(height)),
            cx,
        );
        let input = h_flex().gap_2();
        if !has_slider {
            return input.child(text);
        }
        let input = if compact {
            input.w_full().min_w_0()
        } else {
            let rail = (width * 4.).min(240.);
            input.w(px(rail + 2. * THUMB_RADIUS + 8. + width))
        };
        let position = move |fraction: f32| {
            let range = i64::from(max) - i64::from(min);
            let step = i64::from(step.max(1));
            (i64::from(min)
                + ((fraction.clamp(0., 1.) as f64 * range as f64 / step as f64).round() as i64)
                    * step)
                .clamp(i64::from(min), i64::from(max)) as i32
        };
        let slot = Rc::new(Cell::new(None::<Bounds<Pixels>>));
        let geometry = slot.clone();
        let slider_bar = cx.theme().slider_bar;
        let slider_track = slider_bar.opacity(0.22);
        let slider_thumb = cx.theme().slider_thumb;
        let thumb_ring = slider_bar.opacity(0.65);
        let entity = cx.entity();
        let slider_id = field_id(field);
        let track = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                geometry.set(Some(bounds));
                let ratio =
                    ((value as f64 - min as f64) / (max as f64 - min as f64)).clamp(0., 1.) as f32;
                let inset = px(THUMB_RADIUS);
                let inner_inset = px(THUMB_RADIUS - 2.5);
                let left = bounds.origin.x + inset;
                let width = (bounds.size.width - inset * 2.).max(px(1.));
                let x = left + width * ratio;
                let y = bounds.origin.y + bounds.size.height * 0.5;
                let rail = |length, color| {
                    fill(
                        Bounds::new(point(left, y - px(3.)), size(length, px(6.))),
                        color,
                    )
                    .corner_radii(px(3.))
                };
                window.paint_quad(rail(width, slider_track));
                if x > left {
                    window.paint_quad(rail(x - left, slider_bar));
                }
                window.paint_quad(
                    fill(
                        Bounds::new(point(x - inset, y - inset), size(inset * 2., inset * 2.)),
                        thumb_ring,
                    )
                    .corner_radii(inset),
                );
                window.paint_quad(
                    fill(
                        Bounds::new(
                            point(x - inner_inset, y - inner_inset),
                            size(inner_inset * 2., inner_inset * 2.),
                        ),
                        slider_thumb,
                    )
                    .corner_radii(inner_inset),
                );
                let dragged = entity.clone();
                window.on_mouse_event(move |ev: &MouseMoveEvent, phase, _, cx| {
                    if !phase.bubble() || !ev.dragging() || dragged.read(cx).slider != Some(field) {
                        return;
                    }
                    let v = position(slider_fraction(bounds, ev.position.x));
                    dragged.update(cx, |p, cx| {
                        p.set_number(field, v);
                        cx.notify();
                    });
                });
                window.on_mouse_event(move |ev: &MouseUpEvent, _, _, cx| {
                    if ev.button == MouseButton::Left && entity.read(cx).slider == Some(field) {
                        entity.update(cx, |p, _| p.slider = None);
                    }
                });
            },
        )
        .size_full();

        let track_wrap = pointer_feedback(
            (slider_id, "slider-fb"),
            div()
                .id(("slider-track", slider_index(field)))
                .relative()
                .flex_1()
                .min_w(px(20.))
                .h(px(SLIDER_HEIGHT)),
            cx.theme().accent.opacity(0.55),
            cx.theme().accent,
            f32::from(cx.theme().radius),
        )
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |p, ev: &MouseDownEvent, _, cx| {
                if let Some(bounds) = slot.get() {
                    p.finish_edit(true);
                    p.slider = Some(field);
                    p.set_number(field, position(slider_fraction(bounds, ev.position.x)));
                    cx.notify();
                }
            }),
        )
        .child(track);

        input.child(track_wrap).child(text)
    }

    #[allow(clippy::too_many_arguments)]
    fn number_text(
        &self,
        field: Field,
        value: i32,
        min: i32,
        max: i32,
        step: i32,
        unit: &'static str,
        input_size: Size<Pixels>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let id = field_id(field);
        let edit = self.edit.as_ref().filter(|e| e.field == field);
        let is_editing = edit.is_some();
        let radius = f32::from(cx.theme().radius);
        let rest_border = cx.theme().input;
        let focus_border = cx.theme().ring;
        let focus_id = id.clone();

        let mut base = h_flex()
            .id(id.clone())
            .relative()
            .w(input_size.width)
            .h(input_size.height)
            .px_1p5()
            .flex_shrink_0()
            .justify_center()
            .whitespace_nowrap()
            .overflow_hidden()
            .text_sm()
            .rounded(cx.theme().radius)
            .contour(Some(cx.theme().input_background()), None, radius)
            .child(
                canvas(
                    move |_, window, cx| {
                        smooth_value(
                            (focus_id.clone(), "num-focus"),
                            if is_editing { 1. } else { 0. },
                            Duration::from_millis(130),
                            window,
                            cx,
                        )
                    },
                    move |bounds, amount, window, _| {
                        paint_outline(bounds, rest_border.opacity(1. - amount), radius, window);
                        paint_outline(bounds, focus_border.opacity(amount), radius, window);
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            );
        if !is_editing {
            base = pointer_feedback(
                (id, "num-fb"),
                base,
                cx.theme().secondary_hover,
                cx.theme().secondary_active,
                radius,
            );
        }
        if let Some(edit) = edit {
            let (l, r) = (edit.anchor.min(edit.cursor), edit.anchor.max(edit.cursor));
            return base
                .track_focus(&edit.focus)
                .child(edit.text[..l].to_owned())
                .child(
                    div()
                        .bg(cx.theme().selection)
                        .child(edit.text[l..r].to_owned()),
                )
                .when(l == r, |d| {
                    d.child(div().w(px(1.)).h(px(17.)).bg(cx.theme().caret))
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
        let formatted = format_number(field, value);
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
                focus.focus(window, cx);
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

fn field_id(field: Field) -> ElementId {
    match field {
        Field::Tuning(i) => ("tuning-number", i).into(),
        Field::Video(i) => ("video-number", i).into(),
        Field::Display(i) => ("display-number", i).into(),
        Field::Custom(i) => ("custom-number", i).into(),
        Field::Gamma(i) => ("gamma-number", i).into(),
        Field::Frame(i) => ("frame-number", i).into(),
    }
}

fn slider_index(field: Field) -> usize {
    match field {
        Field::Tuning(i) => i,
        Field::Video(i) => 1_000 + i,
        Field::Display(i) => 2_000 + i,
        Field::Custom(i) => 3_000 + i,
        Field::Gamma(i) => 4_000 + i,
        Field::Frame(i) => 5_000 + i,
    }
}

fn format_number(field: Field, value: i32) -> String {
    if matches!(field, Field::Gamma(_)) {
        format!("{:.2}", value as f64 / 100.)
    } else {
        value.to_string()
    }
}
