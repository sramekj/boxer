use clap::Parser;
use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY, VK_A,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowW, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, PostMessageW,
    WM_KEYDOWN, WM_KEYUP,
};
use windows::core::{Error, PCWSTR};
use windows::{
    Win32::Foundation::{HWND, LPARAM, WPARAM},
    Win32::UI::Input::KeyboardAndMouse::{
        MOD_NOREPEAT, RegisterHotKey, UnregisterHotKey, VK_DELETE, VK_ESCAPE,
    },
    Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_HOTKEY,
    },
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    debug: bool,
}

fn main() -> windows::core::Result<()> {
    let args = Args::parse();
    if args.debug {
        println!("Window HWND listing:");
        enum_windows()?;
        return Ok(());
    }

    // Create shared toggle
    let enabled = Arc::new(AtomicBool::new(false));
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let enabled_clone = enabled.clone();

    const HOTKEY_DEL_ID: i32 = 1;
    const HOTKEY_ESC_ID: i32 = 2;

    // hWnd = HWND(0) => message delivered to thread message queue
    let hwnd = Some(HWND::default());

    unsafe {
        RegisterHotKey(hwnd, HOTKEY_DEL_ID, MOD_NOREPEAT, VK_DELETE.0 as u32)?;
        RegisterHotKey(hwnd, HOTKEY_ESC_ID, MOD_NOREPEAT, VK_ESCAPE.0 as u32)?;
    }

    println!("Hotkey registered: DELETE. Press DELETE to toggle. ESC or Ctrl+C to exit.");

    // Spawn worker thread that sends 'A' when enabled
    let worker = thread::spawn(move || {
        let xxx = find_window_by_title("Untitled - Notepad");
        println!("HWND: {:?}", xxx);

        while running_clone.load(Ordering::SeqCst) {
            if enabled_clone.load(Ordering::SeqCst) {
                println!("Chilling...");
                //send_key_to_window(xxx, VK_A).expect("Failed to send key to window");
                send_key_vk(VK_A).expect("Failed to send key to window");
            }
            thread::sleep(Duration::from_millis(500));
        }
    });

    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, hwnd, 0, 0).into() {
            if msg.message == WM_HOTKEY {
                let id = msg.wParam.0 as i32;
                match id {
                    HOTKEY_DEL_ID => {
                        // toggle
                        let prev = enabled.fetch_xor(true, Ordering::SeqCst);
                        println!("Enabled: {}", !prev);
                        // You could spawn/stop worker threads here based on enabled
                    }
                    HOTKEY_ESC_ID => {
                        println!("Quitting...");
                        enabled.store(false, Ordering::SeqCst);
                        running.store(false, Ordering::SeqCst);
                        break;
                    }
                    _ => {}
                }
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnregisterHotKey(hwnd, HOTKEY_DEL_ID)?;
        UnregisterHotKey(hwnd, HOTKEY_ESC_ID)?;
    }

    let _ = worker.join();

    Ok(())
}

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

fn send_key_vk(vk: VIRTUAL_KEY) -> windows::core::Result<()> {
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

fn send_key_to_window(hwnd: Option<HWND>, vk: VIRTUAL_KEY) -> windows::core::Result<()> {
    unsafe {
        let vk = vk.0 as usize;
        if let Err(_) = PostMessageW(hwnd, WM_KEYDOWN, WPARAM(vk), LPARAM(0)) {
            return Err(Error::from(GetLastError()));
        }
        if let Err(_) = PostMessageW(hwnd, WM_KEYUP, WPARAM(vk), LPARAM(0)) {
            return Err(Error::from(GetLastError()));
        }
        Ok(())
    }
}

fn find_window_by_title(title: &str) -> Option<HWND> {
    unsafe {
        let wide: Vec<u16> = OsStr::new(title)
            .encode_wide()
            .chain(std::iter::once(0)) // null terminator
            .collect();
        let hwnd = FindWindowW(None, PCWSTR::from_raw(wide.as_ptr()));
        hwnd.ok()
    }
}

fn enum_windows() -> windows::core::Result<()> {
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
