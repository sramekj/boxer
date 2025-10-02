use crate::simulation::CharState;
use crate::simulation::skill::Skill;
use crate::simulation::skill_type::SkillType;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct SkillTracker {
    last_cast: Arc<Mutex<HashMap<String, Instant>>>,
    buff_tracker: Arc<Mutex<HashMap<String, Instant>>>,
    debuff_tracker: Arc<Mutex<HashMap<String, Instant>>>,
}

impl SkillTracker {
    pub fn new() -> Self {
        SkillTracker {
            last_cast: Arc::new(Mutex::new(HashMap::new())),
            buff_tracker: Arc::new(Mutex::new(HashMap::new())),
            debuff_tracker: Arc::new(Mutex::new(HashMap::new())),
        }
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
                    "WARN: trying to cast {} which should still be on a cooldown",
                    skill.name
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
            skill.name, is_on_cooldown, can_cast, result
        );
        result
    }

    pub fn should_cast(&self, skill: &Skill, state: CharState) -> bool {
        let should_attack = match skill.skill_type {
            SkillType::Buff => {
                let result = !self.has_buff_applied(skill);
                if result {
                    println!("Buff {} expired", skill.name);
                } else {
                    println!("Buff {} is still applied", skill.name);
                }
                result
            }
            SkillType::Debuff => {
                let result = !self.has_debuff_applied(skill);
                if result {
                    println!("Debuff {} expired", skill.name);
                } else {
                    println!("Debuff {} is still applied", skill.name);
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
