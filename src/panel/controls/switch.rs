use super::Panel;
use crate::panel::motion::{smooth_spring, smooth_value};
use gpui_kit::component::{ActiveTheme as _, h_flex};
use gpui_kit::{
    App, BorderStyle, Bounds, Context, Div, ElementId, Entity, InteractiveElement as _,
    IntoElement, KeyDownEvent, MouseButton, ParentElement as _, Role, Stateful,
    StatefulInteractiveElement as _, Styled as _, Window, canvas, div, fill, point, px, quad, size,
};
use std::{cell::RefCell, rc::Rc, time::Duration};

#[derive(Default, Clone, Copy)]
struct SwitchPointerState {
    hovered: bool,
    pressed: bool,
}

fn win11_switch(
    id: ElementId,
    on: bool,
    enabled: bool,
    on_toggle: impl Fn(&mut App) + 'static,
    cx: &App,
) -> Stateful<Div> {
    let state_id = ElementId::NamedChild(id.clone().into(), "__win11-switch-state".into());
    let hover_id = id.clone();
    let press_id = id.clone();
    let stretch_id = id.clone();
    let slide_id = id.clone();
    let on_id = id.clone();
    let en_id = id.clone();
    let slot = Rc::new(RefCell::new(None::<Entity<SwitchPointerState>>));
    let hovered = slot.clone();
    let pressed = slot.clone();
    let released = slot.clone();
    let outside = slot.clone();

    let transparent = cx.theme().transparent;
    let off_border = cx.theme().foreground.opacity(0.62);
    let off_border_hover = cx.theme().foreground.opacity(0.80);
    let off_fill_hover = cx.theme().accent;
    let off_fill_press = cx.theme().secondary_active;
    let on_fill = cx.theme().primary;
    let on_fill_hover = cx.theme().primary_hover;
    let on_fill_press = cx.theme().primary_active;
    let off_thumb = cx.theme().muted_foreground;
    let off_thumb_hover = cx.theme().foreground.opacity(0.85);
    let on_thumb = cx.theme().switch_thumb;

    let set_press = |slot: &RefCell<Option<Entity<SwitchPointerState>>>,
                     down: bool,
                     window: &mut Window,
                     cx: &mut App| {
        if let Some(state) = slot.borrow().as_ref() {
            state.update(cx, |s, _| s.pressed = down);
            window.refresh();
        }
    };

    let mut switch = h_flex()
        .id(id)
        .relative()
        .w(px(40.))
        .h(px(20.))
        .flex_shrink_0()
        .items_center()
        .role(Role::Switch)
        .aria_selected(on)
        .aria_label(if on { "已开启" } else { "已关闭" })
        .child(
            canvas(
                move |_, window, cx| {
                    let state = window.use_keyed_state(state_id.clone(), cx, |_, _| {
                        SwitchPointerState::default()
                    });
                    *slot.borrow_mut() = Some(state.clone());
                    let s = *state.read(cx);
                    let hover = smooth_value(
                        (hover_id.clone(), "sw-hover"),
                        if enabled && s.hovered { 1. } else { 0. },
                        Duration::from_millis(100),
                        window,
                        cx,
                    );
                    let press = smooth_value(
                        (press_id.clone(), "sw-press"),
                        if enabled && s.pressed { 1. } else { 0. },
                        Duration::from_millis(75),
                        window,
                        cx,
                    );
                    let stretch = smooth_value(
                        (stretch_id.clone(), "sw-stretch"),
                        if enabled && s.pressed { 1. } else { 0. },
                        Duration::from_millis(90),
                        window,
                        cx,
                    );
                    let slide = f32::from(smooth_spring(
                        (slide_id.clone(), "sw-slide"),
                        px(if on { 100. } else { 0. }),
                        window,
                        cx,
                    )) / 100.;
                    let on_mix = smooth_value(
                        (on_id.clone(), "sw-on"),
                        if on { 1. } else { 0. },
                        Duration::from_millis(150),
                        window,
                        cx,
                    );
                    let en_alpha = smooth_value(
                        (en_id.clone(), "sw-en"),
                        if enabled { 1. } else { 0.45 },
                        Duration::from_millis(120),
                        window,
                        cx,
                    );
                    (hover, press, stretch, slide.clamp(0., 1.), on_mix, en_alpha)
                },
                move |bounds, (hover, press, stretch, slide, on_mix, en_alpha), window, _| {
                    let radius = f32::from(bounds.size.height) / 2.;
                    let off_bg = transparent
                        .blend(off_fill_hover.opacity(hover))
                        .blend(off_fill_press.opacity(press));
                    let active_bg = on_fill
                        .blend(on_fill_hover.opacity(hover))
                        .blend(on_fill_press.opacity(press));
                    let track_bg = off_bg.blend(active_bg.opacity(on_mix));
                    let off_stroke = off_border.blend(off_border_hover.opacity(hover));
                    let track_border = off_stroke.blend(active_bg.opacity(on_mix));
                    window.paint_quad(quad(
                        bounds,
                        px(radius),
                        track_bg.opacity(track_bg.a * en_alpha),
                        px(1.1),
                        track_border.opacity(track_border.a * en_alpha),
                        BorderStyle::Solid,
                    ));

                    let thumb_d = 12.;
                    let thumb_w = thumb_d + 3. * stretch;
                    let pad = (f32::from(bounds.size.height) - thumb_d) / 2.;
                    let travel = (f32::from(bounds.size.width) - 2. * pad - thumb_d).max(0.);
                    let base_x = f32::from(bounds.origin.x) + pad + travel * slide;
                    let thumb_x = if on {
                        base_x - (thumb_w - thumb_d) * slide
                    } else {
                        base_x
                    };
                    let thumb_y = f32::from(bounds.origin.y) + pad;
                    let thumb_color = off_thumb
                        .blend(off_thumb_hover.opacity(hover))
                        .blend(on_thumb.opacity(on_mix));
                    window.paint_quad(
                        fill(
                            Bounds::new(
                                point(px(thumb_x), px(thumb_y)),
                                size(px(thumb_w), px(thumb_d)),
                            ),
                            thumb_color.opacity(thumb_color.a * en_alpha),
                        )
                        .corner_radii(px(thumb_d / 2.)),
                    );
                },
            )
            .size_full(),
        );

    if enabled {
        let on_toggle_key = Rc::new(on_toggle);
        let on_toggle_click = on_toggle_key.clone();
        switch = switch
            .focusable()
            .tab_stop(true)
            .on_hover(move |hover, window, cx| {
                if let Some(state) = hovered.borrow().as_ref() {
                    state.update(cx, |s, _| s.hovered = *hover);
                    window.refresh();
                }
            })
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                set_press(&pressed, true, window, cx);
            })
            .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                set_press(&released, false, window, cx);
            })
            .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
                set_press(&outside, false, window, cx);
            })
            .on_click(move |_, _, cx| on_toggle_click(cx))
            .on_key_down(move |e: &KeyDownEvent, _, cx| {
                if matches!(e.keystroke.key.as_str(), "enter" | "space") {
                    cx.stop_propagation();
                    on_toggle_key(cx);
                }
            });
    }
    switch
}

impl Panel {
    pub(in crate::panel) fn toggle_row(
        &self,
        id: impl Into<ElementId>,
        label: impl IntoElement,
        on: bool,
        enabled: bool,
        change: impl Fn(&mut Self, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Div {
        let id: ElementId = id.into();
        let entity = cx.entity();
        let switch = win11_switch(
            id,
            on,
            enabled,
            move |cx| {
                entity.update(cx, |p, cx| {
                    change(p, cx);
                    cx.notify();
                });
            },
            cx,
        );
        self.setting_row(label, div().child(switch))
    }
}
