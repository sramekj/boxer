use crate::configuration::class_config::LootFilterItem;
use crate::configuration::config::WindowConfig;
use crate::simulation::char_state::CharState;
use crate::simulation::interactor::Interactor;
use crate::simulation::loot::{LootQuality, LootTier};
use crate::simulation::maze_solver::{Node, Pos, Solver};
use crate::simulation::rotation::Rotation;
use crate::simulation::shared_state::SharedStateHandle;
use crate::simulation::skill::Skill;
use crate::simulation::skill_tracker::{DEBUG_COOLDOWNS, SkillTrackerHandle};
use crate::simulation::state_checker::{StateChecker, get_move_pixel};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::HWND;

pub struct DebugObj {
    pub test_state: CharState,
    pub test_map: Arc<Mutex<HashMap<Pos, Node>>>,
    pub position_x: AtomicI32,
    pub position_y: AtomicI32,
}

impl DebugObj {
    pub fn new(
        test_state: CharState,
        test_map: Arc<Mutex<HashMap<Pos, Node>>>,
        position_x: AtomicI32,
        position_y: AtomicI32,
    ) -> DebugObj {
        DebugObj {
            test_state,
            test_map,
            position_x,
            position_y,
        }
    }
}

pub struct WindowObj {
    pub hwnd: Option<HWND>,
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
    pub leave_when_full: bool,
    pub auto_explore: bool,
    pub skill_tracker: SkillTrackerHandle,
    pub interactor: Box<dyn Interactor + Send + Sync>,
    pub state_checker: Box<dyn StateChecker + Send + Sync>,
    pub shared_state: Arc<SharedStateHandle>,
    pub maze_solver: Solver,
}

impl SimulationState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sync_interval_ms: u64,
        cast_leeway_ms: u64,
        num_active_characters: usize,
        window_config: WindowConfig,
        rotation: Rotation,
        leave_when_full: bool,
        auto_explore: bool,
        skill_caster: Box<dyn Interactor + Send + Sync>,
        state_checker: Box<dyn StateChecker + Send + Sync>,
        shared_state: Arc<SharedStateHandle>,
        maze_solver: Solver,
    ) -> Self {
        SimulationState {
            is_running: Arc::new(AtomicBool::new(false)),
            is_enabled: Arc::new(AtomicBool::new(false)),
            sync_interval_ms,
            cast_leeway_ms,
            num_active_characters,
            window_config: window_config.clone(),
            rotation,
            leave_when_full,
            auto_explore,
            skill_tracker: SkillTrackerHandle::new(shared_state.clone(), DEBUG_COOLDOWNS),
            interactor: skill_caster,
            state_checker,
            shared_state,
            maze_solver,
        }
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
                _ = self.state_checker.get_loot_quality();
                _ = self.state_checker.get_loot_tier();
            }
            thread::sleep(Duration::from_millis(self.sync_interval_ms));
        }
    }

    pub fn reset(&self) {
        //let's assume we don't have any buffs
        self.shared_state.set_skill_haste_applied(false);
        self.shared_state.set_frenzy_applied(false);
        //and non-full inventory -> this should eventually get autocorrected later
        self.shared_state.set_full_inventory(false);
        self.skill_tracker.reset();
    }

    pub fn run(&self) {
        self.is_running.store(true, Ordering::SeqCst);
        let is_running = self.is_running.clone();
        let is_enabled = self.is_enabled.clone();
        let mut auto_attacking = false;
        let mut prev_state: CharState = CharState::Unknown;
        while is_running.load(Ordering::SeqCst) {
            if !is_enabled.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(self.sync_interval_ms));
                continue;
            }
            let mut skip_wait = false;

            let state = self.state_checker.get_state(self.num_active_characters);
            let state_check_at = Instant::now();

            // did we recently die or left town?
            let need_reset_states = [CharState::InTown, CharState::Dead];
            if need_reset_states.contains(&prev_state) && !need_reset_states.contains(&state) {
                self.skill_tracker.reset();
                self.maze_solver.reset();
                auto_attacking = false;
            }

            // we need to open inventory if it is not opened (it's needed for inventory checks during looting)
            if !self.state_checker.is_inventory_opened() && state != CharState::Unknown {
                self.interactor.inventory_toggle();
            }

            match state {
                CharState::Unknown | CharState::InTown => {
                    // do nothing
                }
                _ => {
                    if self.left_combat(prev_state, state) {
                        //we should track debuffs only during fight, otherwise it would block possible casts
                        self.skill_tracker.reset_debuffs();
                    }

                    if state == CharState::InDungeon
                        && self.state_checker.is_inventory_opened()
                        && self.state_checker.is_inventory_full()
                    {
                        self.shared_state.set_full_inventory(true);
                    }

                    //leave to town if any of the party has full inventory... no point of farming
                    if state == CharState::InDungeon
                        && self.shared_state.get_full_inventory()
                        //only master can leave dungeon
                        && self.window_config.master
                        && self.leave_when_full
                        && self.interactor.leave_to_town()
                    {
                        // let's assume we clear the inventory in a town... so wait and clear shared state
                        thread::sleep(Duration::from_millis(1000));
                        self.shared_state.set_full_inventory(false);
                    }

                    self.loot_shrine(state);

                    if state == CharState::Looting && !self.state_checker.is_inventory_full() {
                        println!("Initiate looting...");
                        let mut loot_counter = 0;
                        loop {
                            //keep looting until the state changes, or we failed to loot (needs manual intervention)
                            let looted = self.loot_cycle();
                            loot_counter += 1;
                            let new_state =
                                self.state_checker.get_state(self.num_active_characters);
                            //let's break if we go over 10 attempts - we might be hung-up because of unknown loot quality check
                            if !looted
                                || new_state != CharState::Looting
                                || self.state_checker.is_inventory_full()
                                || loot_counter > 10
                            {
                                println!("Looting ended");
                                skip_wait = true;
                                break;
                            }
                        }
                    }

                    if [CharState::Fighting, CharState::InDungeon].contains(&state) {
                        if self.entered_combat(prev_state, state) {
                            //wait if we have just started fighting... otherwise the first cast may not go off
                            thread::sleep(Duration::from_millis(500));
                        }
                        if !auto_attacking && state == CharState::Fighting {
                            // auto-attack just once
                            auto_attacking = self
                                .interactor
                                .auto_attack(self.window_config.class_config.auto_attack);
                        }

                        self.do_rotation(state, state_check_at, skip_wait);
                    }

                    if self.process_movement() {
                        skip_wait = true;
                    }
                }
            }
            prev_state = state;
            if !skip_wait {
                println!("Sync sleep for {} ms", self.sync_interval_ms);
                thread::sleep(Duration::from_millis(self.sync_interval_ms));
            }
        }
    }

    fn process_movement(&self) -> bool {
        if self.is_auto_explore_enabled() {
            //rotations and looting can take quite some time... lets update the state before moving
            let updated_state = self.state_checker.get_state(self.num_active_characters);
            if self.can_walk(updated_state) {
                //let's wait a bit in case we have just left the combat or graphics did not load, otherwise the autowalk may not go off
                thread::sleep(Duration::from_millis(500));
                //try to resume walking when in dungeon
                self.interactor.walk(None);
                thread::sleep(Duration::from_millis(100));
                if self.has_recently_moved() {
                    //let's move until we are stationary
                    loop {
                        if self.is_stationary() {
                            println!("Just stopped...");
                            break;
                        }
                        thread::sleep(Duration::from_millis(300));
                    }
                }
            }

            //rotations and looting can take quite some time... lets update the state before moving
            let updated_state = self.state_checker.get_state(self.num_active_characters);
            if self.can_move_trigger(updated_state) {
                println!("Trying to auto-explore");
                // trigger the move step only when stationary
                let everything_explored = self.maze_solver.explore_step();
                if everything_explored {
                    println!("Everything explored");
                    self.interactor.leave_to_town();
                }
                thread::sleep(Duration::from_millis(100));
                if self.has_recently_moved() {
                    //let's move until we are stationary
                    loop {
                        if self.is_stationary() {
                            println!("Just stopped...");
                            break;
                        }
                        thread::sleep(Duration::from_millis(300));
                    }
                }
            }
            return true;
        }
        false
    }

    fn loot_shrine(&self, state: CharState) {
        if state == CharState::AtShrine && self.interactor.interact() {
            println!("Interacted with a shrine");
        }
    }

    fn is_auto_explore_enabled(&self) -> bool {
        self.window_config.master && self.auto_explore
    }

    fn can_walk(&self, state: CharState) -> bool {
        state == CharState::InDungeon && self.is_auto_explore_enabled()
    }

    fn can_move_trigger(&self, state: CharState) -> bool {
        // always take a fresh state when deciding if to move
        if (state == CharState::InDungeon || state == CharState::AtShrine)
            && self.is_auto_explore_enabled()
            && self.is_stationary()
        {
            // let's make really sure we are still in a dungeon and not fighting... this crap is not very reliable and will screw with a maze map
            thread::sleep(Duration::from_millis(200));
            let updated_state = self.state_checker.get_state(self.num_active_characters);
            if updated_state == CharState::InDungeon || updated_state == CharState::AtShrine {
                // let's not forget to loot a shrine if it is there
                self.loot_shrine(updated_state);
                return true;
            }
        }
        false
    }

    fn has_recently_moved(&self) -> bool {
        let px_before = get_move_pixel(self.window_config.hwnd);
        thread::sleep(Duration::from_millis(100));
        let px_after = get_move_pixel(self.window_config.hwnd);
        px_after != px_before
    }

    fn is_stationary(&self) -> bool {
        !self.has_recently_moved()
    }

    fn entered_combat(&self, prev_state: CharState, state: CharState) -> bool {
        prev_state != CharState::Fighting && state == CharState::Fighting
    }

    fn left_combat(&self, prev_state: CharState, state: CharState) -> bool {
        prev_state == CharState::Fighting && state != CharState::Fighting
    }

    fn do_rotation(&self, state: CharState, state_check_at: Instant, mut skip_wait: bool) {
        // try to cast - go through all skills, they are sorted by priority
        self.rotation.skills.clone().into_iter().for_each(|skill| {
            //make sure we did not die inside a long rotation
            let mut updated_state = state;
            if (Instant::now() - state_check_at) > Duration::from_secs(1) {
                updated_state = self.state_checker.get_state(self.num_active_characters);
            }

            // we should try to use a potion if on low HP if it is not on a cooldown
            if updated_state == CharState::Fighting
                && self.state_checker.is_on_low_hp(self.num_active_characters)
                && !self.skill_tracker.is_hp_pot_on_cooldown()
            {
                self.interactor.use_hp_pot();
                self.skill_tracker.track_hp_pot();
            }

            // if we can cast (or buff/debuff is down)
            if self.skill_tracker.should_cast(
                &skill,
                self.window_config.class_config.cd_reductions.as_ref(),
                updated_state,
            ) {
                if let Some(cast_all_skills) = &self.window_config.class_config.cast_all_skills
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
                        //track only self-cast the cooldown
                        if player_index == 0 {
                            self.skill_tracker.track_cast(
                                &skill,
                                self.window_config.class_config.cd_reductions.as_ref(),
                            );
                        }
                    }
                    // re-target himself
                    self.interactor.target_player(0);
                } else {
                    // try to cast a single spell
                    self.cast(&skill);
                    // and track the cooldown
                    self.skill_tracker.track_cast(
                        &skill,
                        self.window_config.class_config.cd_reductions.as_ref(),
                    );
                }
                skip_wait = true;
            }
        });
    }

    fn loot_cycle(&self) -> bool {
        let quality = self.state_checker.get_loot_quality();
        if quality == LootQuality::Unknown {
            //could not figure out quality... cannot loot (needs a manual intervention)
            return false;
        }
        let tier = self.state_checker.get_loot_tier();
        if tier == LootTier::Unknown {
            //could not figure out tier... cannot loot (needs a manual intervention)
            return false;
        }

        //now loot according to the loot filter
        if self
            .window_config
            .class_config
            .loot_filter
            .contains(&LootFilterItem(quality, tier))
        {
            self.interactor.loot()
        } else {
            self.interactor.discard()
        }
    }

    fn ceil_to_two_decimal_places(val: f32) -> f32 {
        (val * 100.0).ceil() / 100.0
    }
    fn cast(&self, skill: &Skill) {
        if !self.interactor.cast_skill(skill) {
            eprintln!("Couldn't cast skill {}", skill.name);
        }
        let cast_time = skill.cast_time(
            self.shared_state.clone(),
            self.window_config.class_config.class,
            self.window_config
                .class_config
                .cast_time_reductions
                .as_ref(),
        );
        let ms = if cast_time > 0.0 {
            //let's wait for a cast time duration
            (Self::ceil_to_two_decimal_places(cast_time) * 1000.0) as u64
        } else if self
            .window_config
            .class_config
            .no_gcd_skills
            .clone()
            .is_some_and(|skills| skills.contains(&skill.name))
        {
            //no gcd skill
            0
        } else {
            (skill.get_gcd(
                self.shared_state.clone(),
                self.window_config.class_config.class,
            ) * 1000.0) as u64
        };
        thread::sleep(Duration::from_millis(ms + self.cast_leeway_ms));
        println!(
            " and it took {}s",
            Self::ceil_to_two_decimal_places(ms as f32 / 1000.0)
        );
    }

    pub fn enable_toggle(&self) {
        let prev = self.is_enabled.fetch_xor(true, Ordering::SeqCst);
        println!(
            "{:?} enabled: {}",
            self.window_config.class_config.class, !prev
        );
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        println!("Stopping {:?} ", self.window_config.class_config.class);
    }
}

#[cfg(test)]
mod tests {
    use crate::amtx;
    use crate::configuration::config::{Class, Config};
    use crate::simulation::char_state::CharState::Fighting;
    use crate::simulation::maze_solver::Solver;
    use crate::simulation::rotation::Rotation;
    use crate::simulation::shared_state::SharedStateHandle;
    use crate::simulation::simulation_state::{DebugObj, SimulationState};
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    #[ignore]
    fn test_simulation() {
        let cfg = Config::default();

        let rotation = Rotation::load_rotation(Class::Enchanter);

        let simulation = SimulationState::new(
            cfg.sync_interval_ms,
            0,
            1,
            cfg.windows.first().unwrap().clone(),
            rotation,
            false,
            false,
            Box::new(DebugObj::new(
                Fighting,
                amtx!(HashMap::new()),
                0.into(),
                0.into(),
            )),
            Box::new(DebugObj::new(
                Fighting,
                amtx!(HashMap::new()),
                0.into(),
                0.into(),
            )),
            Arc::new(SharedStateHandle::new(
                cfg.skill_haste_percent,
                cfg.frenzy_haste_percent,
            )),
            Solver::new(Box::new(DebugObj::new(
                Fighting,
                amtx!(HashMap::new()),
                0.into(),
                0.into(),
            ))),
        );

        simulation.enable_toggle();
        simulation.run();
    }
}
