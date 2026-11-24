use super::wide;
use anyhow::{Context, Result, ensure};
use std::{io, mem::size_of, ptr};
use windows_sys::Win32::{
    Foundation::{
        ERROR_SERVICE_ALREADY_RUNNING, ERROR_SERVICE_DOES_NOT_EXIST, ERROR_SERVICE_NOT_ACTIVE,
        GetLastError,
    },
    System::Services::*,
};

const NAMES: [&str; 2] = ["AMD External Events Utility", "AMD Crash Defender Service"];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Missing,
    Disabled,
    Enabled,
    Mixed,
}

struct Handle(SC_HANDLE);
impl Handle {
    fn new(raw: SC_HANDLE) -> io::Result<Self> {
        if raw.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self(raw))
        }
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { CloseServiceHandle(self.0) };
    }
}
fn open_service(manager: &Handle, name: &str, access: u32) -> io::Result<Option<Handle>> {
    let name = wide(name);
    let missing = |err: &io::Error| err.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST as i32);
    match Handle::new(unsafe { OpenServiceW(manager.0, name.as_ptr(), access) }) {
        Ok(handle) => return Ok(Some(handle)),
        Err(err) if missing(&err) => (),
        Err(err) => return Err(err),
    }
    let mut key = [0u16; 257];
    let mut capacity = key.len() as u32;
    if unsafe { GetServiceKeyNameW(manager.0, name.as_ptr(), key.as_mut_ptr(), &mut capacity) } == 0
    {
        let err = io::Error::last_os_error();
        return if missing(&err) { Ok(None) } else { Err(err) };
    }
    match Handle::new(unsafe { OpenServiceW(manager.0, key.as_ptr(), access) }) {
        Ok(handle) => Ok(Some(handle)),
        Err(err) if missing(&err) => Ok(None),
        Err(err) => Err(err),
    }
}

fn is_enabled(service: &Handle) -> io::Result<bool> {
    // QueryServiceConfigW accepts at most 8 KiB. Use pointer-aligned storage.
    let mut buffer = [0usize; 8192 / size_of::<usize>()];
    let config = buffer.as_mut_ptr().cast::<QUERY_SERVICE_CONFIGW>();
    let mut needed = 0;
    if unsafe { QueryServiceConfigW(service.0, config, 8192, &mut needed) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { (*config).dwStartType } != SERVICE_DISABLED)
}

pub fn state() -> Result<State> {
    let manager =
        Handle::new(unsafe { OpenSCManagerW(ptr::null(), ptr::null(), SC_MANAGER_CONNECT) })?;
    let mut enabled = false;
    let mut disabled = false;
    for name in NAMES {
        if let Some(service) = open_service(&manager, name, SERVICE_QUERY_CONFIG).context(name)? {
            if is_enabled(&service).context(name)? {
                enabled = true;
            } else {
                disabled = true;
            }
        }
    }
    Ok(match (enabled, disabled) {
        (false, false) => State::Missing,
        (false, true) => State::Disabled,
        (true, false) => State::Enabled,
        (true, true) => State::Mixed,
    })
}

fn change(manager: &Handle, name: &str, enable: bool) -> Result<bool> {
    let access = SERVICE_CHANGE_CONFIG
        | SERVICE_QUERY_STATUS
        | if enable { SERVICE_START } else { SERVICE_STOP };
    let Some(service) = open_service(manager, name, access)? else {
        return Ok(false);
    };
    let start = if enable {
        SERVICE_DEMAND_START
    } else {
        SERVICE_DISABLED
    };
    if unsafe {
        ChangeServiceConfigW(
            service.0,
            SERVICE_NO_CHANGE,
            start,
            SERVICE_NO_CHANGE,
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error().into());
    }
    let rc = if enable {
        unsafe { StartServiceW(service.0, 0, ptr::null()) }
    } else {
        let mut status = unsafe { std::mem::zeroed() };
        unsafe { ControlService(service.0, SERVICE_CONTROL_STOP, &mut status) }
    };
    if rc == 0 {
        let error = unsafe { GetLastError() };
        if (enable && error != ERROR_SERVICE_ALREADY_RUNNING)
            || (!enable && error != ERROR_SERVICE_NOT_ACTIVE)
        {
            return Err(io::Error::from_raw_os_error(error as i32).into());
        }
    }
    Ok(true)
}
pub fn set_enabled(enable: bool) -> Result<()> {
    let manager =
        Handle::new(unsafe { OpenSCManagerW(ptr::null(), ptr::null(), SC_MANAGER_CONNECT) })?;
    let mut failures = Vec::new();
    let mut changed = false;
    for name in NAMES {
        match change(&manager, name, enable) {
            Ok(installed) => changed |= installed,
            Err(err) => failures.push(format!("{name}（{err}）")),
        }
    }
    ensure!(
        failures.is_empty(),
        "服务操作未完全成功：{}",
        failures.join("；")
    );
    ensure!(changed, "未检测到 Radeon 服务");
    Ok(())
}
