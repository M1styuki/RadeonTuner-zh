use super::{Device, Driver, Handle, string};

#[repr(C)]
struct Memory {
    bytes: i64,
    kind: [u8; 256],
    bandwidth: i64,
    hyper: i64,
    invisible: i64,
    visible: i64,
    vendor: i64,
    bandwidth2: i64,
    bitrate: i64,
}

impl Driver {
    pub fn information(&self, device: &Device) -> Vec<String> {
        let mut lines = vec![device.name.clone(), device.profile_key.clone()];
        macro_rules! read {
            ($ty:ty,$symbol:literal) => {{
                let mut value: $ty = unsafe { std::mem::zeroed() };
                unsafe {
                    self.library
                        .get::<unsafe extern "C" fn(Handle, i32, *mut $ty) -> i32>(
                            concat!($symbol, "\0").as_bytes(),
                        )
                        .ok()
                        .and_then(|get| {
                            (get(self.context, device.index, &mut value) == 0).then_some(value)
                        })
                }
            }};
        }
        if let Some(v) = read!([[u8; 256]; 4], "ADL2_Graphics_VersionsX3_Get") {
            lines.push(format!(
                "驱动程序版本：{} / {}",
                string(&v[2]),
                string(&v[0])
            ));
        }
        if let Some(v) = read!([[u8; 256]; 3], "ADL2_Adapter_VideoBiosInfo_Get") {
            lines.push(format!(
                "BIOS 版本：{} / {} / {}",
                string(&v[0]),
                string(&v[1]),
                string(&v[2])
            ));
        }
        if let Some(v) = read!(Memory, "ADL2_Adapter_MemoryInfoX4_Get") {
            lines.push(format!(
                "显存：{} MB {}",
                v.bytes / 1024 / 1024,
                string(&v.kind)
            ));
            if v.bandwidth > 0 {
                lines.push(format!("显存带宽：{} MB/s", v.bandwidth));
            }
        }
        if let Some(v) = read!([i32; 5], "ADL2_GcnAsicInfo_Get") {
            for (label, val) in [
                ("计算单元", v[0]),
                ("纹理映射单元", v[1]),
                ("渲染输出单元", v[2]),
            ] {
                if val > 0 {
                    lines.push(format!("{label}：{val}"));
                }
            }
        }
        if let Some(v) = read!([i32; 2], "ADL2_SmartAccessMemory_Mode_Get") {
            lines.push(format!(
                "可变基址寄存器（SAM）：{}",
                if v[0] > 1 { "已启用" } else { "已禁用" }
            ));
        }
        if let Ok(displays) = self.displays(device.index) {
            for display in displays {
                if let Ok(mode) = self.current_mode(&display) {
                    lines.push(format!(
                        "{} · {} × {} · {:.2} Hz",
                        display.name, mode.width, mode.height, mode.refresh
                    ));
                }
            }
        }
        lines
    }
}
