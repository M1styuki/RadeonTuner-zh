use super::wide;
use anyhow::{Result, ensure};
use std::{io, ptr};
use windows_sys::Win32::{
    Foundation::{ERROR_SERVICE_ALREADY_RUNNING, ERROR_SERVICE_NOT_ACTIVE, GetLastError},
    System::Services::*,
};

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
fn change(manager: &Handle, name: &str, enable: bool) -> Result<()> {
    let access = SERVICE_CHANGE_CONFIG
        | SERVICE_QUERY_STATUS
        | if enable { SERVICE_START } else { SERVICE_STOP };
    let service = Handle::new(unsafe { OpenServiceW(manager.0, wide(name).as_ptr(), access) })?;
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
    Ok(())
}
pub fn set_enabled(enable: bool) -> Result<()> {
    let manager =
        Handle::new(unsafe { OpenSCManagerW(ptr::null(), ptr::null(), SC_MANAGER_CONNECT) })?;
    let mut failures = Vec::new();
    for name in ["AMD External Events Utility", "AMD Crash Defender Service"] {
        if let Err(err) = change(&manager, name, enable) {
            failures.push(format!("{name}（{err}）"));
        }
    }
    ensure!(
        failures.is_empty(),
        "服务操作未完全成功：{}",
        failures.join("；")
    );
    Ok(())
}
