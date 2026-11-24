use super::{
    Panel,
    layout::load_icon,
    shape::{self, ContinuousCorners},
    theme::Palette,
};
use gpui::{
    AnyView, App, Context, Div, Render, RenderImage, SharedString, Window, div, img, prelude::*,
    px, rgb,
};
use std::sync::{Arc, OnceLock};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Information,
    Warning,
    Critical,
    Experimental,
    Administrator,
}

impl Kind {
    fn icon(self, light: bool) -> Option<Arc<RenderImage>> {
        macro_rules! image {
            ($name:literal) => {
                include_bytes!(concat!("../../RadeonTuner/", $name, ".png")) as &[u8]
            };
        }
        static ICONS: OnceLock<[[Option<Arc<RenderImage>>; 5]; 2]> = OnceLock::new();
        let icons = ICONS.get_or_init(|| {
            [
                [
                    image!("Information"),
                    image!("WarningOrange"),
                    image!("WarningRed"),
                    image!("Experimental"),
                    image!("Shield"),
                ],
                [
                    image!("InformationDark"),
                    image!("WarningOrange"),
                    image!("WarningRed"),
                    image!("ExperimentalDark"),
                    image!("ShieldDark"),
                ],
            ]
            .map(|theme| theme.map(load_icon))
        });
        icons[usize::from(light)][self as usize].clone()
    }
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
        let hints = native_hints(source);
        if hints.is_empty() {
            return label;
        }
        div()
            .flex()
            .items_center()
            .gap_1()
            .child(label.min_w_0())
            .children(hints.iter().enumerate().filter_map(|(i, &(kind, text))| {
                if (kind == Kind::Administrator && self.admin)
                    || (kind == Kind::Experimental && !self.experimental)
                {
                    return None;
                }
                let icon = kind.icon(self.palette.light)?;
                Some(
                    div()
                        .id((source, i))
                        .size(px(16.))
                        .flex_shrink_0()
                        .child(img(icon).size_full())
                        .tooltip(self.tooltip(text)),
                )
            }))
    }
}

include!("tooltips_native.rs");
