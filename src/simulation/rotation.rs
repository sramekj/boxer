use crate::config::{Class, Config};
use crate::simulation::skill::Skill;
use crate::simulation::skill_type::SkillType;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_9, VK_OEM_MINUS, VK_OEM_PLUS,
};

#[derive(Debug, Clone)]
pub struct Rotation {
    pub skills: Vec<Skill>,
}

pub trait Rotations<T> {
    fn get_rotation(input: T, cfg: &Config) -> Rotation;
}

impl Rotations<Class> for Rotation {
    fn get_rotation(input: Class, cfg: &Config) -> Rotation {
        match input {
            Class::Warrior => Rotation {
                skills: vec![
                    Skill {
                        name: "Intrepid Shout".to_string(),
                        key: VK_OEM_PLUS,
                        cast_time: 0.0,
                        cooldown: 0.0,
                        buff_duration: Some(240.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Frenzy".to_string(),
                        key: VK_5,
                        cast_time: 0.0,
                        cooldown: 60.0,
                        buff_duration: Some(cfg.frenzy_duration),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Bulwark".to_string(),
                        key: VK_OEM_MINUS,
                        cast_time: 0.0,
                        cooldown: 40.0,
                        buff_duration: Some(cfg.bulwark_duration),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Rupture".to_string(),
                        key: VK_4,
                        cast_time: 0.0,
                        cooldown: 0.0,
                        buff_duration: None,
                        debuff_duration: Some(18.0),
                        skill_type: SkillType::Debuff,
                    },
                    Skill {
                        name: "Double Throw".to_string(),
                        key: VK_2,
                        cast_time: 0.0,
                        cooldown: 20.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                    Skill {
                        name: "Furious Cleave".to_string(),
                        key: VK_1,
                        cast_time: 0.0,
                        cooldown: 16.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                    Skill {
                        name: "Rupture".to_string(),
                        key: VK_4,
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
                        key: VK_OEM_MINUS,
                        cast_time: 3.0,
                        cooldown: 0.0,
                        buff_duration: Some(720.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Profane Spirit".to_string(),
                        key: VK_OEM_PLUS,
                        cast_time: 2.5,
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
                        key: VK_9,
                        cast_time: 2.0,
                        cooldown: 0.0,
                        buff_duration: Some(480.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Phase Blade".to_string(),
                        key: VK_0,
                        cast_time: 2.0,
                        cooldown: 0.0,
                        buff_duration: Some(600.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Clarity".to_string(),
                        key: VK_OEM_MINUS,
                        cast_time: 2.5,
                        cooldown: 0.0,
                        buff_duration: Some(720.0),
                        debuff_duration: None,
                        skill_type: SkillType::Buff,
                    },
                    Skill {
                        name: "Color Shift".to_string(),
                        key: VK_2,
                        cast_time: 1.5,
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
                        cast_time: 3.0,
                        cooldown: 12.0,
                        buff_duration: None,
                        debuff_duration: None,
                        skill_type: SkillType::Attack,
                    },
                    Skill {
                        name: "Mind Blitz".to_string(),
                        key: VK_4,
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
