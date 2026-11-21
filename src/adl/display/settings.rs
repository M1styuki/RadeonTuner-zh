use super::{Display, DisplayId, Driver, Handle};
use anyhow::{Result, ensure};

type GetInt = unsafe extern "C" fn(Handle, i32, i32, *mut i32) -> i32;
type HdcpGet = unsafe extern "C" fn(Handle, i32, i32, *mut [i32; 5]) -> i32;
type AdapterQuad = unsafe extern "C" fn(Handle, i32, *mut i32, *mut i32, *mut i32, *mut i32) -> i32;
type AdapterSet = unsafe extern "C" fn(Handle, i32, i32) -> i32;
type SetInt = unsafe extern "C" fn(Handle, i32, i32, i32) -> i32;
type GetPair = unsafe extern "C" fn(Handle, i32, i32, *mut i32, *mut i32) -> i32;
type GetTriple = unsafe extern "C" fn(Handle, i32, i32, *mut i32, *mut i32, *mut i32) -> i32;
type GetQuad =
    unsafe extern "C" fn(Handle, i32, i32, *mut i32, *mut i32, *mut i32, *mut i32) -> i32;
type GetKind = unsafe extern "C" fn(Handle, i32, i32, i32, *mut i32) -> i32;
type SetKind = unsafe extern "C" fn(Handle, i32, i32, i32, i32) -> i32;
type GetColor = unsafe extern "C" fn(
    Handle,
    i32,
    i32,
    i32,
    *mut i32,
    *mut i32,
    *mut i32,
    *mut i32,
    *mut i32,
) -> i32;
type HdrGet = unsafe extern "C" fn(Handle, i32, DisplayId, *mut i32, *mut i32) -> i32;
type HdrSet = unsafe extern "C" fn(Handle, i32, DisplayId, i32) -> i32;
type Property = unsafe extern "C" fn(Handle, i32, i32, *mut [i32; 6]) -> i32;

#[derive(Clone, Copy, PartialEq)]
pub enum Feature {
    Pixel,
    Hdr,
    FreeSync,
    Depth,
    Enhancement,
    TemperatureSource,
    Color(i32),
    Correction(i32),
    Vsr,
    GpuScaling,
    IntegerScaling,
    Scaling,
    Hdcp,
    VariBright,
    VariBrightLevel,
}
use Feature::*;
const FEATURES: &[(Feature, &str)] = &[
    (Pixel, "像素格式"),
    (Hdr, "HDR"),
    (FreeSync, "FreeSync 模式"),
    (Depth, "色深"),
    (Enhancement, "色彩增强"),
    (TemperatureSource, "手动色温"),
    (Color(16), "色温（K）"),
    (Color(1), "亮度"),
    (Color(2), "对比度"),
    (Color(4), "饱和度"),
    (Color(8), "色调"),
    (Correction(0), "色觉校正"),
    (Correction(1), "红色觉校正"),
    (Correction(2), "绿色觉校正"),
    (Correction(3), "蓝色觉校正"),
    (Vsr, "虚拟超级分辨率"),
    (GpuScaling, "GPU 缩放"),
    (IntegerScaling, "整数缩放"),
    (Scaling, "缩放模式"),
    (Hdcp, "HDCP"),
    (VariBright, "Vari-Bright"),
    (VariBrightLevel, "Vari-Bright 级别"),
];

pub struct Setting {
    pub feature: Feature,
    pub label: &'static str,
    pub current: i32,
    pub draft: i32,
    pub default: i32,
    pub min: i32,
    pub max: i32,
    pub step: i32,
}

impl Feature {
    pub fn profile_name(self) -> &'static str {
        match self {
            Pixel => "PixelFormat",
            Hdr => "HdrEnabled",
            Depth => "ColorDepth",
            FreeSync => "FreeSyncMode",
            Enhancement => "ColorEnhancement",
            TemperatureSource => "ColorTemperatureControl",
            Color(1) => "Brightness",
            Color(2) => "Contrast",
            Color(4) => "Saturation",
            Color(8) => "Hue",
            Color(16) => "ColorTemperatureKelvin",
            Correction(0) => "CVDCControl",
            Correction(1) => "CVDCProtanopia",
            Correction(2) => "CVDCDeuteranopia",
            Correction(3) => "CVDCTritanopia",
            Vsr => "VsrEnabled",
            GpuScaling => "GpuScalingEnabled",
            IntegerScaling => "IntegerScalingEnabled",
            Scaling => "ScalingMode",
            Hdcp => "HDCPEnabled",
            VariBright => "VariBrightEnabled",
            VariBrightLevel => "VariBrightLevel",
            Color(_) | Correction(_) => unreachable!("unsupported display feature ID"),
        }
    }
    pub fn per_app(self) -> bool {
        matches!(
            self,
            FreeSync | Enhancement | TemperatureSource | Color(_) | Correction(_)
        )
    }
}
impl Setting {
    pub fn text(&self) -> String {
        let choices: &[&str] = match self.feature {
            Pixel => &[
                "RGB 全范围",
                "YCbCr 4:4:4",
                "YCbCr 4:2:2",
                "RGB 有限范围",
                "YCbCr 4:2:0",
            ],
            Depth => &["6 bpc", "8 bpc", "10 bpc", "12 bpc", "14 bpc", "16 bpc"],
            FreeSync => &["关闭", "可变", "固定"],
            Enhancement => &["关闭", "鲜艳游戏", "动态对比度"],
            Scaling => &["保持纵横比", "全屏", "居中"],
            Hdr | TemperatureSource | Vsr | GpuScaling | IntegerScaling | Correction(0) => {
                &["关", "开"]
            }
            _ => &[],
        };
        choices
            .get(self.draft as usize)
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.draft.to_string())
    }
}
impl Display {
    fn id(&self) -> DisplayId {
        DisplayId {
            logical: self.logical,
            physical: 0,
            logical_adapter: self.adapter,
            physical_adapter: 0,
        }
    }
}

impl Driver {
    pub fn display_settings(&self, d: &Display) -> Vec<Setting> {
        FEATURES
            .iter()
            .filter_map(|&(f, label)| self.read_display_setting(d, f, label))
            .collect()
    }
    fn read_display_setting(
        &self,
        d: &Display,
        f: Feature,
        label: &'static str,
    ) -> Option<Setting> {
        let (a, i) = (d.adapter, d.logical);
        let (mut current, mut default, mut min, mut max, mut step, mut support, mut aux) =
            (0, 0, 0, 1, 1, 0, 0);
        let ctx = self.context;
        macro_rules! read {($ty:ty,$name:literal,$($arg:expr),+)=>{unsafe {self.library.get::<$ty>(concat!($name,"\0").as_bytes()).ok()?(ctx,$($arg),+)}}}
        let status = match f {
            Hdcp => {
                let mut data = [0; 5];
                let code = read!(HdcpGet, "ADL2_Display_HDCP_Get", a, i, &mut data);
                current = data[2];
                default = 1;
                code
            }
            VariBright => {
                let code = read!(
                    AdapterQuad,
                    "ADL2_Adapter_VariBright_CapsX2",
                    a,
                    &mut support,
                    &mut current,
                    &mut aux,
                    &mut default
                );
                if support == 0 {
                    return None;
                }
                default = 1;
                code
            }
            VariBrightLevel => {
                let code = read!(
                    AdapterQuad,
                    "ADL2_Adapter_VariBrightLevel_Get",
                    a,
                    &mut default,
                    &mut max,
                    &mut step,
                    &mut current
                );
                max = max.saturating_sub(1);
                code
            }
            Pixel | Depth | TemperatureSource => {
                let (symbol, table, default_idx): (&[u8], &[i32], i32) = match f {
                    Pixel => (b"ADL2_Display_PixelFormat_Get\0", &[1, 2, 4, 8, 16], 2),
                    Depth => (b"ADL2_Display_ColorDepth_Get\0", &[1, 2, 3, 4, 5, 6], 1),
                    _ => (b"ADL2_Display_ColorTemperatureSource_Get\0", &[32, 64], 1),
                };
                let code =
                    unsafe { self.library.get::<GetInt>(symbol).ok()?(ctx, a, i, &mut current) };
                current = table.iter().position(|x| *x == current)? as i32;
                default = default_idx;
                max = table.len() as i32 - 1;
                let defaults = match f {
                    Pixel => Some(b"ADL2_Display_PixelFormatDefault_Get\0".as_slice()),
                    Depth => Some(b"ADL2_Display_ColorDepthDefault_Get\0".as_slice()),
                    _ => None,
                };
                if let Some(symbol) = defaults {
                    if let Ok(get) = unsafe { self.library.get::<GetInt>(symbol) } {
                        if unsafe { get(ctx, a, i, &mut aux) } == 0 {
                            if let Some(pos) = table.iter().position(|x| *x == aux) {
                                default = pos as i32;
                            }
                        }
                    }
                }
                code
            }
            Hdr => {
                let code = read!(
                    HdrGet,
                    "ADL2_Display_HDRState_Get",
                    a,
                    d.id(),
                    &mut support,
                    &mut current
                );
                if support == 0 {
                    return None;
                }
                code
            }
            FreeSync => {
                let code = read!(
                    GetQuad,
                    "ADL2_Display_FreeSyncState_Get",
                    a,
                    i,
                    &mut current,
                    &mut default,
                    &mut min,
                    &mut max
                );
                current = match current {
                    0 => 0,
                    1 => 2,
                    _ => 1,
                };
                default = 1;
                min = 0;
                max = 2;
                code
            }
            Enhancement => {
                let code = read!(
                    GetPair,
                    "ADL2_Display_SCE_State_Get",
                    a,
                    i,
                    &mut current,
                    &mut aux
                );
                current = [1, 2, 8].iter().position(|v| *v == current)? as i32;
                max = 2;
                code
            }
            Color(kind) => read!(
                GetColor,
                "ADL2_Display_Color_Get",
                a,
                i,
                kind,
                &mut current,
                &mut default,
                &mut min,
                &mut max,
                &mut step
            ),
            Correction(kind) => {
                max = if kind == 0 { 1 } else { 20 };
                read!(GetKind, "ADL2_Display_CVDC_Get", a, i, kind, &mut current)
            }
            Vsr | IntegerScaling => {
                let mut p = [24, if f == Vsr { 11 } else { 12 }, 0, 0, 0, 0];
                let code = read!(Property, "ADL2_Display_Property_Get", a, i, &mut p);
                if p[3] == 0 {
                    return None;
                }
                current = p[4];
                default = p[5];
                code
            }
            GpuScaling => {
                let code = read!(
                    GetTriple,
                    "ADL2_DFP_GPUScalingEnable_Get",
                    a,
                    i,
                    &mut support,
                    &mut current,
                    &mut default
                );
                if support == 0 {
                    return None;
                }
                code
            }
            Scaling => {
                if read!(
                    GetTriple,
                    "ADL2_Display_PreservedAspectRatio_Get",
                    a,
                    i,
                    &mut support,
                    &mut aux,
                    &mut default
                ) != 0
                {
                    return None;
                }
                let code = read!(
                    GetTriple,
                    "ADL2_Display_ImageExpansion_Get",
                    a,
                    i,
                    &mut support,
                    &mut current,
                    &mut default
                );
                current = if aux == 1 {
                    0
                } else if current == 1 {
                    1
                } else {
                    2
                };
                default = 0;
                max = 2;
                code
            }
        };
        if status != 0 || !(min..=max).contains(&current) {
            return None;
        }
        Some(Setting {
            feature: f,
            label,
            current,
            draft: current,
            default,
            min,
            max,
            step: step.max(1),
        })
    }

    pub fn apply_display(&self, d: &Display, values: &[Setting]) -> Result<()> {
        let (ctx, a, i) = (self.context, d.adapter, d.logical);
        macro_rules! set {($ty:ty,$name:literal,$($arg:expr),+)=>{unsafe{self.library.get::<$ty>(concat!($name,"\0").as_bytes())?(ctx,$($arg),+)}}}
        for v in values.iter().filter(|v| v.current != v.draft) {
            let f = v.feature;
            let n = v.draft;
            let live = self
                .read_display_setting(d, f, v.label)
                .ok_or_else(|| anyhow::anyhow!("{} 已不可用", v.label))?;
            ensure!(
                live.current == v.current && (live.min..=live.max).contains(&n),
                "{} 已改变，请刷新",
                v.label
            );
            let code = match f {
                Hdcp => set!(SetKind, "ADL2_Display_HDCP_Set", a, i, 0, n),
                VariBright => set!(AdapterSet, "ADL2_Adapter_VariBrightEnable_Set", a, n),
                VariBrightLevel => set!(SetInt, "ADL2_Adapter_VariBrightLevel_Set", a, n, 1),
                Pixel => set!(SetInt, "ADL2_Display_PixelFormat_Set", a, i, 1 << n),
                Hdr => set!(HdrSet, "ADL2_Display_HDRState_Set", a, d.id(), n),
                Depth => set!(SetInt, "ADL2_Display_ColorDepth_Set", a, i, n + 1),
                FreeSync => set!(
                    SetKind,
                    "ADL2_Display_FreeSyncState_Set",
                    a,
                    i,
                    [0, 7, 1][n as usize],
                    0
                ),
                Enhancement => set!(
                    SetInt,
                    "ADL2_Display_SCE_State_Set",
                    a,
                    i,
                    [1, 2, 8][n as usize]
                ),
                TemperatureSource => set!(
                    SetInt,
                    "ADL2_Display_ColorTemperatureSource_Set",
                    a,
                    i,
                    if n == 1 { 64 } else { 32 }
                ),
                Color(kind) => set!(SetKind, "ADL2_Display_Color_Set", a, i, kind, n),
                Correction(kind) => set!(SetKind, "ADL2_Display_CVDC_Set", a, i, kind, n),
                Vsr | IntegerScaling => {
                    let mut p = [24, if f == Vsr { 11 } else { 12 }, 0, 0, n, 0];
                    set!(Property, "ADL2_Display_Property_Set", a, i, &mut p)
                }
                GpuScaling => set!(SetInt, "ADL2_DFP_GPUScalingEnable_Set", a, i, n),
                Scaling if n == 0 => set!(SetInt, "ADL2_Display_PreservedAspectRatio_Set", a, i, 1),
                Scaling => set!(
                    SetInt,
                    "ADL2_Display_ImageExpansion_Set",
                    a,
                    i,
                    i32::from(n == 1)
                ),
            };
            ensure!(code == 0, "应用{}失败 ({code})", v.label);
            if matches!(f, Pixel | Hdr | Depth) {
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        }
        Ok(())
    }
}
