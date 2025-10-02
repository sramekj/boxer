use crate::simulation::skill::Skill;
use crate::simulation::{DebugObj, WindowObj};
use crate::win_util::{focus_window, send_key_vk};

pub trait SkillCaster {
    fn cast(&self, skill: &Skill) -> bool;
}

impl SkillCaster for DebugObj {
    fn cast(&self, skill: &Skill) -> bool {
        println!("Casting '{}'", skill.name);
        true
    }
}

impl SkillCaster for WindowObj {
    fn cast(&self, skill: &Skill) -> bool {
        println!("Casting '{}'", skill.name);
        focus_window(self.hwnd).as_bool() && send_key_vk(skill.key).is_ok()
    }
}
