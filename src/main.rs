use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use windows::{
    Win32::Foundation::{HWND, LRESULT, WPARAM, LPARAM},
    Win32::UI::Input::KeyboardAndMouse::{MOD_NOREPEAT, RegisterHotKey, UnregisterHotKey},
    Win32::UI::WindowsAndMessaging::{
        GetMessageW, TranslateMessage, DispatchMessageW, MSG, WM_HOTKEY,
    },
};



fn main() -> windows::core::Result<()> {
    // Create shared toggle
    let enabled = Arc::new(AtomicBool::new(false));
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let enabled_clone = enabled.clone();

    const HOTKEY_DEL_ID: i32 = 1;
    const HOTKEY_ESC_ID: i32 = 2;

    let vk_delete: u32 = 0x2E;
    let vk_esc: u32 = 0x1B;

    // hWnd = HWND(0) => message delivered to thread message queue
    let hwnd = Some(HWND::default());

    unsafe {
        RegisterHotKey(hwnd, HOTKEY_DEL_ID, MOD_NOREPEAT, vk_delete)?;
        RegisterHotKey(hwnd, HOTKEY_ESC_ID, MOD_NOREPEAT, vk_esc)?;
    }

    println!("Hotkey registered: DELETE. Press DELETE to toggle. ESC or Ctrl+C to exit.");

    // Spawn worker thread that sends 'A' when enabled
    let worker = thread::spawn(move || {
        while running_clone.load(Ordering::SeqCst) {
            if enabled_clone.load(Ordering::SeqCst) {
                println!("Chilling...")
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
                    },
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
