use crate::config::Class;
use crate::simulation::loot::LootQuality;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ClassConfig {
    pub class: Class,
    pub cast_all_skills: Option<Vec<String>>,
    pub loot_filter: Vec<LootQuality>,
}

impl ClassConfig {
    pub fn new(
        class: Class,
        cast_all_skills: Option<Vec<String>>,
        loot_filter: Vec<LootQuality>,
    ) -> ClassConfig {
        ClassConfig {
            class,
            cast_all_skills,
            loot_filter,
        }
    }
}
