#![windows_subsystem = "windows"]

mod adl;
mod panel;
mod settings;
mod tray;

fn main() {
    let result = if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--panel")) {
        panel::run()
    } else {
        tray::run()
    };
    if let Err(err) = result {
        let message: Vec<u16> = format!("{err:#}\0").encode_utf16().collect();
        let title: Vec<u16> = "RadeonTuner\0".encode_utf16().collect();
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
                std::ptr::null_mut(),
                message.as_ptr(),
                title.as_ptr(),
                0x10,
            );
        }
    }
}
