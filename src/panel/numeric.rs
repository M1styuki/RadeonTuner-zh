use super::Panel;
use crate::panel::shape::ContinuousCorners;
use gpui::{
    Bounds, Context, Div, FocusHandle, KeyDownEvent, Pixels, Size, Stateful, Subscription, div,
    fill, point, prelude::*, px, rgb, size,
};

const THUMB_RADIUS: f32 = 7.;
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
        let input = div().flex().items_center().gap_2();
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
                let inset = px(THUMB_RADIUS);
                let left = bounds.origin.x + inset;
                let width = (bounds.size.width - inset * 2.).max(px(1.));
                let x = left + width * ratio;
                let y = bounds.origin.y + bounds.size.height * 0.5;
                let rail = |length, color| {
                    fill(
                        Bounds::new(point(left, y - px(2.)), size(length, px(4.))),
                        rgb(color),
                    )
                    .corner_radii(px(2.))
                };
                window.paint_quad(rail(width, palette.grid));
                if x > left {
                    window.paint_quad(rail(x - left, palette.accent));
                }
                window.paint_quad(
                    fill(
                        Bounds::new(point(x - inset, y - inset), size(inset * 2., inset * 2.)),
                        rgb(palette.accent),
                    )
                    .corner_radii(inset),
                );
                let dragged = entity.clone();
                window.on_mouse_event(move |ev: &gpui::MouseMoveEvent, phase, _, cx| {
                    if !phase.bubble() || !ev.dragging() || dragged.read(cx).slider != Some(field) {
                        return;
                    }
                    let v = position(slider_fraction(bounds, ev.position.x));
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
        input
            .child(
                div()
                    .flex_1()
                    .min_w(px(20.))
                    .h(px(SLIDER_HEIGHT))
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
                                    position(slider_fraction(bounds, ev.position.x)),
                                );
                                cx.notify();
                            }
                        }),
                    )
                    .child(track),
            )
            .child(text)
    }
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
            .w(input_size.width)
            .h(input_size.height)
            .px_1()
            .flex_shrink_0()
            .flex()
            .justify_center()
            .items_center()
            .whitespace_nowrap()
            .overflow_hidden()
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

fn format_number(field: Field, value: i32) -> String {
    if matches!(field, Field::Gamma(_)) {
        format!("{:.2}", value as f64 / 100.)
    } else {
        value.to_string()
    }
}
