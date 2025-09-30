use crate::config::{Class, Config, WindowConfig};
use crate::util::CyclicIterator;
use crate::win_util::{focus_window, get_pixel_color, send_key_vk};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_9, VK_OEM_MINUS, VK_OEM_PLUS,
};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum CharState {
    InTown,
    InDungeon,
    Fighting,
    Looting,
    Dead,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum SkillType {
    Attack,
    Buff,
    Debuff,
}

const GCD: f32 = 2.5;

pub trait StateChecker {
    fn get_state(&self) -> CharState;
}

pub trait SkillCaster {
    fn cast(&self, skill: &Skill) -> bool;
}

pub struct DebugObj {}
impl DebugObj {
    pub fn new() -> DebugObj {
        DebugObj {}
    }
}
pub struct WindowObj {
    hwnd: Option<HWND>,
}

impl WindowObj {
    pub fn new(hwnd: Option<HWND>) -> WindowObj {
        Self { hwnd }
    }
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

impl StateChecker for DebugObj {
    fn get_state(&self) -> CharState {
        println!("Getting state");
        let state = CharState::Fighting;
        println!("Setting state to {:?}", state);
        state
    }
}

impl StateChecker for WindowObj {
    fn get_state(&self) -> CharState {
        println!("Getting state");
        //TODO.....
        let x = get_pixel_color(self.hwnd, 100, 100);
        let state = CharState::Fighting;
        println!("Setting state to {:?}", state);
        state
    }
}

#[derive(Debug)]
pub struct Skill {
    pub name: String,
    pub key: VIRTUAL_KEY,
    pub cast_time: f32,
    pub cooldown: f32,
    pub buff_duration: Option<f32>,
    pub debuff_duration: Option<f32>,
    pub skill_type: SkillType,
}

impl Skill {
    pub fn has_gcd(&self) -> bool {
        self.cast_time == 0.0
    }

    pub fn get_gcd(&self) -> f32 {
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

#[derive(Debug)]
pub struct Rotation {
    pub skills: Vec<Skill>,
}

pub trait Rotations<T> {
    fn get_rotation(input: T, cfg: &Config) -> Rotation;
}

fn calculate_haste_coef(cfg: &Config) -> f32 {
    let has_enchanter = cfg
        .windows
        .iter()
        .any(|w| w.class == Class::Enchanter && w.active);
    if has_enchanter {
        return (100.0 - cfg.skill_haste_percent) / 100.0;
    }
    1.0
}
impl Rotations<Class> for Rotation {
    fn get_rotation(input: Class, cfg: &Config) -> Rotation {
        match input {
            Class::Warrior => Rotation {
                skills: vec![
                    Skill {
                        name: "Intrepid Shout".to_string(),
                        key: VK_OEM_PLUS,
                        cast_time: 0.0,
                        cooldown: 0.0,
                        buff_duration: Some(240.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Frenzy".to_string(),
                        key: VK_5,
                        cast_time: 0.0,
                        cooldown: 60.0,
                        buff_duration: Some(cfg.frenzy_duration),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Bulwark".to_string(),
                        key: VK_OEM_MINUS,
                        cast_time: 0.0,
                        cooldown: 40.0,
                        buff_duration: Some(cfg.bulwark_duration),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Rupture".to_string(),
                        key: VK_4,
                        cast_time: 0.0,
                        cooldown: 0.0,
                        buff_duration: None,
                        debuff_duration: Some(18.0),
                        skill_type: SkillType::Debuff,
                    },
                    Skill {
                        name: "Double Throw".to_string(),
                        key: VK_2,
                        cast_time: 0.0,
                        cooldown: 20.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                    Skill {
                        name: "Furious Cleave".to_string(),
                        key: VK_1,
                        cast_time: 0.0,
                        cooldown: 16.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                    Skill {
                        name: "Rupture".to_string(),
                        key: VK_4,
                        cast_time: 0.0,
                        cooldown: 0.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                ],
            },
            Class::Warlock => Rotation {
                skills: vec![
                    Skill {
                        name: "Lich Form".to_string(),
                        key: VK_OEM_MINUS,
                        cast_time: 3.0 * calculate_haste_coef(cfg),
                        cooldown: 0.0,
                        buff_duration: Some(720.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Profane Spirit".to_string(),
                        key: VK_OEM_PLUS,
                        cast_time: 2.5 * calculate_haste_coef(cfg),
                        cooldown: 0.0,
                        buff_duration: Some(900.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Engulfing Darkness".to_string(),
                        key: VK_1,
                        cast_time: 0.0,
                        cooldown: 45.0,
                        buff_duration: None,
                        debuff_duration: Some(18.0),
                        skill_type: SkillType::Debuff,
                    },
                    Skill {
                        name: "Explosive Plaque".to_string(),
                        key: VK_3,
                        cast_time: 0.0,
                        cooldown: 0.0,
                        buff_duration: None,
                        debuff_duration: Some(30.0),
                        skill_type: SkillType::Debuff,
                    },
                    Skill {
                        name: "Venom Bolt".to_string(),
                        key: VK_4,
                        cast_time: 3.0 * calculate_haste_coef(cfg),
                        cooldown: 0.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                ],
            },
            Class::Enchanter => Rotation {
                skills: vec![
                    Skill {
                        name: "Augmentation".to_string(),
                        key: VK_9,
                        cast_time: 2.0 * calculate_haste_coef(cfg),
                        cooldown: 0.0,
                        buff_duration: Some(480.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Phase Blade".to_string(),
                        key: VK_0,
                        cast_time: 2.0 * calculate_haste_coef(cfg),
                        cooldown: 0.0,
                        buff_duration: Some(600.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Clarity".to_string(),
                        key: VK_OEM_MINUS,
                        cast_time: 2.5 * calculate_haste_coef(cfg),
                        cooldown: 0.0,
                        buff_duration: Some(720.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Color Shift".to_string(),
                        key: VK_2,
                        cast_time: 1.5 * calculate_haste_coef(cfg),
                        cooldown: 30.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                    Skill {
                        name: "Static Suffocation".to_string(),
                        key: VK_1,
                        cast_time: 0.0,
                        cooldown: 6.0,
                        buff_duration: None,
                        debuff_duration: Some(18.0),
                        skill_type: SkillType::Debuff,
                    },
                    Skill {
                        name: "Enthrall".to_string(),
                        key: VK_3,
                        cast_time: 3.0 * calculate_haste_coef(cfg),
                        cooldown: 12.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                    Skill {
                        name: "Mind Blitz".to_string(),
                        key: VK_4,
                        cast_time: 2.5 * calculate_haste_coef(cfg),
                        cooldown: 0.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                ],
            },
        }
    }
}

#[derive(Debug)]
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
            if diff.as_secs_f32() <= skill.cooldown {
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
                let cooldown = if skill.has_gcd() {
                    skill.cooldown.min(skill.get_gcd())
                } else {
                    skill.cooldown
                };
                diff.as_secs_f32() > cooldown
            }
        }
    }

    pub fn can_cast(&self, skill: &Skill, state: CharState) -> bool {
        let result = !self.is_on_cooldown(skill) && skill.can_cast(state);
        println!(
            "Checking ability to cast: {}. Is on cooldown: {}. Can cast: {}. Result: {}.",
            skill.name,
            self.is_on_cooldown(skill),
            skill.can_cast(state),
            result
        );
        result
    }

    pub fn should_cast(&self, skill: &Skill, state: CharState) -> bool {
        if !self.can_cast(skill, state) {
            return false;
        }
        let result = match skill.skill_type {
            SkillType::Buff => !self.has_buff_applied(skill),
            SkillType::Debuff => !self.has_debuff_applied(skill),
            SkillType::Attack => true,
        };
        if result {
            println!("Buff or debuff {} expired", skill.name);
        } else {
            println!("Buff or debuff {} is still applied", skill.name);
        }
        result
    }

    pub fn has_buff_applied(&self, skill: &Skill) -> bool {
        let now = Instant::now();
        if let Some(last_cast) = self.buff_tracker.get(&skill.name) {
            let diff = now - *last_cast;
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
            let diff = now - *last_cast;
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

pub struct SimulationState {
    pub is_running: Arc<AtomicBool>,
    pub sync_interval_ms: u64,
    pub window_config: WindowConfig,
    pub rotation: Rotation,
    pub state: CharState,
    pub skill_tracker: SkillTracker,
    pub skill_caster: Box<dyn SkillCaster>,
    pub state_checker: Box<dyn StateChecker>,
}

impl SimulationState {
    pub fn new(
        sync_interval_ms: u64,
        window_config: WindowConfig,
        rotation: Rotation,
        skill_tracker: SkillTracker,
        skill_caster: Box<dyn SkillCaster>,
        state_checker: Box<dyn StateChecker>,
    ) -> Self {
        SimulationState {
            is_running: Arc::new(AtomicBool::new(false)),
            sync_interval_ms,
            window_config,
            rotation,
            state: CharState::InTown,
            skill_tracker,
            skill_caster,
            state_checker,
        }
    }

    pub fn run(&mut self) {
        let mut rotation_iter = CyclicIterator::new(&self.rotation.skills);
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = self.is_running.clone();
        while is_running.load(Ordering::SeqCst) {
            let state = self.state_checker.get_state();
            match state {
                CharState::InTown => {
                    // do nothing
                }
                _ => {
                    // try to cast
                    let current_rotation_skill = rotation_iter.next().unwrap();
                    if self
                        .skill_tracker
                        .should_cast(current_rotation_skill, state)
                    {
                        self.skill_caster.cast(current_rotation_skill);
                        self.skill_tracker.track_cast(current_rotation_skill);
                    }
                    if state == CharState::Looting {
                        // TODO: implement looting
                    }
                }
            }
            thread::sleep(Duration::from_millis(self.sync_interval_ms));
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{Class, Config};
    use crate::simulation::{DebugObj, Rotation, Rotations, SimulationState, SkillTracker};

    #[test]
    fn test_simulation() {
        let cfg = Config::default();

        let rotation = Rotation::get_rotation(Class::Enchanter, &cfg);

        let mut simulation = SimulationState::new(
            cfg.sync_interval_ms,
            cfg.windows.first().unwrap().clone(),
            rotation,
            SkillTracker::new(),
            Box::new(DebugObj::new()),
            Box::new(DebugObj::new()),
        );

        simulation.run();
    }
}
