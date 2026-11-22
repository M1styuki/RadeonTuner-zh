use super::*;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

fn clear_folder(root: &Path, relative: &Path, failed: &mut usize) -> io::Result<()> {
    let mut dir = root.to_owned();
    // A relocated cache can be a junction. Never traverse it into another folder.
    for part in relative.components() {
        dir.push(part);
        match fs::symlink_metadata(&dir) {
            Ok(m) if !m.is_dir() || m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 => {
                return Ok(());
            }
            Ok(_) => (),
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e),
        }
    }
    for entry in fs::read_dir(&dir)? {
        let result = entry.and_then(|e| {
            let path = e.path();
            let m = fs::symlink_metadata(&path)?;
            if m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Ok(());
            }
            if m.is_dir() {
                fs::remove_dir_all(path)
            } else {
                fs::remove_file(path)
            }
        });
        if result.is_err() {
            *failed += 1;
        }
    }
    Ok(())
}
pub fn clear_shader_cache() -> Result<()> {
    let local = super::known_folder(&FOLDERID_LocalAppData)?;
    let low = super::known_folder(&FOLDERID_LocalAppDataLow)?;
    let mut failed = 0;
    clear_folder(&local, Path::new("D3DSCache"), &mut failed)?;
    for root in [local, low] {
        for name in ["DX9Cache", "DxCache", "DxcCache", "OglCache", "VkCache"] {
            clear_folder(&root, &Path::new("AMD").join(name), &mut failed)?;
        }
    }
    ensure!(failed == 0, "部分缓存正在使用或无法删除（{failed} 项）");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clears_only_named_cache_contents() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("AMD/DxCache/sub")).unwrap();
        fs::write(root.join("AMD/DxCache/sub/shader"), b"x").unwrap();
        fs::write(root.join("AMD/sibling"), b"x").unwrap();
        let mut failed = 0;
        clear_folder(root, Path::new("AMD/DxCache"), &mut failed).unwrap();
        assert_eq!(failed, 0);
        assert_eq!(fs::read_dir(root.join("AMD/DxCache")).unwrap().count(), 0);
        assert!(root.join("AMD/sibling").is_file());
    }
}
