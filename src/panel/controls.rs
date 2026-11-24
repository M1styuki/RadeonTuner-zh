use super::{
    Panel,
    shape::{self, ContinuousCorners},
};
use gpui::{
    Context, Div, ElementId, FocusHandle, IntoElement, KeyDownEvent, PathBuilder, Window, anchored,
    canvas, deferred, div, point, prelude::*, px, rgb,
};
use std::{collections::HashMap, rc::Rc};

pub(super) const SELECT_MAX_WIDTH: f32 = 216.;

pub(super) struct Dropdown {
    id: ElementId,
    focus: FocusHandle,
    highlighted: usize,
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
            .child(
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
            )
    }

    fn toggle_switch(&self, on: bool, enabled: bool) -> Div {
        let palette = self.palette;
        let fill = if on { palette.accent } else { palette.control };
        let hover = if on {
            palette.accent_hover
        } else {
            palette.hover
        };
        let track = div()
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
            });
        div()
            .relative()
            .w(px(44.))
            .h(px(32.))
            .flex_shrink_0()
            .child(track)
            .child(
                div()
                    .absolute()
                    .top(px(8.))
                    .left(px(if on { 26. } else { 2. }))
                    .w(px(16.))
                    .h(px(16.))
                    .rounded_full()
                    .bg(rgb(if on { 0xffffff } else { palette.text })),
            )
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
            .w_full()
            .min_h(px(44.))
            .py(px(6.))
            .items_center()
            .gap_4()
            .child(div().flex_1().min_w_0().child(label))
            .child(div().flex().flex_shrink_0().justify_end().child(controls))
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
        let trigger = self
            .button_style(count > 0)
            .id(id.clone())
            .w(px(width))
            .justify_between()
            .gap_2()
            .child(div().flex_1().min_w_0().truncate().child(selected))
            .child(select_chevron(palette.text));
        let trigger = match (clipped_selection, placeholder) {
            (Some(text), _) => trigger.tooltip(self.tooltip_owned(text.into())),
            (None, "选择显卡") => trigger.tooltip(self.tooltip("选择当前显卡")),
            (None, "选择显示器") => trigger.tooltip(self.tooltip("选择当前显示器")),
            (None, "选择应用程序") => trigger.tooltip(self.tooltip("选择当前应用程序")),
            _ => trigger,
        };
        let mut wrapper = div().relative().w(px(width)).flex_shrink_0();
        let open = self
            .dropdown
            .as_ref()
            .filter(|menu| menu.id == id && count > 0);
        if count == 0 {
            return wrapper.child(trigger);
        }
        let initial = current.filter(|&i| i < count).unwrap_or(0);
        let open_menu = move |p: &mut Self, window: &mut Window, cx: &mut Context<Self>| {
            let focus = cx.focus_handle();
            focus.focus(window);
            p.dropdown = Some(Dropdown {
                id: id.clone(),
                focus,
                highlighted: initial,
            });
            cx.notify();
        };
        let key_open = open_menu.clone();
        wrapper = wrapper.child(
            trigger
                .cursor_pointer()
                .focusable()
                .tab_index(0)
                .on_click(cx.listener(move |p, _, window, cx| open_menu(p, window, cx)))
                .on_key_down(cx.listener(move |p, event: &KeyDownEvent, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "space" | "enter" | "down") {
                        key_open(p, window, cx);
                        cx.stop_propagation();
                    }
                })),
        );
        if let Some(menu) = open {
            let change = Rc::new(change);
            let key_change = change.clone();
            let mut list = div()
                .id("select-options")
                .w(px(width))
                .max_h(px(300.))
                .p_1()
                .occlude()
                .overflow_y_scroll()
                .track_focus(&menu.focus)
                .smooth_background(palette.control)
                .child(shape::outline(palette.border))
                .on_mouse_down_out(cx.listener(|p, _, _, cx| {
                    p.dropdown = None;
                    cx.notify();
                }))
                .on_key_down(cx.listener(move |p, event: &KeyDownEvent, _, cx| {
                    let Some(menu) = &mut p.dropdown else {
                        return;
                    };
                    match event.keystroke.key.as_str() {
                        "up" => menu.highlighted = menu.highlighted.saturating_sub(1),
                        "down" => menu.highlighted = (menu.highlighted + 1).min(count - 1),
                        "escape" => p.dropdown = None,
                        "enter" | "space" => {
                            let i = menu.highlighted;
                            p.dropdown = None;
                            key_change(p, i, cx);
                        }
                        _ => return,
                    }
                    cx.notify();
                    cx.stop_propagation();
                }));
            for (i, (label, text_width)) in labels.into_iter().zip(option_widths).enumerate() {
                let choose = change.clone();
                let full_text = (text_width + 24. > width).then(|| label.clone());
                list = list.child(
                    div()
                        .id(("option", i))
                        .px_2()
                        .py_1()
                        .cursor_pointer()
                        .smooth_hover_background(
                            (i == menu.highlighted).then_some(palette.selected),
                            palette.hover,
                            shape::CONTROL_CORNER,
                        )
                        .when_some(full_text, |row, text| {
                            row.tooltip(self.tooltip_owned(text.into()))
                        })
                        .child(div().min_w_0().truncate().child(label))
                        .on_click(cx.listener(move |p, _, _, cx| {
                            p.dropdown = None;
                            choose(p, i, cx);
                            cx.notify();
                        })),
                );
            }
            wrapper = wrapper.child(
                deferred(
                    anchored()
                        .offset(point(px(0.), px(3.)))
                        .snap_to_window()
                        .child(list),
                )
                .with_priority(3),
            );
        }
        wrapper
    }
}

fn select_chevron(color: u32) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let x = bounds.origin.x + bounds.size.width * 0.5;
            let y = bounds.origin.y + bounds.size.height * 0.5;
            let mut path = PathBuilder::stroke(px(1.25));
            path.move_to(point(x - px(3.5), y - px(1.75)));
            path.line_to(point(x, y + px(1.75)));
            path.line_to(point(x + px(3.5), y - px(1.75)));
            if let Ok(path) = path.build() {
                window.paint_path(path, rgb(color));
            }
        },
    )
    .size(px(14.))
    .flex_shrink_0()
}
