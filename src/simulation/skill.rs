use crate::simulation::CharState;
use crate::simulation::skill_type::SkillType;
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub key: VIRTUAL_KEY,
    pub cast_time: f32,
    pub cooldown: f32,
    pub buff_duration: Option<f32>,
    pub debuff_duration: Option<f32>,
    pub skill_type: SkillType,
}
const GCD: f32 = 2.5;

impl Skill {
    pub fn has_gcd(&self) -> bool {
        self.cast_time == 0.0
    }

    pub fn get_gcd(&self) -> f32 {
        //TODO: measure enchanter impact
        GCD
    }

    pub fn can_cast(&self, state: CharState) -> bool {
        match state {
            CharState::InTown | CharState::Dead => false,
            CharState::InDungeon => self.skill_type == SkillType::Buff,
            CharState::Fighting | CharState::Looting => true,
        }
    }
}
