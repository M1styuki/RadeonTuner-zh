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
    Foundation::{ERROR_ALREADY_EXISTS, GetLastError, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::{CreateEventW, INFINITE, SetEvent},
    UI::WindowsAndMessaging::*,
};

fn open_panel(child: &mut Option<Child>) -> Result<()> {
    if let Some(process) = child.as_mut() {
        if process.try_wait()?.is_none() {
            let title: Vec<_> = TITLE.encode_utf16().chain(Some(0)).collect();
            unsafe {
                let window = FindWindowW(ptr::null(), title.as_ptr());
                if !window.is_null() {
                    ShowWindow(window, SW_RESTORE);
                    SetForegroundWindow(window);
                }
            }
            return Ok(());
        }
    }
    *child = Some(
        Command::new(std::env::current_exe()?)
            .arg("--panel")
            .spawn()?,
    );
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
    let settings = Settings::open()?;
    let menu = Menu::new();
    let open = MenuItem::new("打开设置", true, None);
    let quit = MenuItem::new("退出", true, None);
    menu.append(&open)?;
    menu.append(&quit)?;
    let rgba =
        image::load_from_memory(include_bytes!("../RadeonTuner/AppIconColor.ico"))?.to_rgba8();
    let (width, height) = rgba.dimensions();
    let icon = Icon::from_rgba(rgba.into_raw(), width, height)?;
    let tray = TrayIconBuilder::new()
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip(TITLE)
        .build()?;
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
    show_status(background.reload());
    let mut next_check = std::time::Instant::now() + std::time::Duration::from_secs(2);
    let mut child = None;
    if settings.flag("FirstLaunch", true)? || settings.flag("StartWindowVisible", false)? {
        open_panel(&mut child)?;
    }
    'host: loop {
        let mut timeout = INFINITE;
        if child.is_none() && background.active() {
            if std::time::Instant::now() >= next_check {
                show_status(background.tick());
                next_check = std::time::Instant::now() + std::time::Duration::from_secs(10);
            }
            timeout = next_check
                .saturating_duration_since(std::time::Instant::now())
                .as_millis() as u32;
        }
        let mut handles = vec![recall.as_raw_handle()];
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
        if handles.len() == 2 && event == WAIT_OBJECT_0 + 1 {
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
        if process.try_wait()?.is_none() {
            process.kill()?;
        }
        process.wait()?;
    }
    Ok(())
}
