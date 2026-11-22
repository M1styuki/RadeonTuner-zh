use super::super::read_registry;
use super::{Control, Device, Driver, Field, Handle, Pair};
use Field::*;
use anyhow::{Context, Result, ensure};
use std::{io, ptr};
use windows_sys::Win32::{
    Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
    System::Registry::*,
    UI::WindowsAndMessaging::{
        HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
    },
};

const AA_NAMES: [&str; 3] = ["ASD", "ASE", "ASTT"];
const AA_METHODS: [[i32; 3]; 3] = [[-1, 0, 0], [1, 0, 1], [1, 1, 1]];
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn env_flag(name: &str) -> bool {
    let mut text = [0u16; 8];
    let mut bytes = std::mem::size_of_val(&text) as u32;
    let code = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            wide("Environment").as_ptr(),
            wide(name).as_ptr(),
            RRF_RT_REG_SZ,
            ptr::null_mut(),
            text.as_mut_ptr().cast(),
            &mut bytes,
        )
    };
    if code != ERROR_SUCCESS {
        return false;
    }
    let s =
        String::from_utf16_lossy(&text[..text.iter().position(|v| *v == 0).unwrap_or(text.len())]);
    s == "1" || s.eq_ignore_ascii_case("true")
}
fn write_env_flag(name: &str, value: Option<bool>) -> Result<()> {
    let group = wide("Environment");
    let name = wide(name);
    let code = match value {
        Some(on) => {
            let text = wide(if on { "1" } else { "0" });
            unsafe {
                RegSetKeyValueW(
                    HKEY_CURRENT_USER,
                    group.as_ptr(),
                    name.as_ptr(),
                    REG_SZ,
                    text.as_ptr().cast(),
                    (text.len() * 2) as u32,
                )
            }
        }
        None => unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, group.as_ptr(), name.as_ptr()) },
    };
    if code != ERROR_SUCCESS && !(value.is_none() && code == ERROR_FILE_NOT_FOUND) {
        return Err(io::Error::from_raw_os_error(code as i32).into());
    }
    Ok(())
}
impl Driver {
    fn aa_method(&self, d: &Device) -> Option<usize> {
        let [a, b, c] = AA_NAMES.map(|key| read_registry(&d.registry_path, key));
        let raw = [a?, b?, c?];
        AA_METHODS.iter().position(|v| v == &raw)
    }
    pub(super) fn read_extras(&self, d: &Device, push: &mut impl FnMut(Field, i32, i32, i32, i32)) {
        push(Watermark, env_flag("MLSR-WATERMARK") as i32, 0, 1, 1);
        push(
            Swapchain,
            env_flag("FFX_DISABLE_ML_SWAPCHAIN") as i32,
            0,
            1,
            1,
        );
        if let Some(method) = self.aa_method(d) {
            push(AntiAliasing, method as i32, 0, 2, 1);
        }
        let (mut default, mut current) = (0, 0);
        if unsafe {
            self.library
                .get::<Pair>(b"ADL2_Workstation_DeepBitDepthX2_Get\0")
                .is_ok_and(|get| get(self.context, d.index, &mut default, &mut current) == 0)
        } {
            push(DeepColor, current, 0, 1, 1);
        }
    }
    pub(super) fn apply_extras(&self, d: &Device, controls: &[Control]) -> Result<()> {
        let change = |field| {
            controls
                .iter()
                .find(|c| c.field == field && c.current != c.draft)
                .map(|c| c.draft)
        };
        if let Some(next) = change(AntiAliasing) {
            let current = self.aa_method(d).context("无法读取抗锯齿方法")?;
            let target = AA_METHODS.get(next as usize).context("抗锯齿选项无效")?;
            for (index, key) in AA_NAMES.iter().enumerate() {
                if let Err(err) = self.set_umd_string(d, key, &target[index].to_string()) {
                    for (j, prior) in AA_NAMES[..index].iter().enumerate() {
                        self.set_umd_string(d, prior, &AA_METHODS[current][j].to_string())
                            .context("恢复原抗锯齿方法失败")?;
                    }
                    return Err(err);
                }
            }
        }
        if let Some(enabled) = change(DeepColor) {
            type Set = unsafe extern "C" fn(Handle, i32, i32) -> i32;
            let code = unsafe {
                self.library
                    .get::<Set>(b"ADL2_Workstation_DeepBitDepthX2_Set\0")?(
                    self.context,
                    d.index,
                    enabled,
                )
            };
            ensure!(
                matches!(code, 0 | 3),
                "设置 OpenGL 10 位像素格式失败 ({code})"
            );
        }
        if let Some(on) = change(Watermark) {
            let value = (on != 0).then_some(true);
            write_env_flag("MLFI-WATERMARK", value)?;
            write_env_flag("MLSR-WATERMARK", value)?;
        }
        if let Some(on) = change(Swapchain) {
            write_env_flag("FFX_DISABLE_ML_SWAPCHAIN", Some(on != 0))?;
        }
        if change(Watermark).is_some() || change(Swapchain).is_some() {
            let group = wide("Environment");
            unsafe {
                SendMessageTimeoutW(
                    HWND_BROADCAST,
                    WM_SETTINGCHANGE,
                    0,
                    group.as_ptr() as isize,
                    SMTO_ABORTIFHUNG,
                    2000,
                    ptr::null_mut(),
                );
            }
        }
        Ok(())
    }
}
