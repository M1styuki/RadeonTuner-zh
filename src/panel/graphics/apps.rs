use super::*;
use crate::adl::graphics::apps::{App, settings};
#[derive(Default)]
pub(in crate::panel) struct Games {
    list: Vec<App>,
    pub(super) selected: Option<App>,
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
        self.graphics_profile = json!({});
        self.confirm_graphics_reset = false;
        self.graphics_dirty = false;
        self.graphics_status.clear();
        self.read_graphics_profile()
    }
    pub(in crate::panel) fn add_game(&mut self, file: String) -> Result<()> {
        let _write = crate::driver_write::lock()?;
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
        let _write = crate::driver_write::lock()?;
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
        let (key, name) = self.graphics_target().context("未选择显卡")?;
        let path = self.settings.profiles_path("GraphicsProfiles.json")?;
        let mut rows = profile_store::load(&path)?;
        profile_store::upsert(&mut rows, &key, &name, &self.graphics_record()?, false)?;
        let staged = profile_store::staged_json(&path, &json!(&rows))?;
        if updated != original {
            driver.write_game_app(&app, &updated)?;
        }
        if let Err(err) = staged.persist(&path) {
            if updated != original {
                driver
                    .write_game_app(&app, &original)
                    .context("保存应用配置失败，无法恢复驱动数据")?;
            }
            return Err(err).context("无法保存应用图形配置");
        }
        self.select_written_game(&selected)
    }
    pub(in crate::panel) fn game_names(&self) -> Vec<String> {
        self.games.list.iter().map(|a| a.file.clone()).collect()
    }
    pub(in crate::panel) fn remove_games(&mut self, names: &[String]) -> Result<()> {
        let _write = crate::driver_write::lock()?;
        let driver = self.driver.as_ref().context("驱动不可用")?;
        let path = self.settings.profiles_path("GraphicsProfiles.json")?;
        let mut rows = profile_store::load(&path)?;
        let mut errors = Vec::new();
        let apps = driver.game_apps()?;
        for name in names {
            for app in apps.iter().filter(|a| a.file.eq_ignore_ascii_case(name)) {
                if let Err(e) = driver.delete_game_app(app) {
                    errors.push(format!("{name}：{e}"));
                }
            }
        }
        let remaining = driver.game_apps()?;
        rows.retain(|r| {
            !r["Application"].as_str().is_some_and(|s| {
                names.iter().any(|n| n.eq_ignore_ascii_case(s))
                    && !remaining.iter().any(|a| a.file.eq_ignore_ascii_case(s))
            })
        });
        profile_store::write_json(&path, &json!(rows))?;
        self.read_device_details();
        ensure!(errors.is_empty(), "{}", errors.join("；"));
        Ok(())
    }
    pub(super) fn game_controls(&self, cx: &mut Context<Self>) -> Div {
        let mut row = div().flex().items_center().gap_2();
        if self.devices.get(self.selected).is_none() {
            return row;
        }
        let apps = self.games.list.clone();
        let selected = 1 + self
            .games
            .selected
            .as_ref()
            .and_then(|app| apps.iter().position(|a| a == app))
            .map(|i| i as isize)
            .unwrap_or(-1);
        let mut labels = vec!["全局".to_owned()];
        labels.extend(apps.iter().map(|a| {
            if a.path == r"*\*" {
                a.file.clone()
            } else {
                format!("{} ({})", a.file, a.path)
            }
        }));
        row = row.child(self.select(
            "game-app",
            labels,
            Some(selected as usize),
            "选择应用程序",
            160.,
            move |p, i, _| {
                if i == 0 {
                    p.read_device_details();
                } else if let Err(e) = p.load_game(apps[i - 1].clone()) {
                    p.graphics_status = e.to_string();
                }
            },
            cx,
        ));
        row.child(self.app_picker_buttons(crate::panel::picker::Target::Graphics, cx))
    }
}
