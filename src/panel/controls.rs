use super::{
    Panel, motion,
    shape::{self, ContinuousCorners},
};
use gpui::{
    App, Bounds, Context, Corner, Div, ElementId, IntoElement, KeyDownEvent, PathBuilder, Pixels,
    Window, anchored, canvas, deferred, div, point, prelude::*, px, rgb,
};
use std::{cell::Cell, rc::Rc};

pub(super) const SELECT_MAX_WIDTH: f32 = 216.;
pub(super) const PROFILE_SELECT_WIDTH: SelectWidth = SelectWidth::Fixed(164.);

pub(super) enum SelectWidth {
    Fit(f32),
    Fixed(f32),
}

impl From<f32> for SelectWidth {
    fn from(max: f32) -> Self {
        Self::Fit(max)
    }
}

pub(super) struct Dropdown {
    id: ElementId,
    keyboard_choice: Option<usize>,
    trigger: Bounds<Pixels>,
    above: bool,
    max_height: Pixels,
}

const ITEM_HEIGHT: f32 = 30.;
const ITEM_GAP: f32 = 3.;
const MENU_PADDING: f32 = 6.;
const MENU_OFFSET: f32 = 7.;
const MENU_LABEL_INSET: f32 = 50.;
const SELECT_CORNER: f32 = 15.;
const MENU_CORNER: f32 = 17.;

fn popup_space(
    trigger: Bounds<Pixels>,
    viewport_height: Pixels,
    num_options: usize,
) -> (bool, Pixels) {
    let inset = px(MENU_OFFSET + 4.);
    let below = viewport_height - trigger.bottom() - inset;
    let above = trigger.top() - inset;
    let content = num_options as f32 * ITEM_HEIGHT
        + num_options.saturating_sub(1) as f32 * ITEM_GAP
        + 2. * MENU_PADDING;
    let desired = px(content.min(300.));
    let open_above = desired > below && above > below;
    let available = if open_above { above } else { below };
    (open_above, desired.min(available.max(px(1.))))
}

impl Panel {
    pub(super) fn modal(&self, content: Div) -> impl IntoElement {
        let mut veil = rgb(self.palette.sidebar);
        veil.a = 0.9;
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
            .bg(veil)
            .child(motion::reveal(
                div()
                    .id("dialog-body")
                    .w_full()
                    .max_w(px(720.))
                    .max_h(gpui::relative(1.))
                    .p_4()
                    .overflow_y_scroll()
                    .smooth_background(self.palette.canvas)
                    .child(shape::outline(self.palette.border))
                    .child(content),
                "dialog-open",
                self.animations,
            ))
    }

    fn toggle_switch(&self, on: bool, enabled: bool) -> Div {
        const TRACK_WIDTH: f32 = 34.;
        const TRACK_HEIGHT: f32 = 20.;
        const THUMB_DIAMETER: f32 = 14.;
        const TRACK_RADIUS: f32 = TRACK_HEIGHT / 2.;
        const THUMB_RADIUS: f32 = THUMB_DIAMETER / 2.;
        let palette = self.palette;
        div()
            .w(px(42.))
            .h(px(32.))
            .flex_shrink_0()
            .flex()
            .justify_end()
            .items_center()
            .group("switch-track")
            .child(motion::transition(
                "switch-slide",
                if on { 1. } else { 0. },
                self.animations,
                move |progress| {
                    let fill = motion::mix(palette.control, palette.accent, progress);
                    let hover = motion::mix(palette.hover, palette.accent_hover, progress);
                    let border = |active| {
                        motion::mix(
                            palette.border,
                            motion::mix(active, 0xffffff, 0.42),
                            progress,
                        )
                    };
                    let rounded = |mut element: Div, radius: f32| {
                        let radius = Some(px(radius).into());
                        let corners = &mut element.style().corner_radii;
                        corners.top_left = radius;
                        corners.top_right = radius;
                        corners.bottom_left = radius;
                        corners.bottom_right = radius;
                        element
                    };
                    let track =
                        rounded(div().absolute().top_0().left_0().size_full(), TRACK_RADIUS)
                            .border_1()
                            .border_color(rgb(border(palette.accent)))
                            .bg(rgb(fill))
                            .when(enabled, |track| {
                                let hover_border = border(palette.accent_hover);
                                track.group_hover("switch-track", move |style| {
                                    style.bg(rgb(hover)).border_color(rgb(hover_border))
                                })
                            });
                    // At either stop, the thumb shares the center of the track's semicircle.
                    let center_x = TRACK_RADIUS + (TRACK_WIDTH - TRACK_HEIGHT) * progress;
                    let thumb = rounded(div().absolute().size(px(THUMB_DIAMETER)), THUMB_RADIUS)
                        .top(px(TRACK_RADIUS - THUMB_RADIUS))
                        .left(px(center_x - THUMB_RADIUS))
                        .bg(rgb(motion::mix(palette.muted_text, 0xffffff, progress)));
                    div()
                        .relative()
                        .w(px(TRACK_WIDTH))
                        .h(px(TRACK_HEIGHT))
                        .child(track)
                        .child(thumb)
                },
            ))
    }

    pub(super) fn button_style(&self, enabled: bool) -> Div {
        self.control_style(enabled, shape::CONTROL_CORNER)
    }

    fn control_style(&self, enabled: bool, corner: f32) -> Div {
        let palette = self.palette;
        div()
            .h(px(32.))
            .min_w(px(70.))
            .px_3()
            .flex()
            .items_center()
            .justify_center()
            .relative()
            .when(enabled, |button| {
                button.smooth_hover_background(Some(palette.control), palette.hover, corner)
            })
            .when(!enabled, |button| {
                button
                    .opacity(0.45)
                    .child(shape::surface_radius(palette.control, corner))
            })
            .child(shape::outline_radius(palette.border, corner))
    }

    pub(super) fn inline_action(&self) -> Div {
        div()
            .px_2()
            .py_1()
            .smooth_hover_background(None, self.palette.hover, shape::CONTROL_CORNER)
            .cursor_pointer()
    }

    pub(super) fn setting_row<L: IntoElement>(&self, label: L, controls: Div) -> Div {
        div()
            .flex()
            .flex_wrap()
            .w_full()
            .min_h(px(44.))
            .py(px(6.))
            .items_center()
            .gap_x_4()
            .gap_y_1()
            .child(div().flex_1().min_w(px(90.)).child(label))
            .child(
                div()
                    .flex()
                    .ml_auto()
                    .flex_shrink_0()
                    .justify_end()
                    .child(controls),
            )
    }

    pub(super) fn setting_sections(&self, sections: impl IntoIterator<Item = Div>) -> Div {
        div()
            .flex()
            .flex_col()
            .children(sections.into_iter().enumerate().map(|(index, section)| {
                div()
                    .min_w_0()
                    .when(index > 0, |group| {
                        group
                            .mt_2()
                            .pt_2()
                            .border_t_1()
                            .border_color(rgb(self.palette.border))
                    })
                    .child(section)
            }))
    }

    pub(super) fn toggle_row(
        &self,
        id: impl Into<ElementId>,
        label: impl IntoElement,
        on: bool,
        enabled: bool,
        change: impl Fn(&mut Self, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Div {
        let switch = self.toggle_switch(on, enabled).id(id);
        let switch = if enabled {
            let change = Rc::new(change);
            let key_change = change.clone();
            switch
                .cursor_pointer()
                .focusable()
                .tab_index(0)
                .on_click(cx.listener(move |p, _, _, cx| {
                    change(p, cx);
                    cx.notify();
                }))
                .on_key_down(cx.listener(move |p, event: &KeyDownEvent, _, cx| {
                    if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                        key_change(p, cx);
                        cx.notify();
                        cx.stop_propagation();
                    }
                }))
        } else {
            switch.opacity(0.45)
        };
        self.setting_row(label, div().child(switch))
    }

    pub(super) fn control_text_width(&self, text: &str, cx: &App) -> f32 {
        let text_system = cx.text_system();
        let font = text_system.resolve_font(&gpui::font("Microsoft YaHei UI"));
        let mut advances = self.control_glyph_widths.borrow_mut();
        text.chars()
            .map(|ch| {
                *advances.entry(ch).or_insert_with(|| {
                    text_system
                        .advance(font, px(13.), ch)
                        .map_or(13., |size| size.width / px(1.))
                })
            })
            .sum()
    }

    pub(super) fn select(
        &self,
        id: impl Into<ElementId>,
        labels: Vec<String>,
        current: Option<usize>,
        placeholder: &str,
        width: impl Into<SelectWidth>,
        change: impl Fn(&mut Self, usize, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Div {
        self.select_with_details(
            id,
            labels.into_iter().map(|label| (label, None)).collect(),
            current,
            placeholder,
            width,
            change,
            cx,
        )
    }

    pub(super) fn select_with_details(
        &self,
        id: impl Into<ElementId>,
        mut options: Vec<(String, Option<String>)>,
        current: Option<usize>,
        placeholder: &str,
        width: impl Into<SelectWidth>,
        change: impl Fn(&mut Self, usize, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Div {
        let id = id.into();
        for (label, _) in &mut options {
            if label.contains(['\r', '\n']) {
                *label = label.split_whitespace().collect::<Vec<_>>().join(" ");
            }
        }
        let selected_option = current.and_then(|index| options.get(index));
        let selected = selected_option
            .map_or(placeholder, |(label, _)| label.as_str())
            .to_string();
        let palette = self.palette;
        let count = options.len();
        let measure = |text: &str| self.control_text_width(text, cx);
        let selected_width = measure(&selected);
        let width = match width.into() {
            SelectWidth::Fit(max) => {
                let widest = options
                    .iter()
                    .map(|(label, _)| measure(label))
                    .fold(selected_width, f32::max);
                (widest + MENU_LABEL_INSET).clamp(84., max)
            }
            SelectWidth::Fixed(width) => width,
        };
        let selected_tip = option_tip(
            &selected,
            selected_option.and_then(|(_, detail)| detail.as_deref()),
            selected_width + 46. > width,
        );
        let open = self
            .dropdown
            .as_ref()
            .filter(|menu| menu.id == id && count > 0);
        let trigger_box = Rc::new(Cell::new(None::<Bounds<Pixels>>));
        let measure_trigger = trigger_box.clone();
        let trigger = self
            .control_style(count > 0, SELECT_CORNER)
            .id(id.clone())
            .w(px(width))
            .justify_between()
            .gap_2()
            .child(
                canvas(
                    move |bounds, _, _| measure_trigger.set(Some(bounds)),
                    |_, _, _, _| {},
                )
                .absolute()
                .left_0()
                .top_0()
                .size_full(),
            )
            .child(div().flex_1().min_w_0().truncate().child(selected))
            .child(motion::transition(
                "select-arrow",
                if open.is_some() { 1. } else { 0. },
                self.animations,
                move |progress| select_chevron(palette.text, progress),
            ));
        let trigger = match (selected_tip, placeholder) {
            (Some(text), _) => trigger.tooltip(self.tooltip_owned(text.into())),
            (None, "选择显卡") => trigger.tooltip(self.tooltip("选择当前显卡")),
            (None, "选择显示器") => trigger.tooltip(self.tooltip("选择当前显示器")),
            (None, "选择应用程序") => trigger.tooltip(self.tooltip("选择当前应用程序")),
            _ => trigger,
        };
        let mut wrapper = div().relative().w(px(width)).flex_shrink_0();
        if count == 0 {
            return wrapper.child(trigger);
        }
        let initial = current.filter(|&i| i < count).unwrap_or(0);
        let toggle = {
            let id = id.clone();
            move |p: &mut Self, window: &mut Window, cx: &mut Context<Self>| {
                if p.dropdown.as_ref().is_some_and(|menu| menu.id == id) {
                    p.dropdown = None;
                } else if let Some(trigger) = trigger_box.get() {
                    let (above, max_height) =
                        popup_space(trigger, window.viewport_size().height, count);
                    p.dropdown = Some(Dropdown {
                        id: id.clone(),
                        keyboard_choice: None,
                        trigger,
                        above,
                        max_height,
                    });
                }
                cx.notify();
            }
        };
        let change = Rc::new(change);
        let key_change = change.clone();
        let key_toggle = toggle.clone();
        let key_id = id.clone();
        wrapper = wrapper.child(
            trigger
                .cursor_pointer()
                .focusable()
                .tab_index(0)
                .on_click(cx.listener(move |p, _, window, cx| {
                    cx.stop_propagation();
                    toggle(p, window, cx);
                }))
                .on_key_down(cx.listener(move |p, event: &KeyDownEvent, window, cx| {
                    let key = event.keystroke.key.as_str();
                    let Some(menu) = p.dropdown.as_mut().filter(|menu| menu.id == key_id) else {
                        if matches!(key, "space" | "enter" | "down" | "up") {
                            key_toggle(p, window, cx);
                            cx.stop_propagation();
                        }
                        return;
                    };
                    let focused = menu.keyboard_choice.unwrap_or(initial);
                    match key {
                        "up" => menu.keyboard_choice = Some(focused.saturating_sub(1)),
                        "down" => menu.keyboard_choice = Some((focused + 1).min(count - 1)),
                        "escape" => p.dropdown = None,
                        "enter" | "space" => {
                            p.dropdown = None;
                            key_change(p, focused, cx);
                        }
                        "tab" => {
                            p.dropdown = None;
                            cx.notify();
                            return;
                        }
                        _ => return,
                    }
                    cx.notify();
                    cx.stop_propagation();
                })),
        );
        if let Some(menu) = open {
            let trigger = menu.trigger;
            let dismiss_id = id.clone();
            let mut list = div()
                .id(ElementId::NamedChild(
                    Box::new(id.clone()),
                    "options".into(),
                ))
                .w(px(width))
                .max_h(menu.max_height)
                .p(px(MENU_PADDING))
                .flex()
                .flex_col()
                .gap(px(ITEM_GAP))
                .occlude()
                .overflow_y_scroll()
                .relative()
                .child(shape::surface_radius(palette.control, MENU_CORNER))
                .child(shape::outline_radius(palette.border, MENU_CORNER))
                .on_mouse_down_out(cx.listener(move |p, event: &gpui::MouseDownEvent, _, cx| {
                    if !trigger.contains(&event.position)
                        && p.dropdown
                            .as_ref()
                            .is_some_and(|current| current.id == dismiss_id)
                    {
                        p.dropdown = None;
                        cx.notify();
                    }
                }));
            for (i, (label, detail)) in options.into_iter().enumerate() {
                let text_width = measure(&label);
                let choose = change.clone();
                let choice_id = id.clone();
                let tip = option_tip(
                    &label,
                    detail.as_deref(),
                    text_width + MENU_LABEL_INSET > width,
                );
                list = list.child(
                    div()
                        .id(("option", i))
                        .h(px(ITEM_HEIGHT))
                        .flex_shrink_0()
                        .px_2()
                        .gap_2()
                        .flex()
                        .items_center()
                        .cursor_pointer()
                        .smooth_hover_background(
                            (Some(i) == menu.keyboard_choice).then_some(palette.hover),
                            palette.hover,
                            shape::CONTROL_CORNER,
                        )
                        .child(
                            div()
                                .id(("option-label", i))
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .when_some(tip, |label, text| {
                                    label.tooltip(self.tooltip_owned(text.into()))
                                })
                                .child(label),
                        )
                        .child(
                            div()
                                .size(px(14.))
                                .flex_shrink_0()
                                .when(current == Some(i), |indicator| {
                                    indicator.child(select_check(palette.accent))
                                }),
                        )
                        .on_click(cx.listener(move |p, _, _, cx| {
                            cx.stop_propagation();
                            if p.dropdown.as_ref().is_some_and(|menu| menu.id == choice_id) {
                                p.dropdown = None;
                                choose(p, i, cx);
                                cx.notify();
                            }
                        })),
                );
            }
            let (corner, y) = if menu.above {
                (Corner::BottomLeft, trigger.top() - px(MENU_OFFSET))
            } else {
                (Corner::TopLeft, trigger.bottom() + px(MENU_OFFSET))
            };
            wrapper = wrapper.child(
                deferred(
                    anchored()
                        .anchor(corner)
                        .position(point(trigger.left(), y))
                        .snap_to_window()
                        .child(motion::reveal(
                            list,
                            ElementId::NamedChild(Box::new(id), "open".into()),
                            self.animations,
                        )),
                )
                .with_priority(3),
            );
        }
        wrapper
    }
}

fn option_tip(label: &str, detail: Option<&str>, clipped: bool) -> Option<String> {
    match (detail, clipped) {
        (Some(detail), true) => Some(format!("{label}\n{detail}")),
        (Some(detail), false) => Some(detail.to_string()),
        (None, true) => Some(label.to_string()),
        (None, false) => None,
    }
}

fn select_check(color: u32) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let at = |x, y| bounds.origin + point(px(x), px(y));
            let mut path = PathBuilder::stroke(px(1.5));
            path.move_to(at(2.5, 7.));
            path.line_to(at(5.5, 10.));
            path.line_to(at(11.5, 4.));
            if let Ok(path) = path.build() {
                window.paint_path(path, rgb(color));
            }
        },
    )
    .size_full()
}

fn select_chevron(color: u32, progress: f32) -> impl IntoElement {
    let (sine, cosine) = (progress * std::f32::consts::PI).sin_cos();
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let center = point(
                bounds.origin.x + bounds.size.width * 0.5,
                bounds.origin.y + bounds.size.height * 0.5,
            );
            let rotate = |x, y| {
                point(
                    center.x + px(x * cosine - y * sine),
                    center.y + px(x * sine + y * cosine),
                )
            };
            let mut path = PathBuilder::stroke(px(1.25));
            path.move_to(rotate(-3.5, -1.75));
            path.line_to(rotate(0., 1.75));
            path.line_to(rotate(3.5, -1.75));
            if let Ok(path) = path.build() {
                window.paint_path(path, rgb(color));
            }
        },
    )
    .size(px(14.))
    .flex_shrink_0()
}

#[cfg(test)]
mod tests {
    use super::{Bounds, point, popup_space, px};
    use gpui::size;

    #[test]
    fn popup_opens_above_a_trigger_near_the_bottom() {
        let trigger = Bounds::new(point(px(100.), px(628.)), size(px(90.), px(32.)));
        let (above, height) = popup_space(trigger, px(720.), 6);
        assert!(above);
        assert_eq!(height, px(207.));
    }
}
