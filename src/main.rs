pub mod config;
mod simulation;
mod win_util;

use crate::config::{Args, Config, WindowConfig, load_config};
use crate::win_util::{
    debug_mouse, debug_mouse_color, enum_windows, find_window_by_title, focus_window, set_window,
};
use clap::Parser;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
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

    if args.debug_mouse {
        if cfg.windows.is_empty() {
            println!("No windows in configuration found");
            return Ok(());
        }
        if let Some(first_window) = cfg.windows.first()
            && let Some(window_title) = first_window.title.as_deref()
        {
            let hwnd_opt = find_window_by_title(window_title);
            println!("Window title: {} HWND: {:?}", window_title, hwnd_opt);
            match hwnd_opt {
                None => {
                    println!("Failed to get window handle for: {}", window_title);
                    return Ok(());
                }
                Some(hwnd) => loop {
                    debug_mouse(hwnd);
                    debug_mouse_color();
                    thread::sleep(Duration::from_millis(args.debug_interval_ms));
                },
            }
        }
        return Ok(());
    }

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

    let active_windows = cfg.windows.iter().filter(|x| x.active);
    active_windows.for_each(|win_config| {
        let mut hwnd_opt = match &win_config.title {
            Some(title) => find_window_by_title(title),
            _ => None,
        };
        if hwnd_opt.is_none() {
            hwnd_opt = win_config.hwnd
        }
        if let Some(hwnd) = hwnd_opt {
            set_window(
                hwnd,
                win_config.position_x,
                win_config.position_y,
                win_config.window_width,
                win_config.window_height,
            )
            .expect("Failed to set window position");
            if !focus_window(hwnd_opt).as_bool() {
                println!("Could not focus a window");
            }
        }
    });

    //////////////////
    let worker = spawn_window_worker(
        running_clone,
        enabled_clone,
        cfg.clone(),
        cfg.windows.first().unwrap(),
        || {},
    );

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

fn spawn_window_worker(
    running: Arc<AtomicBool>,
    enabled: Arc<AtomicBool>,
    config: Config,
    window_config: &WindowConfig,
    worker: fn() -> (),
) -> JoinHandle<()> {
    thread::spawn(move || {
        while running.load(Ordering::SeqCst) {
            if enabled.load(Ordering::SeqCst) {
                worker();
            }
            thread::sleep(Duration::from_millis(config.sync_interval_ms));
        }
    })
}
