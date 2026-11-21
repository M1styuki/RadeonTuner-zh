use super::Panel;
use gpui::{Context, Div, PathBuilder, canvas, div, point, prelude::*, px, rgb};

impl Panel {
    pub(super) fn render_curve(&self, _cx: &mut Context<Self>) -> Div {
        let palette = self.palette;
        let points = self.curve_points();
        let mut section = div().flex().flex_col().gap_1();
        if points.len() != 5 {
            return section;
        }
        section = section.child("风扇转速（%）");
        let plot = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let coord = |x: f32, y: f32| {
                    point(
                        bounds.origin.x + bounds.size.width * x,
                        bounds.origin.y + bounds.size.height * (1. - y),
                    )
                };
                let mut grid = PathBuilder::stroke(px(1.));
                for i in 0..=10 {
                    let f = i as f32 / 10.;
                    grid.move_to(coord(f, 0.));
                    grid.line_to(coord(f, 1.));
                    grid.move_to(coord(0., f));
                    grid.line_to(coord(1., f));
                }
                if let Ok(path) = grid.build() {
                    window.paint_path(path, rgb(palette.grid));
                }
                let mut line = PathBuilder::stroke(px(2.5));
                for (i, (temp, speed)) in points.iter().enumerate() {
                    let p = coord(
                        (*temp as f32 / 100.).clamp(0., 1.),
                        (*speed as f32 / 100.).clamp(0., 1.),
                    );
                    if i == 0 {
                        line.move_to(p);
                    } else {
                        line.line_to(p);
                    }
                }
                if let Ok(path) = line.build() {
                    window.paint_path(path, rgb(palette.accent));
                }
            },
        )
        .w_full()
        .h(px(145.));
        section
            .child(
                div()
                    .border_1()
                    .border_color(rgb(palette.border))
                    .child(plot),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child("0 °C")
                    .child("50 °C")
                    .child("100 °C"),
            )
    }
}
