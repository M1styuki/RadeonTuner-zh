use anyhow::{Result, ensure};
use std::{os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle}, ptr};
use windows_sys::Win32::{
    Foundation::{WAIT_ABANDONED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject},
};

pub struct Guard(OwnedHandle);

pub fn lock() -> Result<Guard> {
    let name: Vec<u16> = "Local\\RadeonTuner.Rust.DriverWrite"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let raw = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };
    if raw.is_null() {
        return Err(std::io::Error::last_os_error().into());
    }
    let handle = unsafe { OwnedHandle::from_raw_handle(raw) };
    let result = unsafe { WaitForSingleObject(handle.as_raw_handle(), 5000) };
    ensure!(result != WAIT_TIMEOUT, "另一进程正在应用驱动设置，请稍后重试");
    if !matches!(result, WAIT_OBJECT_0 | WAIT_ABANDONED) {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(Guard(handle))
}

impl Drop for Guard {
    fn drop(&mut self) {
        unsafe { ReleaseMutex(self.0.as_raw_handle()); }
    }
}
