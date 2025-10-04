use crate::simulation::CharState;
use crate::simulation::shared_state::SharedState;
use crate::simulation::skill::Skill;
use crate::simulation::skill_type::SkillType;
use colored::Colorize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct SkillTracker {
    last_cast: Arc<Mutex<HashMap<String, Instant>>>,
    buff_tracker: Arc<Mutex<HashMap<String, Instant>>>,
    debuff_tracker: Arc<Mutex<HashMap<String, Instant>>>,
    shared_state: Arc<Mutex<SharedState>>,
}

impl SkillTracker {
    pub fn new(shared_state: Arc<Mutex<SharedState>>) -> Self {
        SkillTracker {
            last_cast: Arc::new(Mutex::new(HashMap::new())),
            buff_tracker: Arc::new(Mutex::new(HashMap::new())),
            debuff_tracker: Arc::new(Mutex::new(HashMap::new())),
            shared_state,
        }
    }

    pub fn reset(&self) {
        println!("Resetting skill tracker");
        let last_cast_map = Arc::clone(&self.last_cast);
        let buff_map = Arc::clone(&self.buff_tracker);
        let debuff_map = Arc::clone(&self.debuff_tracker);
        let mut last_cast_map = last_cast_map.lock().unwrap();
        let mut buff_map = buff_map.lock().unwrap();
        let mut debuff_map = debuff_map.lock().unwrap();
        last_cast_map.clear();
        buff_map.clear();
        debuff_map.clear();
        println!("Skill tracker has been reset");
    }

    pub fn track_cast(&self, skill: &Skill) {
        let now = Instant::now();
        let last_cast_map = Arc::clone(&self.last_cast);
        let buff_map = Arc::clone(&self.buff_tracker);
        let debuff_map = Arc::clone(&self.debuff_tracker);
        let last_cast_map = last_cast_map.lock().unwrap();
        let buff_map = buff_map.lock().unwrap();
        let debuff_map = debuff_map.lock().unwrap();
        if let Some(last_cast) = last_cast_map.get(&skill.name) {
            let diff = now - *last_cast;
            if diff.as_secs_f32() < skill.cooldown {
                println!(
                    "{}",
                    format!(
                        "WARN: trying to cast {} which should still be on a cooldown",
                        skill.name
                    )
                    .red()
                );
                return;
            }
        }
        self.track_inner(skill, now, last_cast_map, buff_map, debuff_map);
    }

    fn track_inner(
        &self,
        skill: &Skill,
        now: Instant,
        mut last_cast_map: MutexGuard<HashMap<String, Instant>>,
        mut buff_map: MutexGuard<HashMap<String, Instant>>,
        mut debuff_map: MutexGuard<HashMap<String, Instant>>,
    ) {
        last_cast_map.insert(skill.name.clone(), now);
        match skill.skill_type {
            SkillType::Buff => {
                if skill.name == "Augmentation" {
                    let state = self.shared_state.clone();
                    let mut state = state.lock().unwrap();
                    state.set_skill_haste(true);
                } else if skill.name == "Frenzy" {
                    let state = self.shared_state.clone();
                    let mut state = state.lock().unwrap();
                    state.set_frenzy(true);
                }
                buff_map.insert(skill.name.clone(), now);
            }
            SkillType::Debuff => {
                debuff_map.insert(skill.name.clone(), now);
            }
            _ => {}
        }
    }

    pub fn is_on_cooldown(&self, skill: &Skill) -> bool {
        let map = Arc::clone(&self.last_cast);
        let map = map.lock().unwrap();
        match map.get(&skill.name) {
            None => false,
            Some(last_cast) => {
                let now = Instant::now();
                let diff = now - *last_cast;
                diff.as_secs_f32() < skill.cooldown
            }
        }
    }

    pub fn can_cast(&self, skill: &Skill, state: CharState) -> bool {
        let is_on_cooldown = self.is_on_cooldown(skill);
        let can_cast = skill.can_cast(state);
        let result = !is_on_cooldown && can_cast;
        println!(
            "Checking ability: {}. Is on cooldown: {}. Can cast: {}. Result: {}.",
            skill.name,
            if is_on_cooldown {
                is_on_cooldown.to_string().red()
            } else {
                is_on_cooldown.to_string().green()
            },
            if can_cast {
                can_cast.to_string().green()
            } else {
                can_cast.to_string().red()
            },
            if result {
                result.to_string().green()
            } else {
                result.to_string().red()
            }
        );
        result
    }

    pub fn should_cast(&self, skill: &Skill, state: CharState) -> bool {
        let should_attack = match skill.skill_type {
            SkillType::Buff => {
                let result = !self.has_buff_applied(skill);
                if result {
                    if skill.name == "Augmentation" {
                        let state = self.shared_state.clone();
                        let mut state = state.lock().unwrap();
                        state.set_skill_haste(false);
                    } else if skill.name == "Frenzy" {
                        let state = self.shared_state.clone();
                        let mut state = state.lock().unwrap();
                        state.set_frenzy(false);
                    }
                    println!("{}", format!("Buff {} expired", skill.name).yellow());
                } else {
                    println!(
                        "{}",
                        format!("Buff {} is still applied", skill.name).bright_green()
                    );
                }
                result
            }
            SkillType::Debuff => {
                let result = !self.has_debuff_applied(skill);
                if result {
                    println!("{}", format!("Debuff {} expired", skill.name).yellow());
                } else {
                    println!(
                        "{}",
                        format!("Debuff {} is still applied", skill.name).bright_green()
                    );
                }
                result
            }
            SkillType::Attack => true,
        };
        self.can_cast(skill, state) && should_attack
    }

    //TODO: change to 5.0 after testing
    const BUFF_DEBUFF_DURATION_TOLERANCE: u64 = 0;

    pub fn has_buff_applied(&self, skill: &Skill) -> bool {
        let now = Instant::now();
        let map = Arc::clone(&self.buff_tracker);
        let map = map.lock().unwrap();
        if let Some(last_cast) = map.get(&skill.name) {
            let diff = now - Duration::from_secs(Self::BUFF_DEBUFF_DURATION_TOLERANCE) - *last_cast;
            if let Some(buff_duration) = skill.buff_duration {
                diff.as_secs_f32() < buff_duration
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn has_debuff_applied(&self, skill: &Skill) -> bool {
        let now = Instant::now();
        let map = Arc::clone(&self.debuff_tracker);
        let map = map.lock().unwrap();
        if let Some(last_cast) = map.get(&skill.name) {
            let diff = now - Duration::from_secs(Self::BUFF_DEBUFF_DURATION_TOLERANCE) - *last_cast;
            if let Some(debuff_duration) = skill.debuff_duration {
                diff.as_secs_f32() < debuff_duration
            } else {
                false
            }
        } else {
            false
        }
    }
}
