use gpui::{
    Animation, AnimationExt, AnyElement, App, Bounds, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Styled, Window,
};
use std::time::{Duration, Instant};

const DURATION: Duration = Duration::from_millis(210);

fn spring(t: f32) -> f32 {
    let t = t.clamp(0., 1.);
    let decay = 7. * t;
    (1. - (1. + decay) * (-decay).exp()) / (1. - 8. * (-7_f32).exp())
}

pub(super) fn enabled() -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SPI_GETCLIENTAREAANIMATION, SystemParametersInfoW,
    };
    let mut animate = 1i32;
    unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            std::ptr::from_mut(&mut animate).cast(),
            0,
        );
    }
    animate != 0
}

pub(super) fn reveal<E>(element: E, id: impl Into<ElementId>, enabled: bool) -> AnyElement
where
    E: IntoElement + Styled + 'static,
{
    if enabled {
        element
            .with_animation(
                id,
                Animation::new(DURATION).with_easing(spring),
                |elem, t| elem.opacity(0.55 + 0.45 * t),
            )
            .into_any_element()
    } else {
        element.into_any_element()
    }
}

struct State {
    from: f32,
    to: f32,
    changed: Instant,
}

impl State {
    fn stationary(value: f32) -> Self {
        Self {
            from: value,
            to: value,
            changed: Instant::now(),
        }
    }

    fn sample(&self, now: Instant) -> f32 {
        let t = now.duration_since(self.changed).as_secs_f32() / DURATION.as_secs_f32();
        self.from + (self.to - self.from) * spring(t)
    }
}

pub(super) fn transition<E: IntoElement + 'static>(
    id: impl Into<ElementId>,
    target: f32,
    enabled: bool,
    draw: impl FnOnce(f32) -> E + 'static,
) -> AnyElement {
    if !enabled {
        return draw(target).into_any_element();
    }
    Transition {
        id: id.into(),
        target,
        draw: Some(Box::new(move |value| draw(value).into_any_element())),
    }
    .into_any_element()
}

struct Transition {
    id: ElementId,
    target: f32,
    draw: Option<Box<dyn FnOnce(f32) -> AnyElement>>,
}

impl IntoElement for Transition {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for Transition {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        window.with_element_state(
            id.expect("transition has an ID"),
            |state: Option<State>, window| {
                let mut state = state.unwrap_or_else(|| State::stationary(self.target));
                let now = Instant::now();
                if state.to != self.target {
                    state = State {
                        from: state.sample(now),
                        to: self.target,
                        changed: now,
                    };
                }
                let value = state.sample(now);
                if state.from != state.to && now.duration_since(state.changed) < DURATION {
                    window.request_animation_frame();
                }
                let mut child = self.draw.take().expect("transition draws once")(value);
                ((child.request_layout(window, cx), child), state)
            },
        )
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        child: &mut AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) {
        child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        child: &mut AnyElement,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        child.paint(window, cx);
    }
}

pub(super) fn mix(a: u32, b: u32, t: f32) -> u32 {
    [0u32, 8, 16].into_iter().fold(0, |color, shift| {
        let from = ((a >> shift) & 0xff) as f32;
        let to = ((b >> shift) & 0xff) as f32;
        color | (((from + (to - from) * t).round() as u32) << shift)
    })
}
