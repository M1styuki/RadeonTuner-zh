use gpui::{
    Bounds, InteractiveElement, ParentElement, PathBuilder, Pixels, Styled, canvas, div, point, px,
    rgb,
};

pub(super) const OUTLINE_WIDTH: f32 = 1.5;
pub(super) const CONTROL_CORNER: f32 = 11.;
pub(super) const PANEL_CORNER: f32 = 15.;
pub(super) const NAV_CORNER: f32 = 18.;

pub(super) trait ContinuousCorners: Styled + ParentElement + Sized {
    fn smooth_background(self, color: u32) -> Self {
        self.relative().child(surface_radius(color, PANEL_CORNER))
    }

    fn smooth_hover_background(self, normal: Option<u32>, hover: u32, radius: f32) -> Self
    where
        Self: InteractiveElement,
    {
        let layer = || div().absolute().top_0().left_0().size_full();
        self.relative()
            .group("continuous-hover")
            .children(normal.map(|color| {
                layer()
                    .group_hover("continuous-hover", |style| style.opacity(0.))
                    .child(surface_radius(color, radius))
            }))
            .child(
                layer()
                    .opacity(0.)
                    .group_hover("continuous-hover", |layer| layer.opacity(1.))
                    .child(surface_radius(hover, radius)),
            )
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
    // The first two handles sit on the incoming edge, giving it zero
    // curvature at the join. Reflection gives the other edge the same
    // join and matches tangent and curvature at the corner's diagonal.
    path.move_to(point(left + radius, top));
    let mut corner = |x, y, dx, dy| {
        let at = |along: f32, across: f32| {
            point(x + dx * along - dy * across, y + dy * along + dx * across)
        };
        path.line_to(at(0., 0.));
        path.cubic_bezier_to(at(0.8, 0.2), at(0.275, 0.), at(0.6, 0.));
        path.cubic_bezier_to(at(1., 1.), at(1., 0.4), at(1., 0.725));
    };
    corner(right - radius, top, radius, px(0.));
    corner(right, bottom - radius, px(0.), radius);
    corner(left + radius, bottom, -radius, px(0.));
    corner(left, top + radius, px(0.), -radius);
    path.close();
    path
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
    outline_radius(color, PANEL_CORNER)
}

pub(super) fn outline_radius(color: u32, radius: f32) -> impl gpui::IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let inset = OUTLINE_WIDTH / 2.;
            if let Ok(path) = draw(
                bounds.dilate(px(-inset)),
                radius - inset,
                PathBuilder::stroke(px(OUTLINE_WIDTH)),
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
