use crate::simulation::CharState;
use crate::simulation::skill::Skill;
use crate::simulation::skill_type::SkillType;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct SkillTracker {
    last_cast: HashMap<String, Instant>,
    buff_tracker: HashMap<String, Instant>,
    debuff_tracker: HashMap<String, Instant>,
}

impl SkillTracker {
    pub fn new() -> Self {
        SkillTracker {
            last_cast: HashMap::new(),
            buff_tracker: HashMap::new(),
            debuff_tracker: HashMap::new(),
        }
    }

    pub fn track_cast(&mut self, skill: &Skill) {
        let now = Instant::now();
        if let Some(last_cast) = self.last_cast.get(&skill.name) {
            let diff = now - *last_cast;
            if diff.as_secs_f32() < skill.cooldown {
                println!(
                    "WARN: trying to cast {} which should still be on a cooldown",
                    skill.name
                );
                return;
            }
            self.track_inner(skill, now);
        } else {
            self.track_inner(skill, now);
        }
    }

    fn track_inner(&mut self, skill: &Skill, now: Instant) {
        self.last_cast.insert(skill.name.clone(), now);
        match skill.skill_type {
            SkillType::Buff => {
                self.buff_tracker.insert(skill.name.clone(), now);
            }
            SkillType::Debuff => {
                self.debuff_tracker.insert(skill.name.clone(), now);
            }
            _ => {}
        }
    }

    pub fn is_on_cooldown(&self, skill: &Skill) -> bool {
        match self.last_cast.get(&skill.name) {
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
        if let Some(last_cast) = self.buff_tracker.get(&skill.name) {
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
        if let Some(last_cast) = self.debuff_tracker.get(&skill.name) {
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
