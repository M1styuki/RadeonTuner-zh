use super::motion::smooth_value;
use gpui_kit::{
    AnyElement, Bounds, ElementId, Hsla, IntoElement, ParentElement, PathBuilder, Pixels, Styled,
    Window, canvas, fill, point, px,
};
use std::time::Duration;

fn path_corners(
    bounds: Bounds<Pixels>,
    corners: gpui_kit::Corners<Pixels>,
    mut path: PathBuilder,
) -> PathBuilder {
    let left = f32::from(bounds.origin.x);
    let top = f32::from(bounds.origin.y);
    let right = f32::from(bounds.right());
    let bottom = f32::from(bounds.bottom());
    let w = (right - left).max(0.);
    let h = (bottom - top).max(0.);

    let mut tl = (f32::from(corners.top_left) * 1.65).max(0.);
    let mut tr = (f32::from(corners.top_right) * 1.65).max(0.);
    let mut br = (f32::from(corners.bottom_right) * 1.65).max(0.);
    let mut bl = (f32::from(corners.bottom_left) * 1.65).max(0.);

    let mut scale = 1.0f32;
    if tl + tr > w && tl + tr > 0. {
        scale = scale.min(w / (tl + tr));
    }
    if bl + br > w && bl + br > 0. {
        scale = scale.min(w / (bl + br));
    }
    if tl + bl > h && tl + bl > 0. {
        scale = scale.min(h / (tl + bl));
    }
    if tr + br > h && tr + br > 0. {
        scale = scale.min(h / (tr + br));
    }
    tl *= scale;
    tr *= scale;
    br *= scale;
    bl *= scale;

    let corner = |x: f32,
                  y: f32,
                  dx: f32,
                  dy: f32,
                  sharp_x: f32,
                  sharp_y: f32,
                  r: f32,
                  path: &mut PathBuilder,
                  start: bool| {
        if r <= 0. {
            let p = point(px(sharp_x), px(sharp_y));
            if start {
                path.move_to(p);
            } else {
                path.line_to(p);
            }
            return;
        }
        let at = |along: f32, across: f32| {
            point(
                px(x + dx * along - dy * across),
                px(y + dy * along + dx * across),
            )
        };
        if start {
            path.move_to(at(0., 0.));
        } else {
            path.line_to(at(0., 0.));
        }
        path.cubic_bezier_to(at(0.8, 0.2), at(0.275, 0.), at(0.6, 0.));
        path.cubic_bezier_to(at(1., 1.), at(1., 0.4), at(1., 0.725));
    };
    corner(right - tr, top, tr, 0., right, top, tr, &mut path, true);
    corner(
        right,
        bottom - br,
        0.,
        br,
        right,
        bottom,
        br,
        &mut path,
        false,
    );
    corner(
        left + bl,
        bottom,
        -bl,
        0.,
        left,
        bottom,
        bl,
        &mut path,
        false,
    );
    corner(left, top + tl, 0., -tl, left, top, tl, &mut path, false);
    path.close();
    path
}

fn path(bounds: Bounds<Pixels>, radius: f32, path: PathBuilder) -> PathBuilder {
    path_corners(bounds, gpui_kit::Corners::all(px(radius)), path)
}

pub fn paint_fill(bounds: Bounds<Pixels>, color: Hsla, radius: f32, window: &mut Window) {
    if color.a == 0. || bounds.size.width <= px(0.) || bounds.size.height <= px(0.) {
        return;
    }
    if radius <= 0. {
        window.paint_quad(fill(bounds, color));
    } else if let Ok(shape) = path(bounds, radius, PathBuilder::fill()).build() {
        window.paint_path(shape, color);
    }
}

pub fn paint_fill_corners(
    bounds: Bounds<Pixels>,
    color: Hsla,
    corners: gpui_kit::Corners<Pixels>,
    window: &mut Window,
) {
    if color.a == 0. || bounds.size.width <= px(0.) || bounds.size.height <= px(0.) {
        return;
    }
    if corners.top_left <= px(0.)
        && corners.top_right <= px(0.)
        && corners.bottom_right <= px(0.)
        && corners.bottom_left <= px(0.)
    {
        window.paint_quad(fill(bounds, color));
    } else if let Ok(shape) = path_corners(bounds, corners, PathBuilder::fill()).build() {
        window.paint_path(shape, color);
    }
}

pub const HIGHLIGHT_STROKE: f32 = 0.65;
const CAD_OFFSET_FACTOR: f32 = 0.887_555_2;

pub fn offset_radius(outer_radius: f32, inset: f32) -> f32 {
    (outer_radius - inset * CAD_OFFSET_FACTOR).max(0.)
}

pub fn paint_outline(bounds: Bounds<Pixels>, color: Hsla, radius: f32, window: &mut Window) {
    if color.a > 0.
        && bounds.size.width > px(1.)
        && bounds.size.height > px(1.)
        && let Ok(shape) = path(
            bounds.dilate(px(-0.5)),
            offset_radius(radius, 0.5),
            PathBuilder::stroke(px(1.)),
        )
        .build()
    {
        window.paint_path(shape, color);
    }
}

#[derive(Clone, Copy)]
pub struct HighlightRing {
    pub top: Hsla,
    pub rest: Hsla,
    pub stop: f32,
}

pub fn paint_ring_highlight(
    bounds: Bounds<Pixels>,
    ring: HighlightRing,
    radius: f32,
    window: &mut Window,
) {
    if (ring.top.a <= 0. && ring.rest.a <= 0.)
        || bounds.size.width <= px(2.)
        || bounds.size.height <= px(2.)
    {
        return;
    }
    let inset = HIGHLIGHT_STROKE * 0.5;
    if let Ok(shape) = path(
        bounds.dilate(px(-inset)),
        offset_radius(radius, inset),
        PathBuilder::stroke(px(HIGHLIGHT_STROKE)),
    )
    .build()
    {
        let sheen = gpui_kit::linear_gradient(
            180.,
            gpui_kit::linear_color_stop(ring.top, 0.0),
            gpui_kit::linear_color_stop(ring.rest, ring.stop),
        );
        window.paint_path(shape, sheen);
    }
}

pub fn layer(fill: Option<Hsla>, stroke: Option<Hsla>, radius: f32) -> AnyElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            if let Some(fill) = fill {
                paint_fill(bounds, fill, radius, window);
            }
            if let Some(stroke) = stroke {
                paint_outline(bounds, stroke, radius, window);
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
    .into_any_element()
}

pub fn fill_layer(fill: Hsla, radius: f32) -> AnyElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            paint_fill(bounds, fill, radius, window);
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
    .into_any_element()
}

pub fn fill_corners_layer(fill: Hsla, corners: gpui_kit::Corners<Pixels>) -> AnyElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            paint_fill_corners(bounds, fill, corners, window);
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
    .into_any_element()
}

pub fn ring_layer(highlight: HighlightRing, radius: f32) -> AnyElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            paint_ring_highlight(bounds, highlight, radius, window);
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
    .into_any_element()
}

pub fn focus_ring_layer(
    id: impl Into<ElementId>,
    active: bool,
    color: Hsla,
    radius: f32,
    duration: Duration,
) -> AnyElement {
    let id: ElementId = id.into();
    canvas(
        move |_, window, cx| {
            smooth_value(
                id.clone(),
                if active { 1. } else { 0. },
                duration,
                window,
                cx,
            )
        },
        move |bounds, amount, window, _| {
            paint_outline(bounds, color.opacity(amount), radius, window);
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
    .into_any_element()
}

pub trait ContinuousCorners: Styled + ParentElement + Sized {
    fn contour(self, fill: Option<Hsla>, stroke: Option<Hsla>, radius: f32) -> Self {
        self.child(layer(fill, stroke, radius))
    }

    fn contour_fill(self, fill: Hsla, radius: f32) -> Self {
        self.child(fill_layer(fill, radius))
    }

    fn contour_fill_corners(self, fill: Hsla, corners: gpui_kit::Corners<Pixels>) -> Self {
        self.child(fill_corners_layer(fill, corners))
    }

    fn ring_highlight(self, highlight: HighlightRing, radius: f32) -> Self {
        self.child(ring_layer(highlight, radius))
    }
}

impl<T: Styled + ParentElement> ContinuousCorners for T {}

#[cfg(test)]
mod tests {
    use super::{offset_radius, path_corners};
    use gpui_kit::{Bounds, Corners, PathBuilder, point, px, size};

    #[test]
    fn offset_radius_keeps_exact_equidistant_diagonal_gap() {
        for (outer, inset) in [(10.0_f32, 5.0_f32), (5.0, 3.0), (5.0, 0.325)] {
            let inner = offset_radius(outer, inset);
            let diag_outer = 0.2 * std::f32::consts::SQRT_2 * 1.65 * outer;
            let diag_inner =
                inset * std::f32::consts::SQRT_2 + 0.2 * std::f32::consts::SQRT_2 * 1.65 * inner;
            assert!(((diag_inner - diag_outer) - inset).abs() < 1e-4);
        }
        assert_eq!(offset_radius(2.0, 5.0), 0.0);
    }

    #[test]
    fn path_corners_builds_for_uniform_top_bottom_and_narrow_rects() {
        let standard = Bounds::new(point(px(0.), px(0.)), size(px(240.), px(56.)));
        let narrow = Bounds::new(point(px(0.), px(0.)), size(px(12.), px(8.)));
        let r = px(10.);
        let cases = [
            (standard, Corners::all(r)),
            (
                standard,
                Corners {
                    top_left: r,
                    top_right: r,
                    bottom_right: px(0.),
                    bottom_left: px(0.),
                },
            ),
            (
                standard,
                Corners {
                    top_left: px(0.),
                    top_right: px(0.),
                    bottom_right: r,
                    bottom_left: r,
                },
            ),
            (narrow, Corners::all(r)),
        ];
        for (bounds, corners) in cases {
            assert!(
                path_corners(bounds, corners, PathBuilder::fill())
                    .build()
                    .is_ok()
            );
        }
    }
}
