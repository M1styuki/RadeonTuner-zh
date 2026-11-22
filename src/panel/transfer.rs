use super::Panel;
use crate::profile_store;
use anyhow::{Result, ensure};
use gpui::{Context, Div, PathPromptOptions, div, prelude::*};
use serde_json::Value;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Kind {
    Tuning,
    Graphics,
    Display,
}

pub(super) struct Pending {
    kind: Kind,
    target: (String, String),
    record: Value,
}

impl Kind {
    fn extension(self) -> &'static str {
        match self {
            Self::Tuning => "radt",
            Self::Graphics => "radg",
            Self::Display => "radd",
        }
    }
    fn target(self, p: &Panel) -> Option<(String, String)> {
        match self {
            Self::Tuning => p.tuning_target(),
            Self::Graphics => p.graphics_target(),
            Self::Display => p.display_target(),
        }
    }
    fn record(self, p: &Panel) -> Result<Value> {
        match self {
            Self::Tuning => p.tuning_record(),
            Self::Graphics => p.graphics_record(),
            Self::Display => p.display_record(),
        }
    }
    fn stage(self, p: &mut Panel, value: &Value) -> Result<()> {
        match self {
            Self::Tuning => p.stage_tuning(value),
            Self::Graphics => p.stage_graphics(value),
            Self::Display => p.stage_display(value),
        }
    }
}
impl Panel {
    fn transfer_status(&mut self, kind: Kind, result: Result<()>, message: &str) {
        let status = result.map_or_else(|err| err.to_string(), |()| message.to_owned());
        match kind {
            Kind::Tuning => self.tuning_status = status,
            Kind::Graphics => self.graphics_status = status,
            Kind::Display => self.display.set_status(status),
        }
    }
    pub(super) fn transfer_buttons(&self, kind: Kind, cx: &mut Context<Self>) -> Div {
        let mut row = div().flex().gap_3();
        let Some(target) = kind.target(self) else {
            return row;
        };
        if self
            .pending_import
            .as_ref()
            .is_some_and(|pending| pending.kind == kind && pending.target == target)
        {
            return row
                .child("设置文件中的设备与当前选择不同，继续导入？")
                .child(
                    div()
                        .id("confirm-import")
                        .cursor_pointer()
                        .child("继续导入")
                        .on_click(cx.listener(move |p, _, _, cx| {
                            if let Some(pending) = p.pending_import.take() {
                                if kind.target(p).as_ref() == Some(&pending.target) {
                                    let result = kind.stage(p, &pending.record);
                                    p.transfer_status(kind, result, "已导入，请确认后应用");
                                }
                            }
                            cx.notify();
                        })),
                )
                .child(
                    div()
                        .id("cancel-import")
                        .cursor_pointer()
                        .child("取消")
                        .on_click(cx.listener(|p, _, _, cx| {
                            p.pending_import = None;
                            cx.notify();
                        })),
                );
        }
        row = row.child(
            div()
                .id("import-profile")
                .cursor_pointer()
                .child("导入配置…")
                .on_click(cx.listener(move |p, _, _, cx| {
                    let Some(target) = kind.target(p) else {
                        return;
                    };
                    let pick = cx.prompt_for_paths(PathPromptOptions {
                        files: true,
                        directories: false,
                        multiple: false,
                        prompt: Some(format!("选择 .{} 配置文件", kind.extension()).into()),
                    });
                    cx.spawn(async move |view, cx| {
                        let result: Result<Option<Value>> = async {
                            let Some(paths) = pick.await?? else {
                                return Ok(None);
                            };
                            let Some(path) = paths.first() else {
                                return Ok(None);
                            };
                            ensure!(
                                path.extension()
                                    .is_some_and(|s| s.eq_ignore_ascii_case(kind.extension())),
                                "请选择 .{} 配置文件",
                                kind.extension()
                            );
                            Ok(Some(profile_store::read_export(path)?))
                        }
                        .await;
                        let _ = view.update(cx, |p, cx| {
                            if kind.target(p).as_ref() != Some(&target) {
                                return;
                            }
                            match result {
                                Ok(Some(record)) => {
                                    let mismatch = kind != Kind::Graphics
                                        && record["DeviceId"].as_str().is_some_and(|key| {
                                            !key.is_empty()
                                                && !target.0.is_empty()
                                                && key != target.0
                                        });
                                    if mismatch {
                                        p.pending_import = Some(Pending {
                                            kind,
                                            target,
                                            record,
                                        });
                                    } else {
                                        let result = kind.stage(p, &record);
                                        p.transfer_status(kind, result, "已导入，请确认后应用");
                                    }
                                }
                                Err(err) => p.transfer_status(kind, Err(err), ""),
                                Ok(None) => {}
                            }
                            cx.notify();
                        });
                    })
                    .detach();
                })),
        );
        row.child(
            div()
                .id("export-profile")
                .cursor_pointer()
                .child("导出配置…")
                .on_click(cx.listener(move |p, _, _, cx| {
                    let Some(target) = kind.target(p) else {
                        return;
                    };
                    let record = match kind.record(p) {
                        Ok(row) => row,
                        Err(err) => {
                            p.transfer_status(kind, Err(err), "");
                            cx.notify();
                            return;
                        }
                    };
                    let default_dir = std::env::var_os("USERPROFILE")
                        .map(std::path::PathBuf::from)
                        .unwrap_or_default();
                    let suggested = match kind {
                        Kind::Tuning => "RadeonTuner.radt",
                        Kind::Graphics => "RadeonTuner.radg",
                        Kind::Display => "RadeonTuner.radd",
                    };
                    let pick = cx.prompt_for_new_path(&default_dir, Some(suggested));
                    cx.spawn(async move |view, cx| {
                        let result: Result<Option<()>> = async {
                            let Some(path) = pick.await?? else {
                                return Ok(None);
                            };
                            profile_store::write_json(&path, &record)?;
                            Ok(Some(()))
                        }
                        .await;
                        let _ = view.update(cx, |p, cx| {
                            if kind.target(p).as_ref() == Some(&target) {
                                match result {
                                    Ok(None) => {}
                                    Ok(Some(())) => p.transfer_status(kind, Ok(()), "已导出配置"),
                                    Err(err) => p.transfer_status(kind, Err(err), ""),
                                }
                                cx.notify();
                            }
                        });
                    })
                    .detach();
                })),
        )
    }
}
