use crate::configuration::class_config::AutoAttack;
use crate::simulation::global_lock::CRITICAL_SECTION;
use crate::simulation::keys::{
    AUTO_ATTACK, AUTO_RANGED_ATTACK, AUTO_WALK, DISCARD, HEALTH_POT, INVENTORY, Key, LOOT_INTERACT,
};
use crate::simulation::maze_solver::Direction;
use crate::simulation::simulation_state::{DebugObj, WindowObj};
use crate::simulation::skill::Skill;
use crate::simulation::state_checker::get_move_pixel;
use crate::win_util::{focus_window, send_key_vk, set_mouse};
use crate::with_critical_section;
use colored::Colorize;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

pub trait Interactor {
    fn cast_skill(&self, skill: &Skill) -> bool;
    fn loot(&self) -> bool;
    fn interact(&self) -> bool;
    fn discard(&self) -> bool;
    fn target_player(&self, player_index: usize) -> bool;
    fn auto_attack(&self, auto_attack: AutoAttack) -> bool;
    fn use_hp_pot(&self) -> bool;
    fn inventory_toggle(&self) -> bool;
    fn leave_to_town(&self) -> bool;
    fn try_direction(&self, direction: Direction) -> bool;
    fn walk(&self, direction: Option<Direction>, walk_duration_ms: u64) -> bool;
}

impl Interactor for DebugObj {
    fn cast_skill(&self, skill: &Skill) -> bool {
        print!("Casting ");
        print!("{}", format!("'{}'", skill.name).bright_magenta());
        true
    }

    fn loot(&self) -> bool {
        println!("{}", "Looting an item".green());
        true
    }

    fn interact(&self) -> bool {
        println!("{}", "Interacting".green());
        true
    }

    fn discard(&self) -> bool {
        println!("{}", "Discarding an item".red());
        true
    }

    fn target_player(&self, player_index: usize) -> bool {
        println!("Targeting player {}", player_index + 1);
        true
    }

    fn auto_attack(&self, auto_attack: AutoAttack) -> bool {
        println!("{}", format!("Auto-attacking {:?}", auto_attack).magenta());
        true
    }

    fn use_hp_pot(&self) -> bool {
        println!("{}", "Using a HP potion".red());
        true
    }

    fn inventory_toggle(&self) -> bool {
        println!("{}", "Toggling an inventory".bright_purple());
        true
    }

    fn leave_to_town(&self) -> bool {
        println!("{}", "Leaving to town".red());
        true
    }

    fn try_direction(&self, direction: Direction) -> bool {
        let position = (
            self.position_x.load(Ordering::SeqCst),
            self.position_y.load(Ordering::SeqCst),
        );
        print!(
            "{}",
            format!("At {:?} and trying to go: {:?}. ", position, direction).bright_yellow()
        );
        let map = self.test_map.clone();
        let map = map.lock().unwrap();
        let result = map
            .get(&position)
            .is_some_and(|node| node.neighbors.contains_key(&direction));
        println!(
            "{}: {}",
            format!("Can go {:?}", direction).white(),
            if result {
                format!("{:?}", result).green()
            } else {
                format!("{:?}", result).red()
            }
        );
        result
    }

    fn walk(&self, direction: Option<Direction>, _: u64) -> bool {
        println!(
            "{}",
            format!(
                "Walking {}...",
                direction.map_or_else(|| "".to_string(), |d| d.to_string())
            )
            .bright_yellow()
        );
        let delta = direction.unwrap().delta();
        self.position_x.store(
            self.position_x.load(Ordering::SeqCst) + delta.0,
            Ordering::SeqCst,
        );
        self.position_y.store(
            self.position_y.load(Ordering::SeqCst) + delta.1,
            Ordering::SeqCst,
        );
        let new_position = (
            self.position_x.load(Ordering::SeqCst),
            self.position_y.load(Ordering::SeqCst),
        );
        println!("New position: {:?}", new_position);
        let map = self.test_map.clone();
        let mut map = map.lock().unwrap();
        if let Some(new_pos_node) = map.get(&new_position) {
            let visited = new_pos_node.make_visited();
            map.insert(new_position, visited);
        }
        true
    }
}

const WAIT_TO_REGISTER_MS: u64 = 200;
impl Interactor for WindowObj {
    fn cast_skill(&self, skill: &Skill) -> bool {
        print!("Casting ");
        print!("{}", format!("'{}'", skill.name).bright_magenta());
        with_critical_section!(WAIT_TO_REGISTER_MS, {
            focus_window(self.hwnd).as_bool() && send_key_vk(skill.key).is_ok()
        })
    }

    fn loot(&self) -> bool {
        println!("{}", "Looting an item".green());
        with_critical_section!(WAIT_TO_REGISTER_MS, {
            focus_window(self.hwnd).as_bool() && send_key_vk(LOOT_INTERACT).is_ok()
        })
    }

    fn interact(&self) -> bool {
        println!("{}", "Interacting".green());
        with_critical_section!(WAIT_TO_REGISTER_MS, {
            focus_window(self.hwnd).as_bool() && send_key_vk(LOOT_INTERACT).is_ok()
        })
    }

    fn discard(&self) -> bool {
        println!("{}", "Discarding an item".red());
        with_critical_section!(WAIT_TO_REGISTER_MS, {
            focus_window(self.hwnd).as_bool() && send_key_vk(DISCARD).is_ok()
        })
    }

    fn target_player(&self, player_index: usize) -> bool {
        println!("Targeting player {}", player_index + 1);
        if let Some(key) = Key::get_party_keys().get(player_index) {
            with_critical_section!(WAIT_TO_REGISTER_MS, {
                focus_window(self.hwnd).as_bool() && send_key_vk(*key).is_ok()
            })
        } else {
            false
        }
    }

    fn auto_attack(&self, auto_attack: AutoAttack) -> bool {
        println!("{}", format!("Auto-attacking {:?}", auto_attack).magenta());
        with_critical_section!(WAIT_TO_REGISTER_MS, {
            let key = match auto_attack {
                AutoAttack::Primary => AUTO_ATTACK,
                AutoAttack::Ranged => AUTO_RANGED_ATTACK,
            };
            focus_window(self.hwnd).as_bool() && send_key_vk(key).is_ok()
        })
    }

    fn use_hp_pot(&self) -> bool {
        println!("{}", "Using a HP potion".red());
        with_critical_section!(WAIT_TO_REGISTER_MS, {
            focus_window(self.hwnd).as_bool() && send_key_vk(HEALTH_POT).is_ok()
        })
    }

    fn inventory_toggle(&self) -> bool {
        println!("{}", "Toggling an inventory".bright_purple());
        with_critical_section!(WAIT_TO_REGISTER_MS, {
            focus_window(self.hwnd).as_bool() && send_key_vk(INVENTORY).is_ok()
        })
    }

    fn leave_to_town(&self) -> bool {
        println!("{}", "Leaving to town".red());
        let _lock = CRITICAL_SECTION.lock().unwrap();
        let result1 = focus_window(self.hwnd).as_bool() && set_mouse(self.hwnd, 948, 304, true);
        thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
        let result2 = focus_window(self.hwnd).as_bool() && set_mouse(self.hwnd, 1022, 432, true);
        thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
        drop(_lock);
        result1 && result2
    }

    fn try_direction(&self, direction: Direction) -> bool {
        print!(
            "{}",
            format!("Trying to go: {:?}. ", direction).bright_yellow()
        );

        let px_before = get_move_pixel(self.hwnd);
        with_critical_section!(WAIT_TO_REGISTER_MS, {
            focus_window(self.hwnd).as_bool() && send_key_vk(direction.to_key()).is_ok()
        });
        //thread::sleep(Duration::from_millis(1000));
        let px_after = get_move_pixel(self.hwnd);

        let result = px_before != px_after;
        println!(
            "{}: {}",
            format!("Can go {:?}", direction).white(),
            if result {
                format!("{:?}", result).green()
            } else {
                format!("{:?}", result).red()
            }
        );
        result
    }

    fn walk(&self, direction: Option<Direction>, walk_duration_ms: u64) -> bool {
        println!(
            "{}",
            format!(
                "Walking {}...",
                direction.map_or_else(|| "".to_string(), |d| d.to_string())
            )
            .bright_yellow()
        );
        let result = with_critical_section!(WAIT_TO_REGISTER_MS, {
            focus_window(self.hwnd).as_bool() && send_key_vk(AUTO_WALK).is_ok()
        });
        thread::sleep(Duration::from_millis(walk_duration_ms));
        result
    }
}
