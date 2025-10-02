pub mod char_state;
pub mod rotation;
mod skill;
mod skill_caster;
mod skill_tracker;
mod skill_type;
mod state_checker;

use crate::config::WindowConfig;
pub(crate) use crate::simulation::char_state::CharState;
pub(crate) use crate::simulation::rotation::Rotation;
use crate::simulation::skill_caster::SkillCaster;
use crate::simulation::skill_tracker::SkillTracker;
use crate::simulation::state_checker::StateChecker;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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

impl WindowObj {
    pub fn new(hwnd: Option<HWND>) -> WindowObj {
        Self { hwnd }
    }
}

pub struct SimulationState {
    pub is_running: Arc<AtomicBool>,
    pub is_enabled: Arc<AtomicBool>,
    pub sync_interval_ms: u64,
    pub window_config: WindowConfig,
    pub rotation: Rotation,
    pub skill_tracker: SkillTracker,
    pub skill_caster: Box<dyn SkillCaster + Send + Sync>,
    pub state_checker: Box<dyn StateChecker + Send + Sync>,
}

impl SimulationState {
    pub fn new(
        sync_interval_ms: u64,
        window_config: WindowConfig,
        rotation: Rotation,
        skill_caster: Box<dyn SkillCaster + Send + Sync>,
        state_checker: Box<dyn StateChecker + Send + Sync>,
    ) -> Self {
        SimulationState {
            is_running: Arc::new(AtomicBool::new(false)),
            is_enabled: Arc::new(AtomicBool::new(false)),
            sync_interval_ms,
            window_config,
            rotation,
            skill_tracker: SkillTracker::new(),
            skill_caster,
            state_checker,
        }
    }

    pub fn run(&mut self) {
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = self.is_running.clone();
        let is_enabled = self.is_enabled.clone();
        while is_running.load(Ordering::SeqCst) {
            if !is_enabled.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(self.sync_interval_ms));
                continue;
            }
            let mut casted = false;
            let mut looted = false;
            let state = self.state_checker.get_state();
            match state {
                CharState::InTown => {
                    // do nothing
                }
                _ => {
                    // try to cast - go through all skills, they are sorted by priority
                    self.rotation.skills.clone().into_iter().for_each(|skill| {
                        // if we can cast (or buff/debuff is down)
                        if self.skill_tracker.should_cast(&skill, state) {
                            // try to cast
                            self.skill_caster.cast(&skill);
                            let ms = if skill.cast_time > 0.0 {
                                //let's wait for a cast time duration
                                let ms = (skill.cast_time * 1000.0) as u64;
                                println!("Casting for {} seconds", skill.cast_time);
                                ms
                            } else {
                                let ms = (skill.get_gcd() * 1000.0) as u64;
                                println!(
                                    "Casting instant and waiting for GCD for {} seconds",
                                    skill.get_gcd()
                                );
                                ms
                            };
                            thread::sleep(Duration::from_millis(ms));
                            // and track the cooldown
                            self.skill_tracker.track_cast(&skill);
                            casted = true;
                        }
                    });
                    if state == CharState::Looting {
                        // TODO: implement looting

                        looted = true;
                    }
                }
            }
            if !casted && !looted {
                println!("Sync sleep for {} ms", self.sync_interval_ms);
                thread::sleep(Duration::from_millis(self.sync_interval_ms));
            }
            println!("Simulation cycle finished")
        }
    }

    pub fn enable_toggle(&self) {
        let prev = self.is_enabled.fetch_xor(true, Ordering::SeqCst);
        println!(
            "Enabled: {} for class: {:?}",
            !prev, self.window_config.class
        );
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        println!("Stopping for class: {:?}", self.window_config.class);
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{Class, Config};
    use crate::simulation::rotation::Rotations;
    use crate::simulation::{CharState, DebugObj, Rotation, SimulationState};

    #[test]
    fn test_simulation() {
        let cfg = Config::default();

        let rotation = Rotation::get_rotation(Class::Enchanter, &cfg);

        let mut simulation = SimulationState::new(
            cfg.sync_interval_ms,
            cfg.windows.first().unwrap().clone(),
            rotation,
            Box::new(DebugObj::new(CharState::Fighting)),
            Box::new(DebugObj::new(CharState::Fighting)),
        );

        simulation.enable_toggle();
        simulation.run();
    }
}
