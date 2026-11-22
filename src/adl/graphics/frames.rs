use super::{Device, Driver, Handle, ReadInt};
use anyhow::{Context, Result, ensure};

#[derive(Clone, Copy, PartialEq)]
pub enum Field {
    Chill,
    Low,
    High,
    Cap,
    Target,
    Generate,
    Algorithm,
}
use Field::*;

impl Field {
    pub fn name(self) -> &'static str {
        match self {
            Chill => "ChillEnabled.Current",
            Low => "ChillMinFps.Current",
            High => "ChillMaxFps.Current",
            Cap => "FrtcEnabled.Current",
            Target => "FrtcFrameRateTarget.Current",
            Generate => "FrameGenEnabled.Current",
            Algorithm => "FrameGenAlgorithm.Current",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Chill => "Radeon Chill",
            Low => "Chill 最低帧率",
            High => "Chill 最高帧率",
            Cap => "帧率目标控制",
            Target => "帧率上限",
            Generate => "AMD 流体运动帧",
            Algorithm => "帧生成算法",
        }
    }
    pub fn entries(self) -> &'static [&'static str] {
        match self {
            Chill | Cap | Generate => &["关", "开"],
            Algorithm => &["自动", "增强", "标准"],
            _ => &[],
        }
    }
    pub fn enabled(self, settings: &[Control]) -> bool {
        let parent = match self {
            Low | High => Chill,
            Target => Cap,
            Algorithm => Generate,
            _ => return true,
        };
        settings.iter().any(|c| c.field == parent && c.draft != 0)
    }
}
#[derive(Clone)]
pub struct Control {
    pub field: Field,
    pub current: i32,
    pub draft: i32,
    pub min: i32,
    pub max: i32,
    pub step: i32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct ChillState {
    hotkey: i32,
    on: i32,
    low: i32,
    high: i32,
    min: i32,
    max: i32,
    step: i32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct FrameGeneration {
    value: i32,
    reserved: [i32; 3],
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct CapState {
    size: i32,
    ac_on: i32,
    dc_on: i32,
    ac: i32,
    dc: i32,
    ac_max: i32,
    ac_min: i32,
    dc_max: i32,
    dc_min: i32,
}
#[repr(C)]
#[derive(Default)]
struct CapInput {
    size: i32,
    global: i32,
    ac: i32,
    dc: i32,
    reserved: [i32; 6],
}
#[repr(C)]
struct ChillFlags {
    hotkey: i32,
    on: i32,
    low: i32,
    high: i32,
}

type Pair = unsafe extern "C" fn(Handle, i32, *mut i32, *mut i32) -> i32;
type Get<T> = unsafe extern "C" fn(Handle, i32, *mut T) -> i32;
type ChillSet = unsafe extern "C" fn(Handle, i32, ChillState, ChillFlags, *mut [i32; 4]) -> i32;
type CapSet = unsafe extern "C" fn(Handle, i32, CapInput) -> i32;
type FrameGet =
    unsafe extern "C" fn(Handle, i32, *mut FrameGeneration, *mut FrameGeneration) -> i32;
type FrameSet = unsafe extern "C" fn(Handle, i32, FrameGeneration, FrameGeneration) -> i32;

pub fn validate(values: &[Control]) -> Result<()> {
    let draft = |field| values.iter().find(|c| c.field == field).map(|c| c.draft);
    if let (Some(low), Some(high)) = (draft(Low), draft(High)) {
        ensure!(low <= high, "Chill 最低帧率不能高于最高帧率");
    }
    Ok(())
}

impl Driver {
    fn frame_caps(&self, index: i32, symbol: &[u8]) -> bool {
        let (mut supported, mut version) = (0, 0);
        unsafe {
            self.library.get::<Pair>(symbol).is_ok_and(|get| {
                get(self.context, index, &mut supported, &mut version) == 0 && supported > 0
            })
        }
    }
    fn get_frame_state<T>(&self, idx: i32, symbol: &[u8], mut value: T) -> Option<T> {
        (unsafe { self.library.get::<Get<T>>(symbol).ok()?(self.context, idx, &mut value) } == 0)
            .then_some(value)
    }
    fn generation(&self, idx: i32) -> Option<(FrameGeneration, FrameGeneration)> {
        let (mut on, mut algorithm) = (FrameGeneration::default(), FrameGeneration::default());
        (unsafe {
            self.library
                .get::<FrameGet>(b"ADL2_DriverFrameGeneration_SettingsX2_Get\0")
                .ok()?(self.context, idx, &mut on, &mut algorithm)
        } == 0)
            .then_some((on, algorithm))
    }

    pub fn frame_controls(&self, d: &Device, skip_caps: bool) -> Vec<Control> {
        let mut controls = Vec::new();
        let mut push = |field, current, min, max, step: i32| {
            if (min..=max).contains(&current) {
                controls.push(Control {
                    field,
                    current,
                    draft: current,
                    min,
                    max,
                    step: step.max(1),
                });
            }
        };
        let i = d.index;
        if skip_caps || self.frame_caps(i, b"ADL2_Chill_Caps_Get\0") {
            if let Some(v) =
                self.get_frame_state(i, b"ADL2_CHILL_SettingsX2_Get\0", ChillState::default())
            {
                push(Chill, v.on, 0, 1, 1);
                push(Low, v.low, v.min, v.max, v.step);
                push(High, v.high, v.min, v.max, v.step);
            }
        }
        if skip_caps || self.frame_caps(i, b"ADL2_FPS_Caps\0") {
            let blank = CapState {
                size: size_of::<CapState>() as i32,
                ..Default::default()
            };
            if let Some(v) = self.get_frame_state(i, b"ADL2_FPS_Settings_Get\0", blank) {
                push(Cap, v.ac_on, 0, 1, 1);
                if v.ac_min <= v.ac_max && v.ac_max > 0 {
                    let target = if v.ac_on == 0 && v.ac == 0 {
                        60i32.clamp(v.ac_min, v.ac_max)
                    } else {
                        v.ac
                    };
                    push(Target, target, v.ac_min, v.ac_max, 1);
                }
            }
        }
        let mut supported = 0;
        if skip_caps
            || unsafe {
                self.library
                    .get::<ReadInt>(b"ADL2_DriverFrameGenerationSupport_Get\0")
                    .is_ok_and(|get| get(self.context, i, &mut supported) == 0 && supported > 0)
            }
        {
            if let Some((enabled, mode)) = self.generation(i) {
                push(Generate, enabled.value, 0, 1, 1);
                push(Algorithm, mode.value, 0, 2, 1);
            }
        }
        controls
    }

    pub fn apply_frames(&self, d: &Device, values: &[Control]) -> Result<()> {
        let changed = |f| values.iter().any(|c| c.field == f && c.draft != c.current);
        if !values.iter().any(|c| c.current != c.draft) {
            return Ok(());
        }
        let latest = self.frame_controls(d, true);
        for c in values.iter().filter(|c| c.draft != c.current) {
            let live = latest
                .iter()
                .find(|item| item.field == c.field)
                .context("驱动功能已不可用")?;
            ensure!(
                live.current == c.current && (live.min..=live.max).contains(&c.draft),
                "{} 已变化，请刷新",
                c.field.label()
            );
        }
        let value = |field| values.iter().find(|c| c.field == field).map(|c| c.draft);
        let i = d.index;
        if changed(Chill) || changed(Low) || changed(High) {
            let mut state = self
                .get_frame_state(i, b"ADL2_CHILL_SettingsX2_Get\0", ChillState::default())
                .context("无法读取 Chill 设置")?;
            let flags = ChillFlags {
                hotkey: 0,
                on: changed(Chill) as i32,
                low: changed(Low) as i32,
                high: changed(High) as i32,
            };
            if changed(Chill) {
                state.on = value(Chill).unwrap();
            }
            if changed(Low) {
                state.low = value(Low).unwrap();
            }
            if changed(High) {
                state.high = value(High).unwrap();
            }
            ensure!(state.low <= state.high, "Chill 最低帧率不能高于最高帧率");
            let mut reason = [0; 4];
            let code = unsafe {
                self.library
                    .get::<ChillSet>(b"ADL2_CHILL_SettingsX2_Set\0")?(
                    self.context,
                    i,
                    state,
                    flags,
                    &mut reason,
                )
            };
            ensure!(code == 0, "应用 Chill 设置失败 ({code})");
            type Notify = unsafe extern "C" fn(Handle, i32, i32, i32) -> i32;
            let code = unsafe {
                self.library.get::<Notify>(b"ADL2_User_Settings_Notify\0")?(self.context, i, 2, 1)
            };
            ensure!(code == 0, "Chill 已保存，通知驱动失败 ({code})");
        }
        if changed(Cap) || changed(Target) {
            let on = latest
                .iter()
                .find(|c| c.field == Cap)
                .context("无法读取帧率上限开关")?
                .current;
            let enabled = if changed(Cap) {
                value(Cap).unwrap()
            } else {
                on
            };
            if changed(Cap) || enabled != 0 {
                let rate = if enabled == 0 {
                    0
                } else {
                    value(Target).context("无法读取目标帧率")?
                };
                let input = CapInput {
                    size: size_of::<CapInput>() as i32,
                    global: 1,
                    ac: rate,
                    dc: rate,
                    ..Default::default()
                };
                let code = unsafe {
                    self.library.get::<CapSet>(b"ADL2_FPS_Settings_Set\0")?(self.context, i, input)
                };
                ensure!(code == 0, "应用帧率上限失败 ({code})");
            }
        }
        if changed(Generate) || changed(Algorithm) {
            let (mut on, mut algo) = self.generation(i).context("无法读取流体运动帧设置")?;
            if changed(Generate) {
                on.value = value(Generate).unwrap();
            }
            if changed(Algorithm) {
                algo.value = value(Algorithm).unwrap();
            }
            let code = unsafe {
                self.library
                    .get::<FrameSet>(b"ADL2_DriverFrameGeneration_SettingsX2_Set\0")?(
                    self.context,
                    i,
                    on,
                    algo,
                )
            };
            ensure!(code == 0, "应用流体运动帧设置失败 ({code})");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chill_limits_validate_and_follow_parent_setting() {
        let control = |field, draft| Control {
            field,
            current: draft,
            draft,
            min: 0,
            max: 300,
            step: 1,
        };
        let mut values = [control(Chill, 0), control(Low, 60), control(High, 144)];
        assert!(!Low.enabled(&values));
        values[0].draft = 1;
        assert!(High.enabled(&values));
        assert!(validate(&values).is_ok());
        values[1].draft = 150;
        assert!(validate(&values).is_err());
    }

    #[test]
    fn frame_structs_match_bundled_adl_headers() {
        assert_eq!(size_of::<FrameGeneration>(), 16);
        assert_eq!(size_of::<ChillState>(), 28);
        assert_eq!(size_of::<ChillFlags>(), 16);
        assert_eq!(size_of::<CapState>(), 36);
        assert_eq!(size_of::<CapInput>(), 40);
        assert_eq!(std::mem::offset_of!(CapState, ac_min), 24);
    }
}
