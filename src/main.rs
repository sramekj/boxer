pub mod config;
mod game_loop;
mod win_util;

use crate::config::{Args, load_config};
use crate::win_util::{enum_windows, find_window_by_title, focus_window, send_key_vk};
use clap::Parser;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_I;
use windows::{
    Win32::Foundation::HWND,
    Win32::UI::Input::KeyboardAndMouse::{
        MOD_NOREPEAT, RegisterHotKey, UnregisterHotKey, VK_DELETE, VK_ESCAPE,
    },
    Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_HOTKEY,
    },
};

fn main() -> windows::core::Result<()> {
    let args = Args::parse();
    if args.debug {
        println!("Window HWND listing:");
        enum_windows()?;
        return Ok(());
    }

    let cfg = load_config();

    println!("Configuration: {:?}", cfg);

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

    let worker = thread::spawn(move || {
        //debug....
        let xxx = find_window_by_title("[#] Nevergrind [#]");
        println!("HWND: {:?}", xxx);

        while running_clone.load(Ordering::SeqCst) {
            if enabled_clone.load(Ordering::SeqCst) {
                //TODO... just testing now

                println!("Chilling...");

                let _ = focus_window(xxx);
                send_key_vk(VK_I).expect("Failed to send key to window");
            }
            thread::sleep(Duration::from_millis(1000));
        }
    });

    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, hwnd, 0, 0).into() {
            if msg.message == WM_HOTKEY {
                let id = msg.wParam.0 as i32;
                match id {
                    HOTKEY_DEL_ID => {
                        let prev = enabled.fetch_xor(true, Ordering::SeqCst);
                        println!("Enabled: {}", !prev);
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
