use super::{NAV_WIDTH, Panel};
use gpui::{Context, Div, PathBuilder, Pixels, canvas, div, point, prelude::*, px, rgb};
use std::collections::HashSet;

pub(super) fn nav_icon(index: usize, color: u32) -> Div {
    div().size(px(20.)).child(
        canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let at = |x, y| point(bounds.origin.x + px(x), bounds.origin.y + px(y));
                let mut path = PathBuilder::stroke(px(1.5));
                let line = |path: &mut PathBuilder, points: &[(f32, f32)], closed: bool| {
                    let (&(x, y), tail) = points.split_first().expect("nonempty icon path");
                    path.move_to(at(x, y));
                    for &(x, y) in tail {
                        path.line_to(at(x, y));
                    }
                    if closed {
                        path.close();
                    }
                };
                let arc = |path: &mut PathBuilder, radius, large, sweep, x, y| {
                    path.arc_to(
                        point(px(radius), px(radius)),
                        px(0.),
                        large,
                        sweep,
                        at(x, y),
                    );
                };
                let circle = |path: &mut PathBuilder, x, y, r| {
                    path.move_to(at(x + r, y));
                    arc(path, r, false, true, x - r, y);
                    arc(path, r, false, true, x + r, y);
                    path.close();
                };
                let box_outline = |path: &mut PathBuilder, left, top, right, bottom| {
                    let r = 2.;
                    path.move_to(at(left + r, top));
                    path.line_to(at(right - r, top));
                    arc(path, r, false, true, right, top + r);
                    path.line_to(at(right, bottom - r));
                    arc(path, r, false, true, right - r, bottom);
                    path.line_to(at(left + r, bottom));
                    arc(path, r, false, true, left, bottom - r);
                    path.line_to(at(left, top + r));
                    arc(path, r, false, true, left + r, top);
                    path.close();
                };
                match index {
                    0 => {
                        path.move_to(at(4.4, 16.));
                        arc(&mut path, 8., true, true, 15.6, 16.);
                        line(&mut path, &[(7., 16.), (13., 16.)], false);
                        line(&mut path, &[(10., 11.5), (13.4, 7.6)], false);
                    }
                    1 => {
                        circle(&mut path, 10., 10., 1.7);
                        for blade in 0..3 {
                            let angle = blade as f32 * std::f32::consts::TAU / 3.;
                            let (sin, cos) = angle.sin_cos();
                            let rotate = |x: f32, y: f32| {
                                at(10. + cos * x - sin * y, 10. + sin * x + cos * y)
                            };
                            path.move_to(rotate(-1.6, -1.));
                            path.cubic_bezier_to(
                                rotate(0., -8.),
                                rotate(-4., -4.2),
                                rotate(-2.5, -8.),
                            );
                            path.cubic_bezier_to(
                                rotate(1.5, -1.),
                                rotate(3.8, -8.),
                                rotate(0.8, -4.3),
                            );
                        }
                    }
                    2 => {
                        box_outline(&mut path, 3., 5., 18., 15.);
                        line(&mut path, &[(3., 4.), (1.5, 4.), (1.5, 16.)], false);
                        circle(&mut path, 8.1, 10., 2.2);
                        circle(&mut path, 13.6, 10., 2.2);
                        line(
                            &mut path,
                            &[(7., 15.), (7., 17.), (13., 17.), (13., 15.)],
                            false,
                        );
                    }
                    3 => {
                        box_outline(&mut path, 2., 3.5, 18., 14.);
                        line(&mut path, &[(10., 14.), (10., 17.)], false);
                        line(&mut path, &[(6.5, 17.), (13.5, 17.)], false);
                    }
                    4 => {
                        box_outline(&mut path, 2., 3.5, 18., 16.5);
                        line(&mut path, &[(8., 7.), (13., 10.), (8., 13.)], true);
                    }
                    5 => {
                        for (y, stop) in [(4., 7.), (10., 13.), (16., 9.)] {
                            line(&mut path, &[(2., y), (stop - 1.8, y)], false);
                            circle(&mut path, stop, y, 1.8);
                            line(&mut path, &[(stop + 1.8, y), (18., y)], false);
                        }
                    }
                    6 => {
                        circle(&mut path, 10., 10., 8.);
                        line(&mut path, &[(10., 5.5), (10., 7.)], false);
                        line(&mut path, &[(10., 9.), (10., 14.5)], false);
                    }
                    7 => {
                        path.move_to(at(5.5, 4.6));
                        arc(&mut path, 7.5, true, false, 14.5, 4.6);
                        line(&mut path, &[(10., 2.), (10., 10.)], false);
                    }
                    _ => return,
                }
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgb(color));
                }
            },
        )
        .size_full(),
    )
}

impl Panel {
    pub(super) fn render_header(&self, viewport: Pixels, cx: &mut Context<Self>) -> Div {
        let mut header = div()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap_2()
            .px_4()
            .pt_3()
            .pb_2();
        if self.page == 5 {
            return header;
        }
        let mut locations = HashSet::new();
        let devices = self
            .devices
            .iter()
            .enumerate()
            .filter(|(_, d)| d.pci_location.is_none_or(|pci| locations.insert(pci)))
            .collect::<Vec<_>>();
        let selected = devices.iter().position(|(i, _)| *i == self.selected);
        let indices = devices.iter().map(|(i, _)| *i).collect::<Vec<_>>();
        let labels = devices
            .iter()
            .map(|(_, d)| {
                if devices
                    .iter()
                    .filter(|(_, other)| other.name == d.name)
                    .count()
                    > 1
                {
                    format!("{} ({})", d.name, d.index)
                } else {
                    d.name.clone()
                }
            })
            .collect();
        let mut toolbar = div()
            .flex()
            .items_center()
            .flex_wrap()
            .gap_2()
            .child(self.select(
                "select-gpu",
                labels,
                selected,
                "选择显卡",
                208.,
                move |p, i, _| {
                    p.finish_preview(false);
                    p.selected = indices[i];
                    p.reset_tuning_editor();
                    p.video_app = "Global".into();
                    p.read_device_details();
                    p.read_displays();
                    p.read_controls();
                    p.refresh();
                },
                cx,
            ));
        toolbar = match self.page {
            0 | 1 => toolbar.child(self.tuning_profiles(cx)),
            2 => toolbar.child(self.game_controls(cx)),
            3 => toolbar.child(self.display_header(cx)),
            4 => toolbar.child(self.video_profiles(cx)),
            _ => toolbar,
        };
        header = header.child(toolbar);
        if !self.metrics.is_empty() {
            let columns =
                (((viewport - px(NAV_WIDTH + 32.)) / px(118.)).floor() as u16).clamp(1, 5);
            let mut stats = div().w_full().grid().grid_cols(columns).gap_x_4().gap_y_2();
            for (i, item) in self.metrics.iter().enumerate() {
                let reading = item
                    .value
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "—".into());
                stats = stats.child(
                    div()
                        .id(("metric", i))
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .tooltip(self.tooltip(item.label))
                        .child(
                            div()
                                .text_xs()
                                .truncate()
                                .text_color(rgb(self.palette.muted_text))
                                .child(item.label),
                        )
                        .child(
                            div()
                                .flex()
                                .justify_end()
                                .items_baseline()
                                .gap_1()
                                .whitespace_nowrap()
                                .child(reading)
                                .when(item.value.is_some(), |value| {
                                    value.child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(self.palette.muted_text))
                                            .child(item.unit),
                                    )
                                }),
                        ),
                );
            }
            header = header.child(stats);
        }
        let (actions, status) = match self.page {
            0 | 1 => (
                (!self.controls.is_empty()).then(|| self.tuning_actions(cx)),
                &self.tuning_status,
            ),
            2 => (Some(self.graphics_actions(cx)), &self.graphics_status),
            3 => (Some(self.display_actions(cx)), &self.display.status),
            4 => (
                (!self.video.is_empty()).then(|| self.video_actions(cx)),
                &self.video_status,
            ),
            _ => return header,
        };
        header.children(actions).when(!status.is_empty(), |view| {
            view.child(
                div()
                    .id("action-status")
                    .text_xs()
                    .truncate()
                    .tooltip(self.tooltip_owned(status.clone().into()))
                    .child(status.clone()),
            )
        })
    }
}
