use super::{
    apps::{App, settings},
    profile_drafts, registry_data, registry_text,
};
use crate::{
    adl::{Device, Driver, Handle},
    profile_store,
    settings::Settings,
};
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::collections::HashSet;

impl Driver {
    fn initialize_app_gpu(&self, d: &Device) -> Result<()> {
        type Get = unsafe extern "C" fn(Handle, i32, i32, *const i8, *const i8, *mut i32) -> i32;
        type Set = unsafe extern "C" fn(Handle, i32, i32, *const i8, *const i8, i32) -> i32;
        let mut val = 0;
        unsafe {
            let get = self.library.get::<Get>(b"ADL2_Adapter_RegValueInt_Get\0")?;
            if get(
                self.context,
                d.index,
                1,
                c"".as_ptr(),
                c"KMD_IsGamingDriver".as_ptr(),
                &mut val,
            ) != 0
                || val != 1
            {
                let rc = self.library.get::<Set>(b"ADL2_Adapter_RegValueInt_Set\0")?(
                    self.context,
                    d.index,
                    1,
                    c"".as_ptr(),
                    c"KMD_IsGamingDriver".as_ptr(),
                    1,
                );
                ensure!(rc == 0, "设置 AMD 游戏驱动标志失败 ({rc})");
            }
        }
        if registry_data(&d.registry_path, "AppGpuId")
            .and_then(|(k, b)| registry_text(k, &b))
            .as_deref()
            != Some(super::apps::GPU)
        {
            self.set_umd_string(d, "AppGpuId", super::apps::GPU)?;
        }
        Ok(())
    }
    fn restore_game(&self, d: &Device, name: &str, row: &Value) -> Result<()> {
        let (mut choices, fields) = settings::settings(&[], |n| self.app_kind(n).is_some());
        let (indices, fields) = profile_drafts(&choices, &fields, row)?;
        for (c, i) in choices.iter_mut().zip(indices) {
            c.draft = i;
        }
        let mut library = self
            .app_kind("FfxDllPath")
            .and_then(|_| self.fsr_library(d, true));
        if let Some(l) = &mut library {
            l.select(
                row["FsrOvrDLLPath.Current"]
                    .as_str()
                    .unwrap_or_default()
                    .trim()
                    .into(),
            );
        }
        let props = self.game_properties_from_controls(
            Vec::new(),
            &choices,
            &fields,
            library.as_ref(),
            true,
        )?;
        ensure!(!props.is_empty(), "驱动未提供应用配置属性");
        self.write_game_app(&App::new(name.into()), &props)
    }
    pub fn initialize_profiles(&self, config: &Settings) -> Result<()> {
        let devices = self.enumerate_devices(false)?;
        if devices.is_empty() {
            return Ok(());
        }
        let mut errors = Vec::new();
        for d in &devices {
            if let Err(e) = self.initialize_app_gpu(d) {
                errors.push(format!("{}：{e}", d.name));
            }
        }
        let path = config.profiles_path("GraphicsProfiles.json")?;
        if path.is_file() {
            let rows = profile_store::load(&path)?;
            let apps = self.game_apps()?;
            let names: HashSet<_> = rows
                .iter()
                .filter_map(|r| r["Application"].as_str())
                .map(str::to_lowercase)
                .collect();
            // Remove stale RadeonTuner assignments only after loading the saved
            // inventory. Another application's driver profiles have their own owner.
            for a in &apps {
                if a.profile.starts_with("RADT{")
                    && a.path == r"*\*"
                    && !names.contains(&a.file.to_lowercase())
                {
                    if let Err(e) = self.delete_game_app(a) {
                        errors.push(format!("{}：{e}", a.file));
                    }
                }
            }
            let mut assigned: HashSet<_> = apps
                .iter()
                .filter(|a| a.path == r"*\*")
                .map(|a| a.file.to_lowercase())
                .collect();
            for row in &rows {
                let Some(name) = row["Application"]
                    .as_str()
                    .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("Global"))
                else {
                    continue;
                };
                if assigned.contains(&name.to_lowercase()) {
                    continue;
                }
                if let Some(device) = devices
                    .iter()
                    .find(|d| row["DeviceId"].as_str() == Some(&d.profile_key))
                {
                    match self.restore_game(device, name, row) {
                        Ok(()) => {
                            assigned.insert(name.to_lowercase());
                        }
                        Err(e) => errors.push(format!("{name}：{e}")),
                    }
                }
            }
        }
        ensure!(errors.is_empty(), "{}", errors.join("；"));
        Ok(())
    }
}
