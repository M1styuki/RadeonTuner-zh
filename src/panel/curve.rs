use super::Panel;
use gpui::{
    Bounds, Context, Div, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    PathBuilder, Pixels, Point, canvas, div, point, prelude::*, px, rgb,
};
use std::{cell::Cell, rc::Rc};

fn plot_point(bounds: Bounds<Pixels>, x: f32, y: f32) -> Point<Pixels> {
    point(
        bounds.origin.x + bounds.size.width * x,
        bounds.origin.y + bounds.size.height * (1. - y),
    )
}

fn percent(offset: Pixels, length: Pixels) -> i32 {
    ((offset / length).clamp(0., 1.) * 100.).round() as i32
}

impl Panel {
    pub(super) fn render_curve(&self, cx: &mut Context<Self>) -> Div {
        let palette = self.palette;
        let points = self.curve_points();
        let mut section = div().flex().flex_col().gap_1();
        if points.len() != 5 {
            return section;
        }
        section = section.child("风扇转速（%）");
        let editable = self.editable_curve();
        let geometry = Rc::new(Cell::new(None::<Bounds<Pixels>>));
        let paint_geometry = geometry.clone();
        let grab_points = points.clone();
        let entity = cx.entity();
        let plot = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                paint_geometry.set(Some(bounds));
                let coord = |x, y| plot_point(bounds, x, y);
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
                if !editable {
                    return;
                }
                for &(temp, speed) in &points {
                    let p = coord(temp as f32 / 100., speed as f32 / 100.);
                    let mut marker = PathBuilder::fill();
                    marker.move_to(point(p.x, p.y - px(4.)));
                    marker.line_to(point(p.x + px(4.), p.y));
                    marker.line_to(point(p.x, p.y + px(4.)));
                    marker.line_to(point(p.x - px(4.), p.y));
                    marker.close();
                    if let Ok(path) = marker.build() {
                        window.paint_path(path, rgb(palette.accent));
                    }
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
        .h(px(145.));
        let mut graph = div().relative().child(super::shape::outline(palette.border)).child(plot);
        if editable {
            graph = graph.cursor_pointer().on_mouse_down(
                MouseButton::Left,
                cx.listener(move |panel, ev: &MouseDownEvent, _, _| {
                    if let Some(bounds) = geometry.get() {
                        panel.curve_drag = grab_points.iter().enumerate().filter_map(|(i, &(temp, speed))| {
                            let p = plot_point(bounds, temp as f32 / 100., speed as f32 / 100.);
                            let x = (p.x - ev.position.x) / px(1.);
                            let y = (p.y - ev.position.y) / px(1.);
                            let distance = x * x + y * y;
                            (distance <= 144.).then_some((i, distance))
                        }).min_by(|a, b| a.1.total_cmp(&b.1)).map(|(i, _)| i);
                    }
                }),
            );
        }
        section
            .child(graph)
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
