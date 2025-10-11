use std::any::type_name;

pub mod char_state;
pub mod global_lock;
pub mod interactor;
pub mod keys;
pub mod loot;
pub mod rotation;
pub mod shared_state;
pub mod simulation_state;
pub mod skill;
pub mod skill_tracker;
pub mod skill_type;
pub mod state_checker;

pub fn type_of<T>(_: &T) -> &'static str {
    type_name::<T>()
}
