use crate::{
    adl::{Device, Driver, overdrive::Control},
    settings::Settings,
};
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs, io, mem,
    os::windows::io::{FromRawHandle, OwnedHandle},
    path::PathBuf,
};
use windows_sys::Win32::{
    Foundation::{ERROR_NO_MORE_FILES, GetLastError, INVALID_HANDLE_VALUE},
    System::Diagnostics::ToolHelp::*,
};

fn process_names() -> Result<HashSet<String>> {
    unsafe {
        let handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        ensure!(handle != INVALID_HANDLE_VALUE, "无法读取正在运行的应用");
        let _owner = OwnedHandle::from_raw_handle(handle);
        let mut entry: PROCESSENTRY32W = mem::zeroed();
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut ok = Process32FirstW(handle, &mut entry);
        let mut names = HashSet::new();
        while ok != 0 {
            let length = entry
                .szExeFile
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(entry.szExeFile.len());
            names.insert(String::from_utf16_lossy(&entry.szExeFile[..length]).to_lowercase());
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

fn curve(start: i32, end: i32, quiet: bool) -> [i32; 5] {
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
    path: PathBuf,
    profiles: Vec<Value>,
    driver: Option<Driver>,
    applied: HashMap<i32, Value>,
}

impl Background {
    pub fn new(settings: &Settings) -> Result<Self> {
        Ok(Self {
            path: settings.profiles_path("TuningProfiles.json")?,
            profiles: Vec::new(),
            driver: None,
            applied: HashMap::new(),
        })
    }
    pub fn reload(&mut self) -> Result<()> {
        self.applied.clear();
        self.profiles.clear();
        match fs::read(&self.path) {
            Ok(bytes) => {
                self.profiles = serde_json::from_slice(
                    bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes),
                )?;
                Ok(())
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
    pub fn active(&self) -> bool {
        !self.profiles.is_empty()
    }
    pub fn tick(&mut self) -> Result<()> {
        if self.profiles.is_empty() {
            return Ok(());
        }
        if self.driver.is_none() {
            self.driver = Some(Driver::open()?);
        }
        let driver = self.driver.as_ref().context("无法打开 AMD 驱动")?;
        let running = process_names()?;
        for Device {
            index, profile_key, ..
        } in driver.devices()?
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
    fn legacy_fan_curve_interpolation() {
        assert_eq!(curve(25, 75, false), [25, 38, 50, 63, 75]);
        assert_eq!(curve(25, 75, true), [25, 35, 45, 55, 75]);
    }
}
