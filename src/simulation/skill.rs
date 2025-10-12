use crate::configuration::config::Class;
use crate::simulation::char_state::CharState;
use crate::simulation::keys::Key;
use crate::simulation::shared_state::SharedStateHandle;
use crate::simulation::skill_type::SkillType;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub key: Key,
    pub cast_time: f32,
    pub cooldown: f32,
    pub buff_duration: Option<f32>,
    pub debuff_duration: Option<f32>,
    pub skill_type: SkillType,
}
const GCD: f32 = 2.5;

impl Skill {
    pub fn get_gcd(&self, shared_state: Arc<SharedStateHandle>, class: Class) -> f32 {
        GCD * self.get_haste_coef(shared_state, class)
    }

    pub fn get_cooldown(&self, reductions: Option<&Vec<(String, f32)>>) -> f32 {
        let reduction = self.calculate_reduction_coef(reductions);
        self.cooldown * reduction
    }

    pub fn can_cast(&self, state: CharState) -> bool {
        match state {
            CharState::InTown | CharState::Dead | CharState::Unknown => false,
            CharState::InDungeon | CharState::AtShrine => self.skill_type == SkillType::Buff,
            CharState::Fighting | CharState::Looting => true,
        }
    }

    pub fn cast_time(
        &self,
        shared_state: Arc<SharedStateHandle>,
        class: Class,
        reductions: Option<&Vec<(String, f32)>>,
    ) -> f32 {
        let reduction = self.calculate_reduction_coef(reductions);
        if self.cast_time == 0.0 {
            0.0
        } else {
            self.cast_time * reduction * self.get_haste_coef(shared_state, class)
        }
    }

    fn calculate_reduction_coef(&self, reductions: Option<&Vec<(String, f32)>>) -> f32 {
        if let Some(reductions) = reductions {
            reductions
                .iter()
                .find(|(k, _)| k == &self.name)
                .map_or_else(|| 1.0, |(_, v)| (100.0 - *v) / 100.0)
        } else {
            1.0
        }
    }

    fn get_haste_coef(&self, shared_state: Arc<SharedStateHandle>, class: Class) -> f32 {
        let mut coef = 1.0f32;
        if shared_state.get_skill_haste_applied() {
            coef = coef * (100.0 - shared_state.get_skill_haste_percent()) / 100.0;
        }
        // frenzy is Warrior only self-buff
        if shared_state.get_frenzy_applied() && class == Class::Warrior {
            coef = coef * (100.0 - shared_state.get_frenzy_percent()) / 100.0;
        }
        coef
    }
}

#[cfg(test)]
mod tests {
    use crate::configuration::class_config::{AutoAttack, ClassConfig};
    use crate::configuration::config::Class;
    use crate::simulation::keys::{SKILL_BUTTON_1, SKILL_BUTTON_4};
    use crate::simulation::shared_state::SharedStateHandle;
    use crate::simulation::skill::Skill;
    use crate::simulation::skill_type::SkillType;
    use std::sync::Arc;

    #[test]
    fn test_gcd_no_skill_haste() {
        let (skill, shared_state) = get_gcd_skill(10f32, 20f32, false, false);
        let coef = skill.get_haste_coef(shared_state.clone(), Class::Warrior);
        let cast_time = skill.cast_time(shared_state.clone(), Class::Warrior, None);
        let gcd = skill.get_gcd(shared_state.clone(), Class::Warrior);
        assert(coef, 1.0);
        assert(cast_time, 0.0);
        assert(gcd, 2.5);
        shared_state.stop();
    }

    #[test]
    fn test_gcd_skill_haste() {
        let (skill, shared_state) = get_gcd_skill(10f32, 20f32, true, false);
        let coef = skill.get_haste_coef(shared_state.clone(), Class::Warrior);
        let cast_time = skill.cast_time(shared_state.clone(), Class::Warrior, None);
        let gcd = skill.get_gcd(shared_state.clone(), Class::Warrior);
        assert(coef, 0.9);
        assert(cast_time, 0.0);
        assert(gcd, 2.25);
        shared_state.stop();
    }

    #[test]
    fn test_gcd_skill_frenzy_no_warrior_haste() {
        let (skill, shared_state) = get_gcd_skill(10f32, 20f32, true, true);
        let coef = skill.get_haste_coef(shared_state.clone(), Class::Enchanter);
        let cast_time = skill.cast_time(shared_state.clone(), Class::Enchanter, None);
        let gcd = skill.get_gcd(shared_state.clone(), Class::Enchanter);
        assert(coef, 0.9);
        assert(cast_time, 0.0);
        assert(gcd, 2.25);
        shared_state.stop();
    }

    #[test]
    fn test_gcd_skill_frenzy_warrior_haste() {
        let (skill, shared_state) = get_gcd_skill(10f32, 20f32, true, true);
        let coef = skill.get_haste_coef(shared_state.clone(), Class::Warrior);
        let cast_time = skill.cast_time(shared_state.clone(), Class::Warrior, None);
        let gcd = skill.get_gcd(shared_state.clone(), Class::Warrior);
        assert(coef, 0.72);
        assert(cast_time, 0.0);
        assert(gcd, 1.8);
        shared_state.stop();
    }

    #[test]
    fn test_gcd_skill_frenzy_no_haste_warrior_haste() {
        let (skill, shared_state) = get_gcd_skill(10f32, 20f32, false, true);
        let coef = skill.get_haste_coef(shared_state.clone(), Class::Warrior);
        let cast_time = skill.cast_time(shared_state.clone(), Class::Warrior, None);
        let gcd = skill.get_gcd(shared_state.clone(), Class::Warrior);
        assert(coef, 0.8);
        assert(cast_time, 0.0);
        assert(gcd, 2.0);
        shared_state.stop();
    }

    #[test]
    fn test_no_skill_haste() {
        let (skill, shared_state) = get_long_cast_skill(10f32, 20f32, false, false);
        let coef = skill.get_haste_coef(shared_state.clone(), Class::Warrior);
        let cast_time = skill.cast_time(shared_state.clone(), Class::Warrior, None);
        assert(coef, 1.0);
        assert(cast_time, 2.5);
        shared_state.stop();
    }

    #[test]
    fn test_skill_haste() {
        let (skill, shared_state) = get_long_cast_skill(10f32, 20f32, true, false);
        let coef = skill.get_haste_coef(shared_state.clone(), Class::Warrior);
        let cast_time = skill.cast_time(shared_state.clone(), Class::Warrior, None);
        assert(coef, 0.9);
        assert(cast_time, 2.25);
        shared_state.stop();
    }

    #[test]
    fn test_skill_frenzy_no_warrior_haste() {
        let (skill, shared_state) = get_long_cast_skill(10f32, 20f32, true, true);
        let coef = skill.get_haste_coef(shared_state.clone(), Class::Enchanter);
        let cast_time = skill.cast_time(shared_state.clone(), Class::Enchanter, None);
        assert(coef, 0.9);
        assert(cast_time, 2.25);
        shared_state.stop();
    }

    #[test]
    fn test_skill_frenzy_warrior_haste() {
        let (skill, shared_state) = get_long_cast_skill(10f32, 20f32, true, true);
        let coef = skill.get_haste_coef(shared_state.clone(), Class::Warrior);
        let cast_time = skill.cast_time(shared_state.clone(), Class::Warrior, None);
        assert(coef, 0.72);
        assert(cast_time, 1.8);
        shared_state.stop();
    }

    #[test]
    fn test_skill_frenzy_no_haste_warrior_haste() {
        let (skill, shared_state) = get_long_cast_skill(10f32, 20f32, false, true);
        let coef = skill.get_haste_coef(shared_state.clone(), Class::Warrior);
        let cast_time = skill.cast_time(shared_state.clone(), Class::Warrior, None);
        assert(coef, 0.8);
        assert(cast_time, 2.0);
        shared_state.stop();
    }

    #[test]
    fn test_skill_reduction() {
        let shared_state = Arc::new(SharedStateHandle::new(1.0, 1.0));
        let skill = Skill {
            name: "Engulfing Darkness".to_string(),
            key: SKILL_BUTTON_1,
            cast_time: 5.0,
            cooldown: 45.0,
            buff_duration: None,
            debuff_duration: Some(18.0),
            skill_type: SkillType::Debuff,
        };

        let mut class_config = ClassConfig::new(
            Class::Warlock,
            None,
            None,
            Some(vec![("Engulfing Darkness".to_string(), 49.0)]),
            Some(vec![("Engulfing Darkness".to_string(), 50.0)]),
            vec![],
            AutoAttack::Primary,
        );

        assert(
            skill.get_cooldown(class_config.cd_reductions.as_ref()),
            22.95,
        );

        assert(
            skill.cast_time(
                shared_state.clone(),
                Class::Enchanter,
                class_config.cast_time_reductions.as_ref(),
            ),
            2.5,
        );

        class_config = ClassConfig::new(
            Class::Warlock,
            None,
            None,
            None,
            None,
            vec![],
            AutoAttack::Primary,
        );

        assert(
            skill.get_cooldown(class_config.cd_reductions.as_ref()),
            45.0,
        );

        assert(
            skill.cast_time(
                shared_state.clone(),
                Class::Enchanter,
                class_config.cast_time_reductions.as_ref(),
            ),
            5.0,
        );

        shared_state.stop();
    }

    fn assert(a: f32, b: f32) {
        let epsilon = 1e-5;
        assert!((a - b).abs() < epsilon, "a: {}, b: {}", a, b);
    }
    fn get_gcd_skill(
        skill_haste: f32,
        frenzy_haste: f32,
        enable_haste: bool,
        enable_frenzy: bool,
    ) -> (Skill, Arc<SharedStateHandle>) {
        let skill = Skill {
            name: "Rupture".to_string(),
            key: SKILL_BUTTON_4,
            cast_time: 0.0,
            cooldown: 0.0,
            buff_duration: None,
            debuff_duration: Some(18.0),
            skill_type: SkillType::Debuff,
        };
        let state = SharedStateHandle::new(skill_haste, frenzy_haste);
        state.set_skill_haste_applied(enable_haste);
        state.set_frenzy_applied(enable_frenzy);
        (skill, Arc::new(state))
    }

    fn get_long_cast_skill(
        skill_haste: f32,
        frenzy_haste: f32,
        enable_haste: bool,
        enable_frenzy: bool,
    ) -> (Skill, Arc<SharedStateHandle>) {
        let skill = Skill {
            name: "Mind Blitz".to_string(),
            key: SKILL_BUTTON_4,
            cast_time: 2.5,
            cooldown: 0.0,
            buff_duration: None,
            debuff_duration: None,
            skill_type: SkillType::Attack,
        };
        let state = SharedStateHandle::new(skill_haste, frenzy_haste);
        state.set_skill_haste_applied(enable_haste);
        state.set_frenzy_applied(enable_frenzy);
        (skill, Arc::new(state))
    }
}
