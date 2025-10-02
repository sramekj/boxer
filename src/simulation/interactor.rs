use crate::simulation::skill::Skill;
use crate::simulation::{DebugObj, WindowObj};
use crate::win_util::{focus_window, send_key_vk};

pub trait Interactor {
    fn cast_skill(&self, skill: &Skill) -> bool;
}

impl Interactor for DebugObj {
    fn cast_skill(&self, skill: &Skill) -> bool {
        println!("Casting '{}'", skill.name);
        true
    }
}

impl Interactor for WindowObj {
    fn cast_skill(&self, skill: &Skill) -> bool {
        println!("Casting '{}'", skill.name);
        focus_window(self.hwnd).as_bool() && send_key_vk(skill.key).is_ok()
    }
}
