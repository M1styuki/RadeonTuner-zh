use super::Panel;
use crate::update;
use gpui::{Context, Div, div, prelude::*};
#[derive(Default)]
pub(super) struct Updates {
    busy: bool,
    status: String,
    found: Option<update::Update>,
}
impl Updates {
    pub(super) fn has_notice(&self) -> bool {
        self.found.is_some() || !self.status.is_empty()
    }
}

impl Panel {
    pub(super) fn check_updates(&mut self, manual: bool, cx: &mut Context<Self>) {
        if self.updates.busy {
            return;
        }
        self.updates.busy = true;
        if manual {
            self.updates.status = "正在检查更新…".into();
        }
        let task = cx.background_executor().spawn(async { update::check() });
        cx.spawn(async move |view, cx| {
            let result = task.await;
            let _ = view.update(cx, |p, cx| {
                p.updates.busy = false;
                match result {
                    Ok(update) => {
                        p.updates.found = update;
                        p.updates.status = if p.updates.found.is_none() && manual {
                            "已是最新的 Rust 发行版".into()
                        } else {
                            String::new()
                        };
                    }
                    Err(e) => p.updates.status = e.to_string(),
                }
                cx.notify();
            });
        })
        .detach();
    }
    pub(super) fn update_controls(&self, cx: &mut Context<Self>) -> Div {
        let mut row = div().flex().flex_col().gap_2();
        if let Some(update) = &self.updates.found {
            let release = update.clone();
            let action = self
                .button_style(!self.updates.busy)
                .id("install-update")
                .child(format!("更新至 {} 并重新启动", release.version));
            row = row.child(if self.updates.busy {
                action
            } else {
                action
                    .cursor_pointer()
                    .on_click(cx.listener(move |p, _, _, cx| {
                        if p.updates.busy {
                            return;
                        }
                        p.updates.busy = true;
                        p.updates.status = "正在下载并校验更新…".into();
                        let release = release.clone();
                        let install = cx
                            .background_executor()
                            .spawn(async move { update::install(&release) });
                        cx.spawn(async move |view, cx| {
                            let result = install
                                .await
                                .and_then(|()| crate::tray::restart_after_update());
                            let _ = view.update(cx, |p, cx| {
                                p.updates.busy = false;
                                p.updates.status = match result {
                                    Ok(()) => "已安装，正在重新启动".into(),
                                    Err(e) => e.to_string(),
                                };
                                cx.notify();
                            });
                        })
                        .detach();
                        cx.notify();
                    }))
            });
        }
        if self.page == 6 {
            let check = self
                .button_style(!self.updates.busy)
                .id("check-update")
                .child("检查 Rust 版更新");
            row = row.child(if self.updates.busy {
                check
            } else {
                check.cursor_pointer().on_click(cx.listener(|p, _, _, cx| {
                    p.check_updates(true, cx);
                    cx.notify();
                }))
            });
            row = row.child(
                self.button_style(true)
                    .id("update-page")
                    .cursor_pointer()
                    .child("打开 GitHub 下载页")
                    .on_click(cx.listener(|p, _, _, cx| {
                        if let Err(e) = crate::integration::open(update::RELEASES) {
                            p.updates.status = e.to_string();
                        }
                        cx.notify();
                    })),
            );
        }
        if self.page == 6 || self.updates.found.is_some() {
            row = row.child(self.updates.status.clone());
        }
        row
    }
}
