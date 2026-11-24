use super::{Panel, tuning::FanCurve};
use gpui::{
    Bounds, Context, Div, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PathBuilder,
    Pixels, Point, canvas, div, fill, point, prelude::*, px, relative, rgb, size,
};
use std::{cell::Cell, rc::Rc};

const PLOT_HEIGHT: f32 = 150.;

fn plot_point(bounds: Bounds<Pixels>, temperature: i32, speed: i32) -> Point<Pixels> {
    point(
        bounds.origin.x + bounds.size.width * (temperature.clamp(0, 100) as f32 / 100.),
        bounds.bottom() - bounds.size.height * (speed.clamp(0, 100) as f32 / 100.),
    )
}

fn curve_samples(points: FanCurve) -> [(i32, i32); 7] {
    let [first, second, third, fourth, last] = points;
    [
        (0, first.1),
        first,
        second,
        third,
        fourth,
        last,
        (100, last.1),
    ]
}

fn percent(offset: Pixels, length: Pixels) -> i32 {
    ((offset / length).clamp(0., 1.) * 100.).round() as i32
}

impl Panel {
    pub(super) fn render_curve(&self, cx: &mut Context<Self>) -> Option<Div> {
        let (current, target) = self.fan_curves()?;
        let palette = self.palette;
        let editable = self.fan_curve_editable();
        let geometry = Rc::new(Cell::new(None::<Bounds<Pixels>>));
        let paint_geometry = geometry.clone();
        let entity = cx.entity();
        let plot = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                paint_geometry.set(Some(bounds));
                let coord = |t, s| plot_point(bounds, t, s);
                let mut grid = PathBuilder::stroke(px(1.));
                for i in 1..10 {
                    let mid = i * 10;
                    grid.move_to(coord(mid, 0));
                    grid.line_to(coord(mid, 100));
                    grid.move_to(coord(0, mid));
                    grid.line_to(coord(100, mid));
                }
                if let Ok(path) = grid.build() {
                    window.paint_path(path, rgb(palette.grid));
                }
                for (points, color, width) in [
                    (current, palette.muted_text, 1.5),
                    (target, palette.accent, 2.25),
                ] {
                    let samples = curve_samples(points);
                    let mut line = PathBuilder::stroke(px(width));
                    line.move_to(coord(samples[0].0, samples[0].1));
                    for (temp, speed) in samples.into_iter().skip(1) {
                        line.line_to(coord(temp, speed));
                    }
                    if let Ok(path) = line.build() {
                        window.paint_path(path, rgb(color));
                    }
                }
                if !editable {
                    return;
                }
                for (temp, speed) in target {
                    let p = coord(temp, speed);
                    window.paint_quad(
                        fill(
                            Bounds::new(point(p.x - px(3.5), p.y - px(3.5)), size(px(7.), px(7.))),
                            rgb(palette.accent),
                        )
                        .corner_radii(px(3.5)),
                    );
                }

                let dragged = entity.clone();
                window.on_mouse_event(move |ev: &MouseMoveEvent, phase, _, cx| {
                    if !phase.bubble() || !ev.dragging() {
                        return;
                    }
                    if let Some(i) = dragged.read(cx).curve_drag {
                        dragged.update(cx, |panel, cx| {
                            panel.move_curve_point(
                                i,
                                percent(ev.position.x - bounds.origin.x, bounds.size.width),
                                percent(bounds.bottom() - ev.position.y, bounds.size.height),
                            );
                            cx.notify();
                        });
                    }
                });
                window.on_mouse_event(move |ev: &MouseUpEvent, _, _, cx| {
                    if ev.button == MouseButton::Left && entity.read(cx).curve_drag.is_some() {
                        entity.update(cx, |panel, _| panel.curve_drag = None);
                    }
                });
            },
        )
        .w_full()
        .h(px(PLOT_HEIGHT));

        let mut graph = div()
            .border_1()
            .border_color(rgb(palette.border))
            .child(plot);
        if editable {
            graph = graph.on_mouse_down(
                MouseButton::Left,
                cx.listener(move |panel, ev: &MouseDownEvent, _, _| {
                    if let Some(bounds) = geometry.get() {
                        panel.curve_drag = target
                            .iter()
                            .enumerate()
                            .filter_map(|(i, &(temp, speed))| {
                                let p = plot_point(bounds, temp, speed);
                                let x = (p.x - ev.position.x) / px(1.);
                                let y = (p.y - ev.position.y) / px(1.);
                                let distance = x * x + y * y;
                                (distance <= 144.).then_some((i, distance))
                            })
                            .min_by(|a, b| a.1.total_cmp(&b.1))
                            .map(|(i, _)| i);
                    }
                }),
            );
        }
        let y_labels = div()
            .relative()
            .w(px(32.))
            .flex_shrink_0()
            .children((0..=5).map(|i| {
                div()
                    .absolute()
                    .top(px(i as f32 * PLOT_HEIGHT / 5. - 6.))
                    .right_2()
                    .child((100 - i * 20).to_string())
            }));
        let x_labels = div().relative().h(px(18.)).children((0..=5).map(|i| {
            div()
                .absolute()
                .left(relative(i as f32 / 5.))
                .ml(px(-16.))
                .w(px(32.))
                .text_center()
                .child((i * 20).to_string())
        }));
        let key = |label, color| {
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().w(px(15.)).h(px(2.)).bg(rgb(color)))
                .child(label)
        };
        Some(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .pr(px(16.))
                .text_xs()
                .text_color(rgb(palette.muted_text))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child("转速（%）")
                        .child(
                            div()
                                .flex()
                                .gap_4()
                                .child(key("当前曲线", palette.muted_text))
                                .child(key("目标曲线", palette.accent)),
                        ),
                )
                .child(
                    div().flex().child(y_labels).child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(graph)
                            .child(x_labels)
                            .child(div().text_center().child("温度（°C）")),
                    ),
                ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::curve_samples;

    #[test]
    fn fan_curve_keeps_the_end_speeds_outside_the_five_points() {
        let points = [(28, 22), (45, 32), (55, 42), (70, 56), (88, 72)];
        let samples = curve_samples(points);
        assert_eq!(samples[0], (0, 22));
        assert_eq!(samples[1..=5], points);
        assert_eq!(samples[6], (100, 72));
    }
}
