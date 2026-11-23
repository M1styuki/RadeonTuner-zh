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
    // Both controls sit at the corner. Curvature reaches zero at each
    // endpoint, matching the straight edge on both sides of the join.
    path.move_to(point(left + radius, top));
    path.line_to(point(right - radius, top));
    path.cubic_bezier_to(
        point(right, top + radius),
        point(right, top),
        point(right, top),
    );
    path.line_to(point(right, bottom - radius));
    path.cubic_bezier_to(
        point(right - radius, bottom),
        point(right, bottom),
        point(right, bottom),
    );
    path.line_to(point(left + radius, bottom));
    path.cubic_bezier_to(
        point(left, bottom - radius),
        point(left, bottom),
        point(left, bottom),
    );
    path.line_to(point(left, top + radius));
    path.cubic_bezier_to(
        point(left + radius, top),
        point(left, top),
        point(left, top),
    );
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
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            if let Ok(path) =
                draw(bounds.dilate(px(-0.5)), 7.5, PathBuilder::stroke(px(1.))).build()
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
