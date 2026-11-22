use super::wide;
use anyhow::{Context, Result, ensure};
use std::{
    collections::{BTreeSet, HashSet},
    fs,
    io::{BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
use windows_sys::Win32::{
    Foundation::{ERROR_SUCCESS, HWND, LPARAM},
    System::Registry::*,
    UI::WindowsAndMessaging::*,
};

pub type Entry = (String, String);
pub fn running() -> Result<Vec<Entry>> {
    unsafe extern "system" fn window(hwnd: HWND, param: LPARAM) -> i32 {
        if unsafe { IsWindowVisible(hwnd) } != 0 && unsafe { GetWindow(hwnd, GW_OWNER) }.is_null() {
            let mut pid = 0;
            unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
            let pids = unsafe { &mut *(param as *mut HashSet<u32>) };
            pids.insert(pid);
        }
        1
    }
    let mut pids = HashSet::<u32>::new();
    ensure!(
        unsafe { EnumWindows(Some(window), (&mut pids as *mut HashSet<u32>) as isize) } != 0,
        "读取运行窗口失败"
    );
    let own = std::env::current_exe()?
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    let mut names = BTreeSet::new();
    for (pid, name) in crate::profiles::process_list()? {
        let lower = name.to_lowercase();
        if pids.contains(&pid)
            && lower != own
            && lower != "explorer.exe"
            && lower != "radeontuner.exe"
        {
            names.insert((name.clone(), name));
        }
    }
    Ok(names.into_iter().collect())
}
fn steam_path() -> Result<PathBuf> {
    for (hive, path, key) in [
        (HKEY_CURRENT_USER, r"Software\Valve\Steam", "SteamPath"),
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\WOW6432Node\Valve\Steam",
            "InstallPath",
        ),
    ] {
        let mut buf = [0u16; 1024];
        let mut bytes = (buf.len() * 2) as u32;
        let rc = unsafe {
            RegGetValueW(
                hive,
                wide(path).as_ptr(),
                wide(key).as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                buf.as_mut_ptr().cast(),
                &mut bytes,
            )
        };
        if rc == ERROR_SUCCESS && bytes >= 2 {
            return Ok(String::from_utf16(&buf[..(bytes as usize / 2) - 1])?.into());
        }
    }
    anyhow::bail!("未检测到 Steam");
}
fn installed(libraries: &str) -> HashSet<u32> {
    let mut ids = HashSet::new();
    let mut apps = false;
    for line in libraries.lines().map(str::trim) {
        if line == "\"apps\"" {
            apps = true;
        } else if line == "}" {
            apps = false;
        } else if apps {
            if let Some(id) = line.split('"').nth(1).and_then(|s| s.parse::<u32>().ok()) {
                ids.insert(id);
            }
        }
    }
    ids
}
fn take<'a>(data: &mut &'a [u8], n: usize) -> Result<&'a [u8]> {
    let (head, rest) = data.split_at_checked(n).context("Steam 游戏数据不完整")?;
    *data = rest;
    Ok(head)
}
fn integer(data: &mut &[u8]) -> Result<u32> {
    Ok(u32::from_le_bytes(take(data, 4)?.try_into()?))
}
fn string<'a>(data: &mut &'a [u8]) -> Result<&'a str> {
    let n = data
        .iter()
        .position(|&b| b == 0)
        .context("Steam 字符串未结束")?;
    let bytes = take(data, n + 1)?;
    Ok(std::str::from_utf8(&bytes[..n])?)
}
fn game(mut data: &[u8], keys: &[&str]) -> Result<Vec<Entry>> {
    let mut stack = Vec::new();
    let mut title = None;
    let mut exe = BTreeSet::new();
    while !data.is_empty() {
        let kind = take(&mut data, 1)?[0];
        if kind == 8 {
            if stack.pop().is_none() {
                break;
            }
            continue;
        }
        let key = *keys
            .get(integer(&mut data)? as usize)
            .context("Steam 游戏属性索引无效")?;
        match kind {
            0 => stack.push(key),
            1 => {
                let val = string(&mut data)?;
                if stack == ["appinfo", "common"] && key == "name" {
                    title = Some(val.to_owned());
                }
                if stack.starts_with(&["appinfo", "config", "launch"]) && key == "executable" {
                    let file = val.rsplit(['\\', '/']).next().unwrap_or(val);
                    let lower = file.to_ascii_lowercase();
                    if lower.ends_with(".exe")
                        && ![
                            "launcher",
                            "launcer",
                            "starter",
                            "config",
                            "setup",
                            "tools",
                            "install",
                            "bootstrapper",
                            "dedicated",
                            "server",
                            "service",
                            "boot.exe",
                        ]
                        .iter()
                        .any(|s| lower.contains(s))
                    {
                        exe.insert(file.to_owned());
                    }
                }
            }
            2 | 3 | 4 | 6 => {
                take(&mut data, 4)?;
            }
            7 | 10 => {
                take(&mut data, 8)?;
            }
            _ => anyhow::bail!("Steam 游戏属性格式不受支持"),
        }
    }
    Ok(title
        .map(|t| exe.into_iter().map(|name| (t.clone(), name)).collect())
        .unwrap_or_default())
}
fn read_u32(r: &mut impl Read) -> Result<u32> {
    let mut b = [0; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}
fn steam(root: &Path) -> Result<Vec<Entry>> {
    let ids = installed(&fs::read_to_string(
        root.join("steamapps/libraryfolders.vdf"),
    )?);
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(root.join("appcache/appinfo.vdf"))?;
    let len = file.metadata()?.len();
    let mut r = BufReader::new(file);
    ensure!(read_u32(&mut r)? & 255 == 41, "不支持此 Steam 游戏索引版本");
    read_u32(&mut r)?;
    let mut b = [0; 8];
    r.read_exact(&mut b)?;
    let table = u64::from_le_bytes(b);
    ensure!((16..len).contains(&table), "Steam 游戏索引长度无效");
    r.seek(SeekFrom::Start(table))?;
    let count = read_u32(&mut r)? as usize;
    let mut strings = Vec::new();
    r.read_to_end(&mut strings)?;
    ensure!(count <= strings.len(), "Steam 游戏索引字符串数量无效");
    let mut data = strings.as_slice();
    let keys = (0..count)
        .map(|_| string(&mut data))
        .collect::<Result<Vec<_>>>()?;
    r.seek(SeekFrom::Start(16))?;
    let mut list = BTreeSet::new();
    while r.stream_position()? < table {
        let id = read_u32(&mut r)?;
        if id == 0 {
            break;
        }
        let size = read_u32(&mut r)? as usize;
        let start = r.stream_position()?;
        let end = start
            .checked_add(size as u64)
            .filter(|p| *p <= table)
            .context("Steam 游戏记录无效")?;
        if ids.contains(&id) && size >= 60 {
            r.seek(SeekFrom::Start(start + 60))?;
            let mut record = vec![0; size - 60];
            r.read_exact(&mut record)?;
            if let Ok(entries) = game(&record, &keys) {
                list.extend(entries);
            }
        } else {
            r.seek(SeekFrom::Start(end))?;
        }
    }
    Ok(list.into_iter().collect())
}
pub fn launcher() -> Result<Vec<Entry>> {
    steam(&steam_path()?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn steam_index_reads_launch_entries_of_installed_games() {
        let ids = installed("\"apps\"\n{\n\"42\" \"1000\"\n}\n\"1\" \"0\"");
        assert_eq!(ids, HashSet::from([42]));
        let keys = [
            "appinfo",
            "common",
            "name",
            "config",
            "launch",
            "0",
            "executable",
        ];
        let mut data = Vec::new();
        for (kind, key, s) in [
            (0, 0, ""),
            (0, 1, ""),
            (1, 2, "游戏"),
            (8, 0, ""),
            (0, 3, ""),
            (0, 4, ""),
            (0, 5, ""),
            (1, 6, r"bin\game.exe"),
        ] {
            data.push(kind);
            if kind != 8 {
                data.extend((key as u32).to_le_bytes());
            }
            if kind == 1 {
                data.extend(s.as_bytes());
                data.push(0);
            }
        }
        assert_eq!(
            game(&data, &keys).unwrap(),
            [("游戏".into(), "game.exe".into())]
        );
    }
}
