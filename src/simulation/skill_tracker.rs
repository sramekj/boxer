use crate::simulation::char_state::CharState;
use crate::simulation::shared_state::SharedStateHandle;
use crate::simulation::skill::Skill;
use crate::simulation::skill_tracker::SkillTrackerMessage::*;
use crate::simulation::skill_type::SkillType;
use crate::simulation::type_of;
use colored::Colorize;
use std::collections::HashMap;
use std::string::ToString;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Instant;

const HP_POT_COOLDOWN: f32 = 24.0;
const HP_POT_KEY: &str = "hp-potion";
// we want to reapply buffs/debuffs before they drop down
const BUFF_DURATION_TOLERANCE_SEC: f32 = 3.0;
const DEBUFF_DURATION_TOLERANCE_SEC: f32 = 1.0;

#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
enum SkillTrackerMessage {
    TrackCast(Skill, Option<Vec<(String, f32)>>, Sender<()>),
    IsOnCooldown(Skill, Option<Vec<(String, f32)>>, Sender<bool>),
    CanCast(Skill, Option<Vec<(String, f32)>>, CharState, Sender<bool>),
    ShouldCast(Skill, Option<Vec<(String, f32)>>, CharState, Sender<bool>),
    HasBuffApplied(Skill, Sender<bool>),
    HasDebuffApplied(Skill, Sender<bool>),
    TrackHpPot(Sender<()>),
    IsHpPotOnCooldown(Sender<bool>),
    ResetDebuffs(Sender<()>),
    Reset(Sender<()>),
    Stop(Sender<()>),
}

#[derive(Debug)]
struct SkillTrackerActor {
    last_cast: HashMap<String, Instant>,
    buff_tracker: HashMap<String, Instant>,
    debuff_tracker: HashMap<String, Instant>,
    potion_tracker: HashMap<String, Instant>,
    shared_state: Arc<SharedStateHandle>,
    receiver: Receiver<SkillTrackerMessage>,
}

impl SkillTrackerActor {
    pub fn new(
        shared_state: Arc<SharedStateHandle>,
        receiver: Receiver<SkillTrackerMessage>,
    ) -> SkillTrackerActor {
        SkillTrackerActor {
            last_cast: HashMap::new(),
            buff_tracker: HashMap::new(),
            debuff_tracker: HashMap::new(),
            potion_tracker: HashMap::new(),
            shared_state,
            receiver,
        }
    }

    fn run(mut self) {
        while let Ok(msg) = self.receiver.recv() {
            //println!("Received message: {:?}", type_of(&msg));
            match msg {
                TrackCast(skill, reductions, sender) => {
                    self.track_skill(&skill, reductions.as_ref());
                    let _ = sender.send(());
                }
                IsOnCooldown(skill, reductions, sender) => {
                    let _ = sender.send(self.is_on_cooldown(&skill, reductions.as_ref()));
                }
                CanCast(skill, reductions, state, sender) => {
                    let _ = sender.send(self.can_cast(&skill, reductions.as_ref(), state));
                }
                ShouldCast(skill, reductions, state, sender) => {
                    let _ = sender.send(self.should_cast(&skill, reductions.as_ref(), state));
                }
                HasBuffApplied(skill, sender) => {
                    let _ = sender.send(self.has_buff_applied(&skill));
                }
                HasDebuffApplied(skill, sender) => {
                    let _ = sender.send(self.has_debuff_applied(&skill));
                }
                TrackHpPot(sender) => {
                    self.track_hp_pot();
                    let _ = sender.send(());
                }
                IsHpPotOnCooldown(sender) => {
                    let _ = sender.send(self.is_hp_pot_on_cooldown());
                }
                Stop(sender) => {
                    print!("Shutting down {}", type_of(&self));
                    let _ = sender.send(());
                    break;
                }
                ResetDebuffs(sender) => {
                    self.debuff_tracker.clear();
                    let _ = sender.send(());
                }
                Reset(sender) => {
                    println!("Resetting skill tracker");
                    self.last_cast.clear();
                    self.buff_tracker.clear();
                    self.debuff_tracker.clear();
                    self.potion_tracker.clear();
                    let _ = sender.send(());
                }
            }
        }
    }

    fn track_skill(&mut self, skill: &Skill, reductions: Option<&Vec<(String, f32)>>) {
        let now = Instant::now();
        if let Some(last_cast) = self.last_cast.get(&skill.name) {
            let diff = now - *last_cast;
            if diff.as_secs_f32() < skill.get_cooldown(reductions) {
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
        println!("Tracking skill: {}", skill.name);
        self.last_cast.insert(skill.name.clone(), now);
        match skill.skill_type {
            SkillType::Buff => {
                if skill.name == "Augmentation" {
                    self.shared_state.set_skill_haste_applied(true);
                } else if skill.name == "Frenzy" {
                    self.shared_state.set_frenzy_applied(true);
                }
                self.buff_tracker.insert(skill.name.clone(), now);
            }
            SkillType::Debuff => {
                self.debuff_tracker.insert(skill.name.clone(), now);
            }
            _ => {}
        }
    }

    fn is_on_cooldown(&self, skill: &Skill, reductions: Option<&Vec<(String, f32)>>) -> bool {
        match self.last_cast.get(&skill.name) {
            None => false,
            Some(last_cast) => {
                let now = Instant::now();
                let diff = now - *last_cast;
                diff.as_secs_f32() < skill.get_cooldown(reductions)
            }
        }
    }

    fn track_hp_pot(&mut self) {
        let now = Instant::now();
        if self.is_hp_pot_on_cooldown() {
            println!(
                "{}",
                "WARN: trying to use a health potion which should still be on a cooldown".red()
            );
        } else {
            self.potion_tracker.insert(HP_POT_KEY.to_string(), now);
        }
    }

    fn can_cast(
        &self,
        skill: &Skill,
        reductions: Option<&Vec<(String, f32)>>,
        state: CharState,
    ) -> bool {
        let is_on_cooldown = self.is_on_cooldown(skill, reductions);
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

    fn should_cast(
        &self,
        skill: &Skill,
        reductions: Option<&Vec<(String, f32)>>,
        state: CharState,
    ) -> bool {
        let should_attack = match skill.skill_type {
            SkillType::Buff => {
                let result = !self.has_buff_applied(skill);
                if result {
                    if skill.name == "Augmentation" {
                        self.shared_state.set_skill_haste_applied(false);
                    } else if skill.name == "Frenzy" {
                        self.shared_state.set_frenzy_applied(false);
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
        self.can_cast(skill, reductions, state) && should_attack
    }

    fn has_buff_applied(&self, skill: &Skill) -> bool {
        let now = Instant::now();
        if let Some(last_cast) = self.buff_tracker.get(&skill.name) {
            let diff = now - *last_cast;
            if let Some(buff_duration) = skill.buff_duration {
                diff.as_secs_f32() < (buff_duration - BUFF_DURATION_TOLERANCE_SEC)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn has_debuff_applied(&self, skill: &Skill) -> bool {
        let now = Instant::now();
        if let Some(last_cast) = self.debuff_tracker.get(&skill.name) {
            let diff = now - *last_cast;
            if let Some(debuff_duration) = skill.debuff_duration {
                diff.as_secs_f32() < (debuff_duration - DEBUFF_DURATION_TOLERANCE_SEC)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_hp_pot_on_cooldown(&self) -> bool {
        match self.potion_tracker.get(HP_POT_KEY) {
            None => false,
            Some(last_cast) => {
                let now = Instant::now();
                let diff = now - *last_cast;
                diff.as_secs_f32() < HP_POT_COOLDOWN
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillTrackerHandle {
    sender: Sender<SkillTrackerMessage>,
}

impl SkillTrackerHandle {
    pub fn new(shared_state_handle: Arc<SharedStateHandle>) -> Self {
        let (tx, rx) = mpsc::channel();
        let actor = SkillTrackerActor::new(shared_state_handle, rx);
        println!("Starting {}", type_of(&actor));
        thread::spawn(move || {
            if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| actor.run())) {
                eprintln!("{}", format!("Actor panicked: {:?}", e).red());
            }
        });
        Self { sender: tx }
    }

    fn ask<T>(&self, msg: impl FnOnce(Sender<T>) -> SkillTrackerMessage) -> T {
        let (tx, rx) = mpsc::channel();
        let _ = self.sender.send(msg(tx));
        rx.recv()
            .expect("Actor thread died or failed to send reply")
    }

    pub fn reset(&self) {
        self.ask(Reset);
    }

    pub fn reset_debuffs(&self) {
        self.ask(ResetDebuffs);
    }

    pub fn track_hp_pot(&self) {
        self.ask(TrackHpPot);
    }

    pub fn is_hp_pot_on_cooldown(&self) -> bool {
        self.ask(IsHpPotOnCooldown)
    }

    pub fn can_cast(
        &self,
        skill: &Skill,
        reductions: Option<&Vec<(String, f32)>>,
        state: CharState,
    ) -> bool {
        self.ask(|tx| CanCast(skill.clone(), reductions.cloned(), state, tx))
    }

    pub fn should_cast(
        &self,
        skill: &Skill,
        reductions: Option<&Vec<(String, f32)>>,
        state: CharState,
    ) -> bool {
        self.ask(|tx| ShouldCast(skill.clone(), reductions.cloned(), state, tx))
    }

    pub fn track_cast(&self, skill: &Skill, reductions: Option<&Vec<(String, f32)>>) {
        self.ask(|tx| TrackCast(skill.clone(), reductions.cloned(), tx))
    }

    pub fn is_on_cooldown(&self, skill: &Skill, reductions: Option<&Vec<(String, f32)>>) -> bool {
        self.ask(|tx| IsOnCooldown(skill.clone(), reductions.cloned(), tx))
    }

    pub fn stop(&self) {
        let (tx, _) = mpsc::channel();
        let _ = self.sender.send(Stop(tx));
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation::keys::SKILL_BUTTON_2;
    use crate::simulation::shared_state::SharedStateHandle;
    use crate::simulation::skill::Skill;
    use crate::simulation::skill_tracker::SkillTrackerHandle;
    use crate::simulation::skill_type::SkillType;
    use colored::Colorize;
    use std::sync::Arc;

    #[test]
    fn test_skill_tracker_is_transient() {
        std::panic::set_hook(Box::new(|panic_info| {
            println!("{}", format!("Panic occurred: {:?}", panic_info).red());
        }));

        let shared_state = Arc::new(SharedStateHandle::new(1.0, 1.0));
        let h1 = SkillTrackerHandle::new(shared_state.clone());
        let h2 = SkillTrackerHandle::new(shared_state.clone());
        let skill = Skill {
            name: "Color Shift".to_string(),
            key: SKILL_BUTTON_2,
            cast_time: 1.5,
            cooldown: 30.0,
            buff_duration: None,
            debuff_duration: None,
            skill_type: SkillType::Attack,
        };

        h1.track_cast(&skill, None);
        let r1 = h1.is_on_cooldown(&skill, None);
        let r2 = h2.is_on_cooldown(&skill, None);

        assert!(r1);
        assert!(!r2);

        h1.stop();
        h2.stop();
        shared_state.stop();
    }
}
