use super::{Allocation, Driver, Handle, string};
pub mod settings;
use anyhow::{Context, Result, ensure};
use std::{ptr, slice};

#[repr(C)]
#[derive(Clone, Copy)]
struct DisplayId {
    logical: i32,
    physical: i32,
    logical_adapter: i32,
    physical_adapter: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct DisplayInfo {
    id: DisplayId,
    controller: i32,
    name: [u8; 256],
    manufacturer: [u8; 256],
    display_type: i32,
    output: i32,
    connector: i32,
    mask: i32,
    flags: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Mode {
    adapter: i32,
    display: DisplayId,
    x: i32,
    y: i32,
    pub width: i32,
    pub height: i32,
    depth: i32,
    pub refresh: f32,
    pub orientation: i32,
    flag: i32,
    mask: i32,
    value: i32,
}
#[derive(Clone)]
pub struct Display {
    pub adapter: i32,
    pub logical: i32,
    pub name: String,
}
type Displays = unsafe extern "C" fn(Handle, i32, *mut i32, *mut *mut DisplayInfo, i32) -> i32;
type Get = unsafe extern "C" fn(Handle, i32, i32, *mut i32, *mut *mut Mode) -> i32;
type Set = unsafe extern "C" fn(Handle, i32, i32, i32, *const Mode) -> i32;
type Possible = unsafe extern "C" fn(Handle, i32, *mut i32, *mut *mut Mode) -> i32;

impl<T: Copy> Allocation<T> {
    fn copy_result(&self, code: i32, count: i32) -> Result<Vec<T>> {
        ensure!(code == 0, "显示驱动返回错误 ({code})");
        if count == 0 {
            return Ok(Vec::new());
        }
        ensure!(count > 0 && !self.0.is_null(), "显示驱动返回无效列表");
        Ok(unsafe { slice::from_raw_parts(self.0, count as usize) }.to_vec())
    }
}
impl Driver {
    pub fn displays(&self, adapter: i32) -> Result<Vec<Display>> {
        let mut count = 0;
        let mut data = Allocation(ptr::null_mut());
        let code = unsafe {
            self.library
                .get::<Displays>(b"ADL2_Display_DisplayInfo_Get\0")?(
                self.context,
                adapter,
                &mut count,
                &mut data.0,
                1,
            )
        };
        Ok(data
            .copy_result(code, count)?
            .into_iter()
            .filter(|d| d.flags & 3 == 3 && d.id.logical >= 0)
            .map(|d| Display {
                adapter,
                logical: d.id.logical,
                name: string(&d.name),
            })
            .filter(|d| self.current_mode(d).is_ok())
            .collect())
    }
    pub fn current_mode(&self, display: &Display) -> Result<Mode> {
        let mut count = 0;
        let mut data = Allocation(ptr::null_mut());
        let code = unsafe {
            self.library.get::<Get>(b"ADL2_Display_Modes_Get\0")?(
                self.context,
                display.adapter,
                display.logical,
                &mut count,
                &mut data.0,
            )
        };
        data.copy_result(code, count)?
            .into_iter()
            .find(|m| m.value > 0)
            .context("当前显示模式不可用")
    }
    pub fn possible_modes(&self, display: &Display) -> Result<Vec<Mode>> {
        let mut count = 0;
        let mut data = Allocation(ptr::null_mut());
        let code = unsafe {
            self.library
                .get::<Possible>(b"ADL2_Display_PossibleMode_Get\0")?(
                self.context,
                display.adapter,
                &mut count,
                &mut data.0,
            )
        };
        let mut modes = data.copy_result(code, count)?;
        modes.retain(|m| m.width > 0 && m.height > 0 && m.refresh.is_finite() && m.refresh > 0.0);
        modes.sort_by(|a, b| {
            b.width
                .cmp(&a.width)
                .then(b.height.cmp(&a.height))
                .then(b.refresh.total_cmp(&a.refresh))
        });
        modes.dedup_by(|a, b| a.width == b.width && a.height == b.height && a.refresh == b.refresh);
        Ok(modes)
    }
    pub fn set_mode(&self, display: &Display, mode: &Mode) -> Result<()> {
        let code = unsafe {
            self.library.get::<Set>(b"ADL2_Display_Modes_Set\0")?(
                self.context,
                display.adapter,
                display.logical,
                1,
                mode,
            )
        };
        ensure!(code == 0, "更改显示模式失败 ({code})");
        Ok(())
    }
}
