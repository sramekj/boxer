use crate::simulation::keys::{DISCARD, Key, LOOT_INTERACT};
use crate::simulation::skill::Skill;
use crate::simulation::{DebugObj, WindowObj};
use crate::win_util::{focus_window, send_key_vk};
use colored::Colorize;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub trait Interactor {
    fn cast_skill(&self, skill: &Skill) -> bool;
    fn loot(&self) -> bool;
    fn interact(&self) -> bool;
    fn discard(&self) -> bool;
    fn target_player(&self, player_index: usize) -> bool;
}

impl Interactor for DebugObj {
    fn cast_skill(&self, skill: &Skill) -> bool {
        print!("Casting '{}'", skill.name);
        true
    }

    fn loot(&self) -> bool {
        println!("Looting an item");
        true
    }

    fn interact(&self) -> bool {
        println!("Interacting");
        true
    }

    fn discard(&self) -> bool {
        println!("Discarding an item");
        true
    }

    fn target_player(&self, player_index: usize) -> bool {
        println!("Targeting player {}", player_index + 1);
        true
    }
}

lazy_static::lazy_static! {
    static ref CRITICAL_SECTION: Mutex<()> = Mutex::new(());
}

const WAIT_TO_REGISTER_MS: u64 = 100;
impl Interactor for WindowObj {
    fn cast_skill(&self, skill: &Skill) -> bool {
        print!("Casting ");
        print!("{}", format!("'{}'", skill.name).bright_magenta());
        let _lock = CRITICAL_SECTION.lock().unwrap();
        let result = focus_window(self.hwnd).as_bool() && send_key_vk(skill.key).is_ok();
        drop(_lock);
        thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
        result
    }

    fn loot(&self) -> bool {
        println!("{}", "Looting an item".green());
        let _lock = CRITICAL_SECTION.lock().unwrap();
        let result = focus_window(self.hwnd).as_bool() && send_key_vk(LOOT_INTERACT).is_ok();
        drop(_lock);
        thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
        result
    }

    fn interact(&self) -> bool {
        println!("{}", "Interacting".green());
        let _lock = CRITICAL_SECTION.lock().unwrap();
        let result = focus_window(self.hwnd).as_bool() && send_key_vk(LOOT_INTERACT).is_ok();
        drop(_lock);
        thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
        result
    }

    fn discard(&self) -> bool {
        println!("{}", "Discarding an item".red());
        let _lock = CRITICAL_SECTION.lock().unwrap();
        let result = focus_window(self.hwnd).as_bool() && send_key_vk(DISCARD).is_ok();
        drop(_lock);
        thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
        result
    }

    fn target_player(&self, player_index: usize) -> bool {
        println!("Targeting player {}", player_index + 1);
        if let Some(key) = Key::get_party_keys().get(player_index) {
            let _lock = CRITICAL_SECTION.lock().unwrap();
            let result = focus_window(self.hwnd).as_bool() && send_key_vk(*key).is_ok();
            drop(_lock);
            thread::sleep(Duration::from_millis(WAIT_TO_REGISTER_MS));
            result
        } else {
            false
        }
    }
}
