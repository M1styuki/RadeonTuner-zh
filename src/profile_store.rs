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
    let dir = path.parent().context("配置目录无效")?;
    fs::create_dir_all(dir)?;
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    serde_json::to_writer_pretty(tmp.as_file_mut(), &rows)?;
    tmp.flush()?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;
    Ok(())
}
