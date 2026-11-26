use gpui_kit::{
    AnyElement, Bounds, Hsla, IntoElement, ParentElement, PathBuilder, Pixels, Styled, Window,
    canvas, fill, point, px,
};

fn path(bounds: Bounds<Pixels>, radius: f32, mut path: PathBuilder) -> PathBuilder {
    let left = f32::from(bounds.origin.x);
    let top = f32::from(bounds.origin.y);
    let right = f32::from(bounds.right());
    let bottom = f32::from(bounds.bottom());
    let r = (radius * 1.65)
        .min((right - left) / 2.)
        .min((bottom - top) / 2.)
        .max(0.);
    let corner = |x: f32, y: f32, dx: f32, dy: f32, path: &mut PathBuilder, start: bool| {
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
    corner(right - r, top, r, 0., &mut path, true);
    corner(right, bottom - r, 0., r, &mut path, false);
    corner(left + r, bottom, -r, 0., &mut path, false);
    corner(left, top + r, 0., -r, &mut path, false);
    path.close();
    path
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
    window.paint_quad(fill(bounds, color).corner_radii(corners));
}

pub fn paint_outline(bounds: Bounds<Pixels>, color: Hsla, radius: f32, window: &mut Window) {
    if color.a > 0.
        && bounds.size.width > px(1.)
        && bounds.size.height > px(1.)
        && let Ok(shape) = path(
            bounds.dilate(px(-0.5)),
            (radius - 0.5).max(0.),
            PathBuilder::stroke(px(1.)),
        )
        .build()
    {
        window.paint_path(shape, color);
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

pub trait ContinuousCorners: Styled + ParentElement + Sized {
    fn contour(self, fill: Option<Hsla>, stroke: Option<Hsla>, radius: f32) -> Self {
        self.child(layer(fill, stroke, radius))
    }
}

impl<T: Styled + ParentElement> ContinuousCorners for T {}
