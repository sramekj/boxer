use crate::configuration::class_config::AutoAttack;
use crate::simulation::global_lock::CRITICAL_SECTION;
use crate::simulation::keys::{
    AUTO_ATTACK, AUTO_RANGED_ATTACK, DISCARD, HEALTH_POT, INVENTORY, Key, LOOT_INTERACT,
};
use crate::simulation::simulation_state::{DebugObj, WindowObj};
use crate::simulation::skill::Skill;
use crate::win_util::{focus_window, send_key_vk, set_mouse};
use colored::Colorize;
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
}

const WAIT_TO_REGISTER_MS: u64 = 200;
impl Interactor for WindowObj {
    fn cast_skill(&self, skill: &Skill) -> bool {
        print!("Casting ");
        print!("{}", format!("'{}'", skill.name).bright_magenta());
        let _lock = CRITICAL_SECTION.lock().unwrap();
        let result = focus_window(self.hwnd).as_bool() && send_key_vk(skill.key).is_ok();
        thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
        drop(_lock);
        result
    }

    fn loot(&self) -> bool {
        println!("{}", "Looting an item".green());
        let _lock = CRITICAL_SECTION.lock().unwrap();
        let result = focus_window(self.hwnd).as_bool() && send_key_vk(LOOT_INTERACT).is_ok();
        thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
        drop(_lock);
        result
    }

    fn interact(&self) -> bool {
        println!("{}", "Interacting".green());
        let _lock = CRITICAL_SECTION.lock().unwrap();
        let result = focus_window(self.hwnd).as_bool() && send_key_vk(LOOT_INTERACT).is_ok();
        thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
        drop(_lock);
        result
    }

    fn discard(&self) -> bool {
        println!("{}", "Discarding an item".red());
        let _lock = CRITICAL_SECTION.lock().unwrap();
        let result = focus_window(self.hwnd).as_bool() && send_key_vk(DISCARD).is_ok();
        thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
        drop(_lock);
        result
    }

    fn target_player(&self, player_index: usize) -> bool {
        println!("Targeting player {}", player_index + 1);
        if let Some(key) = Key::get_party_keys().get(player_index) {
            let _lock = CRITICAL_SECTION.lock().unwrap();
            let result = focus_window(self.hwnd).as_bool() && send_key_vk(*key).is_ok();
            thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
            drop(_lock);
            result
        } else {
            false
        }
    }

    fn auto_attack(&self, auto_attack: AutoAttack) -> bool {
        println!("{}", format!("Auto-attacking {:?}", auto_attack).magenta());
        let _lock = CRITICAL_SECTION.lock().unwrap();
        let key = match auto_attack {
            AutoAttack::Primary => AUTO_ATTACK,
            AutoAttack::Ranged => AUTO_RANGED_ATTACK,
        };
        let result = focus_window(self.hwnd).as_bool() && send_key_vk(key).is_ok();
        thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
        drop(_lock);
        result
    }

    fn use_hp_pot(&self) -> bool {
        println!("{}", "Using a HP potion".red());
        let _lock = CRITICAL_SECTION.lock().unwrap();
        let result = focus_window(self.hwnd).as_bool() && send_key_vk(HEALTH_POT).is_ok();
        thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
        drop(_lock);
        result
    }

    fn inventory_toggle(&self) -> bool {
        println!("{}", "Toggling an inventory".bright_purple());
        let _lock = CRITICAL_SECTION.lock().unwrap();
        let result = focus_window(self.hwnd).as_bool() && send_key_vk(INVENTORY).is_ok();
        thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
        drop(_lock);
        result
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
}
