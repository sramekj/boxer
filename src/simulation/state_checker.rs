use crate::simulation::{CharState, DebugObj, WindowObj};
use crate::win_util::{PixelColor, focus_window, get_pixel_color_local};
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

        if let Some(s) = check_location(self.hwnd, get_town_marker(), CharState::InTown) {
            state = s;
        } else if let Some(s) = check_location(self.hwnd, get_loot_marker(), CharState::Looting) {
            state = s;
        } else if let Some(s) = check_location(self.hwnd, get_shrine1_marker(), CharState::AtShrine)
        {
            state = s;
        } else if let Some(s) = check_location(self.hwnd, get_shrine2_marker(), CharState::AtShrine)
        {
            state = s;
        } else if let Some(s) = check_location(self.hwnd, get_shrine3_marker(), CharState::AtShrine)
        {
            state = s;
        } else if let Some(s) = check_location(self.hwnd, get_shrine4_marker(), CharState::AtShrine)
        {
            state = s;
        } else if let Some(s) = check_location(self.hwnd, get_shrine5_marker(), CharState::AtShrine)
        {
            state = s;
        } else if let Some(s) =
            check_location(self.hwnd, get_dungeon_marker(), CharState::InDungeon)
        {
            state = s;
        } else if let Some(s) = check_location(self.hwnd, get_fight_marker(), CharState::Fighting) {
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
    _ = focus_window(hwnd).as_bool();
    if let Ok(color) = get_pixel_color_local(hwnd, location.0, location.1) {
        println!("Found color: {}", color);
        if location.2.iter().any(|c| *c == color) {
            return Some(result_state);
        }
    }
    None
}

//x, y, vector of colors (or)
struct Location(i32, i32, Vec<PixelColor>);

fn get_town_marker() -> Location {
    Location(1127, 11, vec![PixelColor(0x00D5FE)])
}

fn get_dungeon_marker() -> Location {
    Location(948, 304, vec![PixelColor(0xF0F66C), PixelColor(0xA0A448)])
}

fn get_shrine1_marker() -> Location {
    Location(
        635,
        246,
        vec![
            PixelColor(0xF0F0F0),
            PixelColor(0xFFFFFF),
            PixelColor(0x666666),
        ],
    )
}

fn get_shrine2_marker() -> Location {
    Location(669, 252, vec![PixelColor(0x99B9BE), PixelColor(0xB0DEE5)])
}

fn get_shrine3_marker() -> Location {
    Location(620, 402, vec![PixelColor(0x6F6360), PixelColor(0x887772)])
}

fn get_shrine4_marker() -> Location {
    Location(626, 452, vec![PixelColor(0xE2D2C3), PixelColor(0xFFFCE7)])
}

fn get_shrine5_marker() -> Location {
    Location(591, 471, vec![PixelColor(0x091E4F), PixelColor(0x042268)])
}

fn get_loot_marker() -> Location {
    Location(446, 507, vec![PixelColor(0x6E969A), PixelColor(0x85D3DB)])
}

fn get_fight_marker() -> Location {
    Location(1231, 598, vec![PixelColor(0x4D2209)])
}
