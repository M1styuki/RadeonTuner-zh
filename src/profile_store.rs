use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::{
    fs,
    io::{self, Write},
    path::Path,
};

pub fn load(path: &Path) -> Result<Vec<Value>> {
    match fs::read(path) {
        Ok(bytes) => Ok(serde_json::from_slice(
            bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes),
        )?),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(err) => Err(err.into()),
    }
}
pub fn matches(row: &Value, key: &str, app: &str) -> bool {
    row["DeviceId"].as_str() == Some(key)
        && row["Application"]
            .as_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(app))
}

pub fn save(path: &Path, device: &str, app: &str, update: &Value) -> Result<()> {
    ensure!(!device.is_empty() && !app.is_empty(), "配置身份无效");
    let mut rows = load(path)?;
    if let Some(row) = rows.iter_mut().find(|row| matches(row, device, app)) {
        let object = row.as_object_mut().context("配置格式无效")?;
        for (key, value) in update.as_object().context("配置格式无效")? {
            object.insert(key.clone(), value.clone());
        }
    } else {
        rows.push(update.clone());
    }
    write_json(path, &Value::Array(rows))
}

pub fn read_export(path: &Path) -> Result<Value> {
    let bytes = fs::read(path)?;
    let value: Value =
        serde_json::from_slice(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes))?;
    ensure!(value.is_object(), "设置文件应包含一个配置");
    ensure!(
        value["DeviceId"].is_string()
            && value
                .as_object()
                .is_some_and(|o| o.keys().any(|k| k.ends_with(".Current"))),
        "文件中缺少设备标识或设置项"
    );
    Ok(value)
}

pub fn write_json(path: &Path, data: &Value) -> Result<()> {
    let dir = path.parent().context("配置目录无效")?;
    fs::create_dir_all(dir)?;
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    serde_json::to_writer_pretty(tmp.as_file_mut(), data)?;
    tmp.flush()?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;
    Ok(())
}

#[cfg(test)]
mod transfer_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn native_profile_files_round_trip_as_single_objects() {
        let dir = tempfile::tempdir().unwrap();
        for (filename, row) in [
            (
                "original.radg",
                json!({"DeviceId":"AMD","Application":"Global","MfgRatio.Current":2,"FsrOvrDLLPath.Current":" ","ChillMaxFps.Current":140}),
            ),
            (
                "original.radt",
                json!({"DeviceId":"AMD","Application":"游戏.exe","FanZeroRpm.Current":1,"PowerLimit.Current":-4}),
            ),
            (
                "original.radd",
                json!({"DeviceId":"23 TV 72","Application":"Global","GammaBase.Current":1.25,"HdrEnabled.Current":0}),
            ),
        ] {
            let path = dir.path().join(filename);
            let mut original = vec![0xef, 0xbb, 0xbf];
            original.extend(serde_json::to_vec(&row).unwrap());
            fs::write(&path, original).unwrap();
            assert_eq!(read_export(&path).unwrap(), row);
            write_json(&path, &row).unwrap();
            assert_eq!(read_export(&path).unwrap(), row);
            write_json(&path, &json!([row])).unwrap();
            assert!(read_export(&path).is_err());
        }
    }
}
