use super::{Panel, chevron_right};
use crate::panel::{motion::pointer_feedback_corners, shape::ContinuousCorners as _};
use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::{
    App, Context, Corners, Div, ElementId, FontWeight, InteractiveElement as _, IntoElement,
    KeyDownEvent, ParentElement as _, Pixels, Role, StatefulInteractiveElement as _, Styled as _,
    Window, div, px,
};
use std::rc::Rc;

pub(in crate::panel) fn section_heading(title: &'static str, cx: &App) -> Div {
    div()
        .px_1()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(cx.theme().muted_foreground)
        .child(title)
}

pub(in crate::panel) fn section_card(cx: &App) -> Div {
    let radius = f32::from(cx.theme().radius_lg);
    v_flex()
        .relative()
        .w_full()
        .overflow_hidden()
        // `.rounded` shapes only `.shadow_xs()`; surface fill uses continuous corners.
        .rounded(cx.theme().radius_lg)
        .shadow_xs()
        .contour_fill(cx.theme().group_box, radius)
}

pub(in crate::panel) enum CardItem {
    Static(Div),
    Interactive(Box<dyn FnOnce(Corners<Pixels>) -> Div>),
}

impl From<Div> for CardItem {
    fn from(row: Div) -> Self {
        Self::Static(row)
    }
}

pub(in crate::panel) fn card_rows<I: Into<CardItem>>(
    rows: impl IntoIterator<Item = I>,
    cx: &App,
) -> Div {
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
            // Keep `.rounded*()` solely to shape `.shadow_xs()`; tile fill uses continuous corners.
            let tile = div()
                .relative()
                .w_full()
                .overflow_hidden()
                .shadow_xs()
                .contour_fill_corners(surface, corners)
                .child(row.w_full());
            match (top, bottom) {
                (true, true) => tile.rounded(radius),
                (true, false) => tile.rounded_t(radius),
                (false, true) => tile.rounded_b(radius),
                (false, false) => tile,
            }
        }))
}

impl Panel {
    pub(in crate::panel) fn setting_row<L: IntoElement>(&self, label: L, controls: Div) -> Div {
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

    pub(in crate::panel) fn navigation_row<L: IntoElement + 'static>(
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

    pub(in crate::panel) fn setting_sections(
        &self,
        sections: impl IntoIterator<Item = Div>,
        _cx: &App,
    ) -> Div {
        v_flex().gap_3().children(sections)
    }

    pub(in crate::panel) fn setting_sections_titled<'a>(
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
}
