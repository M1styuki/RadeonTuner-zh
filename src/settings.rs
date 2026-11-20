use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

pub struct Settings {
    path: PathBuf,
}

impl Settings {
    pub fn open() -> Result<Self> {
        let root = std::env::var_os("LOCALAPPDATA").context("无法定位本地应用数据目录")?;
        Ok(Self {
            path: Path::new(&root).join("RadeonTuner-zh-CN/App/Settings/Settings.json"),
        })
    }

    fn read(&self) -> Result<Map<String, Value>> {
        match fs::read(&self.path) {
            Ok(bytes) => Ok(serde_json::from_slice(
                bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes),
            )?),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Map::new()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn flag(&self, key: &str, default: bool) -> Result<bool> {
        Ok(self
            .read()?
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or(default))
    }

    pub fn set_flag(&self, key: &str, value: bool) -> Result<()> {
        let mut settings = self.read()?;
        settings.insert(key.into(), Value::Bool(value));
        let dir = self.path.parent().context("无效设置路径")?;
        fs::create_dir_all(dir)?;
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
        serde_json::to_writer_pretty(tmp.as_file_mut(), &settings)?;
        tmp.flush()?;
        tmp.as_file().sync_all()?;
        tmp.persist(&self.path)?;
        Ok(())
    }

    pub fn profiles_path(&self, filename: &str) -> Result<PathBuf> {
        Ok(self
            .path
            .parent()
            .and_then(Path::parent)
            .context("配置目录无效")?
            .join("Profiles")
            .join(filename))
    }
}
