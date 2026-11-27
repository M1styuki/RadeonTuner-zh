use super::Panel;
use crate::update;
use gpui_kit::component::{ActiveTheme as _, Sizable as _, alert::Alert, h_flex, v_flex};
use gpui_kit::{Context, Div, ParentElement as _, Styled as _, div};

#[derive(Default)]
pub(super) struct Updates {
    pub(super) busy: bool,
    pub(super) found: Option<update::Update>,
    pub(super) status: String,
}

impl Updates {
    pub(super) fn has_notice(&self) -> bool {
        self.found.is_some()
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
                            "已是最新版本".into()
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
        let mut container = v_flex().gap_2();
        let mut row = h_flex().flex_wrap().gap_2();
        if let Some(update) = &self.updates.found {
            let release = update.clone();
            row = row.child(self.primary_button(
                "install-update",
                format!("更新至 {} 并重新启动", release.version),
                !self.updates.busy,
                None,
                move |p, _, cx| {
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
                },
                cx,
            ));
        }
        if self.page == 6 {
            row = row.child(self.contour_button(
                "check-update",
                "检查更新",
                !self.updates.busy,
                None,
                |p, _, cx| p.check_updates(true, cx),
                cx,
            ));
            row = row.child(self.contour_button(
                "update-page",
                "打开 GitHub 下载页",
                true,
                None,
                |p, _, _| {
                    if let Err(e) = crate::integration::open(update::RELEASES) {
                        p.updates.status = e.to_string();
                    }
                },
                cx,
            ));
        }
        container = container.child(row);
        if (self.page == 6 || self.updates.found.is_some()) && !self.updates.status.is_empty() {
            container = container.child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(Alert::info("update-status", self.updates.status.clone()).small()),
            );
        }
        container
    }
}
