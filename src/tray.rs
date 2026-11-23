use crate::{panel::TITLE, settings::Settings};
use anyhow::{Context, Result, bail};
use std::{
    mem,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    process::{Child, Command},
    ptr,
};
use tray_icon::{
    Icon, MouseButton, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem},
};
use windows_sys::Win32::{
    Foundation::{
        ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, GetLastError, WAIT_FAILED, WAIT_OBJECT_0,
        WAIT_TIMEOUT,
    },
    System::Threading::{CreateEventW, EVENT_MODIFY_STATE, INFINITE, OpenEventW, SetEvent},
    UI::WindowsAndMessaging::*,
};

const OPTIONS: &str = "Local\\RadeonTuner.Rust.Options";
const RESTART: &str = "Local\\RadeonTuner.Rust.Restart";
const EXIT: &str = "Local\\RadeonTuner.Rust.Exit";
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn event(name: &str) -> Result<OwnedHandle> {
    let raw = unsafe { CreateEventW(ptr::null(), 0, 0, wide(name).as_ptr()) };
    if raw.is_null() {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(unsafe { OwnedHandle::from_raw_handle(raw) })
}
fn signal(name: &str) -> Result<()> {
    let raw = unsafe { OpenEventW(EVENT_MODIFY_STATE, 0, wide(name).as_ptr()) };
    if raw.is_null() {
        return Err(std::io::Error::last_os_error().into());
    }
    let _handle = unsafe { OwnedHandle::from_raw_handle(raw) };
    if unsafe { SetEvent(raw) } == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
pub fn settings_changed() -> Result<()> {
    match signal(OPTIONS) {
        Err(err)
            if err
                .downcast_ref::<std::io::Error>()
                .and_then(std::io::Error::raw_os_error)
                == Some(ERROR_FILE_NOT_FOUND as i32) =>
        {
            Ok(())
        }
        result => result,
    }
}
pub fn restart_after_update() -> Result<()> {
    signal(RESTART)
}
pub fn request_exit() -> Result<()> {
    signal(EXIT)
}
fn panel_window(pid: u32) -> windows_sys::Win32::Foundation::HWND {
    unsafe {
        let w = FindWindowW(ptr::null(), wide(TITLE).as_ptr());
        let mut owner = 0;
        if !w.is_null() {
            GetWindowThreadProcessId(w, &mut owner);
        }
        if owner == pid { w } else { ptr::null_mut() }
    }
}
pub fn topmost(on: bool) -> Result<()> {
    let w = panel_window(std::process::id());
    if w.is_null() {
        anyhow::bail!("找不到设置窗口");
    }
    if unsafe {
        SetWindowPos(
            w,
            if on { HWND_TOPMOST } else { HWND_NOTOPMOST },
            0,
            0,
            0,
            0,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
fn icon(monochrome: bool) -> Result<Icon> {
    let bytes: &[u8] = if monochrome {
        include_bytes!("../RadeonTuner/AppIconMono.ico")
    } else {
        include_bytes!("../RadeonTuner/AppIconColor.ico")
    };
    let rgba = image::load_from_memory(bytes)?.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(Icon::from_rgba(rgba.into_raw(), width, height)?)
}

fn open_panel(child: &mut Option<Child>) -> Result<()> {
    if let Some(process) = child.as_mut() {
        if process.try_wait()?.is_none() {
            unsafe {
                let window = panel_window(process.id());
                if !window.is_null() {
                    ShowWindow(window, SW_RESTORE);
                    SetForegroundWindow(window);
                }
            }
            return Ok(());
        }
    }
    static FIRST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
    let mut cmd = Command::new(std::env::current_exe()?);
    cmd.arg("--panel");
    if FIRST.load(std::sync::atomic::Ordering::Relaxed) {
        cmd.arg("--first-panel");
    }
    *child = Some(cmd.spawn()?);
    FIRST.store(false, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

pub fn run() -> Result<()> {
    let name: Vec<_> = "Local\\RadeonTuner.Rust.Open"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let (recall, already_running) = unsafe {
        let event = CreateEventW(ptr::null(), 0, 0, name.as_ptr());
        if event.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }
        let exists = GetLastError() == ERROR_ALREADY_EXISTS;
        (OwnedHandle::from_raw_handle(event), exists)
    };
    if already_running {
        unsafe {
            SetEvent(recall.as_raw_handle());
        }
        return Ok(());
    }
    let options_changed = event(OPTIONS)?;
    let exit_request = event(EXIT)?;
    let restart_request = event(RESTART)?;
    let installed = std::env::current_exe()?;
    let mut restarting = false;
    let settings = Settings::open()?;
    let menu = Menu::new();
    let open = MenuItem::new("打开设置", true, None);
    let quit = MenuItem::new("退出", true, None);
    menu.append(&open)?;
    let website = MenuItem::new("项目网站", true, None);
    menu.append(&website)?;
    menu.append(&quit)?;
    let mut mono = settings.flag("ThemeIconMono", false)?;
    let tray = TrayIconBuilder::new()
        .with_icon(icon(mono)?)
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip(TITLE)
        .build()?;
    tray.set_visible(!settings.flag("HideTrayIcon", false)?)?;
    let mut background = crate::profiles::Background::new(&settings)?;
    let mut last_error: Option<String> = None;
    let mut show_status = |result: Result<()>| {
        let error = result.err().map(|e| format!("{e:#}"));
        if last_error != error {
            let message = error
                .as_ref()
                .map(|e| format!("RadeonTuner · {e}"))
                .unwrap_or_else(|| TITLE.into());
            let _ = tray.set_tooltip(Some(message));
            last_error = error;
        }
    };
    show_status((|| {
        let _write = crate::driver_write::lock()?;
        if let Ok(driver) = crate::adl::Driver::open() {
            driver.initialize_profiles(&settings)?;
        }
        Ok(())
    })());
    if let Err(err) = background.reload() {
        show_status(Err(err));
    }
    let mut next_check = std::time::Instant::now() + std::time::Duration::from_secs(2);
    let mut child = None;
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--updated"))
        || settings.flag("FirstLaunch", true)?
        || settings.flag("StartWindowVisible", false)?
    {
        show_status((|| {
            let _write = crate::driver_write::lock()?;
            background.reload()?;
            background.tick()
        })());
        open_panel(&mut child)?;
    }
    'host: loop {
        let mut timeout = INFINITE;
        // The panel owns interactive writes until its process exits, as in the
        // WinUI host. A direct --panel process is covered by the shared mutex.
        if child.is_none() && background.active() {
            if std::time::Instant::now() >= next_check {
                show_status((|| {
                    let _write = crate::driver_write::lock()?;
                    background.reload()?;
                    background.tick()
                })());
                next_check = std::time::Instant::now() + std::time::Duration::from_secs(10);
            }
            timeout = next_check
                .saturating_duration_since(std::time::Instant::now())
                .as_millis() as u32;
        }
        let mut handles = vec![
            recall.as_raw_handle(),
            options_changed.as_raw_handle(),
            exit_request.as_raw_handle(),
            restart_request.as_raw_handle(),
        ];
        if let Some(process) = child.as_ref() {
            handles.push(process.as_raw_handle());
        }
        let event = unsafe {
            MsgWaitForMultipleObjectsEx(
                handles.len() as u32,
                handles.as_ptr(),
                timeout,
                QS_ALLINPUT,
                MWMO_INPUTAVAILABLE,
            )
        };
        if event == WAIT_TIMEOUT {
            continue;
        }
        if event == WAIT_FAILED {
            return Err(std::io::Error::last_os_error()).context("等待托盘消息失败");
        }
        if event == WAIT_OBJECT_0 {
            open_panel(&mut child)?;
            continue;
        }
        if event == WAIT_OBJECT_0 + 1 {
            show_status(background.reload());
            next_check = std::time::Instant::now();
            let new_mono = settings.flag("ThemeIconMono", false)?;
            if new_mono != mono {
                tray.set_icon(Some(icon(new_mono)?))?;
                mono = new_mono;
            }
            tray.set_visible(!settings.flag("HideTrayIcon", false)?)?;
            continue;
        }
        if event == WAIT_OBJECT_0 + 2 {
            break;
        }
        if event == WAIT_OBJECT_0 + 3 {
            restarting = true;
            break;
        }
        if handles.len() == 5 && event == WAIT_OBJECT_0 + 4 {
            child.take().context("面板进程已丢失")?.wait()?;
            if !settings.flag("CloseTray", true)? {
                break;
            }
            show_status(background.reload());
            next_check = std::time::Instant::now();
            continue;
        }
        if event != WAIT_OBJECT_0 + handles.len() as u32 {
            bail!("未知的 Windows 消息状态 {event}");
        }
        unsafe {
            let mut msg = mem::zeroed();
            while PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    break 'host;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == *website.id() {
                crate::integration::open("https://projects.arnoldvink.com")?;
            }
            if event.id == *quit.id() {
                break 'host;
            }
            if event.id == *open.id() {
                open_panel(&mut child)?;
            }
        }
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if matches!(
                event,
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            ) {
                open_panel(&mut child)?;
            }
        }
    }
    if let Some(mut process) = child {
        while process.try_wait()?.is_none() {
            let window = panel_window(process.id());
            if !window.is_null() {
                unsafe {
                    PostMessageW(window, WM_CLOSE, 0, 0);
                }
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        process.wait()?;
    }
    let final_result = (|| {
        let _write = crate::driver_write::lock()?;
        background.finish_display_policy()
    })();
    if restarting {
        drop(recall);
        Command::new(installed).arg("--updated").spawn()?;
    }
    final_result
}
