use super::{Driver, Handle, string};
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::{ptr, slice};

type Read<T> = unsafe extern "C" fn(Handle, i32, *mut *mut T, *mut i32) -> i32;
type Set = unsafe extern "C" fn(Handle, i32, *mut FeatureValue, i32, i32) -> i32;

// ADLFeatureCaps and ADLFeatureValues from the bundled AMD SDK.
#[repr(C)]
#[derive(Clone, Copy)]
struct FeatureCaps {
    name: [u8; 16],
    group: i32,
    visual: i32,
    page: i32,
    mask: i32,
    properties: i32,
    control_type: i32,
    control_style: i32,
    adjustment_type: i32,
    adjustment_style: i32,
    boolean_default: i32,
    integer_limits: [i32; 4],
    float_limits: [f32; 4],
    enum_mask: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FeatureValue {
    name: [u8; 16],
    boolean: i32,
    integer: i32,
    float: f32,
    states: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    Upscale,
    Sharpness,
    Brightness,
}

impl Feature {
    fn name(self) -> &'static str {
        match self {
            Self::Upscale => "VideoUpScale",
            Self::Sharpness => "Sharpness",
            Self::Brightness => "Brightness",
        }
    }

    pub fn profile_key(self) -> &'static str {
        match self {
            Self::Upscale => "VideoUpscaling.Current",
            Self::Sharpness => "VideoSharpening.Current",
            Self::Brightness => "VideoBrightness.Current",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Upscale => "视频超分辨率",
            Self::Sharpness => "视频锐化",
            Self::Brightness => "视频亮度",
        }
    }

    pub fn limits(self) -> (i32, i32, i32) {
        match self {
            Self::Upscale => (0, 1, 0),
            Self::Sharpness => (1, 100, 50),
            Self::Brightness => (-100, 100, 0),
        }
    }
}

pub struct VideoControl {
    pub feature: Feature,
    pub current: i32,
    pub saved: i32,
    pub draft: i32,
}

pub fn video_draft(controls: &mut [VideoControl], row: &Value) -> Result<()> {
    for control in controls {
        let value = &row[control.feature.profile_key()];
        if value.is_null() {
            continue;
        }
        let value = value.as_i64().or_else(|| value.as_bool().map(i64::from))
            .context("视频配置数值无效")?;
        let (min, max, _) = control.feature.limits();
        ensure!((i64::from(min)..=i64::from(max)).contains(&value), "{} 超出范围", control.feature.label());
        control.draft = value as i32;
    }
    Ok(())
}

impl Driver {
    unsafe fn read_video_table<T: Copy>(&self, index: i32, symbol: &[u8]) -> Result<Vec<T>> {
        let mut data = ptr::null_mut();
        let mut count = 0;
        let code = unsafe {
            self.library.get::<Read<T>>(symbol)?(self.context, index, &mut data, &mut count)
        };
        let allocation = super::Allocation(data);
        ensure!(code == 0, "无法读取视频设置 ({code})");
        if count == 0 {
            return Ok(Vec::new());
        }
        ensure!(count > 0 && !allocation.0.is_null(), "视频设置数据无效");
        Ok(unsafe { slice::from_raw_parts(allocation.0, count as usize) }.to_vec())
    }

    pub fn video_controls(&self, index: i32) -> Result<Vec<VideoControl>> {
        let caps =
            unsafe { self.read_video_table::<FeatureCaps>(index, b"ADL2_MMD_Features_Caps\0")? };
        let values = unsafe {
            self.read_video_table::<FeatureValue>(index, b"ADL2_MMD_FeatureValues_Get\0")?
        };
        Ok([Feature::Upscale, Feature::Sharpness, Feature::Brightness]
            .into_iter()
            .filter_map(|feature| {
                let value = values
                    .iter()
                    .find(|row| string(&row.name) == feature.name())?;
                if !caps
                    .iter()
                    .any(|row| row.name == value.name && row.properties & 1 != 0)
                {
                    return None;
                }
                let current = if feature == Feature::Upscale {
                    i32::from(value.boolean != 0)
                } else {
                    value.float as i32
                };
                Some(VideoControl {
                    feature,
                    current,
                    saved: current,
                    draft: current,
                })
            })
            .collect())
    }

    pub fn apply_video(&self, index: i32, controls: &[VideoControl]) -> Result<()> {
        let latest = self.video_controls(index)?;
        let mut values = unsafe {
            self.read_video_table::<FeatureValue>(index, b"ADL2_MMD_FeatureValues_Get\0")?
        };
        let mut changed = false;
        for draft in controls.iter().filter(|c| c.current != c.draft) {
            let (min, max, _) = draft.feature.limits();
            ensure!((min..=max).contains(&draft.draft), "视频参数超出范围");
            ensure!(
                latest
                    .iter()
                    .any(|v| v.feature == draft.feature && v.current == draft.current),
                "驱动设置已变化，请重新读取"
            );
            let value = values
                .iter_mut()
                .find(|row| string(&row.name) == draft.feature.name())
                .ok_or_else(|| anyhow::anyhow!("视频参数已失效"))?;
            if draft.feature == Feature::Upscale {
                value.boolean = draft.draft;
            } else {
                value.float = draft.draft as f32;
            }
            changed = true;
        }
        if !changed {
            return Ok(());
        }
        if let Some(preset) = values.iter_mut().find(|row| string(&row.name) == "Preset") {
            preset.states = 128;
        }
        let code = unsafe {
            self.library.get::<Set>(b"ADL2_MMD_FeatureValues_Set\0")?(
                self.context,
                index,
                values.as_mut_ptr(),
                values.len() as i32,
                0,
            )
        };
        ensure!(code == 0, "驱动拒绝了视频设置 ({code})");
        Ok(())
    }
}
