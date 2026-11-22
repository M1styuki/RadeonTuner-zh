use super::{Allocation, DisplayId, Driver, Handle, Mode};
use anyhow::{Context, Result, ensure};
use std::{ffi::c_void, ptr};

#[repr(C)]
#[derive(Clone, Copy)]
struct DisplayTarget {
    id: DisplayId,
    map_index: i32,
    mask: i32,
    value: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct DisplayMap {
    index: i32,
    mode: Mode,
    target_count: i32,
    first_target: i32,
    mask: i32,
    value: i32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Grid {
    adapter: i32,
    index: i32,
    rows: i32,
    columns: i32,
    mask: i32,
    value: i32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct SlsMap {
    adapter: i32,
    index: i32,
    grid: Grid,
    surface: i32,
    orientation: i32,
    targets: i32,
    first_target: i32,
    native_modes: i32,
    first_native: i32,
    bezel_modes: i32,
    first_bezel: i32,
    bezel_offsets: i32,
    first_offset: i32,
    mask: i32,
    value: i32,
}
#[repr(C)]
struct SlsTarget {
    adapter: i32,
    map_index: i32,
    target: DisplayTarget,
    x: i32,
    y: i32,
    view: Mode,
    mask: i32,
    value: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct SlsMode {
    adapter: i32,
    map_index: i32,
    index: i32,
    mode: Mode,
    mask: i32,
    value: i32,
}
type Maps = unsafe extern "C" fn(
    Handle,
    i32,
    *mut i32,
    *mut *mut DisplayMap,
    *mut i32,
    *mut *mut DisplayTarget,
    i32,
) -> i32;
type Index = unsafe extern "C" fn(Handle, i32, i32, *const DisplayTarget, *mut i32) -> i32;
type Config = unsafe extern "C" fn(
    Handle,
    i32,
    i32,
    *mut SlsMap,
    *mut i32,
    *mut *mut c_void,
    *mut i32,
    *mut *mut SlsMode,
    *mut i32,
    *mut *mut c_void,
    *mut i32,
    *mut *mut c_void,
    *mut i32,
    *mut *mut c_void,
    i32,
) -> i32;
type Create =
    unsafe extern "C" fn(Handle, i32, SlsMap, i32, *const SlsTarget, i32, *mut i32, i32) -> i32;
type State = unsafe extern "C" fn(Handle, i32, i32, i32) -> i32;
type List = unsafe extern "C" fn(Handle, i32, *mut i32, *mut *mut i32, i32) -> i32;
type Delete = unsafe extern "C" fn(Handle, i32, i32) -> i32;

fn layout(
    adapter: i32,
    columns: i32,
    rows: i32,
    angle: i32,
    displays: &[i32],
) -> Result<(SlsMap, Vec<SlsTarget>)> {
    ensure!(
        (1..=6).contains(&columns) && (1..=6).contains(&rows) && columns + rows <= 8,
        "行列数超出范围"
    );
    ensure!(
        columns * rows >= 2 && (columns * rows) as usize == displays.len(),
        "网格大小必须与所选显示器数量一致"
    );
    ensure!(matches!(angle, 0 | 90 | 180 | 270), "显示方向无效");
    let map = SlsMap {
        adapter,
        index: -1,
        grid: Grid {
            rows,
            columns,
            ..Default::default()
        },
        orientation: angle,
        native_modes: columns * rows,
        bezel_modes: columns * rows,
        value: 0x200,
        ..Default::default()
    };
    let mut targets = Vec::with_capacity(displays.len());
    for (i, &logical) in displays.iter().enumerate() {
        ensure!(
            logical >= 0 && !displays[..i].contains(&logical),
            "显示器重复或无效"
        );
        let mut target: SlsTarget = unsafe { std::mem::zeroed() };
        target.adapter = adapter;
        target.map_index = -1;
        target.target.id.logical = logical;
        target.target.id.logical_adapter = adapter;
        target.x = i as i32 / rows;
        target.y = i as i32 % rows;
        target.view.adapter = adapter;
        targets.push(target);
    }
    Ok((map, targets))
}
impl Driver {
    pub fn create_eyefinity(
        &self,
        adapter: i32,
        columns: i32,
        rows: i32,
        angle: i32,
        displays: &[i32],
    ) -> Result<()> {
        let (map, targets) = layout(adapter, columns, rows, angle, displays)?;
        let connected = self.display_connections(adapter)?;
        ensure!(
            displays
                .iter()
                .all(|id| connected.iter().any(|d| d.logical == *id)),
            "显示器连接已变化"
        );
        let mut index = -1;
        let rc = unsafe {
            self.library
                .get::<Create>(b"ADL2_Display_SLSMapConfig_Create\0")?(
                self.context,
                adapter,
                map,
                targets.len() as i32,
                targets.as_ptr(),
                0,
                &mut index,
                2,
            )
        };
        ensure!(rc == 0 && index >= 0, "创建 Eyefinity 失败 ({rc})");
        Ok(())
    }
    pub fn eyefinity_state(&self, adapter: i32) -> Result<(i32, bool)> {
        let (mut mc, mut tc) = (0, 0);
        let (mut maps, mut targets) = (Allocation(ptr::null_mut()), Allocation(ptr::null_mut()));
        let rc = unsafe {
            self.library
                .get::<Maps>(b"ADL2_Display_DisplayMapConfig_Get\0")?(
                self.context,
                adapter,
                &mut mc,
                &mut maps.0,
                &mut tc,
                &mut targets.0,
                1,
            )
        };
        let maps = maps.copy_result(rc, mc)?;
        let current = maps.first().context("未找到显示映射")?.mode;
        ensure!(tc > 0 && !targets.0.is_null(), "未找到显示器");
        let mut index = -1;
        let rc = unsafe {
            self.library
                .get::<Index>(b"ADL2_Display_SLSMapIndex_Get\0")?(
                self.context,
                adapter,
                tc,
                targets.0,
                &mut index,
            )
        };
        ensure!(rc == 0 && index >= 0, "未找到此显示器组合的 Eyefinity 配置");
        let mut map = SlsMap::default();
        let mut modes = Allocation(ptr::null_mut());
        let mut native_count = 0;
        let [mut t, mut b, mut tr, mut bo] = std::array::from_fn(|_| Allocation(ptr::null_mut()));
        let [mut nt, mut nb, mut ntr, mut nbo] = [0; 4];
        let rc = unsafe {
            self.library
                .get::<Config>(b"ADL2_Display_SLSMapConfig_Get\0")?(
                self.context,
                adapter,
                index,
                &mut map,
                &mut nt,
                &mut t.0,
                &mut native_count,
                &mut modes.0,
                &mut nb,
                &mut b.0,
                &mut ntr,
                &mut tr.0,
                &mut nbo,
                &mut bo.0,
                2,
            )
        };
        let active = modes
            .copy_result(rc, native_count)?
            .iter()
            .any(|m| m.mode.width == current.width && m.mode.height == current.height);
        Ok((index, active))
    }
    pub fn set_eyefinity(&self, adapter: i32, enable: bool) -> Result<()> {
        let (index, active) = self.eyefinity_state(adapter)?;
        if active != enable {
            let rc = unsafe {
                self.library
                    .get::<State>(b"ADL2_Display_SLSMapConfig_SetState\0")?(
                    self.context,
                    adapter,
                    index,
                    i32::from(enable),
                )
            };
            ensure!(rc == 0, "切换 Eyefinity 失败 ({rc})");
        }
        Ok(())
    }
    fn active_maps(&self, adapter: i32) -> Result<Vec<i32>> {
        let mut data = Allocation(ptr::null_mut());
        let mut count = 0;
        let rc = unsafe {
            self.library
                .get::<List>(b"ADL2_Display_SLSMapIndexList_Get\0")?(
                self.context,
                adapter,
                &mut count,
                &mut data.0,
                1,
            )
        };
        data.copy_result(rc, count)
    }
    pub fn remove_eyefinity(&self, adapter: i32) -> Result<()> {
        let indices = self.active_maps(adapter)?;
        ensure!(!indices.is_empty(), "没有可移除的 Eyefinity 配置");
        let delete = unsafe {
            self.library
                .get::<Delete>(b"ADL2_Display_SLSMapConfig_Delete\0")?
        };
        let mut failed = Vec::new();
        for i in &indices {
            let rc = unsafe { delete(self.context, adapter, *i) };
            if rc != 0 {
                failed.push((*i, rc));
            }
        }
        let remaining = self.active_maps(adapter)?;
        ensure!(
            failed.is_empty() && !indices.iter().any(|i| remaining.contains(i)),
            "部分 Eyefinity 配置仍未移除：{failed:?}"
        );
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grid_order_matches_native_column_major_coordinates() {
        assert_eq!(
            (
                size_of::<Mode>(),
                size_of::<DisplayTarget>(),
                size_of::<SlsTarget>(),
                size_of::<SlsMap>()
            ),
            (60, 28, 112, 80)
        );
        let (m, t) = layout(3, 2, 2, 90, &[5, 2, 9, 6]).unwrap();
        assert_eq!((m.grid.rows, m.grid.columns, m.orientation), (2, 2, 90));
        assert_eq!(
            t.iter()
                .map(|t| (t.target.id.logical, t.x, t.y))
                .collect::<Vec<_>>(),
            [(5, 0, 0), (2, 0, 1), (9, 1, 0), (6, 1, 1)]
        );
        assert!(layout(0, 2, 2, 0, &[0, 1, 2]).is_err());
        assert!(layout(0, 2, 1, 0, &[0, 0]).is_err());
    }
}
