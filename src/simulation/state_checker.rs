use crate::simulation::loot::LootQuality;
use crate::simulation::shared_state::CRITICAL_SECTION;
use crate::simulation::{CharState, DebugObj, WindowObj};
use crate::win_util::{PixelColor, focus_window, get_pixel_color_local};
use colored::Colorize;
use std::collections::HashMap;
use windows::Win32::Foundation::HWND;

const DEBUG_RUNE_COLOR: bool = false;
const DEBUG_LOCATION_COLOR: bool = false;
const COLOR_DISTANCE_TOLERANCE: u8 = 2;

pub trait StateChecker {
    fn get_state(&self, number_of_players: usize) -> CharState;
    fn get_loot_quality(&self) -> LootQuality;
    fn is_rune(&self) -> bool;
    fn is_inventory_full(&self) -> bool;
    fn is_inventory_opened(&self) -> bool;
    fn is_on_low_hp(&self, number_of_players: usize) -> bool;
}

impl StateChecker for DebugObj {
    fn get_state(&self, _: usize) -> CharState {
        let state = self.test_state;
        println!("State: {:?}", state);
        state
    }

    fn get_loot_quality(&self) -> LootQuality {
        let quality = LootQuality::Epic;
        println!("Loot quality: {:?}", quality);
        quality
    }

    fn is_rune(&self) -> bool {
        println!("Is rune: false");
        false
    }

    fn is_inventory_full(&self) -> bool {
        false
    }

    fn is_inventory_opened(&self) -> bool {
        true
    }

    fn is_on_low_hp(&self, _: usize) -> bool {
        false
    }
}

impl StateChecker for WindowObj {
    fn get_state(&self, number_of_players: usize) -> CharState {
        let mut state = CharState::Unknown;

        if let Some(s) = check_location(
            self.hwnd,
            get_loot_marker(),
            CharState::Looting,
            DEBUG_LOCATION_COLOR,
        ) {
            state = s;
        } else if let Some(s) = check_location(
            self.hwnd,
            get_town_marker(),
            CharState::InTown,
            DEBUG_LOCATION_COLOR,
        ) {
            state = s;
        } else if let Some(s) = check_location(
            self.hwnd,
            get_dead_marker(number_of_players),
            CharState::Dead,
            DEBUG_LOCATION_COLOR,
        ) {
            state = s;
        } else if let Some(s) = check_location(
            self.hwnd,
            get_shrine1_marker(),
            CharState::AtShrine,
            DEBUG_LOCATION_COLOR,
        ) {
            state = s;
        } else if let Some(s) = check_location(
            self.hwnd,
            get_shrine2_marker(),
            CharState::AtShrine,
            DEBUG_LOCATION_COLOR,
        ) {
            state = s;
        } else if let Some(s) = check_location(
            self.hwnd,
            get_shrine3_marker(),
            CharState::AtShrine,
            DEBUG_LOCATION_COLOR,
        ) {
            state = s;
        } else if let Some(s) = check_location(
            self.hwnd,
            get_shrine4_marker(),
            CharState::AtShrine,
            DEBUG_LOCATION_COLOR,
        ) {
            state = s;
        } else if let Some(s) = check_location(
            self.hwnd,
            get_shrine5_marker(),
            CharState::AtShrine,
            DEBUG_LOCATION_COLOR,
        ) {
            state = s;
        } else if let Some(s) = check_location(
            self.hwnd,
            get_dungeon_marker(),
            CharState::InDungeon,
            DEBUG_LOCATION_COLOR,
        ) {
            state = s;
        } else if let Some(s) = check_location(
            self.hwnd,
            get_fight_marker(),
            CharState::Fighting,
            DEBUG_LOCATION_COLOR,
        ) {
            state = s;
        }

        println!("State: {}", format!("{:?}", state).cyan());
        state
    }

    fn get_loot_quality(&self) -> LootQuality {
        let mut quality = LootQuality::Unknown;
        for (loc, q) in get_loot_quality_markers() {
            if let Some(q) = check_location(self.hwnd, loc, q, DEBUG_LOCATION_COLOR) {
                quality = q;
                break;
            }
        }
        if quality == LootQuality::Unknown {
            //debug print color
            _ = get_loot_quality_markers()
                .keys()
                .last()
                .cloned()
                .and_then(|loc| check_location(self.hwnd, loc, LootQuality::Unknown, true));
        }
        println!("Loot quality: {:?}", quality);
        quality
    }

    fn is_rune(&self) -> bool {
        let location_offset = -10..10;
        let locations: Vec<_> = location_offset
            .into_iter()
            .map(|offset| Location(650 + offset, 488, vec![PixelColor(0x0091CB)]))
            .collect();
        let _lock = CRITICAL_SECTION.lock().unwrap();
        _ = focus_window(self.hwnd).as_bool();
        let result = locations.iter().any(|loc| {
            check_location_no_focus(self.hwnd, loc.clone(), true, DEBUG_RUNE_COLOR).is_some()
        });
        drop(_lock);
        println!("Is rune: {:?}", result);
        result
    }

    fn is_inventory_full(&self) -> bool {
        let result = check_location(
            self.hwnd,
            get_inventory_full_marker(),
            true,
            DEBUG_LOCATION_COLOR,
        )
        .is_none();
        if result {
            println!("{}", "Inventory full".red());
        }
        result
    }

    fn is_inventory_opened(&self) -> bool {
        check_location(self.hwnd, get_inventory_opened_marker(), true, true).is_some()
    }

    fn is_on_low_hp(&self, number_of_players: usize) -> bool {
        check_location(
            self.hwnd,
            get_low_hp_marker(number_of_players),
            true,
            DEBUG_LOCATION_COLOR,
        )
        .is_some()
    }
}

fn check_location_no_focus<T>(
    hwnd: Option<HWND>,
    location: Location,
    result_state: T,
    debug_color: bool,
) -> Option<T> {
    if let Ok(color) = get_pixel_color_local(hwnd, location.0, location.1) {
        if debug_color {
            print!("Color: ");
            color.println();
        }
        if location
            .2
            .iter()
            .any(|c| c.is_similar_to(color, COLOR_DISTANCE_TOLERANCE))
        {
            return Some(result_state);
        }
    }
    None
}

fn check_location<T>(
    hwnd: Option<HWND>,
    location: Location,
    result_state: T,
    debug_color: bool,
) -> Option<T> {
    let _lock = CRITICAL_SECTION.lock().unwrap();
    _ = focus_window(hwnd).as_bool();
    let result = check_location_no_focus(hwnd, location, result_state, debug_color);
    drop(_lock);
    result
}

//x, y, vector of colors (or)
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
struct Location(i32, i32, Vec<PixelColor>);

fn get_town_marker() -> Location {
    Location(1218, 12, vec![PixelColor(0x2B99CE)])
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
            PixelColor(0x7A7A7A),
            PixelColor(0x5C6263),
            PixelColor(0x6D7677),
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
    Location(1232, 536, vec![PixelColor(0x4D2209), PixelColor(0x5E2D0E)])
}

fn get_dead_marker(number_of_players: usize) -> Location {
    let x = match number_of_players {
        1 => 597,
        2 => 550,
        3 => 503,
        4 => 456,
        5 => 409,
        _ => 597,
    };
    Location(x, 623, vec![PixelColor(0x313131)])
}

fn get_low_hp_marker(number_of_players: usize) -> Location {
    let marker = get_dead_marker(number_of_players);
    Location(
        marker.0 + 23,
        marker.1,
        vec![PixelColor(0x414141), PixelColor(0x434343)],
    )
}

fn get_inventory_full_marker() -> Location {
    Location(145, 422, vec![PixelColor(0x1B1B1B), PixelColor(0x070707)])
}

fn get_inventory_opened_marker() -> Location {
    Location(68, 473, vec![PixelColor(0x455D7D), PixelColor(0x45566C)])
}

fn get_loot_quality_markers() -> HashMap<Location, LootQuality> {
    let mut hm: HashMap<Location, LootQuality> = HashMap::new();
    let x = 485;
    let y = 506;
    hm.insert(
        Location(
            x,
            y,
            vec![
                PixelColor(0x4D4D74),
                PixelColor(0x777777),
                PixelColor(0x767676),
            ],
        ),
        LootQuality::Normal,
    );
    hm.insert(
        Location(
            x,
            y,
            vec![
                PixelColor(0x111138),
                PixelColor(0x1A1A1A),
                PixelColor(0x100F38),
            ],
        ),
        LootQuality::Socketed,
    );
    hm.insert(
        Location(
            x,
            y,
            vec![
                PixelColor(0xFF9A2A),
                PixelColor(0xA46342),
                PixelColor(0x8C5440),
                PixelColor(0xB97728),
                PixelColor(0xCA7A22),
                PixelColor(0xF59428),
                PixelColor(0xE18D2E),
            ],
        ),
        LootQuality::Magic,
    );
    hm.insert(
        Location(
            x,
            y,
            vec![
                PixelColor(0x00FFFF),
                PixelColor(0x00A4CB),
                PixelColor(0x079696),
                PixelColor(0x07FDFC),
            ],
        ),
        LootQuality::Rare,
    );
    hm.insert(
        Location(
            x,
            y,
            vec![
                PixelColor(0x00C400),
                PixelColor(0x026B2A),
                PixelColor(0x007E27),
                PixelColor(0x01CACA),
                PixelColor(0x27C91C),
            ],
        ),
        LootQuality::Set,
    );
    hm.insert(
        Location(
            x,
            y,
            vec![
                PixelColor(0x9F4396),
                PixelColor(0xF868AD),
                PixelColor(0xFF82D8),
                PixelColor(0xF97BAF),
                PixelColor(0xDB639D),
                PixelColor(0xF171AE),
            ],
        ),
        LootQuality::Epic,
    );
    hm.insert(
        Location(x, y, vec![PixelColor(0x0158BB)]),
        LootQuality::Legendary,
    );
    hm
}
