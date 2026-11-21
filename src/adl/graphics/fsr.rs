use super::{Device, Driver, Handle, ReadInt, registry_data, registry_text};
use anyhow::{Result, ensure};
use std::{
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
    ptr,
};
use windows_sys::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VS_FIXEDFILEINFO, VerQueryValueW,
};

fn release(version: &str) -> Option<&'static str> {
    Some(match version {
        "1.0.0.38222" | "1.0.0.38733" | "1.0.0.39037" => "4.0.0 预览版",
        "1.0.0.39849" => "4.0.0",
        "1.0.0.40394" => "4.0.1",
        "1.0.2.44374" => "4.0.2",
        "2.1.0.551" => "4.0.3",
        "2.1.0.968" | "2.2.0.1328" | "2.2.0.0" => "4.1.0",
        "2.3.0.2740" | "2.3.0.2913" | "2.3.0.3193" | "2.3.0.0" => "4.1.1",
        "2.4.0.4370" | "2.4.0.4541" | "2.4.0.0" => "4.1.2",
        _ => return None,
    })
}
fn file_version(path: &Path) -> Option<String> {
    let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        let size = GetFileVersionInfoSizeW(path.as_ptr(), ptr::null_mut());
        if size == 0 {
            return None;
        }
        let mut buffer = vec![0u32; (size as usize).div_ceil(4)];
        if GetFileVersionInfoW(path.as_ptr(), 0, size, buffer.as_mut_ptr().cast()) == 0 {
            return None;
        }
        let mut info = ptr::null_mut();
        let mut length = 0;
        if VerQueryValueW(
            buffer.as_ptr().cast(),
            [92u16, 0].as_ptr(),
            &mut info,
            &mut length,
        ) == 0
            || info.is_null()
            || (length as usize) < size_of::<VS_FIXEDFILEINFO>()
        {
            return None;
        }
        let v = info.cast::<VS_FIXEDFILEINFO>().read_unaligned();
        Some(format!(
            "{}.{}.{}.{}",
            v.dwFileVersionMS >> 16,
            v.dwFileVersionMS & 65535,
            v.dwFileVersionLS >> 16,
            v.dwFileVersionLS & 65535
        ))
    }
}
pub struct Library {
    pub current: String,
    pub draft: String,
    pub description: String,
    default: PathBuf,
}
impl Library {
    pub fn select(&mut self, path: String) {
        self.draft = path;
        let default = self.draft.trim().is_empty();
        let path = if default {
            &self.default
        } else {
            Path::new(&self.draft)
        };
        let version = file_version(path)
            .map(|v| match release(&v) {
                Some(fsr) => format!("{v} · FSR {fsr}"),
                None => v,
            })
            .unwrap_or_else(|| "未能读取版本".into());
        self.description = format!(
            "{} · {version}",
            if default {
                "驱动默认库"
            } else {
                "自定义库"
            }
        );
    }
    pub fn path(&self) -> &Path {
        if self.draft.trim().is_empty() {
            &self.default
        } else {
            Path::new(&self.draft)
        }
    }
}
impl Driver {
    fn fsr_override(&self, device: &Device) -> String {
        registry_data(&device.registry_path, "FsrOvrDLLPath")
            .and_then(|(kind, data)| registry_text(kind, &data))
            .unwrap_or_default()
            .trim()
            .to_owned()
    }
    pub fn fsr_library(&self, device: &Device, skip: bool) -> Option<Library> {
        if !skip {
            let mut support = 0;
            if unsafe {
                self.library
                    .get::<ReadInt>(b"ADL2_DriverMLSRSupport_Get\0")
                    .ok()?(self.context, device.index, &mut support)
            } != 0
                || support == 0
            {
                return None;
            }
        }
        let mut default = PathBuf::new();
        type PathGet = unsafe extern "C" fn(Handle, i32, i32, *mut u8) -> i32;
        let mut buffer = [0u8; 260];
        if let Ok(get) = unsafe { self.library.get::<PathGet>(b"ADL2_Driver_Path_Get\0") } {
            if unsafe { get(self.context, device.index, 260, buffer.as_mut_ptr()) } == 0 {
                let dir = crate::adl::string(&buffer);
                if !dir.is_empty() {
                    default = PathBuf::from(dir).join("amdxcffx64.dll");
                }
            }
        }
        let current = self.fsr_override(device);
        let mut library = Library {
            current: current.clone(),
            draft: String::new(),
            description: String::new(),
            default,
        };
        library.select(current);
        Some(library)
    }
    pub fn apply_fsr_override(&self, d: &Device, library: &Library) -> Result<()> {
        if library.draft != library.current {
            ensure!(
                self.fsr_override(d) == library.current,
                "FSR 库设置已改变，请刷新"
            );
            self.set_umd_string(
                d,
                "FsrOvrDLLPath",
                if library.draft.trim().is_empty() {
                    " "
                } else {
                    &library.draft
                },
            )?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognizes_new_driver_build() {
        assert_eq!(release("2.4.0.4541"), Some("4.1.2"));
        assert_eq!(release("2.4.0.9999"), None);
    }
}
