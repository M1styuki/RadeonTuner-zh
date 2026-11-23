use super::{
    Panel,
    shape::{self, ContinuousCorners},
};
use gpui::{
    Context, Div, ElementId, FocusHandle, IntoElement, KeyDownEvent, Window, anchored, deferred,
    div, point, prelude::*, px, rgb,
};
use std::rc::Rc;

pub(super) struct Dropdown {
    id: ElementId,
    focus: FocusHandle,
    highlighted: usize,
}

impl Panel {
    pub(super) fn modal(&self, content: Div) -> impl IntoElement {
        let mut veil = rgb(self.palette.sidebar);
        veil.a = 0.9;
        deferred(
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
                ),
        )
        .with_priority(2)
    }

    pub(super) fn toggle_switch(&self, on: bool) -> Div {
        let palette = self.palette;
        div()
            .relative()
            .w(px(44.))
            .h(px(32.))
            .flex_shrink_0()
            .child(
                div()
                    .absolute()
                    .top(px(6.))
                    .w_full()
                    .h(px(20.))
                    .child(shape::surface_radius(palette.border, 10.))
                    .child(
                        div()
                            .absolute()
                            .top(px(1.))
                            .bottom(px(1.))
                            .left(px(1.))
                            .right(px(1.))
                            .child(shape::surface_radius(
                                if on { palette.accent } else { palette.control },
                                9.,
                            )),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top(px(8.))
                    .left(px(if on { 24. } else { 4. }))
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
            .child(shape::surface_radius(palette.control, 6.))
            .when(!enabled, |button| button.opacity(0.45))
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
        let switch = self.toggle_switch(on).id(id);
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
        labels: Vec<String>,
        current: Option<usize>,
        placeholder: &str,
        width: f32,
        change: impl Fn(&mut Self, usize, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Div {
        let id = id.into();
        let selected = labels
            .get(current.unwrap_or(usize::MAX))
            .cloned()
            .unwrap_or_else(|| placeholder.into());
        let palette = self.palette;
        let count = labels.len();
        let trigger = self
            .button_style(count > 0)
            .id(id.clone())
            .w(px(width))
            .justify_between()
            .gap_2()
            .child(div().flex_1().min_w_0().truncate().child(selected))
            .child("⌄");
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
                .w(px(width.max(184.)))
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
            for (i, label) in labels.into_iter().enumerate() {
                let choose = change.clone();
                list = list.child(
                    div()
                        .id(("option", i))
                        .px_2()
                        .py_1()
                        .cursor_pointer()
                        .when(i == menu.highlighted, |row| {
                            row.smooth_background(palette.selected)
                        })
                        .hover(move |row| row.bg(rgb(palette.hover)))
                        .child(label)
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
                        .offset(point(px(0.), px(35.)))
                        .snap_to_window()
                        .child(list),
                )
                .with_priority(3),
            );
        }
        wrapper
    }
}
