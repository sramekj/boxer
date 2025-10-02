use crate::simulation::{CharState, DebugObj, WindowObj};
use crate::win_util::{PixelColor, focus_window, get_pixel_color};
use windows::Win32::Foundation::HWND;

pub trait StateChecker {
    fn get_state(&self) -> CharState;
}

impl StateChecker for DebugObj {
    fn get_state(&self) -> CharState {
        println!("Getting state");
        let state = self.test_state;
        println!("New state:  {:?}", state);
        state
    }
}

impl StateChecker for WindowObj {
    fn get_state(&self) -> CharState {
        println!("Getting state");
        let mut state = CharState::Unknown;
        _ = focus_window(self.hwnd).as_bool();

        if let Some(s) = check_location(self.hwnd, TOWN_MARKER, CharState::InTown) {
            state = s;
        }
        if let Some(s) = check_location(self.hwnd, DUNGEON_MARKER, CharState::InDungeon) {
            state = s;
        }

        println!("New state: {:?}", state);
        state
    }
}

fn check_location(
    hwnd: Option<HWND>,
    location: Location,
    result_state: CharState,
) -> Option<CharState> {
    if let Ok(color) = get_pixel_color(hwnd, location.0, location.1) {
        println!("Found color: {}", color);
        if color == location.2 {
            return Some(result_state);
        }
    }
    None
}

struct Location(i32, i32, PixelColor);

const TOWN_MARKER: Location = Location(1127, 10, PixelColor(0x03CDF4));
const DUNGEON_MARKER: Location = Location(937, 303, PixelColor(0xFFFFFF));
