pub mod config;
mod simulation;
mod win_util;

use crate::config::{Args, load_config};
use crate::simulation::rotation::Rotations;
use crate::simulation::shared_state::SharedState;
use crate::simulation::{CharState, DebugObj, Rotation, SimulationState, WindowObj};
use crate::win_util::{
    debug_mouse, debug_mouse_color, enum_windows, find_window_by_title, set_window,
};
use clap::Parser;
use std::sync::{Arc, Mutex};
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

    const HOTKEY_DEL_ID: i32 = 1;
    const HOTKEY_ESC_ID: i32 = 2;

    // hWnd = HWND(0) => message delivered to thread message queue
    let hwnd_screen = Some(HWND::default());

    unsafe {
        RegisterHotKey(hwnd_screen, HOTKEY_DEL_ID, MOD_NOREPEAT, VK_DELETE.0 as u32)?;
        RegisterHotKey(hwnd_screen, HOTKEY_ESC_ID, MOD_NOREPEAT, VK_ESCAPE.0 as u32)?;
    }

    println!("Hotkey registered: DELETE. Press DELETE to toggle. ESC or Ctrl+C to exit.");

    let mut handles: Vec<JoinHandle<()>> = vec![];
    let mut simulations: Vec<Arc<SimulationState>> = vec![];
    let shared_state = Arc::new(Mutex::new(SharedState::new(cfg.skill_haste_percent)));

    let active_windows = cfg
        .windows
        .iter()
        .filter(|x| x.active)
        .cloned()
        .collect::<Vec<_>>();
    for active_window in active_windows {
        let mut hwnd_opt = match &active_window.title {
            Some(title) => find_window_by_title(title),
            _ => None,
        };
        if hwnd_opt.is_none() {
            hwnd_opt = active_window.hwnd
        }
        if let Some(hwnd) = hwnd_opt {
            set_window(
                hwnd,
                active_window.position_x,
                active_window.position_y,
                active_window.window_width,
                active_window.window_height,
            )
            .expect("Failed to set window position");
        }

        let rotation = Rotation::get_rotation(active_window.class, &cfg);

        let simulation = if args.debug_sim {
            Arc::new(SimulationState::new(
                cfg.sync_interval_ms,
                cfg.cast_leeway_ms,
                active_window,
                rotation,
                Box::new(DebugObj::new(CharState::Fighting)),
                Box::new(DebugObj::new(CharState::Fighting)),
                shared_state.clone(),
            ))
        } else {
            Arc::new(SimulationState::new(
                cfg.sync_interval_ms,
                cfg.cast_leeway_ms,
                active_window,
                rotation,
                Box::new(WindowObj::new(hwnd_opt)),
                Box::new(WindowObj::new(hwnd_opt)),
                shared_state.clone(),
            ))
        };

        let handle = thread::spawn({
            let sim = Arc::clone(&simulation);
            simulations.push(sim.clone());
            move || {
                if args.debug_checker {
                    sim.debug_checker();
                } else {
                    sim.run();
                }
            }
        });
        handles.push(handle);
    }

    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, hwnd_screen, 0, 0).into() {
            if msg.message == WM_HOTKEY {
                let id = msg.wParam.0 as i32;
                match id {
                    HOTKEY_DEL_ID => {
                        simulations.iter().for_each(|sim| {
                            sim.enable_toggle();
                            thread::sleep(Duration::from_millis(cfg.start_offset_ms));
                        });
                    }
                    HOTKEY_ESC_ID => {
                        simulations.iter().for_each(|sim| {
                            sim.stop();
                        });
                        println!("Quitting application...");
                        break;
                    }
                    _ => {}
                }
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnregisterHotKey(hwnd_screen, HOTKEY_DEL_ID)?;
        UnregisterHotKey(hwnd_screen, HOTKEY_ESC_ID)?;
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    Ok(())
}
