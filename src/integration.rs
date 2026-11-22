use anyhow::{Result, ensure};
use std::{
    fs, io,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
    ptr,
};
use windows_sys::Win32::{
    Foundation::ERROR_SUCCESS,
    System::{Com::CoTaskMemFree, Registry::*},
    UI::{Shell::*, WindowsAndMessaging::SW_SHOWNORMAL},
};
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

pub fn open(target: &str) -> Result<()> {
    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            ptr::null(),
            wide(target).as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    ensure!((result as isize) > 32, "无法打开 {target}");
    Ok(())
}
#[derive(Clone, Copy, PartialEq)]
pub enum Link {
    Startup,
    StartMenu,
    ContextMenu,
}
const CONTEXT: &str = r"Directory\background\shell\RadeonTunerZhCN";
impl Link {
    fn path(self) -> Result<PathBuf> {
        let id = if matches!(self, Self::Startup) {
            FOLDERID_Startup
        } else {
            FOLDERID_StartMenu
        };
        let mut value = ptr::null_mut();
        let status = unsafe { SHGetKnownFolderPath(&id, 0, ptr::null_mut(), &mut value) };
        ensure!(status >= 0 && !value.is_null(), "无法获取 Windows 菜单目录");
        let path = unsafe {
            let mut len = 0;
            while *value.add(len) != 0 {
                len += 1;
            }
            let path = std::ffi::OsString::from_wide(std::slice::from_raw_parts(value, len));
            CoTaskMemFree(value.cast());
            PathBuf::from(path)
        };
        Ok(path.join("RadeonTuner 简体中文版.url"))
    }
    pub fn enabled(self) -> bool {
        if self == Self::ContextMenu {
            let mut key = ptr::null_mut();
            let result = unsafe {
                RegOpenKeyExW(
                    HKEY_CLASSES_ROOT,
                    wide(CONTEXT).as_ptr(),
                    0,
                    KEY_READ,
                    &mut key,
                )
            };
            if result == ERROR_SUCCESS {
                unsafe { RegCloseKey(key) };
                true
            } else {
                false
            }
        } else {
            self.path().is_ok_and(|p| p.is_file())
        }
    }
    pub fn set(self, on: bool) -> Result<()> {
        let exe = std::env::current_exe()?;
        if self == Self::ContextMenu {
            if on {
                let target = exe.to_string_lossy();
                for (sub, name, value) in [
                    (format!("{CONTEXT}\\command"), "", format!("\"{target}\"")),
                    (CONTEXT.into(), "Icon", target.to_string()),
                    (CONTEXT.into(), "", "打开 RadeonTuner".into()),
                ] {
                    let bytes = wide(&value);
                    let code = unsafe {
                        RegSetKeyValueW(
                            HKEY_CLASSES_ROOT,
                            wide(&sub).as_ptr(),
                            wide(name).as_ptr(),
                            REG_SZ,
                            bytes.as_ptr().cast(),
                            (bytes.len() * 2) as u32,
                        )
                    };
                    if code != ERROR_SUCCESS {
                        return Err(io::Error::from_raw_os_error(code as i32).into());
                    }
                }
            } else {
                let code = unsafe { RegDeleteTreeW(HKEY_CLASSES_ROOT, wide(CONTEXT).as_ptr()) };
                if code != ERROR_SUCCESS {
                    return Err(io::Error::from_raw_os_error(code as i32).into());
                }
            }
        } else {
            let path = self.path()?;
            if on {
                let mut text = vec![0xfeff];
                for (label, value) in [
                    ("[InternetShortcut]\r\nURL=", true),
                    ("\r\nIconFile=", true),
                    ("\r\nIconIndex=0\r\n", false),
                ] {
                    text.extend(label.encode_utf16());
                    if value {
                        text.extend(exe.as_os_str().encode_wide());
                    }
                }
                let bytes: Vec<_> = text.into_iter().flat_map(u16::to_le_bytes).collect();
                fs::write(path, bytes)?;
            } else {
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }
}

const DWM: &str = r"Software\Microsoft\Windows\Dwm";
const OVERLAY: &str = "OverlayTestMode";

pub fn hags_enabled() -> Option<bool> {
    let mut mode = 0u32;
    let mut bytes = size_of::<u32>() as u32;
    let code = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            wide(r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers").as_ptr(),
            wide("HwSchMode").as_ptr(),
            RRF_RT_REG_DWORD,
            ptr::null_mut(),
            (&mut mode as *mut u32).cast(),
            &mut bytes,
        )
    };
    (code == ERROR_SUCCESS).then_some(mode == 2)
}

pub fn mpo_disabled() -> bool {
    unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            wide(DWM).as_ptr(),
            wide(OVERLAY).as_ptr(),
            RRF_RT_REG_DWORD,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        ) == ERROR_SUCCESS
    }
}

pub fn set_mpo_disabled(disabled: bool) -> Result<()> {
    use windows_sys::Win32::Foundation::ERROR_FILE_NOT_FOUND;
    let path = wide(DWM);
    let value = wide(OVERLAY);
    let result = if disabled {
        let mode = 5u32;
        unsafe {
            RegSetKeyValueW(
                HKEY_LOCAL_MACHINE,
                path.as_ptr(),
                value.as_ptr(),
                REG_DWORD,
                (&mode as *const u32).cast(),
                size_of::<u32>() as u32,
            )
        }
    } else {
        unsafe { RegDeleteKeyValueW(HKEY_LOCAL_MACHINE, path.as_ptr(), value.as_ptr()) }
    };
    if result != ERROR_SUCCESS && (disabled || result != ERROR_FILE_NOT_FOUND) {
        return Err(io::Error::from_raw_os_error(result as i32).into());
    }
    Ok(())
}
