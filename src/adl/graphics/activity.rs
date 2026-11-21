use std::ptr;
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_MORE_DATA, ERROR_SUCCESS, WAIT_TIMEOUT},
    System::{
        Registry::*,
        Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
    },
};

const FEATURES: [(&str, &str); 5] = [
    ("fsr4_upscale", "超分辨率"),
    ("fsr4_mlfi", "帧生成"),
    ("fsr4_mfg", "多帧生成"),
    ("fsr4_mld", "光线再生降噪"),
    ("fsr4_nrc", "神经辐射缓存"),
];

fn active_features(data: &str) -> Vec<&'static str> {
    FEATURES
        .iter()
        .filter_map(|(key, label)| {
            data.split('*')
                .filter_map(|field| field.split_once('?'))
                .any(|(name, value)| name == *key && value.starts_with("1_"))
                .then_some(*label)
        })
        .collect()
}

pub fn active_games() -> Vec<String> {
    let mut result = Vec::new();
    let path: Vec<u16> = r"Software\AMD\HKIDs"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut key = ptr::null_mut();
    unsafe {
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut key,
        ) != ERROR_SUCCESS
        {
            return result;
        }
        let mut name = vec![0u16; 256];
        let mut data = vec![0u16; 1024];
        let mut index = 0;
        loop {
            let mut name_len = name.len() as u32;
            let mut data_len = (data.len() * 2) as u32;
            let mut kind = 0;
            let code = RegEnumValueW(
                key,
                index,
                name.as_mut_ptr(),
                &mut name_len,
                ptr::null(),
                &mut kind,
                data.as_mut_ptr().cast(),
                &mut data_len,
            );
            if code == ERROR_MORE_DATA {
                // Windows limits a value name to 16,383 UTF-16 characters.
                if name.len() < 16384 {
                    name.resize((name.len() * 2).min(16384), 0);
                }
                if data_len as usize > data.len() * 2 {
                    data.resize((data_len as usize).div_ceil(2), 0);
                }
                continue;
            }
            if code != ERROR_SUCCESS {
                break;
            }
            index += 1;
            if kind != REG_SZ || data_len % 2 != 0 {
                continue;
            }
            let value = String::from_utf16_lossy(&data[..data_len as usize / 2]);
            let features = active_features(value.trim_end_matches('\0'));
            if features.is_empty() {
                continue;
            }
            let name = String::from_utf16_lossy(&name[..name_len as usize]);
            let mut parts = name.split('*');
            let Some(pid) = parts
                .next()
                .and_then(|s| u32::from_str_radix(s.trim_start_matches("0x"), 16).ok())
            else {
                continue;
            };
            let Some(process) = parts.next().filter(|s| !s.is_empty()) else {
                continue;
            };
            let handle = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
            if handle.is_null() {
                continue;
            }
            let running = WaitForSingleObject(handle, 0) == WAIT_TIMEOUT;
            CloseHandle(handle);
            if running {
                result.push(format!("{process} · FSR：{}", features.join("、")));
            }
        }
        RegCloseKey(key);
    }
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reports_only_active_driver_features() {
        assert_eq!(
            active_features("api?DX12*fsr4_mlfi?0_1*fsr4_upscale?1_2*fsr4_mfg?1*fsr4_mld?1_0"),
            ["超分辨率", "光线再生降噪"]
        );
    }
}
