use super::{Panel, shape::ContinuousCorners};
use gpui::{Context, Div, RenderImage, div, img, prelude::*, px, rgb};
use std::{
    collections::HashSet,
    sync::{Arc, OnceLock},
};

pub(super) fn nav_icon(index: usize, light: bool) -> Div {
    macro_rules! asset {
        ($name:literal) => {
            (
                include_bytes!(concat!("../../RadeonTuner/", $name, ".png")) as &[u8],
                include_bytes!(concat!("../../RadeonTuner/", $name, "Dark.png")) as &[u8],
            )
        };
    }
    const ASSETS: [(&[u8], &[u8]); 8] = [
        asset!("Tuning"),
        asset!("Fans"),
        asset!("Graphics"),
        asset!("Display"),
        asset!("Multimedia"),
        asset!("Settings"),
        asset!("Information"),
        asset!("Shutdown"),
    ];
    static ICONS: OnceLock<[[Option<Arc<RenderImage>>; 8]; 2]> = OnceLock::new();
    let icons = ICONS.get_or_init(|| {
        [false, true].map(|light| {
            ASSETS.map(|(dark_image, light_image)| {
                let mut rgba =
                    image::load_from_memory(if light { light_image } else { dark_image })
                        .ok()?
                        .into_rgba8();
                for pixel in rgba.pixels_mut() {
                    pixel.0.swap(0, 2);
                }
                Some(Arc::new(RenderImage::new([image::Frame::new(rgba)])))
            })
        })
    });
    div().w(px(34.)).h(px(34.)).children(
        icons[usize::from(light)][index]
            .clone()
            .map(|source| img(source).size_full()),
    )
}

impl Panel {
    pub(super) fn render_header(&self, cx: &mut Context<Self>) -> Div {
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
                238.,
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
            let mut stats = div()
                .flex()
                .flex_wrap()
                .gap_x_4()
                .gap_y_1()
                .px_3()
                .py_2()
                .smooth_background(self.palette.sidebar);
            for item in &self.metrics {
                if let Some(value) = item.value {
                    stats = stats.child(
                        div()
                            .flex_shrink_0()
                            .child(format!("{} {value} {}", item.label, item.unit)),
                    );
                }
            }
            header = header.child(stats);
        }
        header
    }
}
