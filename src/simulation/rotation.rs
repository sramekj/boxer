use crate::configuration::config::{Class, Config};
use crate::simulation::keys::{
    SKILL_BUTTON_1, SKILL_BUTTON_2, SKILL_BUTTON_3, SKILL_BUTTON_4, SKILL_BUTTON_5, SKILL_BUTTON_9,
    SKILL_BUTTON_10, SKILL_BUTTON_11, SKILL_BUTTON_12,
};
use crate::simulation::skill::Skill;
use crate::simulation::skill_type::SkillType;

#[derive(Debug, Clone)]
pub struct Rotation {
    pub skills: Vec<Skill>,
}

pub trait Rotations {
    fn get_rotation(input: Class, cfg: &Config) -> Rotation;
}

impl Rotations for Rotation {
    fn get_rotation(input: Class, cfg: &Config) -> Rotation {
        match input {
            Class::Warrior => Rotation {
                skills: vec![
                    Skill {
                        name: "Intrepid Shout".to_string(),
                        key: SKILL_BUTTON_12,
                        cast_time: 0.0,
                        cooldown: 0.0,
                        buff_duration: Some(240.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Frenzy".to_string(),
                        key: SKILL_BUTTON_5,
                        cast_time: 0.0,
                        cooldown: 60.0,
                        buff_duration: Some(cfg.frenzy_duration),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Bulwark".to_string(),
                        key: SKILL_BUTTON_11,
                        cast_time: 0.0,
                        cooldown: 40.0,
                        buff_duration: Some(cfg.bulwark_duration),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Rupture".to_string(),
                        key: SKILL_BUTTON_4,
                        cast_time: 0.0,
                        cooldown: 0.0,
                        buff_duration: None,
                        debuff_duration: Some(18.0),
                        skill_type: SkillType::Debuff,
                    },
                    Skill {
                        name: "Double Throw".to_string(),
                        key: SKILL_BUTTON_2,
                        cast_time: 0.0,
                        cooldown: 20.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                    Skill {
                        name: "Furious Cleave".to_string(),
                        key: SKILL_BUTTON_1,
                        cast_time: 0.0,
                        cooldown: 16.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                    Skill {
                        name: "Rupture".to_string(),
                        key: SKILL_BUTTON_4,
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
                        key: SKILL_BUTTON_11,
                        cast_time: 3.0,
                        cooldown: 0.0,
                        buff_duration: Some(720.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Profane Spirit".to_string(),
                        key: SKILL_BUTTON_12,
                        cast_time: 2.5,
                        cooldown: 0.0,
                        buff_duration: Some(900.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Engulfing Darkness".to_string(),
                        key: SKILL_BUTTON_1,
                        cast_time: 0.0,
                        cooldown: 45.0,
                        buff_duration: None,
                        debuff_duration: Some(18.0),
                        skill_type: SkillType::Debuff,
                    },
                    Skill {
                        name: "Explosive Plaque".to_string(),
                        key: SKILL_BUTTON_3,
                        cast_time: 0.0,
                        cooldown: 0.0,
                        buff_duration: None,
                        debuff_duration: Some(30.0),
                        skill_type: SkillType::Debuff,
                    },
                    Skill {
                        name: "Venom Bolt".to_string(),
                        key: SKILL_BUTTON_4,
                        cast_time: 3.0,
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
                        key: SKILL_BUTTON_9,
                        cast_time: 2.0,
                        cooldown: 0.0,
                        buff_duration: Some(480.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Phase Blade".to_string(),
                        key: SKILL_BUTTON_10,
                        cast_time: 2.25,
                        cooldown: 0.0,
                        buff_duration: Some(600.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Clarity".to_string(),
                        key: SKILL_BUTTON_11,
                        cast_time: 2.75,
                        cooldown: 0.0,
                        buff_duration: Some(720.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Color Shift".to_string(),
                        key: SKILL_BUTTON_2,
                        cast_time: 1.5,
                        cooldown: 30.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                    Skill {
                        name: "Static Suffocation".to_string(),
                        key: SKILL_BUTTON_1,
                        cast_time: 0.0,
                        cooldown: 6.0,
                        buff_duration: None,
                        debuff_duration: Some(18.0),
                        skill_type: SkillType::Debuff,
                    },
                    Skill {
                        name: "Enthrall".to_string(),
                        key: SKILL_BUTTON_3,
                        cast_time: 3.0,
                        cooldown: 12.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                    Skill {
                        name: "Mind Blitz".to_string(),
                        key: SKILL_BUTTON_4,
                        cast_time: 2.5,
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
