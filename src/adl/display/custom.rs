use super::{Allocation, Display, DisplayId, Driver, Handle};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::ptr;

pub const STANDARDS: [(&str, i32); 5] = [
    ("显示器", 8),
    ("CVT", 1),
    ("CVT-RB", 32),
    ("GTF", 2),
    ("DMT", 4),
];

#[repr(C)]
#[derive(Default, Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Axis {
    pub total: i32,
    pub active: i32,
    pub start: i32,
    pub sync: i32,
}
#[repr(C)]
#[derive(Default, Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Detail {
    size: i32,
    pub flags: i32,
    pub h: Axis,
    pub v: Axis,
    pub clock: i32,
    overscan: [i16; 6],
}
#[repr(C)]
#[derive(Default, Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Timing {
    standard: i32,
    possible: i32,
    pub hz: i32,
    pub width: i32,
    pub height: i32,
    pub detail: Detail,
}

#[repr(C)]
#[derive(Default)]
struct Request {
    width: i32,
    height: i32,
    scan: i32,
    hz: i32,
    standard: i32,
}
#[repr(C)]
#[derive(Default)]
struct Legacy {
    header: [i32; 5],
    size: i32,
    values: [i16; 10],
    overscan: [i16; 6],
}

impl Timing {
    pub fn key(&self) -> (i32, i32, i32) {
        (self.width, self.height, self.hz)
    }
    pub fn rate(&self) -> f64 {
        f64::from(self.detail.clock) * 10000.
            / (f64::from(self.detail.h.total) * f64::from(self.detail.v.total))
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.width > 0 && self.height > 0 && self.hz > 0 && self.detail.clock > 0,
            "分辨率、刷新率和像素时钟必须为正数"
        );
        for ax in [self.detail.h, self.detail.v] {
            ensure!(
                ax.active > 0
                    && ax.start >= ax.active
                    && ax.sync > 0
                    && i64::from(ax.start) + i64::from(ax.sync) <= i64::from(ax.total),
                "时序总长不足以容纳有效区域、前沿和同步宽度"
            );
        }
        Ok(())
    }
    pub fn label(&self) -> String {
        format!("{} × {} · {} Hz", self.width, self.height, self.hz)
    }
}
type List = unsafe extern "C" fn(Handle, i32, DisplayId, *mut i32, *mut *mut Timing) -> i32;
type Get = unsafe extern "C" fn(Handle, i32, DisplayId, *const Request, *mut Timing) -> i32;
type GetLegacy = unsafe extern "C" fn(Handle, i32, i32, *const [i32; 4], *mut Legacy) -> i32;
type Set = unsafe extern "C" fn(Handle, i32, i32, *mut Timing, i32) -> i32;
type Delete = unsafe extern "C" fn(Handle, i32, DisplayId, *const Request, i32) -> i32;

impl Driver {
    pub fn custom_timings(&self, d: &Display) -> Result<Vec<Timing>> {
        let mut count = 0;
        let mut data = Allocation(ptr::null_mut());
        let code = unsafe {
            self.library
                .get::<List>(b"ADL2_Display_ModeTimingOverrideListX3_Get\0")?(
                self.context,
                d.adapter,
                d.id(),
                &mut count,
                &mut data.0,
            )
        };
        data.copy_result(code, count)
    }
    pub fn calculate_timing(
        &self,
        d: &Display,
        width: i32,
        height: i32,
        hz: i32,
        standard: usize,
        interlaced: bool,
    ) -> Result<Timing> {
        let (_, standard) = *STANDARDS.get(standard).context("时序标准无效")?;
        ensure!(width > 0 && height > 0 && hz > 0, "请输入有效的显示模式");
        let mut out = Timing::default();
        if standard == 8 {
            let mut v = Legacy::default();
            let code = unsafe {
                self.library
                    .get::<GetLegacy>(b"ADL2_Display_ModeTimingOverride_Get\0")?(
                    self.context,
                    d.adapter,
                    d.logical,
                    &[height, width, 0, hz],
                    &mut v,
                )
            };
            ensure!(code == 0, "驱动没有这个显示模式的原始时序 ({code})");
            let [flags, ht, ha, hs, hw, vt, va, vs, vw, _] = v.values.map(i32::from);
            let clock = i64::from(ht) * i64::from(vt) * i64::from(hz) / 10000;
            out = Timing {
                width,
                height,
                hz,
                standard,
                possible: 0,
                detail: Detail {
                    size: size_of::<Detail>() as i32,
                    flags,
                    h: Axis {
                        total: ht,
                        active: ha,
                        start: hs,
                        sync: hw,
                    },
                    v: Axis {
                        total: vt,
                        active: va,
                        start: vs,
                        sync: vw,
                    },
                    clock: i32::try_from(clock)?,
                    overscan: v.overscan,
                },
            };
        } else {
            let req = Request {
                width,
                height,
                hz,
                standard,
                scan: if interlaced { 2 } else { 0 },
            };
            let code = unsafe {
                self.library
                    .get::<Get>(b"ADL2_Display_ModeTimingOverrideX3_Get\0")?(
                    self.context,
                    d.adapter,
                    d.id(),
                    &req,
                    &mut out,
                )
            };
            ensure!(code == 0, "驱动无法为该模式计算时序 ({code})");
        }
        out.detail.flags = (out.detail.flags & !3) | if interlaced { 2 } else { 0 };
        Ok(out)
    }
    pub fn create_timing(&self, d: &Display, mut t: Timing) -> Result<()> {
        t.validate()?;
        t.standard = 8;
        t.detail.flags ^= 12;
        self.write_timing(d, t)
    }
    pub fn write_timing(&self, d: &Display, mut t: Timing) -> Result<()> {
        let code = unsafe {
            self.library
                .get::<Set>(b"ADL2_Display_ModeTimingOverrideX2_Set\0")?(
                self.context,
                d.adapter,
                d.logical,
                &mut t,
                1,
            )
        };
        ensure!(code == 0, "驱动拒绝自定义时序 ({code})");
        Ok(())
    }
    pub fn delete_timing(&self, d: &Display, t: Timing) -> Result<()> {
        let req = Request {
            width: t.width,
            height: t.height,
            hz: t.hz,
            ..Default::default()
        };
        let code = unsafe {
            self.library
                .get::<Delete>(b"ADL2_Display_ModeTimingOverride_Delete\0")?(
                self.context,
                d.adapter,
                d.id(),
                &req,
                1,
            )
        };
        ensure!(code == 0, "无法删除自定义分辨率 ({code})");
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timing_layout_and_wide_clock_calculation() {
        assert_eq!(
            (
                size_of::<Timing>(),
                size_of::<Legacy>(),
                size_of::<Request>()
            ),
            (76, 56, 20)
        );
        let mut t = Timing {
            width: 1920,
            height: 1080,
            hz: 60,
            ..Default::default()
        };
        t.detail.h = Axis {
            total: 2200,
            active: 1920,
            start: 2008,
            sync: 44,
        };
        t.detail.v = Axis {
            total: 1125,
            active: 1080,
            start: 1084,
            sync: 5,
        };
        t.detail.clock = 14850;
        assert_eq!(t.rate(), 60.);
        assert!(t.validate().is_ok());
        t.detail.h.sync = i32::MAX;
        assert!(t.validate().is_err());
    }
}
