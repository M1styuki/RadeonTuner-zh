use super::Panel;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::{
    AnyView, App, Div, InteractiveElement as _, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Information,
    Warning,
    Critical,
    Experimental,
    Administrator,
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
        move |window, cx| Tooltip::new(text.clone()).build(window, cx)
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
