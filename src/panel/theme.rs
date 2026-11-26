use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{App, Hsla, Window, WindowAppearance, px, rgb, rgba};
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

#[inline]
fn c(hex: u32) -> Hsla {
    rgb(hex).into()
}

#[inline]
fn a(hex: u32) -> Hsla {
    rgba(hex).into()
}

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

pub(super) fn is_system_light(appearance: WindowAppearance) -> bool {
    matches!(
        appearance,
        WindowAppearance::Light | WindowAppearance::VibrantLight
    )
}

pub(super) fn is_light(style: i64, system_light: bool) -> bool {
    style == 2 || (style == 3 && system_light)
}

/// Applies the Windows 11 Fluent 2 light/dark theme tokens with refined Radeon
/// crimson accents, preserving `ThemeStyle` (0=gray, 1=black, 2=light, 3=system)
/// and native Mica surface hierarchy via `Theme::update`.
pub(super) fn sync_theme(
    window: Option<&mut Window>,
    style: i64,
    has_mica: bool,
    cx: &mut App,
) {
    let system_light = is_system_light(
        window
            .as_ref()
            .map(|w| w.appearance())
            .unwrap_or_else(|| cx.window_appearance()),
    );
    let light = is_light(style, system_light);
    let pure_black = !light && style == 1;

    let mode = if light {
        ThemeMode::Light
    } else {
        ThemeMode::Dark
    };
    if Theme::global(cx).mode != mode {
        Theme::change(mode, None, cx);
    }

    Theme::update(cx, |theme| {
        theme.font_size = px(16.);
        theme.radius = px(6.);
        theme.radius_lg = px(10.);
        theme.shadow = true;
        theme.focus_ring = true;

        if light {
            theme.colors.background = if has_mica { a(0xf3f3f3c8) } else { c(0xf3f3f3) };
            theme.colors.foreground = c(0x1a1a1a);
            theme.colors.muted = c(0xeaeaea);
            theme.colors.muted_foreground = c(0x5c5c5c);

            theme.colors.group_box = if has_mica { a(0xffffffdc) } else { c(0xffffff) };
            theme.colors.group_box_foreground = c(0x1a1a1a);

            theme.colors.popover = c(0xf9f9f9);
            theme.colors.popover_foreground = c(0x1a1a1a);

            theme.colors.sidebar = if has_mica { a(0xf3f3f348) } else { c(0xeeeeee) };
            theme.colors.sidebar_foreground = c(0x1a1a1a);
            theme.colors.sidebar_accent = a(0x0000000e);
            theme.colors.sidebar_accent_foreground = c(0x1a1a1a);
            theme.colors.sidebar_border = a(0x00000014);

            theme.colors.title_bar = if has_mica { a(0xf3f3f330) } else { c(0xeeeeee) };
            theme.colors.title_bar_border = a(0x00000012);

            theme.colors.border = a(0x00000018);
            theme.colors.input = a(0x00000024);

            theme.colors.primary = c(0xb8323c);
            theme.colors.primary_hover = c(0xa32932);
            theme.colors.primary_active = c(0x8f222a);
            theme.colors.primary_foreground = c(0xffffff);
            theme.colors.button_primary = c(0xb8323c);
            theme.colors.button_primary_hover = c(0xa32932);
            theme.colors.button_primary_active = c(0x8f222a);
            theme.colors.button_primary_foreground = c(0xffffff);

            theme.colors.secondary = a(0xffffffc8);
            theme.colors.secondary_hover = a(0xf9f9f9ee);
            theme.colors.secondary_active = a(0xf0f0f0dd);
            theme.colors.secondary_foreground = c(0x1a1a1a);
            theme.colors.button_secondary = a(0xffffffc8);
            theme.colors.button_secondary_hover = a(0xf9f9f9ee);
            theme.colors.button_secondary_active = a(0xf0f0f0dd);
            theme.colors.button_secondary_foreground = c(0x1a1a1a);

            theme.colors.button = a(0xffffffcc);
            theme.colors.button_hover = a(0xf9f9f9f0);
            theme.colors.button_active = a(0xf0f0f0e0);
            theme.colors.button_foreground = c(0x1a1a1a);

            theme.colors.accent = a(0x0000000c);
            theme.colors.accent_foreground = c(0x1a1a1a);
            theme.colors.ring = c(0xb8323c);

            theme.colors.slider_bar = c(0xb8323c);
            theme.colors.slider_thumb = c(0xffffff);
            theme.colors.progress_bar = c(0xb8323c);

            theme.colors.danger = c(0xc42b1c);
            theme.colors.danger_hover = c(0xb02315);
            theme.colors.danger_active = c(0x9a1d11);
            theme.colors.danger_foreground = c(0xffffff);
            theme.colors.button_danger = c(0xc42b1c);
            theme.colors.button_danger_hover = c(0xb02315);
            theme.colors.button_danger_active = c(0x9a1d11);
            theme.colors.button_danger_foreground = c(0xffffff);

            theme.colors.chart_grid = a(0x00000018);
            theme.colors.overlay = a(0x00000050);
            theme.colors.switch = a(0x00000028);
            theme.colors.switch_thumb = c(0xffffff);
            theme.colors.caret = c(0x1a1a1a);
            theme.colors.selection = a(0xb8323c40);
            theme.colors.scrollbar_thumb = a(0x00000030);
            theme.colors.scrollbar_thumb_hover = a(0x0000004c);
        } else {
            theme.colors.background = if pure_black {
                if has_mica { a(0x0a0a0ad8) } else { c(0x0a0a0a) }
            } else if has_mica {
                a(0x202020c8)
            } else {
                c(0x202020)
            };
            theme.colors.foreground = c(0xf5f5f5);
            theme.colors.muted = c(0x2b2b2b);
            theme.colors.muted_foreground = c(0xa3a3a3);

            theme.colors.group_box = if pure_black {
                c(0x14161a)
            } else if has_mica {
                a(0xffffff0d)
            } else {
                c(0x2b2b2b)
            };
            theme.colors.group_box_foreground = c(0xf5f5f5);

            theme.colors.popover = if pure_black { c(0x16181c) } else { c(0x2c2c2c) };
            theme.colors.popover_foreground = c(0xf5f5f5);

            theme.colors.sidebar = if pure_black {
                if has_mica { a(0x08080880) } else { c(0x080808) }
            } else if has_mica {
                a(0x20202040)
            } else {
                c(0x1c1c1c)
            };
            theme.colors.sidebar_foreground = c(0xf5f5f5);
            theme.colors.sidebar_accent = a(0xffffff12);
            theme.colors.sidebar_accent_foreground = c(0xffffff);
            theme.colors.sidebar_border = a(0xffffff14);

            theme.colors.title_bar = if pure_black {
                if has_mica { a(0x08080860) } else { c(0x080808) }
            } else if has_mica {
                a(0x20202028)
            } else {
                c(0x1c1c1c)
            };
            theme.colors.title_bar_border = a(0xffffff12);

            theme.colors.border = a(0xffffff18);
            theme.colors.input = a(0xffffff26);

            theme.colors.primary = c(0xd94b54);
            theme.colors.primary_hover = c(0xe35d66);
            theme.colors.primary_active = c(0xc03b44);
            theme.colors.primary_foreground = c(0xffffff);
            theme.colors.button_primary = c(0xd94b54);
            theme.colors.button_primary_hover = c(0xe35d66);
            theme.colors.button_primary_active = c(0xc03b44);
            theme.colors.button_primary_foreground = c(0xffffff);

            theme.colors.secondary = a(0xffffff10);
            theme.colors.secondary_hover = a(0xffffff18);
            theme.colors.secondary_active = a(0xffffff0b);
            theme.colors.secondary_foreground = c(0xf5f5f5);
            theme.colors.button_secondary = a(0xffffff10);
            theme.colors.button_secondary_hover = a(0xffffff18);
            theme.colors.button_secondary_active = a(0xffffff0b);
            theme.colors.button_secondary_foreground = c(0xf5f5f5);

            theme.colors.button = a(0xffffff10);
            theme.colors.button_hover = a(0xffffff18);
            theme.colors.button_active = a(0xffffff0b);
            theme.colors.button_foreground = c(0xf5f5f5);

            theme.colors.accent = a(0xffffff0f);
            theme.colors.accent_foreground = c(0xf5f5f5);
            theme.colors.ring = c(0xd94b54);

            theme.colors.slider_bar = c(0xd94b54);
            theme.colors.slider_thumb = c(0xffffff);
            theme.colors.progress_bar = c(0xd94b54);

            theme.colors.danger = c(0xd13438);
            theme.colors.danger_hover = c(0xbe2b2f);
            theme.colors.danger_active = c(0xa52226);
            theme.colors.danger_foreground = c(0xffffff);
            theme.colors.button_danger = c(0xd13438);
            theme.colors.button_danger_hover = c(0xbe2b2f);
            theme.colors.button_danger_active = c(0xa52226);
            theme.colors.button_danger_foreground = c(0xffffff);

            theme.colors.chart_grid = a(0xffffff16);
            theme.colors.overlay = a(0x00000078);
            theme.colors.switch = a(0xffffff26);
            theme.colors.switch_thumb = c(0xffffff);
            theme.colors.caret = c(0xf5f5f5);
            theme.colors.selection = a(0xd94b5455);
            theme.colors.scrollbar_thumb = a(0xffffff38);
            theme.colors.scrollbar_thumb_hover = a(0xffffff58);
        }
    });
}
