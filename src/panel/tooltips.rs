use super::{
    Panel,
    shape::{self, ContinuousCorners},
    theme::Palette,
};
use gpui::{AnyView, App, Context, Div, Render, SharedString, Window, div, prelude::*, px, rgb};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Information,
    Warning,
    Critical,
    Experimental,
    Administrator,
}

struct Tooltip {
    text: SharedString,
    palette: Palette,
}

impl Render for Tooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .max_w(px(366.))
            .px(px(10.))
            .py(px(7.))
            .text_size(px(12.))
            .font_family("Microsoft YaHei UI")
            .text_color(rgb(self.palette.text))
            .smooth_background(self.palette.control)
            .child(shape::outline(self.palette.border))
            .child(self.text.clone())
    }
}

impl Panel {
    pub(super) fn tooltip(
        &self,
        text: &'static str,
    ) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static + use<> {
        self.tooltip_owned(text.into())
    }

    pub(super) fn tooltip_owned(
        &self,
        text: SharedString,
    ) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static + use<> {
        let palette = self.palette;
        move |_, cx| {
            cx.new(|_| Tooltip {
                text: text.clone(),
                palette,
            })
            .into()
        }
    }

    pub(super) fn hint_label(&self, label: &'static str) -> Div {
        self.with_native_hints(div().child(label), label)
    }

    pub(super) fn with_native_hints(&self, label: Div, source: &'static str) -> Div {
        let hints = native_hints(source)
            .iter()
            .filter(|(kind, _)| match kind {
                Kind::Administrator => !self.admin,
                Kind::Experimental => self.experimental,
                _ => true,
            })
            .map(|&(_, text)| text)
            .collect::<Vec<_>>()
            .join("\n\n");
        if hints.is_empty() {
            return label;
        }
        div().flex().items_center().min_w_0().child(
            label
                .id(source)
                .min_w_0()
                .tooltip(self.tooltip_owned(hints.into())),
        )
    }
}

include!("tooltips_native.rs");
