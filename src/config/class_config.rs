use crate::config::Class;
use crate::simulation::loot::LootQuality;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ClassConfig {
    pub class: Class,
    pub cast_all_skills: Option<Vec<String>>,
    pub no_gcd_skills: Option<Vec<String>>,
    pub cd_reductions: Option<Vec<(String, f32)>>,
    pub loot_filter: Vec<LootQuality>,
    pub auto_attack: AutoAttack,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum AutoAttack {
    Primary,
    Ranged,
}

impl ClassConfig {
    pub fn new(
        class: Class,
        cast_all_skills: Option<Vec<String>>,
        no_gcd_skills: Option<Vec<String>>,
        cd_reductions: Option<Vec<(String, f32)>>,
        loot_filter: Vec<LootQuality>,
        auto_attack: AutoAttack,
    ) -> ClassConfig {
        ClassConfig {
            class,
            cast_all_skills,
            no_gcd_skills,
            cd_reductions,
            loot_filter,
            auto_attack,
        }
    }
}
