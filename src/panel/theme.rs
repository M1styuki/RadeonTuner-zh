use gpui::{Hsla, Window, rgb};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::{
    Foundation::HWND,
    Graphics::Dwm::{
        DWMSBT_MAINWINDOW, DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE,
        DwmExtendFrameIntoClientArea, DwmSetWindowAttribute,
    },
    UI::{
        Controls::MARGINS,
        WindowsAndMessaging::{GWL_EXSTYLE, GetWindowLongW, WS_EX_NOREDIRECTIONBITMAP},
    },
};

fn hwnd(window: &Window) -> Option<HWND> {
    match HasWindowHandle::window_handle(window).ok()?.as_raw() {
        RawWindowHandle::Win32(handle) => Some(handle.hwnd.get() as HWND),
        _ => None,
    }
}

pub(super) fn set_caption_dark(window: &Window, dark: bool) {
    if let Some(hwnd) = hwnd(window) {
        let value = i32::from(dark);
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
                std::ptr::from_ref(&value).cast(),
                size_of_val(&value) as u32,
            );
        }
    }
}

pub(super) fn enable_mica(window: &Window) -> bool {
    let Some(hwnd) = hwnd(window) else {
        return false;
    };
    let margins = MARGINS {
        cxLeftWidth: -1,
        cxRightWidth: -1,
        cyTopHeight: -1,
        cyBottomHeight: -1,
    };
    // GPUI's non-composition renderer ignores alpha. Keep its surfaces opaque.
    unsafe {
        GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_NOREDIRECTIONBITMAP != 0
            && DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE as u32,
                std::ptr::from_ref(&DWMSBT_MAINWINDOW).cast(),
                size_of_val(&DWMSBT_MAINWINDOW) as u32,
            ) >= 0
            && DwmExtendFrameIntoClientArea(hwnd, &margins) >= 0
    }
}

impl super::Panel {
    pub(super) fn layer_color(&self, color: u32, opacity: f32) -> Hsla {
        let fill = Hsla::from(rgb(color));
        if self.mica {
            fill.opacity(opacity)
        } else {
            fill
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Palette {
    pub hover: u32,
    pub canvas: u32,
    pub text: u32,
    pub muted_text: u32,
    pub sidebar: u32,
    pub control: u32,
    pub selected: u32,
    pub selection: u32,
    pub border: u32,
    pub grid: u32,
    pub accent: u32,
    pub accent_hover: u32,
}
impl Palette {
    pub(super) fn is_light(style: i64, system_light: bool) -> bool {
        style == 2 || (style == 3 && system_light)
    }

    pub fn resolve(style: i64, system_light: bool) -> Self {
        if Self::is_light(style, system_light) {
            Self {
                hover: 0xe9edf3,
                canvas: 0xffffff,
                text: 0x1b2028,
                muted_text: 0x626c7b,
                sidebar: 0xeef1f5,
                control: 0xffffff,
                selected: 0xe6ebf2,
                selection: 0xe6afb5,
                border: 0xd3d9e2,
                grid: 0xd5d8dc,
                accent: 0xb84b52,
                accent_hover: 0xa24049,
            }
        } else {
            Self {
                hover: 0x2b323d,
                canvas: if style == 1 { 0 } else { 0x15181d },
                text: 0xe6ebf2,
                muted_text: 0xa3adba,
                sidebar: 0x121418,
                control: 0x20252d,
                selected: 0x262c35,
                selection: 0x80525b,
                border: 0x3a424e,
                grid: 0x3c424a,
                accent: 0xc85c63,
                accent_hover: 0xe47780,
            }
        }
    }
}
