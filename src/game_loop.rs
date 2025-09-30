use crate::config::{Class, Config};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_0, VK_1, VK_2, VK_3, VK_4, VK_9, VK_OEM_MINUS, VK_OEM_NEC_EQUAL,
};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum CharState {
    InTown,
    InDungeon,
    InFight,
    Looting,
    Death,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum SkillType {
    Attack,
    Buff,
    Debuff,
}

const GCD: f32 = 2.5;

#[derive(Debug)]
pub struct Skill {
    name: String,
    key: VIRTUAL_KEY,
    cast_time: f32,
    cooldown: f32,
    buff_duration: Option<f32>,
    debuff_duration: Option<f32>,
    skill_type: SkillType,
}

impl Skill {
    fn has_gcd(&self) -> bool {
        self.cast_time == 0.0
    }

    fn get_gcd(&self) -> f32 {
        GCD
    }

    fn can_be_used(&self, state: CharState) -> bool {
        match state {
            CharState::InTown | CharState::Death => false,
            CharState::InDungeon => self.skill_type == SkillType::Buff,
            CharState::InFight | CharState::Looting => true,
        }
    }
}

#[derive(Debug)]
struct Rotation {
    skills: Vec<Skill>,
}

trait Rotations<T> {
    fn get_rotation(input: &T, cfg: &Config) -> Rotation;
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
    fn get_rotation(input: &Class, cfg: &Config) -> Rotation {
        match input {
            Class::Warrior => Rotation { skills: vec![] },
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
                        key: VK_OEM_NEC_EQUAL,
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
