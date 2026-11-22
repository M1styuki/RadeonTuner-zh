use super::{Driver, Handle};
use crate::adl::Device;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::collections::HashSet;

const NAMES: [&str; 4] = [
    "GammaBase.Current",
    "GammaRed.Current",
    "GammaGreen.Current",
    "GammaBlue.Current",
];
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Gamma {
    pub enabled: bool,
    pub values: [i32; 4],
}
impl Default for Gamma {
    fn default() -> Self {
        Self {
            enabled: false,
            values: [100; 4],
        }
    }
}
impl Gamma {
    pub fn read(row: &Value) -> Result<Self> {
        let mut g = Self::default();
        let v = &row["GammaControl.Current"];
        g.enabled = v
            .as_bool()
            .unwrap_or_else(|| v.as_i64().is_some_and(|n| n != 0));
        for (value, key) in g.values.iter_mut().zip(NAMES) {
            if let Some(v) = row.get(key).filter(|v| !v.is_null()) {
                let n = v.as_f64().context("伽马值格式无效")?;
                ensure!((0.5..=3.0).contains(&n), "伽马超出范围 (0.50–3.00)");
                *value = (n * 100.0).round() as i32;
            }
        }
        Ok(g)
    }
    pub fn record(self, row: &mut Value) {
        row["GammaControl.Current"] = json!(i32::from(self.enabled));
        for (v, key) in self.values.into_iter().zip(NAMES) {
            row[key] = json!(v as f64 / 100.0);
        }
    }
}
#[repr(C)]
#[derive(PartialEq, Eq)]
struct Ramp {
    rgb: [u32; 1024],
}
impl Ramp {
    fn build(values: [i32; 4], delta: f32) -> Self {
        let [base, r, g, b] = values.map(|x| x as f32 / 100.);
        let mut rgb = [0u32; 1024];
        for i in 0..255 {
            let l = (i as f32 / 255.).powf(1. / (base - delta)) * 65535.;
            let channel = |gain: f32| (l * gain).clamp(0., 65535.) as u32;
            rgb[3 + 2 * i] = (channel(g) << 16) | channel(r);
            rgb[4 + 2 * i] = channel(b);
        }
        Self { rgb }
    }
}
type Get = unsafe extern "C" fn(Handle, i32, *mut Ramp) -> i32;
type Set = unsafe extern "C" fn(Handle, i32, Ramp) -> i32;
impl Driver {
    fn gamma_ramp(&self, adapter: i32) -> Result<Ramp> {
        let mut ramp = Ramp { rgb: [0; 1024] };
        let rc = unsafe {
            self.library.get::<Get>(b"ADL2_Adapter_Gamma_Get\0")?(self.context, adapter, &mut ramp)
        };
        ensure!(rc == 0, "读取伽马曲线失败 ({rc})");
        Ok(ramp)
    }
    pub fn supports_gamma(&self, adapter: i32) -> bool {
        (unsafe { self.library.get::<Set>(b"ADL2_Adapter_Gamma_Set\0").is_ok() })
            && self.gamma_ramp(adapter).is_ok()
    }
    pub fn sync_gamma(
        &self,
        device: &Device,
        profile: Gamma,
        owned: &mut HashSet<String>,
    ) -> Result<()> {
        if !profile.enabled && !owned.contains(&device.gamma_key) {
            return Ok(());
        }
        ensure!(!device.gamma_key.is_empty(), "无法识别伽马目标显卡");
        let values = if profile.enabled {
            profile.values
        } else {
            [100; 4]
        };
        let desired = Ramp::build(values, 0.0);
        if self.gamma_ramp(device.index)? == desired {
            if !profile.enabled {
                owned.remove(&device.gamma_key);
            }
            return Ok(());
        }
        let set = unsafe { self.library.get::<Set>(b"ADL2_Adapter_Gamma_Set\0")? };
        // A short intermediate ramp forces drivers that cache LUT updates to reload.
        let intermediate = unsafe { set(self.context, device.index, Ramp::build(values, 0.01)) };
        let status = unsafe { set(self.context, device.index, desired) };
        if profile.enabled && (status == 0 || intermediate == 0) {
            owned.insert(device.gamma_key.clone());
        }
        ensure!(status == 0, "设置伽马曲线失败 ({status})");
        if !profile.enabled {
            owned.remove(&device.gamma_key);
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_gamma_encoding_and_profile_precision() {
        let mut row = json!({"GammaControl.Current":1,"GammaRed.Current":0.5,"GammaGreen.Current":3.0,"GammaBase.Current":1.25});
        let settings = Gamma::read(&row).unwrap();
        settings.record(&mut row);
        assert_eq!(Gamma::read(&row).unwrap().values, [125, 50, 300, 100]);
        let ramp = Ramp::build([100, 50, 300, 100], 0.0);
        let i = 128;
        let l = (i as f32 / 255.) * 65535.;
        assert_eq!(ramp.rgb[3 + 2 * i], 0xffff0000 | (l * 0.5) as u32);
        assert_eq!(ramp.rgb[4 + 2 * i], l as u32);
        assert_eq!(ramp.rgb[513..], [0; 511]);
        assert!(Gamma::read(&json!({"GammaBase.Current":0.0})).is_err());
    }
}
