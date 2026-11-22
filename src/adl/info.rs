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

fn vendor(id: i64) -> &'static str {
    match id {
        1 => "Samsung",
        2 => "Infineon",
        3 => "Elpida",
        4 => "Etron",
        5 => "Nanya",
        6 => "Hynix",
        7 => "Mosel",
        8 => "Winbond",
        9 => "Esmt",
        15 => "Micron",
        _ => "未知",
    }
}
fn connector(id: i32) -> &'static str {
    match id {
        1 => "VGA",
        2 => "DVI-D",
        3 => "DVI-I",
        4 => "NTSC",
        5 => "JPN",
        6 => "非 I2C JPN",
        7 => "非 I2C NTSC",
        8 => "专有",
        10 => "HDMI A 型",
        11 => "HDMI B 型",
        12 => "S-Video",
        13 => "复合视频",
        14 => "RCA",
        15 => "DisplayPort",
        16 => "eDP",
        17 => "无线显示",
        18 => "USB Type-C",
        _ => "未知",
    }
}

impl Driver {
    pub fn information(&self, device: &Device) -> Vec<String> {
        let mut lines = vec![
            format!("设备名称：{}", device.name),
            format!("设备标识符：{}", device.profile_key),
        ];
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
                "显存：{} MB {}{}",
                v.bytes / 1024 / 1024,
                string(&v.kind),
                if v.vendor > 0 {
                    format!("（{}）", vendor(v.vendor))
                } else {
                    String::new()
                }
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
        if let Some(bus) = read!([i32; 6], "ADL2_Adapter_ChipSetInfo_Get") {
            let kind = match bus[0] {
                0 => "PCI".into(),
                1 => "AGP".into(),
                2..=11 => format!("PCIe {}.0", bus[0] - 1),
                _ => "未知".into(),
            };
            if bus[3] > 0 {
                lines.push(format!(
                    "总线类型：{kind}（当前 ×{} / 最大 ×{}）",
                    bus[3], bus[2]
                ));
            } else {
                lines.push(format!("总线类型：{kind}"));
            }
        }
        if let Some(on) = crate::integration::hags_enabled() {
            lines.push(format!(
                "GPU 硬件加速调度（HAGS）：{}",
                if on { "已启用" } else { "已禁用" }
            ));
        }
        if let Some(v) = read!([i32; 2], "ADL2_SmartAccessMemory_Mode_Get") {
            lines.push(format!(
                "可变基址寄存器（SAM）：{}",
                if v[0] > 1 { "已启用" } else { "已禁用" }
            ));
        }
        if let Ok(displays) = self.displays(device.index) {
            if displays.is_empty() {
                lines.push("所选 GPU 上没有已连接的显示器".into());
            }
            for display in displays {
                if let Ok(mode) = self.current_mode(&display) {
                    lines.push(format!(
                        "{}（{}） · {} × {} · {:.2} Hz",
                        display.name,
                        connector(display.connector),
                        mode.width,
                        mode.height,
                        mode.refresh
                    ));
                }
            }
        }
        lines
    }
}
