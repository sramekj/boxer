use std::ffi::{OsStr, OsString};
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::mem::zeroed;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr::null_mut;
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::{
    ERROR_INVALID_WINDOW_HANDLE, GetLastError, HWND, LPARAM, POINT, RECT, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, GetPixel, HGDIOBJ, ReleaseDC,
    SRCCOPY, ScreenToClient, SelectObject,
};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, SetFocus, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowW, GetClientRect, GetCursorPos, GetForegroundWindow,
    GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    PostMessageW, SW_RESTORE, SetForegroundWindow, ShowWindow, WM_KEYDOWN, WM_KEYUP,
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

// does not work with games :((
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

pub fn get_pixel_color_blt(
    hwnd_opt: Option<HWND>,
    x: i32,
    y: i32,
) -> windows::core::Result<PixelColor> {
    unsafe {
        let hwnd = hwnd_opt.ok_or_else(|| Error::from(ERROR_INVALID_WINDOW_HANDLE))?;

        let hdc_window = GetDC(hwnd_opt);
        if hdc_window.0 == null_mut() {
            return Err(Error::from(GetLastError()));
        }

        let hdc_mem = CreateCompatibleDC(Some(hdc_window));
        if hdc_mem.0 == null_mut() {
            return Err(Error::from(GetLastError()));
        }

        // Get window size
        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect).map_err(|_| Error::from(GetLastError()))?;

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        let hbitmap = CreateCompatibleBitmap(hdc_window, width, height);
        let old_obj = SelectObject(hdc_mem, HGDIOBJ(hbitmap.0));

        //copy to mem device context
        BitBlt(
            hdc_mem,
            0,
            0,
            width,
            height,
            Some(hdc_window),
            0,
            0,
            SRCCOPY,
        )
        .map_err(|_| Error::from(GetLastError()))?;

        //get image data
        let mut bmi: BITMAPINFO = zeroed();
        bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = width;
        bmi.bmiHeader.biHeight = -height; // negative to indicate top-down DIB
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32; // We want BGRA (4 bytes per pixel)
        bmi.bmiHeader.biCompression = BI_RGB.0;

        let row_stride = (width * 4) as usize;
        let image_size = row_stride * (height as usize);
        let mut buffer = vec![0u8; image_size];

        let res = GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            height as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        if res == 0 {
            return Err(Error::from(GetLastError()));
        }

        //DEBUG: write raw bytes to file
        if cfg!(debug_assertions) {
            let mut f = File::create("capture.raw")?;
            f.write_all(&buffer)?;
        }

        // Calculate the pixel index
        let px = x.clamp(0, width - 1) as usize;
        let py = y.clamp(0, height - 1) as usize;

        let index = py * row_stride + px * 4;
        let blue = buffer[index];
        let green = buffer[index + 1];
        let red = buffer[index + 2];

        let result = rgb_to_colorref(red, green, blue);

        // Clean up
        SelectObject(hdc_mem, old_obj);
        if DeleteObject(HGDIOBJ(hbitmap.0)).as_bool() == false {
            return Err(Error::from(GetLastError()));
        }
        if DeleteDC(hdc_mem).as_bool() == false {
            return Err(Error::from(GetLastError()));
        }
        if ReleaseDC(hwnd_opt, hdc_window) == 0 {
            return Err(Error::from(GetLastError()));
        }

        if result == CLR_INVALID {
            Err(Error::from(GetLastError()))
        } else {
            Ok(PixelColor(result))
        }
    }
}

pub fn get_pixel_color(
    hwnd_opt: Option<HWND>,
    x: i32,
    y: i32,
) -> windows::core::Result<PixelColor> {
    unsafe {
        match hwnd_opt {
            Some(_) => {
                let hdc_window = GetDC(hwnd_opt);
                if hdc_window.0 == null_mut() {
                    return Err(Error::from(GetLastError()));
                }

                let color = GetPixel(hdc_window, x, y);
                let result = color.0;

                if ReleaseDC(hwnd_opt, hdc_window) == 0 {
                    return Err(Error::from(GetLastError()));
                }

                if result == CLR_INVALID {
                    Err(Error::from(GetLastError()))
                } else {
                    Ok(PixelColor(result))
                }
            }
            None => {
                let hdc_screen = GetDC(None);
                if hdc_screen.0 == null_mut() {
                    return Err(Error::from(GetLastError()));
                }

                let color = GetPixel(hdc_screen, x, y);
                let result = color.0;

                if ReleaseDC(None, hdc_screen) == 0 {
                    return Err(Error::from(GetLastError()));
                }

                if result == CLR_INVALID {
                    Err(Error::from(GetLastError()))
                } else {
                    Ok(PixelColor(result))
                }
            }
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

fn rgb_to_colorref(red: u8, green: u8, blue: u8) -> u32 {
    (red as u32) | ((green as u32) << 8) | ((blue as u32) << 16)
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

#[cfg(test)]
mod tests {
    use crate::win_util::PixelColor;

    #[test]
    fn test_pixel_color_display() {
        const COLOR: PixelColor = PixelColor(11189196);
        assert_eq!(COLOR.to_string(), "#AABBCC");
    }
}
