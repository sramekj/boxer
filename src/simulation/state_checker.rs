use crate::simulation::char_state::CharState;
use crate::simulation::global_lock::CRITICAL_SECTION;
use crate::simulation::loot::{LootQuality, LootTier};
use crate::simulation::simulation_state::{DebugObj, WindowObj};
use crate::win_util::{PixelColor, debug_screen, focus_window, get_pixel_color_local, scan_line};
use colored::Colorize;
use std::collections::HashMap;
use windows::Win32::Foundation::HWND;

const DEBUG_LOCATION_COLOR: bool = false;
const DEBUG_BMP: bool = false;
const COLOR_DISTANCE_TOLERANCE: u8 = 2;

pub trait StateChecker {
    fn get_state(&self, number_of_players: usize) -> CharState;
    fn get_loot_quality(&self) -> LootQuality;
    fn get_loot_tier(&self) -> LootTier;
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

    fn get_loot_tier(&self) -> LootTier {
        let tier = LootTier::Normal;
        println!("Loot tier: {:?}", tier);
        tier
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
        for (ll, q) in get_loot_line_locations() {
            if check_line(self.hwnd, ll, DEBUG_LOCATION_COLOR, DEBUG_BMP) {
                quality = q;
                break;
            }
        }
        if quality == LootQuality::Unknown {
            //debug print color
            _ = get_loot_line_locations()
                .keys()
                .last()
                .cloned()
                .map(|loc| check_line(self.hwnd, loc, true, true));
        }
        println!("Loot quality: {:?}", quality);
        quality
    }

    fn get_loot_tier(&self) -> LootTier {
        let mut tier = LootTier::Unknown;
        for (loc, t) in get_tier_markers() {
            if let Some(_tier) = check_location(self.hwnd, loc, t, DEBUG_LOCATION_COLOR) {
                tier = _tier;
                break;
            }
        }

        if tier == LootTier::Unknown {
            //debug print color
            _ = get_tier_markers()
                .keys()
                .last()
                .cloned()
                .map(|loc| check_location(self.hwnd, loc, LootTier::Unknown, true));
            //and flush bmp
            _ = debug_screen(self.hwnd, "loot_tier.bmp");
        }
        println!("Loot tier: {:?}", tier);
        tier
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
        check_location(
            self.hwnd,
            get_inventory_opened_marker(),
            true,
            DEBUG_LOCATION_COLOR,
        )
        .is_some()
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

fn check_line(
    hwnd: Option<HWND>,
    location: LineLocation,
    debug_color: bool,
    debug_bmp: bool,
) -> bool {
    let _lock = CRITICAL_SECTION.lock().unwrap();
    _ = focus_window(hwnd).as_bool();
    let result = check_line_no_focus(hwnd, location, debug_color, debug_bmp);
    drop(_lock);
    result
}

fn check_line_no_focus(
    hwnd: Option<HWND>,
    location: LineLocation,
    debug_color: bool,
    debug_bmp: bool,
) -> bool {
    let colors_to_find = location.3;
    if let Ok(line) = scan_line(hwnd, location.0, location.1, location.2, debug_bmp) {
        let found = colors_to_find.iter().any(|color| {
            line.iter()
                .any(|l| l.is_similar_to(*color, COLOR_DISTANCE_TOLERANCE))
        });
        if !found && debug_color {
            print!("Colors: ");
            for color in line {
                color.print();
                print!(" ");
            }
            println!();
        }
        return found;
    }
    false
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

//x1, x2, y, vector of colors (or)
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
struct LineLocation(i32, i32, i32, Vec<PixelColor>);

fn get_tier_markers() -> HashMap<Location, LootTier> {
    let mut hm: HashMap<Location, LootTier> = HashMap::new();
    let x = 488;
    let y = 475;
    hm.insert(Location(x, y, vec![PixelColor(0x3A75EC)]), LootTier::Elite);
    hm.insert(
        Location(x, y, vec![PixelColor(0x70A1B5), PixelColor(0x8F94B3)]),
        LootTier::Exceptional,
    );
    hm.insert(
        Location(x, y, vec![PixelColor(0x131215), PixelColor(0x1C303A)]),
        LootTier::Normal,
    );
    hm
}

fn get_loot_line_locations() -> HashMap<LineLocation, LootQuality> {
    let mut hm: HashMap<LineLocation, LootQuality> = HashMap::new();
    let x1 = 585;
    let x2 = 722;
    let y = 497;
    hm.insert(
        LineLocation(
            x1,
            x2,
            490,
            vec![
                PixelColor(0xFFFFFF),
                PixelColor(0xFCFCFC),
                PixelColor(0xE6E6E6),
                PixelColor(0xE4E4E4),
                PixelColor(0xF2F2F2),
            ],
        ),
        LootQuality::Normal,
    );
    hm.insert(
        LineLocation(
            x1,
            x2,
            490,
            vec![
                PixelColor(0x706F6F),
                PixelColor(0x676767),
                PixelColor(0x686868),
            ],
        ),
        LootQuality::Socketed,
    );
    hm.insert(
        LineLocation(x1, x2, y, vec![PixelColor(0xD07E22)]),
        LootQuality::Magic,
    );
    hm.insert(
        LineLocation(x1, x2, y, vec![PixelColor(0x00E4E4)]),
        LootQuality::Rare,
    );
    hm.insert(
        LineLocation(
            x1,
            x2,
            y,
            vec![
                PixelColor(0xE35F9E),
                PixelColor(0xCB558E),
                PixelColor(0xC05185),
            ],
        ),
        LootQuality::Epic,
    );
    hm.insert(
        LineLocation(x1, x2, y, vec![PixelColor(0x00B200)]),
        LootQuality::Set,
    );
    hm.insert(
        LineLocation(x1, x2, y, vec![PixelColor(0x0158BB)]),
        LootQuality::Legendary,
    );
    hm.insert(
        LineLocation(
            600,
            800,
            488,
            vec![
                PixelColor(0x0158BB),
                PixelColor(0x0091CB),
                PixelColor(0x047099),
            ],
        ),
        LootQuality::Rune,
    );
    hm
}

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
