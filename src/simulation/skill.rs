use crate::simulation::CharState;
use crate::simulation::keys::Key;
use crate::simulation::shared_state::SharedState;
use crate::simulation::skill_type::SkillType;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub key: Key,
    pub cast_time: f32,
    pub cooldown: f32,
    pub buff_duration: Option<f32>,
    pub debuff_duration: Option<f32>,
    pub skill_type: SkillType,
}
const GCD: f32 = 2.5;

impl Skill {
    pub fn get_gcd(&self) -> f32 {
        //TODO: measure enchanter impact
        GCD
    }

    pub fn can_cast(&self, state: CharState) -> bool {
        match state {
            CharState::InTown | CharState::Dead | CharState::Unknown => false,
            CharState::InDungeon | CharState::AtShrine => self.skill_type == SkillType::Buff,
            CharState::Fighting | CharState::Looting => true,
        }
    }

    pub fn cast_time(&self, shared_state: Arc<Mutex<SharedState>>) -> f32 {
        if self.cast_time == 0.0 {
            0.0
        } else {
            self.cast_time * self.get_haste_coef(shared_state)
        }
    }

    fn get_haste_coef(&self, shared_state: Arc<Mutex<SharedState>>) -> f32 {
        let state = Arc::clone(&shared_state);
        let state = state.lock().unwrap();
        if state.get_skill_haste() {
            return (100.0 - state.get_skill_haste_percent()) / 100.0;
        }
        1.0
    }
}
