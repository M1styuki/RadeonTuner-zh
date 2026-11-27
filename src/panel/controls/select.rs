use super::{Panel, animated_chevron, control_highlight};
use crate::panel::{
    motion::pointer_feedback,
    shape::{ContinuousCorners as _, focus_ring_layer},
};
use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::{
    Anchor, Animation, AnimationExt as _, Bounds, Context, Div, ElementId, Hsla,
    InteractiveElement as _, IntoElement, KeyDownEvent, MouseDownEvent, ParentElement as _,
    PathBuilder, Pixels, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
    anchored, canvas, deferred, div, ease_out_quint, point, prelude::FluentBuilder as _, px,
};
use std::{cell::Cell, rc::Rc, time::Duration};

pub(in crate::panel) const SELECT_MAX_WIDTH: f32 = 340.;
pub(in crate::panel) const PROFILE_SELECT_WIDTH: SelectWidth = SelectWidth::FitSelected(340.);

pub(in crate::panel) enum SelectWidth {
    Fit(f32),
    FitSelected(f32),
}

impl From<f32> for SelectWidth {
    fn from(max: f32) -> Self {
        Self::Fit(max)
    }
}

pub(in crate::panel) struct Dropdown {
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
    pub(in crate::panel) fn select(
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

    pub(in crate::panel) fn select_with_details(
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
            .contour_fill(cx.theme().button, radius)
            .when(count == 0, |d| d.opacity(0.45));

        if count > 0 && !is_open {
            trigger = pointer_feedback(
                (id.clone(), "select-fb"),
                trigger,
                cx.theme().secondary_hover,
                cx.theme().secondary_active,
                radius,
            );
        }

        let trigger = trigger
            .ring_highlight(control_highlight(cx), radius)
            .child(focus_ring_layer(
                (outline_id, "select-open"),
                is_open,
                open_border,
                radius,
                Duration::from_millis(150),
            ))
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
        let (global_trigger, global_menu) =
            resolve_select_widths(31., 230., SelectWidth::FitSelected(340.));
        assert!((84.0..=96.0).contains(&global_trigger));
        assert!(global_menu >= 286.);

        let (theme_trigger, theme_menu) = resolve_select_widths(61.5, 61.5, SelectWidth::Fit(340.));
        assert!(theme_trigger >= 115.);
        assert_eq!(theme_menu, theme_trigger);

        let (gpu_trigger, gpu_menu) =
            resolve_select_widths(175., 175., SelectWidth::FitSelected(440.));
        assert!(gpu_trigger >= 230.);
        assert_eq!(gpu_menu, gpu_trigger);
    }
}
