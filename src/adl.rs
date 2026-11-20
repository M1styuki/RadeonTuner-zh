use anyhow::{Context, Result, bail, ensure};
use libloading::os::windows::Library;
use std::{ffi::c_void, ptr, slice};

type Handle = *mut c_void;
type Allocate = unsafe extern "system" fn(i32) -> *mut c_void;
type Create = unsafe extern "C" fn(Allocate, i32, *mut Handle) -> i32;
type Destroy = unsafe extern "C" fn(Handle) -> i32;
type Enumerate = unsafe extern "C" fn(Handle, i32, *mut i32, *mut *mut AdapterInfo) -> i32;
type Query = unsafe extern "C" fn(Handle, i32, *mut LogData) -> i32;
type ReadInt = unsafe extern "C" fn(Handle, i32, *mut i32) -> i32;

#[repr(C)]
struct AdapterInfo {
    size: i32,
    index: i32,
    udid: [u8; 256],
    bus: i32,
    device: i32,
    function: i32,
    vendor: i32,
    name: [u8; 256],
    display: [u8; 256],
    present: i32,
    exists: i32,
    path: [u8; 256],
    path_ext: [u8; 256],
    pnp: [u8; 256],
    display_index: i32,
}

#[repr(C)]
struct LogData {
    size: i32,
    sensors: [[i32; 2]; 256],
}

pub struct Device {
    pub index: i32,
    pub name: String,
}
pub struct Metric {
    pub label: &'static str,
    pub value: Option<i32>,
    pub unit: &'static str,
}
pub struct Driver {
    context: Handle,
    destroy: Destroy,
    library: Library,
}

unsafe extern "system" fn allocate(bytes: i32) -> *mut c_void {
    if bytes <= 0 {
        return ptr::null_mut();
    }
    unsafe { libc::malloc(bytes as usize) }
}

fn string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

impl Driver {
    pub fn open() -> Result<Self> {
        unsafe {
            let library = Library::load_with_flags(
                "atiadlxx.dll",
                windows_sys::Win32::System::LibraryLoader::LOAD_LIBRARY_SEARCH_SYSTEM32,
            )
            .context("无法打开 AMD 显示驱动的 ADL 接口")?;
            let create = *library.get::<Create>(b"ADL2_Main_Control_Create\0")?;
            let destroy = *library.get::<Destroy>(b"ADL2_Main_Control_Destroy\0")?;
            let mut context = ptr::null_mut();
            let status = create(allocate, 1, &mut context);
            ensure!(
                status == 0 && !context.is_null(),
                "ADL 初始化失败 ({status})"
            );
            Ok(Self {
                context,
                destroy,
                library,
            })
        }
    }

    pub fn devices(&self) -> Result<Vec<Device>> {
        unsafe {
            let enumerate = self
                .library
                .get::<Enumerate>(b"ADL2_Adapter_AdapterInfoX3_Get\0")?;
            let access = self
                .library
                .get::<ReadInt>(b"ADL2_Adapter_Accessibility_Get\0")?;
            let mut count = 0;
            let mut data = ptr::null_mut();
            let status = enumerate(self.context, -1, &mut count, &mut data);
            struct Buffer(*mut AdapterInfo);
            impl Drop for Buffer {
                fn drop(&mut self) {
                    unsafe { libc::free(self.0.cast()) }
                }
            }
            let _owner = Buffer(data);
            ensure!(status == 0, "无法列出显卡 ({status})");
            if count == 0 {
                return Ok(Vec::new());
            }
            ensure!(!data.is_null() && count > 0, "驱动返回无效的适配器列表");
            let mut devices = Vec::new();
            let mut ids = std::collections::HashSet::new();
            for adapter in slice::from_raw_parts(data, count as usize) {
                let mut available = 0;
                if !matches!(adapter.vendor, 1002 | -1002)
                    || access(self.context, adapter.index, &mut available) != 0
                    || available <= 0
                {
                    continue;
                }
                let id = string(&adapter.pnp);
                if id.is_empty() || ids.insert(id) {
                    devices.push(Device {
                        index: adapter.index,
                        name: string(&adapter.name),
                    });
                }
            }
            Ok(devices)
        }
    }

    pub fn metrics(&self, index: i32) -> Result<Vec<Metric>> {
        let mut data = LogData {
            size: size_of::<LogData>() as i32,
            sensors: [[0; 2]; 256],
        };
        let status = unsafe {
            self.library
                .get::<Query>(b"ADL2_New_QueryPMLogData_Get\0")?(
                self.context, index, &mut data
            )
        };
        if status != 0 {
            bail!("无法读取监控数据 ({status})");
        }
        let sensor = |id: usize| {
            let [supported, value] = data.sensors[id];
            (supported != 0 && (0..65535).contains(&value)).then_some(value)
        };
        let mut memory = -1;
        let used = unsafe {
            self.library
                .get::<ReadInt>(b"ADL2_Adapter_DedicatedVRAMUsage_Get\0")
                .ok()
                .filter(|read| read(self.context, index, &mut memory) == 0)
                .and_then(|_| (memory >= 0).then_some(memory))
        };
        Ok([
            ("核心占用", sensor(19), "%"),
            ("显存占用率", sensor(20), "%"),
            ("已用显存", used, "MB"),
            ("核心频率", sensor(1), "MHz"),
            ("显存频率", sensor(2), "MHz"),
            ("功耗", sensor(73).or(sensor(30)), "W"),
            ("核心电压", sensor(21), "mV"),
            ("风扇转速", sensor(14), "RPM"),
            ("风扇转速比", sensor(15), "%"),
            ("核心温度", sensor(8).or(sensor(28)), "°C"),
            ("显存温度", sensor(9), "°C"),
            ("热点温度", sensor(27), "°C"),
            ("进气温度", sensor(74), "°C"),
        ]
        .into_iter()
        .map(|(label, value, unit)| Metric { label, value, unit })
        .collect())
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        unsafe {
            (self.destroy)(self.context);
        }
    }
}
