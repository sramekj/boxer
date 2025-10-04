pub mod char_state;
mod interactor;
pub mod keys;
pub mod loot;
pub mod rotation;
pub(crate) mod shared_state;
mod skill;
mod skill_tracker;
mod skill_type;
pub mod state_checker;

use crate::config::WindowConfig;
pub(crate) use crate::simulation::char_state::CharState;
use crate::simulation::interactor::Interactor;
use crate::simulation::loot::LootQuality;
pub(crate) use crate::simulation::rotation::Rotation;
use crate::simulation::shared_state::SharedState;
use crate::simulation::skill::Skill;
use crate::simulation::skill_tracker::SkillTracker;
use crate::simulation::state_checker::StateChecker;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::HWND;

pub struct DebugObj {
    test_state: CharState,
}

impl DebugObj {
    pub fn new(test_state: CharState) -> DebugObj {
        DebugObj { test_state }
    }
}

pub struct WindowObj {
    hwnd: Option<HWND>,
}

unsafe impl Send for WindowObj {}
unsafe impl Sync for WindowObj {}

impl WindowObj {
    pub fn new(hwnd: Option<HWND>) -> WindowObj {
        Self { hwnd }
    }
}

pub struct SimulationState {
    pub is_running: Arc<AtomicBool>,
    pub is_enabled: Arc<AtomicBool>,
    pub sync_interval_ms: u64,
    pub cast_leeway_ms: u64,
    pub num_active_characters: usize,
    pub window_config: WindowConfig,
    pub rotation: Rotation,
    pub skill_tracker: SkillTracker,
    pub interactor: Box<dyn Interactor + Send + Sync>,
    pub state_checker: Box<dyn StateChecker + Send + Sync>,
    pub shared_state: Arc<Mutex<SharedState>>,
}

impl SimulationState {
    pub fn new(
        sync_interval_ms: u64,
        cast_leeway_ms: u64,
        num_active_characters: usize,
        window_config: WindowConfig,
        rotation: Rotation,
        skill_caster: Box<dyn Interactor + Send + Sync>,
        state_checker: Box<dyn StateChecker + Send + Sync>,
        shared_state: Arc<Mutex<SharedState>>,
    ) -> Self {
        SimulationState {
            is_running: Arc::new(AtomicBool::new(false)),
            is_enabled: Arc::new(AtomicBool::new(false)),
            sync_interval_ms,
            cast_leeway_ms,
            num_active_characters,
            window_config,
            rotation,
            skill_tracker: SkillTracker::new(shared_state.clone()),
            interactor: skill_caster,
            state_checker,
            shared_state,
        }
    }

    pub fn is_master(&self) -> bool {
        self.window_config.master
    }
    pub fn debug_checker(&self) {
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = self.is_running.clone();
        let is_enabled = self.is_enabled.clone();
        while is_running.load(Ordering::SeqCst) {
            if !is_enabled.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(self.sync_interval_ms));
                continue;
            }
            let state = self.state_checker.get_state(self.num_active_characters);
            if state == CharState::Looting {
                let quality = self.state_checker.get_loot_quality();
                if quality == LootQuality::Normal {
                    _ = self.state_checker.is_rune();
                }
            }
            thread::sleep(Duration::from_millis(self.sync_interval_ms));
        }
    }

    pub fn run(&self) {
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = self.is_running.clone();
        let is_enabled = self.is_enabled.clone();
        while is_running.load(Ordering::SeqCst) {
            if !is_enabled.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(self.sync_interval_ms));
                continue;
            }
            let mut skip_wait = false;
            let state = self.state_checker.get_state(self.num_active_characters);
            match state {
                CharState::Unknown => {
                    // do nothing
                }
                CharState::InTown => {
                    self.skill_tracker.reset();
                }
                _ => {
                    if state == CharState::AtShrine && self.interactor.interact() {
                        println!("Interacted with a shrine");
                    }
                    if state == CharState::Looting {
                        println!("Initiate looting...");
                        loop {
                            //keep looting until the state changes, or we failed to loot (needs manual intervention)
                            let looted = self.loot_cycle();
                            thread::sleep(Duration::from_millis(50));
                            let new_state =
                                self.state_checker.get_state(self.num_active_characters);
                            if !looted || new_state != CharState::Looting {
                                println!("Looting ended");
                                break;
                            }
                        }
                        skip_wait = true;
                    }
                    if state != CharState::Dead {
                        // try to cast - go through all skills, they are sorted by priority
                        self.rotation.skills.clone().into_iter().for_each(|skill| {
                            // if we can cast (or buff/debuff is down)
                            if self.skill_tracker.should_cast(&skill, state) {
                                if let Some(cast_all_skills) =
                                    &self.window_config.class_config.cast_all_skills
                                    && cast_all_skills.contains(&skill.name)
                                    && self.num_active_characters > 1
                                {
                                    // let's buff other players
                                    println!(
                                        "Initiating buff sequence for {} in a party of {}",
                                        skill.name, self.num_active_characters
                                    );
                                    for player_index in 0..self.num_active_characters {
                                        self.interactor.target_player(player_index);
                                        self.cast(&skill);
                                    }
                                    // re-target himself
                                    self.interactor.target_player(0);
                                } else {
                                    // try to cast a single spell
                                    self.cast(&skill);
                                }
                                // and track the cooldown
                                self.skill_tracker.track_cast(&skill);
                                skip_wait = true;
                            }
                        });
                    } else {
                        self.skill_tracker.reset();
                    }
                }
            }
            if !skip_wait {
                println!("Sync sleep for {} ms", self.sync_interval_ms);
                thread::sleep(Duration::from_millis(self.sync_interval_ms));
            }
            println!("Simulation cycle finished")
        }
    }

    fn loot_cycle(&self) -> bool {
        let quality = self.state_checker.get_loot_quality();
        if quality == LootQuality::Unknown {
            //could not figure out quality... cannot loot (needs a manual intervention)
            return false;
        }
        //we always loot runes
        if quality == LootQuality::Normal && self.state_checker.is_rune() {
            return self.interactor.loot();
        }
        //now loot according to the loot filter
        if self
            .window_config
            .class_config
            .loot_filter
            .contains(&quality)
        {
            self.interactor.loot()
        } else {
            self.interactor.discard()
        }
    }

    fn cast(&self, skill: &Skill) {
        let cast_result = self.interactor.cast_skill(skill);
        let cast_time = skill.cast_time(
            self.shared_state.clone(),
            self.window_config.class_config.class,
        );
        let ms = if cast_time > 0.0 {
            //let's wait for a cast time duration
            let ms = (cast_time * 1000.0) as u64;
            println!(
                "Casting for {} seconds with cast result: {}",
                cast_time, cast_result
            );
            ms
        } else if self
            .window_config
            .class_config
            .no_gcd_skills
            .clone()
            .is_some_and(|skills| skills.contains(&skill.name))
        {
            //no gcd skill
            println!("Casting non-GCD instant");
            0
        } else {
            let ms = (skill.get_gcd(
                self.shared_state.clone(),
                self.window_config.class_config.class,
            ) * 1000.0) as u64;
            println!(
                "Casting instant and waiting for GCD for {} seconds",
                skill.get_gcd(
                    self.shared_state.clone(),
                    self.window_config.class_config.class
                )
            );
            ms
        };
        thread::sleep(Duration::from_millis(ms + self.cast_leeway_ms));
    }

    pub fn enable_toggle(&self) {
        let prev = self.is_enabled.fetch_xor(true, Ordering::SeqCst);
        println!(
            "Enabled: {} for class: {:?}",
            !prev, self.window_config.class_config.class
        );
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        println!(
            "Stopping for class: {:?}",
            self.window_config.class_config.class
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{Class, Config};
    use crate::simulation::rotation::Rotations;
    use crate::simulation::shared_state::SharedState;
    use crate::simulation::{CharState, DebugObj, Rotation, SimulationState};
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_simulation() {
        let cfg = Config::default();

        let rotation = Rotation::get_rotation(Class::Enchanter, &cfg);

        let simulation = SimulationState::new(
            cfg.sync_interval_ms,
            0,
            1,
            cfg.windows.first().unwrap().clone(),
            rotation,
            Box::new(DebugObj::new(CharState::Fighting)),
            Box::new(DebugObj::new(CharState::Fighting)),
            Arc::new(Mutex::new(SharedState::new(
                cfg.skill_haste_percent,
                cfg.frenzy_haste_percent,
            ))),
        );

        simulation.enable_toggle();
        simulation.run();
    }
}
