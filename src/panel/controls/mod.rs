mod button;
mod card;
mod select;
mod switch;

pub(super) use card::{CardItem, card_rows, section_card};
pub(super) use select::{Dropdown, PROFILE_SELECT_WIDTH, SELECT_MAX_WIDTH, SelectWidth};

use super::{
    Panel,
    layout::nav_icon,
    motion::{pointer_feedback, smooth_value},
    shape::{ContinuousCorners as _, HighlightRing},
};
use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::{
    Animation, AnimationExt as _, App, Context, Div, ElementId, FontWeight, Hsla,
    InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _, PathBuilder, Pixels,
    Role, StatefulInteractiveElement as _, Styled as _, Window, canvas, div, ease_out_quint, point,
    prelude::FluentBuilder as _, px, relative,
};
use std::{rc::Rc, time::Duration};

pub(super) fn sidebar_width(viewport: Pixels) -> Pixels {
    if viewport < px(680.) {
        px(56.)
    } else {
        px(148.)
    }
}

pub(super) fn animated_chevron(
    id: impl Into<ElementId>,
    color: Hsla,
    up: bool,
) -> impl IntoElement {
    let id: ElementId = id.into();
    canvas(
        move |_, window, cx| {
            let duration = cx.theme().motion_tokens().duration_normal;
            smooth_value(
                (id.clone(), "chevron"),
                if up { 1. } else { 0. },
                duration,
                window,
                cx,
            )
        },
        move |bounds, phase, window, _| {
            let center = bounds.center();
            let (sin, cos) = (phase * std::f32::consts::PI).sin_cos();
            let rotate = |x: f32, y: f32| {
                point(
                    center.x + px(x * cos - y * sin),
                    center.y + px(x * sin + y * cos),
                )
            };
            let mut path = PathBuilder::stroke(px(1.35));
            path.move_to(rotate(-3.75, -1.6));
            path.line_to(rotate(0., 1.8));
            path.line_to(rotate(3.75, -1.6));
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(14.))
    .flex_shrink_0()
}

pub(super) fn chevron_right(color: Hsla) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let center = bounds.center();
            let mut path = PathBuilder::stroke(px(1.35));
            path.move_to(point(center.x - px(2.), center.y - px(4.)));
            path.line_to(point(center.x + px(2.), center.y));
            path.line_to(point(center.x - px(2.), center.y + px(4.)));
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .w(px(10.))
    .h(px(14.))
    .flex_shrink_0()
}

pub(super) fn control_highlight(cx: &App) -> HighlightRing {
    if cx.theme().is_dark() {
        HighlightRing {
            top: cx.theme().switch_thumb.opacity(0.16),
            rest: cx.theme().switch_thumb.opacity(0.08),
            stop: 0.28,
        }
    } else {
        HighlightRing {
            top: cx.theme().foreground.opacity(0.08),
            rest: cx.theme().foreground.opacity(0.14),
            stop: 0.82,
        }
    }
}

pub(super) fn accent_highlight(cx: &App) -> HighlightRing {
    HighlightRing {
        top: cx.theme().switch_thumb.opacity(0.24),
        rest: cx.theme().switch_thumb.opacity(0.10),
        stop: 0.28,
    }
}

impl Panel {
    pub(super) fn modal(&self, content: Div, cx: &App) -> impl IntoElement {
        div()
            .id("dialog-veil")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .occlude()
            .p_5()
            .flex()
            .items_center()
            .justify_center()
            .bg(cx.theme().overlay)
            .child(
                v_flex()
                    .id("dialog-body")
                    .relative()
                    .w_full()
                    .max_w(px(720.))
                    .max_h(relative(1.))
                    .p_5()
                    .overflow_y_scroll()
                    .rounded(cx.theme().radius_lg)
                    .contour(
                        Some(cx.theme().popover),
                        None,
                        f32::from(cx.theme().radius_lg),
                    )
                    .shadow_xl()
                    .child(content)
                    .with_animation(
                        "dialog-open",
                        Animation::new(Duration::from_millis(160)).with_easing(ease_out_quint()),
                        |d, t| d.opacity(t).top(px(6. * (1. - t))),
                    ),
            )
    }

    pub(super) fn nav_item(
        &self,
        id: impl Into<ElementId>,
        label: &'static str,
        icon_index: usize,
        compact: bool,
        selected: bool,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id: ElementId = id.into();
        let radius = f32::from(cx.theme().radius);
        let hover_fill = if selected {
            cx.theme().transparent
        } else {
            cx.theme().sidebar_accent.opacity(0.65)
        };
        let press_fill = if selected {
            cx.theme().transparent
        } else {
            cx.theme().sidebar_accent
        };
        let text_color = if selected {
            cx.theme().sidebar_accent_foreground
        } else {
            cx.theme().sidebar_foreground
        };
        let icon_color = if selected {
            cx.theme().primary
        } else {
            cx.theme().muted_foreground
        };

        let on_click = Rc::new(on_click);
        let click_cb = on_click.clone();
        let key_cb = on_click;

        let base = h_flex()
            .id(id.clone())
            .relative()
            .w_full()
            .h_9()
            .flex_shrink_0()
            .items_center()
            .when(compact, |d| d.justify_center())
            .when(!compact, |d| d.justify_start().px_2p5().gap_2p5())
            .focusable()
            .tab_stop(true)
            .role(Role::Button)
            .aria_label(label)
            .aria_selected(selected);

        pointer_feedback((id, "nav-fb"), base, hover_fill, press_fill, radius)
            .child(nav_icon(icon_index, icon_color))
            .when(!compact, |d| {
                d.child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_sm()
                        .font_weight(if selected {
                            FontWeight::MEDIUM
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(text_color)
                        .child(label),
                )
            })
            .when(compact, |d| d.tooltip(self.tooltip(label)))
            .on_click(cx.listener(move |p, _, window, cx| {
                click_cb(p, window, cx);
                cx.notify();
            }))
            .on_key_down(cx.listener(move |p, e: &KeyDownEvent, window, cx| {
                if matches!(e.keystroke.key.as_str(), "enter" | "space") {
                    key_cb(p, window, cx);
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
    }

    pub(super) fn control_text_width(&self, text: &str, cx: &App) -> f32 {
        let text_system = cx.text_system();
        let font = text_system.resolve_font(&gpui_kit::font("Microsoft YaHei UI"));
        let mut advances = self.control_glyph_widths.borrow_mut();
        let sum: f32 = text
            .chars()
            .map(|ch| {
                let measured = *advances.entry(ch).or_insert_with(|| {
                    text_system
                        .advance(font, px(14.), ch)
                        .map_or(14., |size| f32::from(size.width))
                });
                let is_wide = ('\u{2E80}'..='\u{9FFF}').contains(&ch)
                    || ('\u{F900}'..='\u{FAFF}').contains(&ch)
                    || ('\u{FF00}'..='\u{FFEF}').contains(&ch)
                    || ('\u{3000}'..='\u{303F}').contains(&ch);
                if is_wide {
                    measured.max(14.5)
                } else {
                    measured
                }
            })
            .sum();
        sum * 1.06
    }
}
