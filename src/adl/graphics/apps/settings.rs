use super::{Driver, Property};
use crate::adl::graphics::{
    CHOICES, Choice, Setting,
    frames::{Control, Field},
    fsr::Library,
};
use anyhow::{Result, ensure};
fn choice_name(c: &Choice) -> &str {
    match c.profile_key {
        "EnhancedSync.Current" => "TurboSync",
        "AntiAliasingLevel.Current" => "AntiAliasSmpls",
        "SurfaceFormatOptimization.Current" => "SrfcFrmtRplcmnt",
        "TessellationMode.Current" => "Tessellation_OP",
        "OpenGLTripleBuffering.Current" => "EnableTrplBffr",
        _ => c.key,
    }
}
fn default_choice(c: &Choice) -> usize {
    match c.profile_key {
        "TextureFilteringQuality.Current" | "VerticalSync.Current" => 1,
        "TessellationLevel.Current" => 7,
        _ => 0,
    }
}
fn value<'a>(props: &'a [Property], name: &str) -> Option<&'a str> {
    props.iter().find(|p| p.name == name)?.gpu_value()
}
const FIELDS: &[(Field, &str, i32, i32, i32, i32)] = &[
    (Field::AntiLag, "Dlg_PFEnable", 0, 1, 1, 0),
    (Field::Generate, "UFG_PFEnabled", 0, 1, 1, 0),
    (Field::Algorithm, "UFG_Algo", 0, 2, 1, 0),
    (Field::Boost, "Bst_PFEnable", 0, 3, 1, 0),
    (Field::MinResolution, "Bst_MaxScale", 50, 84, 1, 84),
    (Field::Chill, "Chil_PFEnable", 0, 1, 1, 0),
    (Field::Low, "Chil_MinFRate", 30, 300, 1, 75),
    (Field::High, "Chil_MaxFRate", 30, 300, 1, 140),
    (Field::Sharpen, "Ris_PFEnable", 0, 1, 1, 0),
    (Field::Sharpness, "Ris_SHDegree", 10, 100, 10, 80),
    (Field::AntiAliasing, "ASD", 0, 2, 1, 0),
];
pub fn settings(
    props: &[Property],
    supports: impl Fn(&str) -> bool,
) -> (Vec<Setting>, Vec<Control>) {
    let choices = CHOICES
        .iter()
        .filter_map(|choice| {
            let name = choice_name(choice);
            if !supports(name) {
                return None;
            }
            let current = value(props, name)
                .and_then(|s| s.parse::<i32>().ok())
                .and_then(|v| choice.entries.iter().position(|(_, n)| *n == v))
                .unwrap_or_else(|| default_choice(choice));
            Some(Setting {
                choice,
                current,
                draft: current,
            })
        })
        .collect();
    let number = |name| value(props, name).and_then(|s| s.parse::<i32>().ok());
    let fields = FIELDS
        .iter()
        .filter_map(|&(field, name, min, mut max, step, default)| {
            if !supports(name) {
                return None;
            }
            let current = match field {
                Field::Boost => {
                    if !supports("Bst_AdaPFEnable") {
                        max = 1;
                    }
                    (number(name).unwrap_or(0) != 0) as i32
                        + 2 * i32::from(number("Bst_AdaPFEnable").unwrap_or(0) != 0)
                }
                Field::AntiAliasing => {
                    if number("ASE") == Some(1) {
                        2
                    } else if number("ASTT") == Some(1) {
                        1
                    } else {
                        0
                    }
                }
                Field::Sharpness => value(props, name)
                    .and_then(|s| s.parse::<f32>().ok())
                    .map(|v| (v * 100.).round() as i32)
                    .unwrap_or(default),
                _ => number(name).unwrap_or(default),
            }
            .clamp(min, max);
            Some(Control {
                field,
                current,
                draft: current,
                min,
                max,
                step,
            })
        })
        .collect();
    (choices, fields)
}
pub fn defaults(choices: &mut [Setting], controls: &mut [Control]) {
    for c in choices {
        c.draft = default_choice(c.choice);
    }
    for c in controls {
        if let Some(f) = FIELDS.iter().find(|f| f.0 == c.field) {
            c.draft = f.5;
        }
    }
}
impl Driver {
    pub fn game_properties_from_controls(
        &self,
        mut props: Vec<Property>,
        choices: &[Setting],
        fields: &[Control],
        library: Option<&Library>,
        all: bool,
    ) -> Result<Vec<Property>> {
        let mut put = |name: &str, new: String| -> Result<()> {
            ensure!(
                !new.contains(";;") && !new.contains('\0'),
                "游戏配置参数包含无效分隔符"
            );
            if let Some(p) = props.iter_mut().find(|p| p.name == name) {
                p.set_gpu_value(&new);
            } else if let Some(kind) = self.app_kind(name) {
                let mut p = Property {
                    name: name.into(),
                    kind,
                    value: String::new(),
                };
                p.set_gpu_value(&new);
                props.push(p);
            }
            Ok(())
        };
        for c in choices.iter().filter(|c| all || c.current != c.draft) {
            put(
                choice_name(c.choice),
                c.choice.entries[c.draft].1.to_string(),
            )?;
        }
        for c in fields.iter().filter(|c| all || c.current != c.draft) {
            if let Some(entry) = FIELDS.iter().find(|f| f.0 == c.field) {
                let v = c.draft;
                match c.field {
                    Field::Boost => {
                        put(entry.1, (v & 1).to_string())?;
                        put("Bst_AdaPFEnable", (v >> 1).to_string())?;
                    }
                    Field::AntiAliasing => {
                        put("ASD", if v == 0 { "-1" } else { "1" }.into())?;
                        put("ASE", i32::from(v == 2).to_string())?;
                        put("ASTT", i32::from(v > 0).to_string())?;
                    }
                    Field::Sharpness => put(entry.1, format!("{:.1}", v as f32 / 100.))?,
                    _ => put(entry.1, v.to_string())?,
                }
            }
        }
        if let Some(l) = library.filter(|l| all || l.draft != l.current) {
            put(
                "FfxDllPath",
                if l.draft.trim().is_empty() {
                    " ".into()
                } else {
                    l.draft.clone()
                },
            )?;
        }
        Ok(props)
    }
}
pub fn library_path(props: &[Property]) -> String {
    value(props, "FfxDllPath").unwrap_or("").trim().into()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn game_properties_keep_other_adapters_and_decode_native_options() {
        let mut p = Property {
            name: "MfgRatio".into(),
            kind: 5,
            value: "0x0002::3;;0x0001::2;;".into(),
        };
        p.set_gpu_value("4");
        assert_eq!(p.value, "0x0001::4;;0x0002::3;;");
        let (choices, controls) = settings(&[p], |_| true);
        assert_eq!(
            choices
                .iter()
                .find(|s| s.choice.key == "MfgRatio")
                .unwrap()
                .draft,
            3
        );
        assert_eq!(
            controls
                .iter()
                .find(|c| c.field == Field::Sharpness)
                .unwrap()
                .draft,
            80
        );
        assert!(controls.iter().all(|c| c.field != Field::Cap));
    }
}
