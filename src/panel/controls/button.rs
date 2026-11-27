use super::{Panel, accent_highlight, control_highlight};
use crate::panel::{motion::pointer_feedback, shape::ContinuousCorners as _};
use gpui_kit::component::{ActiveTheme as _, h_flex};
use gpui_kit::{
    Context, ElementId, FontWeight, InteractiveElement as _, KeyDownEvent, ParentElement as _,
    Role, SharedString, Stateful, StatefulInteractiveElement as _, Styled as _, Window,
    prelude::FluentBuilder as _, rems,
};
use std::rc::Rc;

#[derive(Clone, Copy)]
pub(in crate::panel) enum ButtonTone {
    Secondary,
    Primary,
    Danger,
    Inline,
}

impl Panel {
    pub(in crate::panel) fn action_button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        tone: ButtonTone,
        enabled: bool,
        tooltip: Option<SharedString>,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<gpui_kit::Div> {
        let id: ElementId = id.into();
        let label: SharedString = label.into();
        let radius = f32::from(cx.theme().radius);
        let (fill, hover, active, fg) = match tone {
            ButtonTone::Secondary => (
                Some(cx.theme().button),
                cx.theme().button_hover,
                cx.theme().button_active,
                cx.theme().button_foreground,
            ),
            ButtonTone::Primary => (
                Some(cx.theme().button_primary),
                cx.theme().button_primary_hover,
                cx.theme().button_primary_active,
                cx.theme().button_primary_foreground,
            ),
            ButtonTone::Danger => (
                Some(cx.theme().button_danger),
                cx.theme().button_danger_hover,
                cx.theme().button_danger_active,
                cx.theme().button_danger_foreground,
            ),
            ButtonTone::Inline => (
                None,
                cx.theme().accent,
                cx.theme().secondary_active,
                cx.theme().foreground,
            ),
        };
        let is_inline = matches!(tone, ButtonTone::Inline);
        let highlight = match tone {
            ButtonTone::Secondary => Some(control_highlight(cx)),
            ButtonTone::Primary | ButtonTone::Danger => Some(accent_highlight(cx)),
            ButtonTone::Inline => None,
        };
        let mut btn = h_flex()
            .id(id.clone())
            .relative()
            .flex_shrink_0()
            .justify_center()
            .items_center()
            .when(is_inline, |d| d.h_7().px_2p5())
            .when(!is_inline, |d| d.h_8().min_w(rems(4.375)).px_3())
            .when_some(fill, |d, color| d.contour_fill(color, radius))
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_color(fg)
            .when_some(tooltip, |d, tip| d.tooltip(self.tooltip_owned(tip)));

        if enabled {
            let on_click = Rc::new(on_click);
            let click_cb = on_click.clone();
            let key_cb = on_click;
            btn = pointer_feedback((id, "btn-fb"), btn, hover, active, radius)
                .focusable()
                .tab_stop(true)
                .role(Role::Button)
                .aria_label(label.clone())
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
                }));
        } else {
            btn = btn.opacity(0.45);
        }

        btn.when_some(highlight, |d, ring| d.ring_highlight(ring, radius))
            .child(label)
    }

    pub(in crate::panel) fn contour_button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        enabled: bool,
        tooltip: Option<&'static str>,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<gpui_kit::Div> {
        self.action_button(
            id,
            label,
            ButtonTone::Secondary,
            enabled,
            tooltip.map(Into::into),
            on_click,
            cx,
        )
    }

    pub(in crate::panel) fn primary_button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        enabled: bool,
        tooltip: Option<&'static str>,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<gpui_kit::Div> {
        self.action_button(
            id,
            label,
            ButtonTone::Primary,
            enabled,
            tooltip.map(Into::into),
            on_click,
            cx,
        )
    }

    pub(in crate::panel) fn danger_button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        tooltip: Option<&'static str>,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<gpui_kit::Div> {
        self.action_button(
            id,
            label,
            ButtonTone::Danger,
            true,
            tooltip.map(Into::into),
            on_click,
            cx,
        )
    }

    pub(in crate::panel) fn inline_button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<gpui_kit::Div> {
        self.action_button(id, label, ButtonTone::Inline, true, None, on_click, cx)
    }
}
