use crate::adl::display::gamma::Gamma;
use crate::{
    adl::{Device, Driver, overdrive::Control},
    settings::Settings,
};
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    mem,
    os::windows::io::{FromRawHandle, OwnedHandle},
    path::PathBuf,
};
use windows_sys::Win32::{
    Foundation::{ERROR_NO_MORE_FILES, GetLastError, INVALID_HANDLE_VALUE},
    System::Diagnostics::ToolHelp::*,
};

pub fn process_names() -> Result<HashSet<String>> {
    Ok(process_list()?
        .into_iter()
        .map(|(_, s)| s.to_lowercase())
        .collect())
}
pub fn process_list() -> Result<Vec<(u32, String)>> {
    unsafe {
        let handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        ensure!(handle != INVALID_HANDLE_VALUE, "无法读取正在运行的应用");
        let _owner = OwnedHandle::from_raw_handle(handle);
        let mut entry: PROCESSENTRY32W = mem::zeroed();
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut ok = Process32FirstW(handle, &mut entry);
        let mut names = Vec::new();
        while ok != 0 {
            let length = entry
                .szExeFile
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(entry.szExeFile.len());
            names.push((
                entry.th32ProcessID,
                String::from_utf16_lossy(&entry.szExeFile[..length]),
            ));
            ok = Process32NextW(handle, &mut entry);
        }
        ensure!(GetLastError() == ERROR_NO_MORE_FILES, "未能读完应用列表");
        Ok(names)
    }
}

pub fn select_profile<'a>(
    profiles: &'a [Value],
    device_key: &str,
    running: &HashSet<String>,
) -> Option<&'a Value> {
    let mut global = None;
    for profile in profiles
        .iter()
        .filter(|p| p["DeviceId"].as_str() == Some(device_key))
    {
        if let Some(app) = profile["Application"].as_str() {
            if app.eq_ignore_ascii_case("Global") {
                global = Some(profile);
            } else if running.contains(&app.to_lowercase()) {
                return Some(profile);
            }
        }
    }
    global
}

pub fn display_profiles<'a>(rows: &'a [Value], key: &str) -> Vec<&'a Value> {
    let legacy = key.rsplit_once(' ').map(|v| v.0);
    let mut profiles: Vec<&Value> = rows
        .iter()
        .filter(|row| row["DeviceId"].as_str() == Some(key))
        .collect();
    let exact = profiles.len();
    for row in rows.iter().filter(|row| {
        legacy.is_some()
            && row["DeviceId"].as_str() == legacy
            && !row["ExcludedDisplayIds"]
                .as_array()
                .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(key)))
    }) {
        if let Some(name) = row["Application"].as_str() {
            if !profiles[..exact].iter().any(|p| {
                p["Application"]
                    .as_str()
                    .is_some_and(|s| s.eq_ignore_ascii_case(name))
            }) {
                profiles.push(row);
            }
        }
    }
    profiles
}

pub fn automatic_eye(profile: &Value) -> bool {
    let v = &profile["EyefinityAutomatic.Current"];
    v.as_bool()
        .unwrap_or_else(|| v.as_i64().is_some_and(|n| n != 0))
}

fn wanted_eyefinity(profiles: &[&Value], running: &HashSet<String>) -> Option<bool> {
    let mut result = None;
    for row in profiles.iter().filter(|r| automatic_eye(r)) {
        if let Some(app) = row["Application"]
            .as_str()
            .filter(|s| !s.eq_ignore_ascii_case("Global"))
        {
            if running.contains(&app.to_lowercase()) {
                return Some(true);
            }
            result = Some(false);
        }
    }
    result
}

pub fn active_display<'a>(profiles: &[&'a Value], running: &HashSet<String>) -> Option<&'a Value> {
    profiles
        .iter()
        .copied()
        .find(|p| {
            p["Application"].as_str().is_some_and(|s| {
                !s.eq_ignore_ascii_case("Global") && running.contains(&s.to_lowercase())
            })
        })
        .or_else(|| {
            profiles.iter().copied().find(|p| {
                p["Application"]
                    .as_str()
                    .is_some_and(|s| s.eq_ignore_ascii_case("Global"))
            })
        })
}

pub fn display_draft(
    settings: &mut [crate::adl::display::settings::Setting],
    profile: &Value,
    per_app: bool,
) -> Result<()> {
    for setting in settings
        .iter_mut()
        .filter(|s| !per_app || s.feature.per_app())
    {
        if let Some(value) = number(profile, setting.feature.profile_name())? {
            ensure!(
                (setting.min..=setting.max).contains(&value),
                "{} 超出显示器支持范围",
                setting.label
            );
            setting.draft = value;
        }
    }
    Ok(())
}

fn number(profile: &Value, name: &str) -> Result<Option<i32>> {
    let value = &profile[format!("{name}.Current")];
    if value.is_null() {
        return Ok(None);
    }
    if let Some(flag) = value.as_bool() {
        return Ok(Some(i32::from(flag)));
    }
    Ok(Some(i32::try_from(
        value.as_i64().context("配置中的数值无效")?,
    )?))
}

pub fn curve(start: i32, end: i32, quiet: bool) -> [i32; 5] {
    let weights = if quiet {
        [0., 0.2, 0.4, 0.6, 1.]
    } else {
        [0., 0.25, 0.5, 0.75, 1.]
    };
    weights.map(|w| (start as f32 + (end as f32 - start as f32) * w).round() as i32)
}

pub fn tuning_draft(controls: &mut [Control], profile: &Value) -> Result<()> {
    let fan = number(profile, "FanControlMode")?.unwrap_or(0);
    ensure!((0..=3).contains(&fan), "配置中的风扇模式无效");
    let range = match (
        number(profile, "FanSpeedMin")?,
        number(profile, "FanSpeedMax")?,
    ) {
        (Some(a), Some(b)) if fan == 2 => {
            ensure!(a <= b, "配置中的风扇转速范围无效");
            Some(curve(
                a,
                b,
                number(profile, "FanRangedCurve")?.unwrap_or(0) != 0,
            ))
        }
        _ => None,
    };
    for control in controls {
        let value = match control.id {
            0 => number(profile, "CoreMax")?,
            1 => number(profile, "CoreMin")?,
            8 => number(profile, "MemoryMax")?,
            crate::adl::overdrive::ECC => number(profile, "MemoryEcc")?,
            9 => number(profile, "PowerLimit")?,
            37 => number(profile, "PowerVoltage")?,
            47 => number(profile, "PowerTDC")?,
            14 => number(profile, "MemoryTiming")?,
            15 if fan != 0 => number(profile, "FanZeroRpm")?,
            15 | 19..=28 if fan < 2 => {
                control.reset = control.current != control.default;
                Some(control.default)
            }
            19..=28 if fan == 2 => range.map(|c| {
                if control.id % 2 == 1 {
                    control.default
                } else {
                    c[(control.id - 20) / 2]
                }
            }),
            19..=28 => {
                let point = (control.id - 19) / 2;
                number(
                    profile,
                    &format!(
                        "Fan{}{point}",
                        if control.id % 2 == 1 { "Temp" } else { "Speed" }
                    ),
                )?
            }
            _ => None,
        };
        if let Some(value) = value {
            ensure!(
                (control.min..=control.max).contains(&value),
                "{} 超出驱动支持范围",
                control.label
            );
            control.draft = value;
        }
    }
    Ok(())
}

pub struct Background {
    gamma_owned: HashSet<String>,
    path: PathBuf,
    profiles: Vec<Value>,
    display_path: PathBuf,
    display_rows: Vec<Value>,
    applied_display: HashMap<(i32, i32, String), Value>,
    driver: Option<Driver>,
    applied: HashMap<i32, Value>,
}

impl Background {
    pub fn new(settings: &Settings) -> Result<Self> {
        Ok(Self {
            path: settings.profiles_path("TuningProfiles.json")?,
            profiles: Vec::new(),
            display_path: settings.profiles_path("DisplayProfiles.json")?,
            display_rows: Vec::new(),
            gamma_owned: HashSet::new(),
            applied_display: HashMap::new(),
            driver: None,
            applied: HashMap::new(),
        })
    }
    pub fn reload(&mut self) -> Result<()> {
        let tuning = crate::profile_store::load(&self.path)?;
        let displays = crate::profile_store::load(&self.display_path)?;
        if tuning != self.profiles {
            self.applied.clear();
            self.profiles = tuning;
        }
        if displays != self.display_rows {
            self.applied_display.clear();
            self.display_rows = displays;
        }
        Ok(())
    }
    pub fn active(&self) -> bool {
        !self.profiles.is_empty() || !self.display_rows.is_empty() || !self.gamma_owned.is_empty()
    }
    pub fn tick(&mut self) -> Result<()> {
        if !self.active() {
            return Ok(());
        }
        if self.driver.is_none() {
            self.driver = Some(Driver::open()?);
        }
        let driver = self.driver.as_ref().context("无法打开 AMD 驱动")?;
        let running = process_names()?;
        let devices = driver.devices()?;
        let mut display_error = None;
        self.gamma_owned
            .retain(|id| devices.iter().any(|d| &d.gamma_key == id));
        if !self.display_rows.is_empty() || !self.gamma_owned.is_empty() {
            let mut connected = HashSet::new();
            for device in &devices {
                let mut eyefinity = None;
                let mut gamma = None;
                let mut gamma_valid = true;
                for display in driver.displays(device.index)? {
                    let Some(key) = driver.display_profile_key(&display) else {
                        continue;
                    };
                    let candidates = display_profiles(&self.display_rows, &key);
                    if let Some(wanted) = wanted_eyefinity(&candidates, &running) {
                        eyefinity = Some(eyefinity.unwrap_or(false) || wanted);
                    }
                    let Some(profile) = active_display(&candidates, &running) else {
                        continue;
                    };
                    let per_app = profile["Application"]
                        .as_str()
                        .is_some_and(|a| !a.eq_ignore_ascii_case("Global"));
                    match Gamma::read(profile) {
                        Ok(g) => {
                            // The LUT is shared across an adapter. Running application
                            // profiles take priority, then enabled global profiles.
                            let rank = (per_app, g.enabled);
                            if gamma.as_ref().is_none_or(|(priority, _)| rank > *priority) {
                                gamma = Some((rank, g));
                            }
                        }
                        Err(e) => {
                            display_error = Some(e);
                            gamma_valid = false;
                        }
                    }
                    let target = (device.index, display.logical, key);
                    connected.insert(target.clone());
                    if self.applied_display.get(&target) == Some(profile) {
                        continue;
                    }
                    let mut settings = driver.display_settings(&display);
                    display_draft(&mut settings, profile, true)?;
                    driver.apply_display(&display, &settings)?;
                    self.applied_display.insert(target, profile.clone());
                }
                if let Some((_, g)) = gamma {
                    if let Err(e) = driver.sync_gamma(device, g, &mut self.gamma_owned) {
                        display_error = Some(e);
                    }
                } else if gamma_valid {
                    if let Err(e) =
                        driver.sync_gamma(device, Gamma::default(), &mut self.gamma_owned)
                    {
                        display_error = Some(e);
                    }
                }
                if let Some(wanted) = eyefinity {
                    if let Err(e) = driver.set_eyefinity(device.index, wanted) {
                        display_error = Some(e);
                    }
                }
            }
            self.applied_display
                .retain(|key, _| connected.contains(key));
        }
        for Device {
            index, profile_key, ..
        } in devices
        {
            if profile_key.is_empty() {
                continue;
            }
            let Some(profile) = select_profile(&self.profiles, &profile_key, &running) else {
                continue;
            };
            if self.applied.get(&index) == Some(profile)
                && number(profile, "KeepActive")?.unwrap_or(0) == 0
            {
                continue;
            }
            let mut controls = driver.controls(index)?;
            tuning_draft(&mut controls, profile)?;
            if controls.iter().any(|c| c.current != c.draft || c.reset) {
                driver.apply(index, &controls)?;
            }
            self.applied.insert(index, profile.clone());
        }
        if let Some(e) = display_error {
            return Err(e);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deleting_an_inherited_game_from_one_monitor_preserves_its_twin() {
        let rows = [
            json!({"DeviceId":"7 Panel","Application":"game.exe","ExcludedDisplayIds":["7 Panel 100"]}),
        ];
        assert!(display_profiles(&rows, "7 Panel 100").is_empty());
        assert_eq!(display_profiles(&rows, "7 Panel 101").len(), 1);
    }
    #[test]
    fn automatic_eyefinity_tracks_only_enabled_application_profiles() {
        let global = json!({"Application":"Global","EyefinityAutomatic.Current":1});
        let game = json!({"Application":"Game.EXE","EyefinityAutomatic.Current":1});
        let disabled = json!({"Application":"other.exe","EyefinityAutomatic.Current":0});
        let running = HashSet::from(["game.exe".into()]);
        assert_eq!(wanted_eyefinity(&[&global, &disabled], &running), None);
        assert_eq!(wanted_eyefinity(&[&game], &HashSet::new()), Some(false));
        assert_eq!(wanted_eyefinity(&[&global, &game], &running), Some(true));
    }
    #[test]
    fn legacy_profile_order_and_global_fallback() {
        let profiles = [
            json!({"DeviceId":"A","Application":"Global"}),
            json!({"DeviceId":"B","Application":"Game.exe"}),
            json!({"DeviceId":"A","Application":"Game.exe"}),
            json!({"DeviceId":"A","Application":"Other.exe"}),
        ];
        let running = ["game.exe".to_string(), "other.exe".to_string()]
            .into_iter()
            .collect();
        assert_eq!(select_profile(&profiles, "A", &running), Some(&profiles[2]));
        assert_eq!(
            select_profile(&profiles, "A", &HashSet::new()),
            Some(&profiles[0])
        );
        assert_eq!(select_profile(&profiles, "C", &running), None);
    }

    #[test]
    fn monitor_serial_overrides_only_the_matching_application() {
        let rows = [
            json!({"DeviceId":"23 TV","Application":"Global"}),
            json!({"DeviceId":"23 TV","Application":"Game.exe"}),
            json!({"DeviceId":"23 TV 17","Application":"game.exe"}),
            json!({"DeviceId":"23 TV 18","Application":"Other.exe"}),
        ];
        let running = ["game.exe".to_string()].into_iter().collect();
        let exact = display_profiles(&rows, "23 TV 17");
        assert_eq!(active_display(&exact, &running), Some(&rows[2]));
        assert_eq!(active_display(&exact, &HashSet::new()), Some(&rows[0]));
        assert_eq!(
            active_display(&display_profiles(&rows, "23 TV 18"), &running),
            Some(&rows[1])
        );
    }
    #[test]
    fn legacy_fan_curve_interpolation() {
        assert_eq!(curve(25, 75, false), [25, 38, 50, 63, 75]);
        assert_eq!(curve(25, 75, true), [25, 35, 45, 55, 75]);
    }
}
