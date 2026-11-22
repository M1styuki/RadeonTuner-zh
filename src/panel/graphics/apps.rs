use super::*;
use crate::adl::graphics::apps::{App, settings};
#[derive(Default)]
pub(in crate::panel) struct Games {
    list: Vec<App>,
    pub(super) selected: Option<App>,
    remove: bool,
}
impl Panel {
    pub(super) fn game_name(&self) -> String {
        self.games
            .selected
            .as_ref()
            .map(|a| a.file.clone())
            .unwrap_or_else(|| "Global".into())
    }
    pub(super) fn refresh_games(&mut self) {
        self.games = Games::default();
        if let Some(driver) = &self.driver {
            match driver.game_apps() {
                Ok(list) => self.games.list = list,
                Err(e) => self.graphics_status = e.to_string(),
            }
        }
    }
    pub(super) fn load_game(&mut self, app: App) -> Result<()> {
        let driver = self.driver.as_ref().context("驱动不可用")?;
        let device = self.devices.get(self.selected).context("未选择显卡")?;
        let props = driver.app_properties(&app)?;
        let (choices, controls) = settings::settings(&props, |n| driver.app_kind(n).is_some());
        let library = if driver.app_kind("FfxDllPath").is_some() {
            driver.fsr_library(device, true).map(|mut f| {
                let path = settings::library_path(&props);
                f.select(path.clone());
                f.current = path;
                f
            })
        } else {
            None
        };
        self.graphics = choices;
        self.frames = controls;
        self.fsr_library = library;
        self.games.selected = Some(app);
        self.games.remove = false;
        self.graphics_profile = json!({});
        self.confirm_graphics_reset = false;
        self.graphics_dirty = false;
        self.graphics_status.clear();
        self.read_graphics_profile()
    }
    fn add_game(&mut self, file: String) -> Result<()> {
        let driver = self.driver.as_ref().context("驱动不可用")?;
        let apps = driver.game_apps()?;
        if let Some(existing) = apps
            .iter()
            .find(|a| a.file.eq_ignore_ascii_case(&file) && a.path == r"*\*")
        {
            let existing = existing.clone();
            self.games.list = apps;
            return self.load_game(existing);
        }
        let app = App::new(file);
        let (choices, controls) = settings::settings(&[], |n| driver.app_kind(n).is_some());
        let props =
            driver.game_properties_from_controls(Vec::new(), &choices, &controls, None, true)?;
        ensure!(!props.is_empty(), "驱动未提供游戏配置属性");
        driver.write_game_app(&app, &props)?;
        self.select_written_game(&app)
    }
    fn select_written_game(&mut self, app: &App) -> Result<()> {
        self.games.list = self.driver.as_ref().context("驱动不可用")?.game_apps()?;
        let selected = self
            .games
            .list
            .iter()
            .find(|a| {
                a.file.eq_ignore_ascii_case(&app.file)
                    && a.path.eq_ignore_ascii_case(&app.path)
                    && a.version == app.version
            })
            .context("驱动中找不到写入的配置")?
            .clone();
        self.load_game(selected)
    }
    pub(super) fn save_game(&mut self, all: bool) -> Result<()> {
        let selected = self.games.selected.clone().context("未选择游戏")?;
        let driver = self.driver.as_ref().context("驱动不可用")?;
        let app = driver
            .game_apps()?
            .into_iter()
            .find(|a| {
                a.file.eq_ignore_ascii_case(&selected.file)
                    && a.path.eq_ignore_ascii_case(&selected.path)
                    && a.version == selected.version
            })
            .context("驱动中的应用配置已被移除")?;
        let original = driver.app_properties(&app)?;
        let updated = driver.game_properties_from_controls(
            original.clone(),
            &self.graphics,
            &self.frames,
            self.fsr_library.as_ref(),
            all,
        )?;
        if updated != original {
            driver.write_game_app(&app, &updated)?;
        }
        let (key, name) = self.graphics_target().context("未选择显卡")?;
        let path = self.settings.profiles_path("GraphicsProfiles.json")?;
        profile_store::save(&path, &key, &name, &self.graphics_record()?)?;
        self.select_written_game(&selected)
    }
    fn remove_game(&mut self) -> Result<()> {
        let app = self.games.selected.as_ref().context("未选择游戏")?;
        let driver = self.driver.as_ref().context("驱动不可用")?;
        driver.delete_game_app(app)?;
        let path = self.settings.profiles_path("GraphicsProfiles.json")?;
        let mut rows = profile_store::load(&path)?;
        rows.retain(|r| {
            !r["Application"]
                .as_str()
                .is_some_and(|s| s.eq_ignore_ascii_case(&app.file))
        });
        profile_store::write_json(&path, &json!(rows))?;
        self.read_device_details();
        Ok(())
    }
    pub(super) fn game_controls(&self, cx: &mut Context<Self>) -> Div {
        let mut row = div().flex().flex_wrap().gap_2();
        if self.devices.get(self.selected).is_none() {
            return row;
        }
        row = row.child(
            div()
                .id("game-global")
                .cursor_pointer()
                .px_2()
                .child("全局")
                .on_click(cx.listener(|p, _, _, cx| {
                    p.read_device_details();
                    cx.notify();
                })),
        );
        for (i, app) in self.games.list.iter().enumerate() {
            let app = app.clone();
            let label = if app.path == r"*\*" {
                app.file.clone()
            } else {
                format!("{} ({})", app.file, app.path)
            };
            let selected = self.games.selected.as_ref() == Some(&app);
            row = row.child(
                div()
                    .id(("game-item", i))
                    .cursor_pointer()
                    .px_2()
                    .bg(gpui::rgb(if selected {
                        self.palette.selected
                    } else {
                        self.palette.control
                    }))
                    .child(label)
                    .on_click(cx.listener(move |p, _, _, cx| {
                        if let Err(e) = p.load_game(app.clone()) {
                            p.graphics_status = e.to_string();
                        }
                        cx.notify();
                    })),
            );
        }
        let device = self.devices[self.selected].index;
        row = row.child(
            div()
                .id("game-add")
                .cursor_pointer()
                .px_2()
                .child("添加应用…")
                .on_click(cx.listener(move |_, _, _, cx| {
                    Self::pick_profile_app(cx, move |p, result| {
                        if p.devices.get(p.selected).is_some_and(|d| d.index == device) {
                            if let Err(e) = result.and_then(|name| match name {
                                Some(n) => p.add_game(n),
                                None => Ok(()),
                            }) {
                                p.graphics_status = e.to_string();
                            }
                        }
                    });
                })),
        );
        if self.games.selected.is_some() {
            row = row.child(
                div()
                    .id("game-delete")
                    .cursor_pointer()
                    .px_2()
                    .child(if self.games.remove {
                        "确认删除此游戏配置"
                    } else {
                        "删除游戏配置"
                    })
                    .on_click(cx.listener(|p, _, _, cx| {
                        if p.games.remove {
                            if let Err(e) = p.remove_game() {
                                p.graphics_status = e.to_string();
                            }
                        } else {
                            p.games.remove = true;
                        }
                        cx.notify();
                    })),
            );
            if self.games.remove {
                row = row.child(
                    div()
                        .id("game-cancel-delete")
                        .cursor_pointer()
                        .child("取消")
                        .on_click(cx.listener(|p, _, _, cx| {
                            p.games.remove = false;
                            cx.notify();
                        })),
                );
            }
        }
        row
    }
}
