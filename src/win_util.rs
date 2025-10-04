use crate::simulation::keys::Key;
use colored::*;
use std::ffi::{OsStr, OsString};
use std::fmt::Display;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr::null_mut;
use windows::Win32::Foundation::{COLORREF, GetLastError, HWND, LPARAM, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    ClientToScreen, CreatePen, DeleteObject, GetDC, GetPixel, GetStockObject, HGDIOBJ, NULL_BRUSH,
    PS_SOLID, Rectangle, ReleaseDC, ScreenToClient, SelectObject, SetPixel,
};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::HiDpi::{PROCESS_PER_MONITOR_DPI_AWARE, SetProcessDpiAwareness};
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

pub fn send_key_vk(key: Key) -> windows::core::Result<()> {
    unsafe {
        let inputs = [make_input(key.into(), false), make_input(key.into(), true)];
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct PixelColor(pub u32);

impl Display for PixelColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:06X}", self.0 & 0xFFFFFF)
    }
}

impl PixelColor {
    fn r(&self) -> u8 {
        (self.0 & 0x000000FF) as u8
    }
    fn g(&self) -> u8 {
        ((self.0 & 0x0000FF00) >> 8) as u8
    }
    fn b(&self) -> u8 {
        ((self.0 & 0x00FF0000) >> 16) as u8
    }

    fn rgb(&self) -> (u8, u8, u8) {
        (self.r(), self.g(), self.b())
    }

    pub fn println(&self) {
        println!(
            "{}",
            format!("{}", self).truecolor(self.r(), self.g(), self.b())
        );
    }

    pub fn print(&self) {
        print!(
            "{}",
            format!("{}", self).truecolor(self.r(), self.g(), self.b())
        );
    }

    pub fn is_similar_to(&self, other: PixelColor, tolerance: u8) -> bool {
        let (r1, g1, b1) = self.rgb();
        let (r2, g2, b2) = other.rgb();
        let dr = r1 as i16 - r2 as i16;
        let dg = g1 as i16 - g2 as i16;
        let db = b1 as i16 - b2 as i16;
        let distance_squared = dr * dr + dg * dg + db * db;
        distance_squared <= (tolerance as i16).pow(2)
    }
}

const CLR_INVALID: u32 = 0xFFFFFFFF;

pub fn get_pixel_color_screen(x: i32, y: i32) -> windows::core::Result<PixelColor> {
    unsafe {
        let hdc_screen = GetDC(None);
        if hdc_screen.0.is_null() {
            return Err(Error::from(GetLastError()));
        }

        let color = GetPixel(hdc_screen, x, y);

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

pub fn get_pixel_color_local(
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

const DEBUG_RECTANGLE: bool = false;
const DEBUG_DOT: bool = false;

pub fn debug_mouse_color(_hwnd: HWND) {
    unsafe {
        let mut pt = POINT::default();
        if GetCursorPos(&mut pt).is_err() {
            return;
        }
        match get_pixel_color_screen(pt.x, pt.y) {
            Ok(color) => {
                print!("Color: ");
                color.println()
            }
            Err(e) => {
                eprintln!("Failed to get color at [{}, {}]: {:?}", pt.x, pt.y, e);
            }
        }

        if DEBUG_RECTANGLE && debug_rectangle(pt.x - 5, pt.y - 5, pt.x + 5, pt.y + 5).is_err() {
            eprintln!("Failed to draw a rectangle");
        }

        if DEBUG_DOT && debug_dot(pt.x, pt.y).is_err() {
            eprintln!("Failed to draw a dot");
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
        let abs_y = pt.y;
        if !ScreenToClient(hwnd, &mut pt).as_bool() {
            return;
        }
        print!(
            "Mouse at: screen[{}, {}] window[{}, {}]\t",
            abs_x, abs_y, pt.x, pt.y
        );
    }
}

pub fn debug_dot(x: i32, y: i32) -> windows::core::Result<()> {
    unsafe {
        let hwnd: Option<HWND> = None;
        let hdc = GetDC(hwnd);
        if hdc.0.is_null() {
            return Err(Error::from(GetLastError()));
        }

        SetPixel(hdc, x, y, COLORREF(0x0000FF));

        ReleaseDC(hwnd, hdc);

        Ok(())
    }
}

pub fn debug_rectangle(left: i32, top: i32, right: i32, bottom: i32) -> windows::core::Result<()> {
    unsafe {
        let hwnd: Option<HWND> = None;
        let hdc = GetDC(hwnd);
        if hdc.0.is_null() {
            return Err(Error::from(GetLastError()));
        }

        // Create a red pen (for border)
        let hpen = CreatePen(PS_SOLID, 2, COLORREF(0x0000FF));
        let old_pen = SelectObject(hdc, HGDIOBJ(hpen.0));

        // Optional: Create a null brush (no fill)
        let hollow_brush = GetStockObject(NULL_BRUSH);
        let old_brush = SelectObject(hdc, hollow_brush);

        let success = Rectangle(hdc, left, top, right, bottom);
        if !success.as_bool() {
            return Err(Error::from(GetLastError()));
        }

        SelectObject(hdc, old_pen);
        SelectObject(hdc, old_brush);
        if !DeleteObject(HGDIOBJ(hpen.0)).as_bool() {
            eprintln!("Could not delete pen object");
        }
        ReleaseDC(hwnd, hdc);

        Ok(())
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

pub fn make_dpi_aware() -> windows::core::Result<()> {
    unsafe { SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE) }
}

#[cfg(test)]
mod tests {
    use crate::win_util::PixelColor;

    #[test]
    fn test_pixel_color_display() {
        const COLOR: PixelColor = PixelColor(11189196);
        assert_eq!(COLOR.to_string(), "#AABBCC");
        assert_eq!(COLOR.b(), 0xAA);
        assert_eq!(COLOR.g(), 0xBB);
        assert_eq!(COLOR.r(), 0xCC);
        assert_eq!(COLOR.rgb(), (0xCC, 0xBB, 0xAA));
    }

    #[test]
    fn test_color_distance() {
        let c1 = PixelColor(0x00C400);
        let c2 = PixelColor(0x00CB00);
        let tolerance = 20;
        assert!(c1.is_similar_to(c2, tolerance));
    }
}
