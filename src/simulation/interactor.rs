use crate::simulation::keys::{DISCARD, LOOT_INTERACT};
use crate::simulation::skill::Skill;
use crate::simulation::{DebugObj, WindowObj};
use crate::win_util::{focus_window, send_key_vk};

pub trait Interactor {
    fn cast_skill(&self, skill: &Skill) -> bool;
    fn interact(&self) -> bool;
    fn discard(&self) -> bool;
}

impl Interactor for DebugObj {
    fn cast_skill(&self, skill: &Skill) -> bool {
        println!("Casting '{}'", skill.name);
        true
    }

    fn interact(&self) -> bool {
        println!("Interacting");
        true
    }

    fn discard(&self) -> bool {
        println!("Discarding a looted item");
        true
    }
}

impl Interactor for WindowObj {
    fn cast_skill(&self, skill: &Skill) -> bool {
        println!("Casting '{}'", skill.name);
        focus_window(self.hwnd).as_bool() && send_key_vk(skill.key).is_ok()
    }

    fn interact(&self) -> bool {
        println!("Interacting");
        focus_window(self.hwnd).as_bool() && send_key_vk(LOOT_INTERACT).is_ok()
    }

    fn discard(&self) -> bool {
        println!("Discarding a looted item");
        focus_window(self.hwnd).as_bool() && send_key_vk(DISCARD).is_ok()
    }
}
