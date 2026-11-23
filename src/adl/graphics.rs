pub mod activity;
pub mod apps;
pub mod frames;
pub mod fsr;
mod startup;
use super::{Device, Driver, Handle, ReadInt};
use anyhow::{Context as _, Result, ensure};
use serde_json::Value;
use std::{ffi::CString, ptr};
use windows_sys::Win32::{Foundation::ERROR_SUCCESS, System::Registry::*};

pub struct Choice {
    pub key: &'static str,
    pub profile_key: &'static str,
    pub label: &'static str,
    pub entries: &'static [(&'static str, i32)],
    caps: Option<&'static [u8]>,
    pub experimental: bool,
}
const SWITCH: &[(&str, i32)] = &[("关", 0), ("开", 1)];
macro_rules! choice {
    ($key:literal,$profile:literal,$label:literal,$entries:expr) => {
        Choice {
            key: $key,
            profile_key: concat!($profile, ".Current"),
            label: $label,
            entries: $entries,
            caps: None,
            experimental: false,
        }
    };
    ($key:literal,$profile:literal,$label:literal,$entries:expr,$caps:literal,$experimental:literal) => {
        Choice {
            key: $key,
            profile_key: concat!($profile, ".Current"),
            label: $label,
            entries: $entries,
            caps: Some(concat!($caps, "\0").as_bytes()),
            experimental: $experimental,
        }
    };
}
const CHOICES: &[Choice] = &[
    choice!(
        "FsrOverride",
        "FsrOverride",
        "FSR 超分辨率覆盖",
        SWITCH,
        "ADL2_DriverMLSRSupport_Get",
        false
    ),
    choice!(
        "MlfiOverride",
        "MlfiOverride",
        "FSR 帧生成覆盖",
        SWITCH,
        "ADL2_DriverMLFISupport_Get",
        false
    ),
    choice!(
        "MfgOverride",
        "MfgOverride",
        "多帧生成覆盖",
        SWITCH,
        "ADL2_DriverMFGSupport_Get",
        true
    ),
    choice!(
        "MldOverride",
        "MldOverride",
        "光线重建覆盖",
        SWITCH,
        "ADL2_DriverMLDSupport_Get",
        true
    ),
    choice!(
        "NrcOverride",
        "NrcOverride",
        "神经辐射缓存覆盖",
        SWITCH,
        "ADL2_DriverNRCSupport_Get",
        true
    ),
    choice!(
        "MfgRatio",
        "MfgRatio",
        "多帧生成倍率",
        &[("使用应用程序设置", 0), ("2×", 2), ("3×", 3), ("4×", 4)],
        "ADL2_DriverMFGSupport_Get",
        true
    ),
    choice!(
        "TurboSync",
        "EnhancedSync",
        "增强同步",
        SWITCH,
        "ADL2_TurboSyncSupport_Get",
        false
    ),
    choice!(
        "VSyncControl",
        "VerticalSync",
        "垂直同步",
        &[
            ("始终关闭", 0),
            ("关闭，除非应用程序指定", 1),
            ("开启，除非应用程序指定", 2),
            ("始终开启", 3)
        ]
    ),
    choice!(
        "AntiAlias",
        "AntiAliasingOverride",
        "抗锯齿覆盖",
        &[("使用应用程序设置", 1), ("覆盖应用程序设置", 2)]
    ),
    choice!(
        "AntiAliasSamples",
        "AntiAliasingLevel",
        "抗锯齿等级",
        &[("2×", 2), ("4×", 4), ("8×", 8)]
    ),
    choice!(
        "EQAA",
        "AntiAliasingEnhancedQuality",
        "增强质量抗锯齿",
        SWITCH
    ),
    choice!("MLF", "AntiAliasingMorphological", "形态抗锯齿", SWITCH),
    choice!(
        "AnisoDegree",
        "AnisotropicOverride",
        "各向异性过滤",
        &[
            ("应用程序设置", 0),
            ("2×", 2),
            ("4×", 4),
            ("8×", 8),
            ("16×", 16)
        ]
    ),
    choice!(
        "TFQ",
        "TextureFilteringQuality",
        "纹理过滤质量",
        &[("高", 0), ("标准", 1), ("性能", 2)]
    ),
    choice!(
        "SurfaceFormatReplacements",
        "SurfaceFormatOptimization",
        "表面格式优化",
        SWITCH
    ),
    choice!(
        "Tessellation_OPTION",
        "TessellationMode",
        "曲面细分模式",
        &[
            ("AMD 优化", 0),
            ("使用应用程序设置", 1),
            ("覆盖应用程序设置", 2)
        ]
    ),
    choice!(
        "Tessellation",
        "TessellationLevel",
        "曲面细分等级",
        &[
            ("关闭", 1),
            ("2×", 2),
            ("4×", 4),
            ("6×", 6),
            ("8×", 8),
            ("16×", 16),
            ("32×", 32),
            ("64×", 64)
        ]
    ),
    choice!(
        "EnableTripleBuffering",
        "OpenGLTripleBuffering",
        "OpenGL 三重缓冲",
        SWITCH
    ),
];
pub struct Setting {
    pub choice: &'static Choice,
    pub current: usize,
    pub draft: usize,
}

fn setting_indices(settings: &[Setting], row: &Value) -> Result<Vec<usize>> {
    settings
        .iter()
        .map(|s| {
            let v = &row[s.choice.profile_key];
            if v.is_null() {
                return Ok(s.draft);
            }
            let index = v
                .as_u64()
                .or_else(|| v.as_bool().map(u64::from))
                .and_then(|n| usize::try_from(n).ok())
                .context("图形配置的选项无效")?;
            ensure!(
                index < s.choice.entries.len(),
                "{} 超出可选范围",
                s.choice.label
            );
            Ok(index)
        })
        .collect()
}
pub fn profile_drafts(
    settings: &[Setting],
    fields: &[frames::Control],
    row: &Value,
) -> Result<(Vec<usize>, Vec<frames::Control>)> {
    let indices = setting_indices(settings, row)?;
    let mut frames = fields.to_vec();
    for c in &mut frames {
        let v = &row[c.field.name()];
        if !v.is_null() {
            let value = v
                .as_i64()
                .or_else(|| v.as_bool().map(i64::from))
                .context("帧率配置数值无效")?;
            if c.field == crate::adl::graphics::frames::Field::Target && value == 0 {
                continue;
            }
            ensure!(
                (i64::from(c.min)..=i64::from(c.max)).contains(&value),
                "{} 超出驱动范围",
                c.field.label()
            );
            c.draft = value as i32;
        }
    }
    crate::adl::graphics::frames::validate(&frames)?;
    Ok((indices, frames))
}
fn registry_data(path: &str, name: &str) -> Option<(u32, Vec<u8>)> {
    let path: Vec<_> = format!("SYSTEM\\CurrentControlSet\\Control\\Class\\{path}\\UMD")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let name: Vec<_> = name.encode_utf16().chain(Some(0)).collect();
    let mut kind = 0;
    let mut bytes = 0;
    let read = |data: *mut std::ffi::c_void, kind: &mut u32, bytes: &mut u32| unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            path.as_ptr(),
            name.as_ptr(),
            RRF_RT_ANY,
            kind,
            data,
            bytes,
        )
    };
    if read(ptr::null_mut(), &mut kind, &mut bytes) != ERROR_SUCCESS || bytes == 0 {
        return None;
    }
    let mut buffer = vec![0u8; bytes as usize];
    if read(buffer.as_mut_ptr().cast(), &mut kind, &mut bytes) != ERROR_SUCCESS {
        return None;
    }
    buffer.truncate(bytes as usize);
    Some((kind, buffer))
}

fn registry_text(kind: u32, buffer: &[u8]) -> Option<String> {
    if !matches!(kind, REG_BINARY | REG_SZ) || buffer.len() % 2 != 0 {
        return None;
    }
    let wide: Vec<_> = buffer
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .take_while(|v| *v != 0)
        .collect();
    String::from_utf16(&wide).ok()
}

fn read_registry(path: &str, name: &str) -> Option<i32> {
    let (kind, buffer) = registry_data(path, name)?;
    if kind == REG_DWORD || (kind == REG_BINARY && buffer.len() == 4 && buffer[2] == 0) {
        return Some(i32::from_le_bytes(buffer.as_slice().try_into().ok()?));
    }
    registry_text(kind, &buffer)?.trim().parse().ok()
}

impl Driver {
    pub fn graphics(&self, d: &Device, skip_caps: bool) -> Vec<Setting> {
        CHOICES
            .iter()
            .filter_map(|choice| {
                if !skip_caps {
                    if let Some(caps) = choice.caps {
                        let mut supported = 0;
                        if unsafe {
                            self.library.get::<ReadInt>(caps).ok()?(
                                self.context,
                                d.index,
                                &mut supported,
                            )
                        } != 0
                            || supported == 0
                        {
                            return None;
                        }
                    }
                }
                let raw = read_registry(&d.registry_path, choice.key)?;
                let current = choice.entries.iter().position(|(_, v)| *v == raw)?;
                Some(Setting {
                    choice,
                    current,
                    draft: current,
                })
            })
            .collect()
    }
    fn set_umd_string(&self, d: &Device, name: &str, text: &str) -> Result<()> {
        type Set =
            unsafe extern "C" fn(Handle, i32, i32, *const i8, *const i8, i32, *const u16) -> i32;

        let set = unsafe {
            self.library
                .get::<Set>(b"ADL2_Adapter_RegValueString_Set\0")?
        };
        let value: Vec<_> = text.encode_utf16().chain(Some(0)).collect();
        let key = CString::new(name)?;
        let code = unsafe {
            set(
                self.context,
                d.index,
                1,
                c"UMD".as_ptr(),
                key.as_ptr(),
                ((value.len() - 1) * 2) as i32,
                value.as_ptr(),
            )
        };
        ensure!(code == 0, "设置{}失败 ({code})", name);
        Ok(())
    }
    pub fn apply_graphics(&self, d: &Device, settings: &[Setting]) -> Result<()> {
        type Notify = unsafe extern "C" fn(Handle, i32, i32, i32) -> i32;
        for s in settings.iter().filter(|s| s.current != s.draft) {
            ensure!(
                read_registry(&d.registry_path, s.choice.key)
                    == Some(s.choice.entries[s.current].1),
                "{}已改变，请刷新",
                s.choice.label
            );
            self.set_umd_string(d, s.choice.key, &s.choice.entries[s.draft].1.to_string())?;
            if s.choice.key == "TurboSync" {
                let status = unsafe {
                    self.library.get::<Notify>(b"ADL2_User_Settings_Notify\0")?(
                        self.context,
                        d.index,
                        1,
                        1,
                    )
                };
                ensure!(status == 0, "增强同步已保存，通知驱动失败 ({status})");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod profile_tests {
    use super::*;

    #[test]
    fn driver_values_map_to_native_profile_indices() {
        for (registry, key, driver_value, expected_index) in [
            ("AntiAlias", "AntiAliasingOverride.Current", 2, 1),
            ("MfgRatio", "MfgRatio.Current", 4, 3),
            ("Tessellation", "TessellationLevel.Current", 64, 7),
            ("AnisoDegree", "AnisotropicOverride.Current", 8, 3),
        ] {
            let choice = CHOICES.iter().find(|s| s.key == registry).unwrap();
            assert_eq!(choice.profile_key, key);
            assert_eq!(choice.entries[expected_index].1, driver_value);
        }
    }
}
