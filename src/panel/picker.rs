use super::{Panel, motion::pointer_feedback, shape::ContinuousCorners as _};
use crate::integration::games::{self, Entry};
use anyhow::{Context as _, Result, ensure};
use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::{
    Context, Div, FontWeight, InteractiveElement as _, ParentElement as _, PathPromptOptions,
    StatefulInteractiveElement as _, Styled as _, div, px,
};
use std::collections::BTreeSet;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Target {
    Graphics,
    Display,
    Tuning,
    Multimedia,
}
pub(super) struct Picker {
    target: Target,
    remove: bool,
    choices: Vec<Entry>,
    selected: BTreeSet<usize>,
    status: String,
}
impl Panel {
    fn open_picker(&mut self, target: Target, remove: bool) {
        self.dropdown = None;
        self.closing_dropdown = None;
        self.picker_generation += 1;
        let mut names = if remove {
            match target {
                Target::Tuning => self.tuning_app_names(),
                Target::Display => self.display_app_names(),
                Target::Graphics => self.game_names(),
                Target::Multimedia => self.video_apps.clone(),
            }
        } else {
            Vec::new()
        };
        names.sort_by_cached_key(|n| n.to_lowercase());
        names.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        names.retain(|n| !n.eq_ignore_ascii_case("Global"));
        self.picker = Some(Picker {
            target,
            remove,
            choices: names.into_iter().map(|n| (n.clone(), n)).collect(),
            selected: BTreeSet::new(),
            status: String::new(),
        });
    }
    pub(super) fn app_picker_buttons(&self, target: Target, cx: &mut Context<Self>) -> Div {
        let mut row = h_flex().gap_2();
        for (id, label, remove) in [
            ("profile-picker-add", "添加应用…", false),
            ("profile-picker-remove", "移除应用…", true),
        ] {
            row = row.child(self.contour_button(
                id,
                label,
                true,
                Some(if remove {
                    "移除应用程序"
                } else {
                    "添加应用程序"
                }),
                move |p, _, _| p.open_picker(target, remove),
                cx,
            ));
        }
        row
    }
    fn show_picker_entries(&mut self, generation: u64, result: Result<Vec<Entry>>) {
        if self.picker_generation != generation {
            return;
        }
        if let Some(p) = &mut self.picker {
            p.selected.clear();
            p.choices.clear();
            p.status.clear();
            match result {
                Ok(items) => {
                    p.choices = items;
                    if p.choices.is_empty() {
                        p.status = "没有找到可选应用".into();
                    }
                }
                Err(e) => p.status = e.to_string(),
            }
        }
    }
    fn apply_picker(&mut self) {
        let Some(p) = self.picker.take() else {
            return;
        };
        let mut seen = BTreeSet::new();
        let names = p
            .selected
            .iter()
            .filter_map(|i| p.choices.get(*i))
            .map(|(_, n)| n.clone())
            .filter(|s| !s.eq_ignore_ascii_case("Global") && seen.insert(s.to_lowercase()))
            .collect::<Vec<_>>();
        let mut failed = Vec::new();
        if p.remove {
            let result = match p.target {
                Target::Display => self.remove_display_apps(&names),
                Target::Tuning => self.remove_tuning_apps(&names),
                Target::Graphics => self.remove_games(&names),
                Target::Multimedia => self.remove_video_apps(&names),
            };
            if let Err(e) = result {
                failed.push(e.to_string());
            }
        } else {
            for name in &names {
                let result = match p.target {
                    Target::Display => self.add_display_app(name),
                    Target::Tuning => self.add_tuning_app(name),
                    Target::Graphics => self.add_game(name.clone()),
                    Target::Multimedia => self.add_video_app(name),
                };
                if let Err(e) = result {
                    failed.push(format!("{name}：{e}"));
                }
            }
        }
        let msg = if failed.is_empty() {
            format!(
                "已{} {} 个应用配置",
                if p.remove { "移除" } else { "选择" },
                names.len()
            )
        } else {
            failed.join("；")
        };
        match p.target {
            Target::Display => self.display.set_status(msg),
            Target::Tuning => self.tuning_status = msg,
            Target::Graphics => self.graphics_status = msg,
            Target::Multimedia => self.video_status = msg,
        }
    }
    pub(super) fn render_picker(&self, cx: &mut Context<Self>) -> Div {
        let picker = self.picker.as_ref().unwrap();
        let mut view =
            v_flex()
                .w_full()
                .gap_3()
                .text_color(cx.theme().popover_foreground)
                .child(div().text_base().font_weight(FontWeight::SEMIBOLD).child(
                    if picker.remove {
                        "移除应用配置"
                    } else {
                        "添加应用配置"
                    },
                ));
        if !picker.remove {
            let mut sources = h_flex().flex_wrap().gap_2();
            for (id, label, steam) in [
                ("picker-processes", "运行中的进程", false),
                ("picker-steam", "Steam 游戏", true),
            ] {
                sources = sources.child(self.contour_button(
                    id,
                    label,
                    true,
                    None,
                    move |p, _, cx| {
                        p.picker_generation += 1;
                        let generation = p.picker_generation;
                        if let Some(v) = &mut p.picker {
                            v.status = "正在读取…".into();
                            v.choices.clear();
                            v.selected.clear();
                        }
                        let read = cx.background_executor().spawn(async move {
                            if steam {
                                games::launcher()
                            } else {
                                games::running()
                            }
                        });
                        cx.spawn(async move |view, cx| {
                            let result = read.await;
                            let _ = view.update(cx, |p, cx| {
                                p.show_picker_entries(generation, result);
                                cx.notify();
                            });
                        })
                        .detach();
                    },
                    cx,
                ));
            }
            sources = sources.child(self.contour_button(
                "picker-files",
                "选择 EXE 或 BIN 文件…",
                true,
                None,
                |p, _, cx| {
                    p.picker_generation += 1;
                    let generation = p.picker_generation;
                    let files = cx.prompt_for_paths(PathPromptOptions {
                        files: true,
                        directories: false,
                        multiple: true,
                        prompt: Some("选择应用程序".into()),
                    });
                    cx.spawn(async move |view, cx| {
                        let result = async {
                            let Some(paths) = files.await?? else {
                                return Ok::<_, anyhow::Error>(None);
                            };
                            let mut rows = Vec::new();
                            for file in paths {
                                ensure!(
                                    file.extension()
                                        .is_some_and(|s| s.eq_ignore_ascii_case("exe")
                                            || s.eq_ignore_ascii_case("bin")),
                                    "请选择 EXE 或 BIN 文件"
                                );
                                let name = file
                                    .file_name()
                                    .context("文件名无效")?
                                    .to_string_lossy()
                                    .into_owned();
                                rows.push((file.display().to_string(), name));
                            }
                            Ok(Some(rows))
                        }
                        .await;
                        let _ = view.update(cx, |p, cx| {
                            if p.picker_generation != generation {
                                return;
                            }
                            match result {
                                Ok(Some(rows)) => {
                                    p.show_picker_entries(generation, Ok(rows));
                                    if let Some(p) = &mut p.picker {
                                        p.selected.extend(0..p.choices.len());
                                    }
                                }
                                Ok(None) => (),
                                Err(e) => p.show_picker_entries(generation, Err(e)),
                            }
                            cx.notify();
                        });
                    })
                    .detach();
                },
                cx,
            ));
            view = view.child(sources);
        }
        let mut list = v_flex()
            .id("picker-list")
            .max_h(px(420.))
            .gap_0p5()
            .overflow_y_scroll();
        let radius = f32::from(cx.theme().radius);
        for (i, (title, file)) in picker.choices.iter().enumerate() {
            let label = if title == file {
                title.clone()
            } else {
                format!("{title}　{file}")
            };
            let chosen = picker.selected.contains(&i);
            let mut row = div()
                .id(("picker-choice", i))
                .relative()
                .px_2p5()
                .py_1p5()
                .text_sm()
                .tooltip(self.tooltip_owned(label.clone().into()));
            if chosen {
                row = row.contour(Some(cx.theme().accent), None, radius);
            }
            let row = pointer_feedback(
                ("picker-choice-fb", i),
                row,
                cx.theme().accent.opacity(0.65),
                cx.theme().accent,
                radius,
            )
            .child(label)
            .on_click(cx.listener(move |p, _, _, cx| {
                if let Some(v) = &mut p.picker {
                    if !v.selected.remove(&i) {
                        v.selected.insert(i);
                    }
                }
                cx.notify();
            }));
            list = list.child(row);
        }
        let mut footer = h_flex().justify_end().gap_2().child(self.contour_button(
            "picker-cancel",
            "取消",
            true,
            None,
            |p, _, _| {
                p.picker = None;
                p.picker_generation += 1;
            },
            cx,
        ));
        if !picker.selected.is_empty() {
            let action_label = format!(
                "{}所选项目（{}）",
                if picker.remove { "移除" } else { "添加" },
                picker.selected.len()
            );
            footer = footer.child(if picker.remove {
                self.danger_button(
                    "picker-apply",
                    action_label,
                    None,
                    |p, _, _| p.apply_picker(),
                    cx,
                )
            } else {
                self.primary_button(
                    "picker-apply",
                    action_label,
                    true,
                    None,
                    |p, _, _| p.apply_picker(),
                    cx,
                )
            });
        }
        view.child(list)
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(picker.status.clone()),
            )
            .child(footer)
    }
}
