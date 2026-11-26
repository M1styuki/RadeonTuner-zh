use super::{
    Panel,
    layout::nav_icon,
    motion::{pointer_feedback, pointer_feedback_corners, smooth_spring, smooth_value},
    shape::{ContinuousCorners as _, paint_outline},
};
use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::{
    Anchor, Animation, AnimationExt as _, App, BorderStyle, Bounds, Context, Corners, Div,
    ElementId, FontWeight, Hsla, InteractiveElement as _, IntoElement, KeyDownEvent, MouseButton,
    MouseDownEvent, ParentElement as _, PathBuilder, Pixels, Role, SharedString, Stateful,
    StatefulInteractiveElement as _, Styled as _, Window, anchored, canvas, deferred, div,
    ease_out_quint, fill, point, prelude::FluentBuilder as _, px, quad, relative, rems, size,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

pub(super) const SELECT_MAX_WIDTH: f32 = 340.;
pub(super) const PROFILE_SELECT_WIDTH: SelectWidth = SelectWidth::FitSelected(340.);

pub(super) enum SelectWidth {
    Fit(f32),
    FitSelected(f32),
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
const TRIGGER_LABEL_INSET: f32 = 56.;
const MENU_LABEL_INSET: f32 = 56.;

pub(super) fn sidebar_width(viewport: Pixels) -> Pixels {
    if viewport < px(680.) {
        px(56.)
    } else {
        px(148.)
    }
}

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

pub(super) fn section_heading(title: &'static str, cx: &App) -> Div {
    div()
        .px_1()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(cx.theme().muted_foreground)
        .child(title)
}

pub(super) fn section_card(cx: &App) -> Div {
    v_flex()
        .relative()
        .w_full()
        .overflow_hidden()
        .rounded(cx.theme().radius_lg)
        .bg(cx.theme().group_box)
        .shadow_xs()
}

pub(super) enum CardItem {
    Static(Div),
    Interactive(Box<dyn FnOnce(Corners<Pixels>) -> Div>),
}

impl From<Div> for CardItem {
    fn from(row: Div) -> Self {
        Self::Static(row)
    }
}

pub(super) fn card_rows<I: Into<CardItem>>(rows: impl IntoIterator<Item = I>, cx: &App) -> Div {
    let rows: Vec<CardItem> = rows.into_iter().map(Into::into).collect();
    let count = rows.len();
    let radius = cx.theme().radius_lg;
    let surface = cx.theme().group_box;
    v_flex()
        .w_full()
        .gap(px(2.))
        .children(rows.into_iter().enumerate().map(|(i, item)| {
            let (top, bottom) = (i == 0, i + 1 == count);
            let corners = Corners {
                top_left: if top { radius } else { px(0.) },
                top_right: if top { radius } else { px(0.) },
                bottom_right: if bottom { radius } else { px(0.) },
                bottom_left: if bottom { radius } else { px(0.) },
            };
            let row = match item {
                CardItem::Static(row) => row,
                CardItem::Interactive(build) => build(corners),
            };
            let tile = row.w_full().overflow_hidden().bg(surface).shadow_xs();
            match (top, bottom) {
                (true, true) => tile.rounded(radius),
                (true, false) => tile.rounded_t(radius),
                (false, true) => tile.rounded_b(radius),
                (false, false) => tile,
            }
        }))
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

pub(super) fn control_highlight(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        cx.theme().switch_thumb.opacity(0.14)
    } else {
        cx.theme().switch_thumb.opacity(0.85)
    }
}

#[derive(Clone, Copy)]
pub(super) enum ButtonTone {
    Secondary,
    Primary,
    Danger,
    Inline,
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

    pub(super) fn action_button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        tone: ButtonTone,
        enabled: bool,
        tooltip: Option<SharedString>,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
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
            ButtonTone::Primary | ButtonTone::Danger => Some(cx.theme().switch_thumb.opacity(0.22)),
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
            .rounded(cx.theme().radius)
            .contour_highlight(fill, highlight, radius)
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

        btn.child(label)
    }

    pub(super) fn contour_button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        enabled: bool,
        tooltip: Option<&'static str>,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
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

    pub(super) fn primary_button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        enabled: bool,
        tooltip: Option<&'static str>,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
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

    pub(super) fn danger_button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        tooltip: Option<&'static str>,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
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

    pub(super) fn inline_button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        self.action_button(id, label, ButtonTone::Inline, true, None, on_click, cx)
    }

    pub(super) fn setting_row<L: IntoElement>(&self, label: L, controls: Div) -> Div {
        h_flex()
            .flex_wrap()
            .w_full()
            .min_h(px(56.))
            .px_4()
            .py_3()
            .items_center()
            .gap_x_4()
            .gap_y_1()
            .child(div().flex_1().min_w(px(96.)).child(label))
            .child(
                h_flex()
                    .ml_auto()
                    .flex_shrink_0()
                    .justify_end()
                    .child(controls),
            )
    }

    pub(super) fn navigation_row<L: IntoElement + 'static>(
        &self,
        id: impl Into<ElementId>,
        label: L,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> CardItem {
        let id: ElementId = id.into();
        let on_click = Rc::new(on_click);
        let click_cb = on_click.clone();
        let key_cb = on_click;
        let hover_fill = cx.theme().accent.opacity(0.65);
        let press_fill = cx.theme().accent;
        let chevron_color = cx.theme().muted_foreground;
        let click_listener = cx.listener(move |p, _, window, cx| {
            click_cb(p, window, cx);
            cx.notify();
        });
        let key_listener = cx.listener(move |p, e: &KeyDownEvent, window, cx| {
            if matches!(e.keystroke.key.as_str(), "enter" | "space") {
                key_cb(p, window, cx);
                cx.stop_propagation();
                cx.notify();
            }
        });

        CardItem::Interactive(Box::new(move |corners| {
            let row = h_flex()
                .id(id.clone())
                .relative()
                .w_full()
                .min_h(px(56.))
                .px_4()
                .py_3()
                .items_center()
                .justify_between()
                .gap_x_4()
                .focusable()
                .tab_stop(true)
                .role(Role::Button);

            let row =
                pointer_feedback_corners((id, "nav-row-fb"), row, hover_fill, press_fill, corners)
                    .child(div().flex_1().min_w(px(96.)).child(label))
                    .child(chevron_right(chevron_color))
                    .on_click(click_listener)
                    .on_key_down(key_listener);

            div().w_full().child(row)
        }))
    }

    pub(super) fn setting_sections(
        &self,
        sections: impl IntoIterator<Item = Div>,
        _cx: &App,
    ) -> Div {
        v_flex().gap_3().children(sections)
    }

    pub(super) fn setting_sections_titled<'a>(
        &self,
        sections: impl IntoIterator<Item = (&'static str, Div)>,
        cx: &App,
    ) -> Div {
        v_flex()
            .gap_4()
            .children(sections.into_iter().map(|(title, section)| {
                v_flex()
                    .gap_1p5()
                    .child(section_heading(title, cx))
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
        let count = options.len();
        let measure = |text: &str| self.control_text_width(text, cx);
        let selected_width = measure(&selected);
        let widest = options
            .iter()
            .map(|(label, _)| measure(label))
            .fold(selected_width, f32::max);
        let (trigger_width, menu_width) =
            resolve_select_widths(selected_width, widest, width.into());
        let selected_tip = option_tip(
            &selected,
            selected_option.and_then(|(_, detail)| detail.as_deref()),
            selected_width + TRIGGER_LABEL_INSET > trigger_width,
        );
        let open = self
            .dropdown
            .as_ref()
            .filter(|menu| menu.id == id && count > 0);
        let is_open = open.is_some();
        let trigger_box = Rc::new(Cell::new(None::<Bounds<Pixels>>));
        let measure_trigger = trigger_box.clone();

        let radius = f32::from(cx.theme().radius);
        let popup_radius = f32::from(cx.theme().radius_lg);
        let open_border = cx.theme().ring;
        let outline_id = id.clone();
        let chevron_id = id.clone();

        let mut trigger = h_flex()
            .id(id.clone())
            .relative()
            .w(px(trigger_width))
            .h_8()
            .px_2p5()
            .flex_shrink_0()
            .justify_between()
            .gap_2()
            .rounded(cx.theme().radius)
            .contour_highlight(Some(cx.theme().button), Some(control_highlight(cx)), radius)
            .when(count == 0, |d| d.opacity(0.45))
            .child(
                canvas(
                    move |_, window, cx| {
                        smooth_value(
                            (outline_id.clone(), "select-open"),
                            if is_open { 1. } else { 0. },
                            Duration::from_millis(150),
                            window,
                            cx,
                        )
                    },
                    move |bounds, amount, window, _| {
                        paint_outline(bounds, open_border.opacity(amount), radius, window);
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
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
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .child(selected),
            )
            .child(animated_chevron(
                chevron_id,
                cx.theme().muted_foreground,
                is_open,
            ));

        if count > 0 && !is_open {
            trigger = pointer_feedback(
                (id.clone(), "select-fb"),
                trigger,
                cx.theme().secondary_hover,
                cx.theme().secondary_active,
                radius,
            );
        }

        let trigger = match (selected_tip, placeholder) {
            (Some(text), _) => trigger.tooltip(self.tooltip_owned(text.into())),
            (None, "选择显卡") => trigger.tooltip(self.tooltip("选择当前显卡")),
            (None, "选择显示器") => trigger.tooltip(self.tooltip("选择当前显示器")),
            (None, "选择应用程序") => trigger.tooltip(self.tooltip("选择当前应用程序")),
            _ => trigger,
        };
        let mut wrapper = div().relative().w(px(trigger_width)).flex_shrink_0();
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
            let popover_bg = cx.theme().popover;
            let popover_border = cx.theme().border;
            let popover_fg = cx.theme().popover_foreground;
            let accent = cx.theme().accent;
            let primary = cx.theme().primary;

            let mut list = v_flex()
                .id(ElementId::NamedChild(
                    std::sync::Arc::new(id.clone()),
                    "options".into(),
                ))
                .relative()
                .w(px(menu_width))
                .max_h(menu.max_height)
                .p(px(MENU_PADDING))
                .gap(px(ITEM_GAP))
                .occlude()
                .overflow_y_scroll()
                .text_color(popover_fg)
                .rounded(cx.theme().radius_lg)
                .contour(Some(popover_bg), Some(popover_border), popup_radius)
                .shadow_md()
                .on_mouse_down_out(cx.listener(move |p, event: &MouseDownEvent, _, cx| {
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
                    text_width + MENU_LABEL_INSET > menu_width,
                );
                let active = Some(i) == menu.keyboard_choice || current == Some(i);
                let row = h_flex()
                    .id(("option", i))
                    .relative()
                    .h(px(ITEM_HEIGHT))
                    .flex_shrink_0()
                    .px_2()
                    .gap_2()
                    .items_center()
                    .when(active, |d| d.contour(Some(accent), None, 4.));

                let row = pointer_feedback(
                    ElementId::NamedChild(
                        std::sync::Arc::new(id.clone()),
                        SharedString::from(i.to_string()),
                    ),
                    row,
                    accent.opacity(0.65),
                    accent,
                    4.,
                );

                list = list.child(
                    row.child(
                        div()
                            .id(("option-label", i))
                            .flex_1()
                            .min_w_0()
                            .text_sm()
                            .truncate()
                            .when_some(tip, |label, text| {
                                label.tooltip(self.tooltip_owned(text.into()))
                            })
                            .child(label),
                    )
                    .child(
                        h_flex()
                            .size(px(14.))
                            .flex_shrink_0()
                            .justify_center()
                            .when(current == Some(i), |indicator| {
                                indicator.child(select_check(primary))
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
            let (anchor, y) = if menu.above {
                (Anchor::BottomLeft, trigger.top() - px(MENU_OFFSET))
            } else {
                (Anchor::TopLeft, trigger.bottom() + px(MENU_OFFSET))
            };
            let offset_y = if menu.above { 4. } else { -4. };
            wrapper = wrapper.child(
                deferred(
                    anchored()
                        .anchor(anchor)
                        .position(point(trigger.left(), y))
                        .snap_to_window()
                        .child(
                            list.with_animation(
                                ElementId::NamedChild(std::sync::Arc::new(id), "open".into()),
                                Animation::new(Duration::from_millis(140))
                                    .with_easing(ease_out_quint()),
                                move |elem, t| elem.opacity(t).top(px(offset_y * (1. - t))),
                            ),
                        ),
                )
                .with_priority(3),
            );
        }
        wrapper
    }
}

fn resolve_select_widths(selected_width: f32, widest_option: f32, spec: SelectWidth) -> (f32, f32) {
    match spec {
        SelectWidth::Fit(max) => {
            let width = (widest_option + TRIGGER_LABEL_INSET).clamp(84., max);
            (width, width)
        }
        SelectWidth::FitSelected(max) => {
            let trigger = (selected_width + TRIGGER_LABEL_INSET).clamp(84., max);
            let menu = trigger.max((widest_option + MENU_LABEL_INSET).clamp(84., max));
            (trigger, menu)
        }
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

fn select_check(color: Hsla) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let at = |x, y| bounds.origin + point(px(x), px(y));
            let mut path = PathBuilder::stroke(px(1.5));
            path.move_to(at(2.5, 7.));
            path.line_to(at(5.5, 10.));
            path.line_to(at(11.5, 4.));
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size_full()
}

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
    let slide_id = id.clone();
    let on_id = id.clone();
    let slot = Rc::new(RefCell::new(None::<gpui_kit::Entity<SwitchPointerState>>));
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

    let set_press = |slot: &RefCell<Option<gpui_kit::Entity<SwitchPointerState>>>,
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
        .when(!enabled, |d| d.opacity(0.45))
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
                        Duration::from_millis(120),
                        window,
                        cx,
                    );
                    let press = smooth_value(
                        (press_id.clone(), "sw-press"),
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
                    (hover, press, slide.clamp(0., 1.), on_mix)
                },
                move |bounds, (hover, press, slide, on_mix), window, _| {
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
                        track_bg,
                        px(1.0),
                        track_border,
                        BorderStyle::Solid,
                    ));

                    let thumb_d = 12.;
                    let pad = (f32::from(bounds.size.height) - thumb_d) / 2.;
                    let travel = (f32::from(bounds.size.width) - 2. * pad - thumb_d).max(0.);
                    let thumb_x = f32::from(bounds.origin.x) + pad + travel * slide;
                    let thumb_y = f32::from(bounds.origin.y) + pad;
                    let thumb_color = off_thumb
                        .blend(off_thumb_hover.opacity(hover))
                        .blend(on_thumb.opacity(on_mix));
                    window.paint_quad(
                        fill(
                            Bounds::new(
                                point(px(thumb_x), px(thumb_y)),
                                size(px(thumb_d), px(thumb_d)),
                            ),
                            thumb_color,
                        )
                        .corner_radii(px(thumb_d / 2.)),
                    );
                },
            )
            .size_full(),
        );

    if enabled {
        switch = switch
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
            .on_click(move |_, _, cx| on_toggle(cx));
    }
    switch
}

#[cfg(test)]
mod tests {
    use super::{Bounds, SelectWidth, point, popup_space, px, resolve_select_widths};
    use gpui_kit::size;

    #[test]
    fn popup_opens_above_a_trigger_near_the_bottom() {
        let trigger = Bounds::new(point(px(100.), px(628.)), size(px(90.), px(32.)));
        let (above, height) = popup_space(trigger, px(720.), 6);
        assert!(above);
        assert_eq!(height, px(207.));
    }

    #[test]
    fn select_widths_fit_gpu_and_cjk_labels_and_keep_global_compact() {
        // "全局" (~31px) fits in a compact trigger while popup widens for long app names.
        let (global_trigger, global_menu) =
            resolve_select_widths(31., 230., SelectWidth::FitSelected(340.));
        assert!((84.0..=96.0).contains(&global_trigger));
        assert!(global_menu >= 286.);

        // "跟随系统" (4 CJK chars * 14.5 * 1.06 ~= 61.5px) must never clip inside trigger or menu.
        let (theme_trigger, theme_menu) = resolve_select_widths(61.5, 61.5, SelectWidth::Fit(340.));
        assert!(theme_trigger >= 115.);
        assert_eq!(theme_menu, theme_trigger);

        // Long GPU name such as "AMD Radeon RX 9070 XT" (~175px) fits without ellipsis.
        let (gpu_trigger, gpu_menu) =
            resolve_select_widths(175., 175., SelectWidth::FitSelected(440.));
        assert!(gpu_trigger >= 230.);
        assert_eq!(gpu_menu, gpu_trigger);
    }
}
