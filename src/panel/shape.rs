use gpui::{Bounds, ParentElement, PathBuilder, Pixels, Styled, canvas, point, px, rgb};

pub(super) trait ContinuousCorners: Styled + ParentElement + Sized {
    fn smooth_background(self, color: u32) -> Self {
        self.relative().child(surface_radius(color, 8.))
    }
}

impl<T: Styled + ParentElement> ContinuousCorners for T {}

fn draw(bounds: Bounds<Pixels>, radius: f32, mut path: PathBuilder) -> PathBuilder {
    let left = bounds.origin.x;
    let top = bounds.origin.y;
    let right = bounds.right();
    let bottom = bounds.bottom();
    let radius = [
        px(radius),
        bounds.size.width * 0.5,
        bounds.size.height * 0.5,
    ]
    .into_iter()
    .reduce(|a, b| if a < b { a } else { b })
    .unwrap_or_default();
    // Each pair of cubics is mirrored at the diagonal. The first two
    // control points lie on the incoming edge and the last two on the
    // outgoing edge, so curvature reaches zero at both straight joins.
    path.move_to(point(left + radius, top));
    let mut corner = |x, y, dx, dy| {
        let at = |along: f32, across: f32| {
            point(x + dx * along - dy * across, y + dy * along + dx * across)
        };
        path.line_to(at(0., 0.));
        path.cubic_bezier_to(at(0.72, 0.28), at(0.22, 0.), at(0.44, 0.));
        path.cubic_bezier_to(at(1., 1.), at(1., 0.56), at(1., 0.78));
    };
    corner(right - radius, top, radius, px(0.));
    corner(right, bottom - radius, px(0.), radius);
    corner(left + radius, bottom, -radius, px(0.));
    corner(left, top + radius, px(0.), -radius);
    path.close();
    path
}

pub(super) fn surface(color: u32) -> impl gpui::IntoElement {
    surface_radius(color, 8.)
}

pub(super) fn surface_radius(color: u32, radius: f32) -> impl gpui::IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            if let Ok(path) = draw(bounds, radius, PathBuilder::fill()).build() {
                window.paint_path(path, rgb(color));
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

pub(super) fn outline(color: u32) -> impl gpui::IntoElement {
    outline_radius(color, 8.)
}

pub(super) fn outline_radius(color: u32, radius: f32) -> impl gpui::IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            if let Ok(path) = draw(
                bounds.dilate(px(-0.5)),
                radius - 0.5,
                PathBuilder::stroke(px(1.)),
            )
            .build()
            {
                window.paint_path(path, rgb(color));
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}
