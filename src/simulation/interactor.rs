use crate::simulation::keys::{DISCARD, Key, LOOT_INTERACT};
use crate::simulation::skill::Skill;
use crate::simulation::{DebugObj, WindowObj};
use crate::win_util::{focus_window, send_key_vk};

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

impl Interactor for WindowObj {
    fn cast_skill(&self, skill: &Skill) -> bool {
        print!("Casting '{}'", skill.name);
        focus_window(self.hwnd).as_bool() && send_key_vk(skill.key).is_ok()
    }

    fn loot(&self) -> bool {
        println!("Looting an item");
        focus_window(self.hwnd).as_bool() && send_key_vk(LOOT_INTERACT).is_ok()
    }

    fn interact(&self) -> bool {
        println!("Interacting");
        focus_window(self.hwnd).as_bool() && send_key_vk(LOOT_INTERACT).is_ok()
    }

    fn discard(&self) -> bool {
        println!("Discarding an item");
        focus_window(self.hwnd).as_bool() && send_key_vk(DISCARD).is_ok()
    }

    fn target_player(&self, player_index: usize) -> bool {
        println!("Targeting player {}", player_index + 1);
        if let Some(key) = Key::get_party_keys().get(player_index) {
            focus_window(self.hwnd).as_bool() && send_key_vk(*key).is_ok()
        } else {
            false
        }
    }
}
