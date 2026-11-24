use super::{
    Panel, motion,
    shape::{self, ContinuousCorners},
};
use gpui::{
    Bounds, Context, Corner, Div, ElementId, IntoElement, KeyDownEvent, PathBuilder, Pixels,
    Window, anchored, canvas, deferred, div, point, prelude::*, px, rgb,
};
use std::{cell::Cell, collections::HashMap, rc::Rc};

pub(super) const SELECT_MAX_WIDTH: f32 = 216.;

pub(super) struct Dropdown {
    id: ElementId,
    highlighted: usize,
    trigger: Bounds<Pixels>,
    above: bool,
    max_height: Pixels,
}

const ITEM_HEIGHT: f32 = 26.;

fn popup_space(
    trigger: Bounds<Pixels>,
    viewport_height: Pixels,
    num_options: usize,
) -> (bool, Pixels) {
    let below = viewport_height - trigger.bottom() - px(8.);
    let above = trigger.top() - px(8.);
    let desired = px((num_options as f32 * ITEM_HEIGHT + 8.).min(300.));
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
        let palette = self.palette;
        div()
            .relative()
            .w(px(44.))
            .h(px(32.))
            .flex_shrink_0()
            .child(motion::transition(
                "switch-slide",
                if on { 1. } else { 0. },
                self.animations,
                move |progress| {
                    let fill = motion::mix(palette.control, palette.accent, progress);
                    let hover = motion::mix(palette.hover, palette.accent_hover, progress);
                    div()
                        .relative()
                        .size_full()
                        .child(
                            div()
                                .absolute()
                                .top(px(6.))
                                .w_full()
                                .h(px(20.))
                                .rounded_full()
                                .border_1()
                                .border_color(rgb(palette.border))
                                .bg(rgb(fill))
                                .when(enabled, |track| {
                                    track.hover(move |style| style.bg(rgb(hover)))
                                }),
                        )
                        .child(
                            div()
                                .absolute()
                                .top(px(8.))
                                .left(px(2. + 24. * progress))
                                .size(px(16.))
                                .rounded_full()
                                .bg(rgb(motion::mix(palette.text, 0xffffff, progress))),
                        )
                },
            ))
    }

    pub(super) fn button_style(&self, enabled: bool) -> Div {
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
                button.smooth_hover_background(
                    Some(palette.control),
                    palette.hover,
                    shape::CONTROL_CORNER,
                )
            })
            .when(!enabled, |button| {
                button.opacity(0.45).child(shape::surface_radius(
                    palette.control,
                    shape::CONTROL_CORNER,
                ))
            })
            .child(shape::outline_radius(palette.border, shape::CONTROL_CORNER))
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

    pub(super) fn setting_sections(
        &self,
        sections: impl IntoIterator<Item = (&'static str, Div)>,
    ) -> Div {
        div()
            .grid()
            .grid_cols(if self.two_columns { 2 } else { 1 })
            .items_start()
            .gap_3()
            .children(sections.into_iter().map(|(title, content)| {
                div()
                    .min_w_0()
                    .p_3()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .smooth_background(self.palette.sidebar)
                    .child(shape::outline(self.palette.border))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .pb_1()
                            .child(title),
                    )
                    .child(content)
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

    pub(super) fn select(
        &self,
        id: impl Into<ElementId>,
        mut labels: Vec<String>,
        current: Option<usize>,
        placeholder: &str,
        max_width: f32,
        change: impl Fn(&mut Self, usize, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Div {
        let id = id.into();
        for label in &mut labels {
            if label.contains(['\r', '\n']) {
                *label = label.split_whitespace().collect::<Vec<_>>().join(" ");
            }
        }
        let selected = labels
            .get(current.unwrap_or(usize::MAX))
            .cloned()
            .unwrap_or_else(|| placeholder.into());
        let palette = self.palette;
        let count = labels.len();
        let text_system = cx.text_system();
        let font = text_system.resolve_font(&gpui::font("Microsoft YaHei UI"));
        let mut advances = HashMap::new();
        let mut measure = |text: &str| {
            text.chars()
                .map(|ch| {
                    *advances.entry(ch).or_insert_with(|| {
                        text_system
                            .advance(font, px(13.), ch)
                            .map_or(13., |size| size.width / px(1.))
                    })
                })
                .sum::<f32>()
        };
        let option_widths = labels
            .iter()
            .map(|label| measure(label))
            .collect::<Vec<_>>();
        let selected_width = measure(&selected);
        let width = (option_widths.iter().copied().fold(selected_width, f32::max) + 50.)
            .clamp(84., max_width);
        let clipped_selection = (selected_width + 46. > width).then(|| selected.clone());
        let open = self
            .dropdown
            .as_ref()
            .filter(|menu| menu.id == id && count > 0);
        let trigger_box = Rc::new(Cell::new(None::<Bounds<Pixels>>));
        let measure_trigger = trigger_box.clone();
        let trigger = self
            .button_style(count > 0)
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
        let trigger = match (clipped_selection, placeholder) {
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
                        highlighted: initial,
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
                    match key {
                        "up" => menu.highlighted = menu.highlighted.saturating_sub(1),
                        "down" => menu.highlighted = (menu.highlighted + 1).min(count - 1),
                        "escape" => p.dropdown = None,
                        "enter" | "space" => {
                            let selected = menu.highlighted;
                            p.dropdown = None;
                            key_change(p, selected, cx);
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
                .p_1()
                .occlude()
                .overflow_y_scroll()
                .smooth_background(palette.control)
                .child(shape::outline(palette.border))
                .on_mouse_down_out(cx.listener(move |p, event, _, cx| {
                    if !trigger.contains(&event.position)
                        && p.dropdown
                            .as_ref()
                            .is_some_and(|current| current.id == dismiss_id)
                    {
                        p.dropdown = None;
                        cx.notify();
                    }
                }));
            for (i, (label, text_width)) in labels.into_iter().zip(option_widths).enumerate() {
                let choose = change.clone();
                let choice_id = id.clone();
                let full_text = (text_width + 24. > width).then(|| label.clone());
                list = list.child(
                    div()
                        .id(("option", i))
                        .h(px(ITEM_HEIGHT))
                        .px_2()
                        .flex()
                        .items_center()
                        .cursor_pointer()
                        .smooth_hover_background(
                            (i == menu.highlighted).then_some(palette.selected),
                            palette.hover,
                            shape::CONTROL_CORNER,
                        )
                        .when_some(full_text, |row, text| {
                            row.tooltip(self.tooltip_owned(text.into()))
                        })
                        .child(div().flex_1().min_w_0().truncate().child(label))
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
                (Corner::BottomLeft, trigger.top() - px(4.))
            } else {
                (Corner::TopLeft, trigger.bottom() + px(4.))
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
        assert_eq!(height, px(164.));
    }
}
