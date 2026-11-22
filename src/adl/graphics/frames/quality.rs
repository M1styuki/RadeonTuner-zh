use super::{Control, Driver, Field, Handle};
use Field::*;
use anyhow::{Context, Result, ensure};

#[repr(C)]
#[derive(Default)]
struct DeLagState {
    hotkey: i32,
    on: i32,
    fps: i32,
    min: i32,
    max: i32,
    step: i32,
    reserved: [i32; 2],
    indicator_hotkey: i32,
}
#[repr(C)]
#[derive(Default)]
struct BoostState {
    hotkey: i32,
    input: i32,
    resolution: i32,
    min: i32,
    max: i32,
    step: i32,
    vsr: i32,
    reserved: i32,
    adaptive: i32,
}
#[repr(C)]
#[derive(Default)]
struct Ris {
    on: i32,
    strength: i32,
    min: i32,
    max: i32,
    step: i32,
}
#[repr(C)]
#[derive(Default)]
struct Ris2 {
    base: Ris,
    desktop: i32,
}

#[repr(C)]
struct DeLagFlags {
    hotkey: i32,
    on: i32,
    fps: i32,
    other: [i32; 3],
}
#[repr(C)]
struct BoostFlags {
    hotkey: i32,
    on: i32,
    resolution: i32,
    reserved: i32,
    adaptive: i32,
}
#[repr(C)]
struct RisFlags {
    on: u32,
    strength: u32,
}
#[repr(C)]
struct Ris2Flags {
    base: RisFlags,
    desktop: u32,
}
type SetChecked<T, F> = unsafe extern "C" fn(Handle, i32, T, F, *mut [i32; 5]) -> i32;
type Set<T, F> = unsafe extern "C" fn(Handle, i32, T, F) -> i32;

enum Sharpening {
    One(Ris),
    Two(Ris2),
}
impl Driver {
    pub(super) fn notify_graphics(&self, i: i32, flag: i32) -> Result<()> {
        type Notify = unsafe extern "C" fn(Handle, i32, i32, i32) -> i32;
        let code = unsafe {
            self.library.get::<Notify>(b"ADL2_User_Settings_Notify\0")?(self.context, i, flag, 1)
        };
        ensure!(code == 0, "设置已保存，通知驱动失败 ({code})");
        Ok(())
    }
    fn sharpening(&self, i: i32) -> Option<Sharpening> {
        self.get_frame_state(i, b"ADL2_RIS_SettingsX2_Get\0", Ris2::default())
            .map(Sharpening::Two)
            .or_else(|| {
                self.get_frame_state(i, b"ADL2_RIS_Settings_Get\0", Ris::default())
                    .map(Sharpening::One)
            })
    }
    pub(super) fn read_quality(&self, i: i32, push: &mut impl FnMut(Field, i32, i32, i32, i32)) {
        if let Some(s) =
            self.get_frame_state(i, b"ADL2_DELAG_SettingsX4_Get\0", DeLagState::default())
        {
            push(AntiLag, s.on, 0, 1, 1);
        }
        if let Some(s) =
            self.get_frame_state(i, b"ADL2_BOOST_SettingsX4_Get\0", BoostState::default())
        {
            push(
                Boost,
                i32::from(s.input != 0) | (i32::from(s.adaptive != 0) << 1),
                0,
                3,
                1,
            );
            push(MinResolution, s.resolution, s.min, s.max, s.step);
        }
        if let Some(sharpen) = self.sharpening(i) {
            let base = match sharpen {
                Sharpening::One(ref base) => base,
                Sharpening::Two(ref s) => {
                    push(Desktop, s.desktop, 0, 1, 1);
                    &s.base
                }
            };
            push(Sharpen, base.on, 0, 1, 1);
            push(Sharpness, base.strength, base.min, base.max, base.step);
        }
    }
    pub(super) fn apply_quality(&self, i: i32, controls: &[Control]) -> Result<()> {
        let change = |field| {
            controls
                .iter()
                .find(|c| c.field == field && c.current != c.draft)
                .map(|c| c.draft)
        };
        if let Some(on) = change(AntiLag) {
            let mut s = self
                .get_frame_state(i, b"ADL2_DELAG_SettingsX4_Get\0", DeLagState::default())
                .context("无法读取延迟降低设置")?;
            s.on = on;
            let reason = DeLagFlags {
                hotkey: 0,
                on: 1,
                fps: 0,
                other: [0; 3],
            };
            let code = unsafe {
                self.library
                    .get::<SetChecked<DeLagState, DeLagFlags>>(b"ADL2_DELAG_SettingsX4_Set\0")?(
                    self.context,
                    i,
                    s,
                    reason,
                    &mut [0; 5],
                )
            };
            ensure!(code == 0, "应用延迟降低设置失败 ({code})");
            self.notify_graphics(i, 4)?;
        }
        let (mode, res) = (change(Boost), change(MinResolution));
        if mode.is_some() || res.is_some() {
            let mut s = self
                .get_frame_state(i, b"ADL2_BOOST_SettingsX4_Get\0", BoostState::default())
                .context("无法读取 Boost 设置")?;
            let flags = BoostFlags {
                hotkey: 0,
                on: mode.is_some() as i32,
                resolution: res.is_some() as i32,
                reserved: 0,
                adaptive: mode.is_some() as i32,
            };
            if let Some(mode) = mode {
                s.input = mode & 1;
                s.adaptive = (mode >> 1) & 1;
            }
            if let Some(res) = res {
                s.resolution = res;
            }
            let code = unsafe {
                self.library
                    .get::<SetChecked<BoostState, BoostFlags>>(b"ADL2_BOOST_SettingsX4_Set\0")?(
                    self.context,
                    i,
                    s,
                    flags,
                    &mut [0; 5],
                )
            };
            ensure!(code == 0, "应用 Boost 设置失败 ({code})");
            self.notify_graphics(i, 8)?;
        }
        let (on, strength, desktop) = (change(Sharpen), change(Sharpness), change(Desktop));
        if on.is_some() || strength.is_some() || desktop.is_some() {
            let mut state = self.sharpening(i).context("无法读取图像锐化设置")?;
            let base = match &mut state {
                Sharpening::One(s) => s,
                Sharpening::Two(s) => &mut s.base,
            };
            if let Some(v) = on {
                base.on = v;
            }
            if let Some(v) = strength {
                base.strength = v;
            }
            let flags = RisFlags {
                on: on.is_some() as u32,
                strength: strength.is_some() as u32,
            };
            let (code, notify) = match state {
                Sharpening::One(s) => {
                    ensure!(desktop.is_none(), "此驱动不支持桌面锐化");
                    (
                        unsafe {
                            self.library
                                .get::<Set<Ris, RisFlags>>(b"ADL2_RIS_Settings_Set\0")?(
                                self.context,
                                i,
                                s,
                                flags,
                            )
                        },
                        16,
                    )
                }
                Sharpening::Two(mut s) => {
                    if let Some(v) = desktop {
                        s.desktop = v;
                    }
                    let flags = Ris2Flags {
                        base: flags,
                        desktop: desktop.is_some() as u32,
                    };
                    (
                        unsafe {
                            self.library
                                .get::<Set<Ris2, Ris2Flags>>(b"ADL2_RIS_SettingsX2_Set\0")?(
                                self.context,
                                i,
                                s,
                                flags,
                            )
                        },
                        512,
                    )
                }
            };
            ensure!(code == 0, "应用图像锐化设置失败 ({code})");
            self.notify_graphics(i, notify)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_and_change_flags_match_driver_headers() {
        assert_eq!((size_of::<DeLagState>(), size_of::<DeLagFlags>()), (36, 24));
        assert_eq!((size_of::<BoostState>(), size_of::<BoostFlags>()), (36, 20));
        assert_eq!((size_of::<Ris>(), size_of::<RisFlags>()), (20, 8));
        assert_eq!((size_of::<Ris2>(), size_of::<Ris2Flags>()), (24, 12));
        assert_eq!(std::mem::offset_of!(BoostState, adaptive), 32);
    }
}
