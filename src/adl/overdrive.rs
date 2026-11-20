use super::{Driver, Handle};
use anyhow::{Result, ensure};
use std::{ptr, slice};

const COUNT: usize = 77; // OD8_COUNT in the bundled AMD headers.
type Limits = unsafe extern "C" fn(Handle, i32, *mut i32, *mut i32, *mut *mut [i32; 4]) -> i32;
type Current = unsafe extern "C" fn(Handle, i32, *mut i32, *mut *mut i32) -> i32;
type Set = unsafe extern "C" fn(Handle, i32, *const Requests, *mut Values) -> i32;

#[repr(C)]
struct Requests {
    count: i32,
    entries: [[i32; 3]; COUNT],
}
#[repr(C)]
struct Values {
    count: i32,
    entries: [i32; COUNT],
}

pub struct Control {
    pub id: usize,
    pub label: &'static str,
    pub unit: &'static str,
    pub min: i32,
    pub max: i32,
    pub step: i32,
    pub current: i32,
    pub draft: i32,
}

const CONTROLS: &[(usize, &str, &str, i32)] = &[
    (1, "最低核心频率", "MHz", 1),
    (0, "最高核心频率", "MHz", 1),
    (8, "最高显存频率", "MHz", 2),
    (9, "功耗限制", "%", 1),
    (37, "电压偏移", "mV", 1),
    (47, "电流限制", "%", 1),
    (19, "曲线点 1 温度", "°C", 1),
    (20, "曲线点 1 转速", "%", 1),
    (21, "曲线点 2 温度", "°C", 1),
    (22, "曲线点 2 转速", "%", 1),
    (23, "曲线点 3 温度", "°C", 1),
    (24, "曲线点 3 转速", "%", 1),
    (25, "曲线点 4 温度", "°C", 1),
    (26, "曲线点 4 转速", "%", 1),
    (27, "曲线点 5 温度", "°C", 1),
    (28, "曲线点 5 转速", "%", 1),
];

struct Allocation<T>(*mut T);
impl<T> Drop for Allocation<T> {
    fn drop(&mut self) {
        unsafe { libc::free(self.0.cast()) }
    }
}

impl Driver {
    pub fn controls(&self, index: i32) -> Result<Vec<Control>> {
        unsafe {
            let limits = self
                .library
                .get::<Limits>(b"ADL2_Overdrive8_Init_SettingX2_Get\0")?;
            let current = self
                .library
                .get::<Current>(b"ADL2_Overdrive8_Current_SettingX2_Get\0")?;
            let (mut caps, mut length, mut values_length) = (0, 0, (COUNT - 1) as i32);
            let mut ranges = Allocation(ptr::null_mut());
            let mut values = Allocation(ptr::null_mut());
            let code = limits(self.context, index, &mut caps, &mut length, &mut ranges.0);
            ensure!(code == 0, "此显卡不支持 Overdrive 调节 ({code})");
            let code = current(self.context, index, &mut values_length, &mut values.0);
            ensure!(code == 0, "读取调节值失败 ({code})");
            let count = length.min(values_length).clamp(0, COUNT as i32) as usize;
            if count == 0 {
                return Ok(Vec::new());
            }
            ensure!(
                !ranges.0.is_null() && !values.0.is_null(),
                "驱动未返回调节范围"
            );
            let ranges = slice::from_raw_parts(ranges.0, count);
            let values = slice::from_raw_parts(values.0, count);
            Ok(CONTROLS
                .iter()
                .filter_map(|&(id, label, unit, step)| {
                    let [_, min, max, _] = *ranges.get(id)?;
                    let current = values[id];
                    if min >= max
                        || !(min..=max).contains(&current)
                        || ((19..=28).contains(&id) && caps & (1 << 13) == 0)
                    {
                        return None;
                    }
                    Some(Control {
                        id,
                        label,
                        unit,
                        min,
                        max,
                        step,
                        current,
                        draft: current,
                    })
                })
                .collect())
        }
    }

    pub fn apply(&self, index: i32, controls: &[Control]) -> Result<()> {
        let latest = self.controls(index)?;
        let mut request = Requests {
            count: (COUNT - 1) as i32,
            entries: [[0; 3]; COUNT],
        };
        let mut effective = std::collections::BTreeMap::new();
        for control in &latest {
            effective.insert(control.id, control.current);
        }
        for draft in controls.iter().filter(|c| c.current != c.draft) {
            let live = latest.iter().find(|c| c.id == draft.id);
            ensure!(
                live.is_some_and(
                    |c| c.current == draft.current && (c.min..=c.max).contains(&draft.draft)
                ),
                "驱动设置已变化，请重新读取"
            );
            request.entries[draft.id] = [draft.draft, 1, 0];
            effective.insert(draft.id, draft.draft);
        }
        if let (Some(min), Some(max)) = (effective.get(&1), effective.get(&0)) {
            ensure!(min <= max, "最低核心频率不能大于最高核心频率");
        }
        for id in 19..=26 {
            if let (Some(a), Some(b)) = (effective.get(&id), effective.get(&(id + 2))) {
                ensure!(
                    if id % 2 == 1 { a < b } else { a <= b },
                    "请检查风扇曲线点的顺序"
                );
            }
        }
        let mut result = Values {
            count: (COUNT - 1) as i32,
            entries: [0; COUNT],
        };
        let code = unsafe {
            self.library.get::<Set>(b"ADL2_Overdrive8_Setting_Set\0")?(
                self.context,
                index,
                &request,
                &mut result,
            )
        };
        ensure!(code == 0, "驱动拒绝了调节请求 ({code})");
        Ok(())
    }
}
