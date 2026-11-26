use super::{Panel, tuning::FanCurve};
use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::{
    Bounds, Context, Div, InteractiveElement as _, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, ParentElement as _, PathBuilder, Pixels, Point, Styled as _, canvas, div, fill,
    point, px, relative, size,
};
use std::{cell::Cell, rc::Rc};

const PLOT_HEIGHT: f32 = 156.;

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
        let editable = self.fan_curve_editable();
        let grid_color = cx.theme().chart_grid;
        let current_color = cx.theme().muted_foreground;
        let target_color = cx.theme().primary;
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
                    window.paint_path(path, grid_color);
                }
                for (points, color, width) in
                    [(current, current_color, 1.5), (target, target_color, 2.25)]
                {
                    let samples = curve_samples(points);
                    let mut line = PathBuilder::stroke(px(width));
                    line.move_to(coord(samples[0].0, samples[0].1));
                    for (temp, speed) in samples.into_iter().skip(1) {
                        line.line_to(coord(temp, speed));
                    }
                    if let Ok(path) = line.build() {
                        window.paint_path(path, color);
                    }
                }
                if !editable {
                    return;
                }
                for (temp, speed) in target {
                    let p = coord(temp, speed);
                    window.paint_quad(
                        fill(
                            Bounds::new(point(p.x - px(4.), p.y - px(4.)), size(px(8.), px(8.))),
                            target_color,
                        )
                        .corner_radii(px(4.)),
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
            .relative()
            .bg(cx.theme().input_background())
            .border_1()
            .border_color(cx.theme().border)
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
                                let x = f32::from(p.x - ev.position.x);
                                let y = f32::from(p.y - ev.position.y);
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
            let tick = div().absolute().w(px(32.)).child((i * 20).to_string());
            match i {
                0 => tick.left_0(),
                5 => tick.right_0().text_right(),
                _ => tick
                    .left(relative(i as f32 / 5.))
                    .ml(px(-16.))
                    .text_center(),
            }
        }));
        let key = |label, color| {
            h_flex()
                .gap_2()
                .child(div().w(px(14.)).h(px(2.)).bg(color))
                .child(label)
        };
        Some(
            v_flex()
                .gap_2()
                .pt_1()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(
                    h_flex().justify_between().child("转速（%）").child(
                        h_flex()
                            .gap_4()
                            .child(key("当前曲线", current_color))
                            .child(key("目标曲线", target_color)),
                    ),
                )
                .child(
                    div().flex().child(y_labels).child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
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
