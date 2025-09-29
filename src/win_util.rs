use std::ffi::{OsStr, OsString};
use std::fmt::Display;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr::null_mut;
use windows::Win32::Foundation::{GetLastError, HWND, LPARAM, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{ClientToScreen, GetDC, GetPixel, ReleaseDC, ScreenToClient};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, SetFocus, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowW, GetCursorPos, GetForegroundWindow, GetWindowTextLengthW,
    GetWindowTextW, GetWindowThreadProcessId, HWND_TOP, IsIconic, IsWindowVisible, PostMessageW,
    SW_RESTORE, SWP_NOZORDER, SWP_SHOWWINDOW, SetForegroundWindow, SetWindowPos, ShowWindow,
    WM_KEYDOWN, WM_KEYUP,
};
use windows::core::{Error, PCWSTR};

fn make_input(vk: VIRTUAL_KEY, key_up: bool) -> INPUT {
    let flags = if key_up {
        KEYEVENTF_KEYUP
    } else {
        Default::default()
    };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

pub fn send_key_vk(vk: VIRTUAL_KEY) -> windows::core::Result<()> {
    unsafe {
        let inputs = [make_input(vk, false), make_input(vk, true)];
        let sent = SendInput(&inputs, size_of::<INPUT>() as i32);
        if sent == 0 {
            Err(Error::from(GetLastError()))
        } else {
            Ok(())
        }
    }
}

// does not work with games >:((
#[allow(dead_code)]
pub fn send_key_to_window(hwnd: Option<HWND>, vk: VIRTUAL_KEY) -> windows::core::Result<()> {
    unsafe {
        let vk = vk.0 as usize;
        if PostMessageW(hwnd, WM_KEYDOWN, WPARAM(vk), LPARAM(0)).is_err() {
            return Err(Error::from(GetLastError()));
        }
        if PostMessageW(hwnd, WM_KEYUP, WPARAM(vk), LPARAM(0)).is_err() {
            return Err(Error::from(GetLastError()));
        }
        Ok(())
    }
}

pub fn find_window_by_title(title: &str) -> Option<HWND> {
    unsafe {
        let wide: Vec<u16> = OsStr::new(title)
            .encode_wide()
            .chain(std::iter::once(0)) // null terminator
            .collect();
        let hwnd = FindWindowW(None, PCWSTR::from_raw(wide.as_ptr()));
        hwnd.ok()
    }
}

pub fn enum_windows() -> windows::core::Result<()> {
    unsafe { EnumWindows(Some(enum_windows_proc), LPARAM(0)) }
}
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> windows::core::BOOL {
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return true.into();
        }
        let length = GetWindowTextLengthW(hwnd);
        if length == 0 {
            return true.into();
        }
        let mut buffer = vec![0u16; length as usize + 1];
        let copied = GetWindowTextW(hwnd, &mut buffer);
        if copied > 0 {
            buffer.truncate(copied as usize);
            let title = OsString::from_wide(&buffer).to_string_lossy().into_owned();

            println!("HWND: {:?}, Title: {}", hwnd, title);
        }
        true.into()
    }
}

pub fn focus_window(hwnd_opt: Option<HWND>) -> windows::core::BOOL {
    unsafe {
        if let Some(hwnd) = hwnd_opt {
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
            let fg_window = GetForegroundWindow();
            let current_thread = GetCurrentThreadId();
            let fg_thread = GetWindowThreadProcessId(fg_window, Some(null_mut()));
            // Attach input threads temporarily
            let attached = AttachThreadInput(fg_thread, current_thread, true).as_bool();
            // Set focus and foreground
            let _ = SetFocus(hwnd_opt);
            let result = SetForegroundWindow(hwnd);
            // Detach input threads
            if attached {
                let _ = AttachThreadInput(fg_thread, current_thread, false);
            }
            return result;
        }
        false.into()
    }
}

#[derive(Debug)]
pub struct PixelColor(pub u32);

impl Display for PixelColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:06X}", self.0 & 0xFFFFFF)
    }
}

const CLR_INVALID: u32 = 0xFFFFFFFF;

pub fn get_pixel_color(
    hwnd_opt: Option<HWND>,
    x: i32,
    y: i32,
) -> windows::core::Result<PixelColor> {
    unsafe {
        let mut point = POINT { x, y };
        if let Some(hwnd) = hwnd_opt
            && !ClientToScreen(hwnd, &mut point).as_bool()
        {
            return Err(Error::from(GetLastError()));
        }

        let hdc_screen = GetDC(None);
        if hdc_screen.0.is_null() {
            return Err(Error::from(GetLastError()));
        }

        let color = GetPixel(hdc_screen, point.x, point.y);

        if ReleaseDC(None, hdc_screen) == 0 {
            return Err(Error::from(GetLastError()));
        }

        if color.0 == CLR_INVALID {
            Err(Error::from(GetLastError()))
        } else {
            Ok(PixelColor(color.0))
        }
    }
}

pub fn debug_mouse_color() {
    unsafe {
        let mut pt = POINT::default();
        if GetCursorPos(&mut pt).is_err() {
            return;
        }
        let x = pt.x;
        let y = pt.y;
        if x < 0 || y < 0 {
            return;
        }
        print_color(None, x, y);
    }
}

fn print_color(hwnd: Option<HWND>, x: i32, y: i32) {
    match get_pixel_color(hwnd, x, y) {
        Ok(color) => {
            println!("Color: {}", color);
        }
        Err(e) => {
            println!("Failed to get color at [{}, {}]: {:?}", x, y, e);
        }
    }
}

pub fn debug_mouse(hwnd: HWND) {
    unsafe {
        let mut pt = POINT::default();
        if GetCursorPos(&mut pt).is_err() {
            return;
        }
        let abs_x = pt.x;
        let abs_y = pt.x;
        if !ScreenToClient(hwnd, &mut pt).as_bool() {
            return;
        }
        print!(
            "Mouse at: screen[{}, {}] window[{}, {}]\t",
            abs_x, abs_y, pt.x, pt.y
        );
    }
}

pub fn set_window(
    hwnd: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> windows::core::Result<()> {
    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOP),
            x,
            y,
            width,
            height,
            SWP_NOZORDER | SWP_SHOWWINDOW,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::win_util::PixelColor;

    #[test]
    fn test_pixel_color_display() {
        const COLOR: PixelColor = PixelColor(11189196);
        assert_eq!(COLOR.to_string(), "#AABBCC");
    }
}
