use crate::configuration::config::Class;
use crate::simulation::loot::{LootQuality, LootTier};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ClassConfig {
    pub class: Class,
    pub cast_all_skills: Option<Vec<String>>,
    pub no_gcd_skills: Option<Vec<String>>,
    pub cd_reductions: Option<Vec<(String, f32)>>,
    pub cast_time_reductions: Option<Vec<(String, f32)>>,
    pub loot_filter: Vec<LootFilterItem>,
    pub auto_attack: AutoAttack,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum AutoAttack {
    Primary,
    Ranged,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct LootFilterItem(pub LootQuality, pub LootTier);

impl ClassConfig {
    pub fn new(
        class: Class,
        cast_all_skills: Option<Vec<String>>,
        no_gcd_skills: Option<Vec<String>>,
        cd_reductions: Option<Vec<(String, f32)>>,
        cast_time_reductions: Option<Vec<(String, f32)>>,
        loot_filter: Vec<LootFilterItem>,
        auto_attack: AutoAttack,
    ) -> ClassConfig {
        ClassConfig {
            class,
            cast_all_skills,
            no_gcd_skills,
            cd_reductions,
            cast_time_reductions,
            loot_filter,
            auto_attack,
        }
    }
}
