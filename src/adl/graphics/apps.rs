use crate::adl::{Allocation, Driver, Handle};
use anyhow::{Result, ensure};
use std::{ffi::c_void, ptr, slice};
use windows_sys::Win32::System::Com::CoCreateGuid;

pub mod settings;
const AREA: &str = "3D_User";
pub const GPU: &str = "0x0001";
type W = *const u16;
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
fn optional(value: &[u16]) -> W {
    if value.len() == 1 {
        ptr::null()
    } else {
        value.as_ptr()
    }
}
unsafe extern "C" {
    fn _msize(block: *mut c_void) -> usize;
    fn wcslen(value: W) -> usize;
}
fn string(p: W) -> String {
    if p.is_null() {
        return String::new();
    }
    unsafe { String::from_utf16_lossy(slice::from_raw_parts(p, wcslen(p))) }
}
#[repr(C)]
struct Record {
    title: *mut u16,
    path: *mut u16,
    file: *mut u16,
    version: *mut u16,
    notes: *mut u16,
    area: *mut u16,
    profile: *mut u16,
    source: i32,
}
#[repr(C)]
struct CreateRecord {
    name: W,
    kind: i32,
    value: W,
}
type List = unsafe extern "C" fn(Handle, W, *mut i32, *mut *mut Record) -> i32;
type Search = unsafe extern "C" fn(Handle, W, W, W, W, *mut *mut u8) -> i32;
type Kind = unsafe extern "C" fn(Handle, W, W, *mut i32) -> i32;
type Create = unsafe extern "C" fn(Handle, W, W, i32, *const CreateRecord) -> i32;
type Assign = unsafe extern "C" fn(Handle, W, W, W, W, W, W) -> i32;
type Remove = unsafe extern "C" fn(Handle, W, W, W, W) -> i32;
type RemoveProfile = unsafe extern "C" fn(Handle, W, W) -> i32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct App {
    pub file: String,
    pub path: String,
    pub profile: String,
    pub title: String,
    pub version: String,
}
impl App {
    pub fn new(file: String) -> Self {
        Self {
            file,
            path: r"*\*".into(),
            profile: String::new(),
            title: String::new(),
            version: String::new(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property {
    pub name: String,
    pub kind: i32,
    pub value: String,
}

impl Property {
    pub fn gpu_value(&self) -> Option<&str> {
        if self.kind != 5 || !self.value.contains("::") {
            return Some(&self.value);
        }
        self.value
            .split(";;")
            .filter_map(|v| v.split_once("::"))
            .find(|(id, _)| id.eq_ignore_ascii_case(GPU))
            .map(|(_, v)| v)
    }
    pub fn set_gpu_value(&mut self, new: &str) {
        if self.kind != 5 {
            self.value = new.into();
            return;
        }
        let mut parts = Vec::new();
        for part in self.value.split(";;").filter(|v| !v.is_empty()) {
            if let Some((id, _)) = part.split_once("::") {
                if !id.eq_ignore_ascii_case(GPU) {
                    parts.push(format!("{part};;"));
                }
            }
        }
        self.value = format!("{GPU}::{new};;{}", parts.concat());
    }
}
fn data_value(kind: i32, data: &[u8]) -> Result<String> {
    match kind {
        5 => {
            ensure!(data.len() % 2 == 0, "驱动字符串长度无效");
            let v = data
                .chunks_exact(2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .take_while(|v| *v != 0)
                .collect::<Vec<_>>();
            Ok(String::from_utf16(&v)?)
        }
        1..=4 => {
            ensure!(data.len() <= 8, "驱动数值宽度无效");
            let mut v = [0; 8];
            v[..data.len()].copy_from_slice(data);
            Ok(u64::from_le_bytes(v).to_string())
        }
        0 => Ok(data.iter().map(|b| format!("{b:02X}")).collect()),
        _ => anyhow::bail!("驱动属性类型无效 ({kind})"),
    }
}
impl Driver {
    pub fn game_apps(&self) -> Result<Vec<App>> {
        let mut count = 0;
        let mut data = Allocation(ptr::null_mut());
        let rc = unsafe {
            self.library
                .get::<List>(b"ADL2_ApplicationProfiles_Applications_Get\0")?(
                self.context,
                wide(AREA).as_ptr(),
                &mut count,
                &mut data.0,
            )
        };
        ensure!(rc == 0, "无法读取 AMD 应用配置 ({rc})");
        if count == 0 {
            return Ok(Vec::new());
        }
        ensure!(count > 0 && !data.0.is_null(), "应用配置列表无效");
        let mut result = Vec::new();
        for r in unsafe { slice::from_raw_parts(data.0, count as usize) } {
            if !r.file.is_null() && string(r.area).eq_ignore_ascii_case(AREA) {
                result.push(App {
                    file: string(r.file),
                    path: string(r.path),
                    profile: string(r.profile),
                    title: string(r.title),
                    version: string(r.version),
                });
            }
            for p in [
                r.file, r.path, r.profile, r.area, r.version, r.notes, r.title,
            ] {
                unsafe { libc::free(p.cast()) }
            }
        }
        result.sort_by_cached_key(|app| (app.file.to_lowercase(), app.path.to_lowercase()));
        Ok(result)
    }
    pub fn app_kind(&self, name: &str) -> Option<i32> {
        let mut kind = 0;
        let rc = unsafe {
            self.library
                .get::<Kind>(b"ADL2_ApplicationProfiles_PropertyType_Get\0")
                .ok()?(
                self.context,
                wide(AREA).as_ptr(),
                wide(name).as_ptr(),
                &mut kind,
            )
        };
        if rc != 0 {
            return None;
        }
        [None, Some(0), Some(3), Some(2), Some(1), Some(4), Some(5)]
            .get(kind as usize)
            .copied()
            .flatten()
    }
    pub fn app_properties(&self, app: &App) -> Result<Vec<Property>> {
        let mut data = Allocation(ptr::null_mut());
        let rc = unsafe {
            self.library
                .get::<Search>(b"ADL2_ApplicationProfiles_ProfileOfAnApplicationX2_Search\0")?(
                self.context,
                wide(&app.file).as_ptr(),
                wide(&app.path).as_ptr(),
                optional(&wide(&app.version)),
                wide(AREA).as_ptr(),
                &mut data.0,
            )
        };
        ensure!(rc == 0 && !data.0.is_null(), "读取游戏驱动属性失败 ({rc})");
        // ADL returns a single callback allocation. Records are packed,
        // so the next header may begin at an unaligned address.
        let len = unsafe { _msize(data.0.cast()) };
        ensure!(len >= 4 && len != usize::MAX, "驱动属性缓冲区无效");
        let count = unsafe { ptr::read_unaligned(data.0.cast::<i32>()) };
        ensure!(
            count >= 0 && count as usize <= (len - 4) / 72,
            "属性数量无效"
        );
        let mut offset = 4;
        let mut properties = Vec::new();
        for _ in 0..count {
            ensure!(len - offset >= 72, "驱动属性标题不完整");
            let base = unsafe { data.0.add(offset) };
            let name = crate::adl::string(unsafe { slice::from_raw_parts(base, 64) });
            let recorded = unsafe { ptr::read_unaligned(base.add(64).cast::<i32>()) };
            let kind = self.app_kind(&name).unwrap_or(recorded);
            let size = unsafe { ptr::read_unaligned(base.add(68).cast::<i32>()) };
            ensure!(
                size >= 0 && size as usize <= len - offset - 72,
                "驱动属性长度无效"
            );
            let raw = unsafe { slice::from_raw_parts(base.add(72), size as usize) };
            properties.push(Property {
                name,
                kind,
                value: data_value(kind, raw)?,
            });
            offset += 72 + size as usize;
        }
        Ok(properties)
    }
    pub fn write_game_app(&self, app: &App, properties: &[Property]) -> Result<()> {
        let mut guid = unsafe { std::mem::zeroed() };
        ensure!(
            unsafe { CoCreateGuid(&mut guid) } == 0,
            "生成驱动配置标识失败"
        );
        let name = format!(
            "RADT{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
            guid.data1,
            guid.data2,
            guid.data3,
            guid.data4[0],
            guid.data4[1],
            guid.data4[2],
            guid.data4[3],
            guid.data4[4],
            guid.data4[5],
            guid.data4[6],
            guid.data4[7]
        );
        let name = wide(&name);
        let area = wide(AREA);
        let strings: Vec<_> = properties
            .iter()
            .map(|p| (wide(&p.name), wide(&p.value)))
            .collect();
        let records: Vec<_> = properties
            .iter()
            .zip(&strings)
            .map(|(p, (n, v))| CreateRecord {
                name: n.as_ptr(),
                kind: p.kind,
                value: v.as_ptr(),
            })
            .collect();
        unsafe {
            let create = self
                .library
                .get::<Create>(b"ADL2_ApplicationProfiles_Profile_Create\0")?;
            let assign = self
                .library
                .get::<Assign>(b"ADL2_ApplicationProfiles_ProfileApplicationX2_Assign\0")?;
            let remove = self
                .library
                .get::<RemoveProfile>(b"ADL2_ApplicationProfiles_Profile_Remove\0")?;
            let rc = create(
                self.context,
                area.as_ptr(),
                name.as_ptr(),
                records.len().try_into()?,
                records.as_ptr(),
            );
            ensure!(rc == 0, "创建游戏配置失败 ({rc})");
            let rc = assign(
                self.context,
                wide(&app.file).as_ptr(),
                wide(&app.path).as_ptr(),
                optional(&wide(&app.version)),
                optional(&wide(&app.title)),
                area.as_ptr(),
                name.as_ptr(),
            );
            if rc != 0 {
                remove(self.context, area.as_ptr(), name.as_ptr());
                anyhow::bail!("应用游戏配置失败 ({rc})");
            }
            if app.profile.starts_with("RADT{")
                && self.game_apps().is_ok_and(|apps| {
                    apps.iter()
                        .all(|a| !a.profile.eq_ignore_ascii_case(&app.profile))
                })
            {
                remove(self.context, area.as_ptr(), wide(&app.profile).as_ptr());
            }
        }
        Ok(())
    }
    pub fn delete_game_app(&self, app: &App) -> Result<()> {
        let rc = unsafe {
            self.library
                .get::<Remove>(b"ADL2_ApplicationProfiles_RemoveApplication\0")?(
                self.context,
                wide(&app.file).as_ptr(),
                wide(&app.path).as_ptr(),
                optional(&wide(&app.version)),
                wide(AREA).as_ptr(),
            )
        };
        ensure!(rc == 0, "删除游戏驱动配置失败 ({rc})");
        Ok(())
    }
}
