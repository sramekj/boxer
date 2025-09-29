use crate::config::{Class, Config};
use windows::Win32::UI::Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_0, VK_2, VK_3, VK_4, VK_9};

#[derive(Debug)]
enum Phase {
    InTown,
    InDungeon,
    InFight,
    Looting,
}

const GCD: f32 = 2.5;

#[derive(Debug)]
pub struct Skill {
    name: String,
    key: VIRTUAL_KEY,
    cast_time: f32,
    has_gcd: bool,
    cooldown: f32,
    buff_duration: Option<f32>,
    is_buff: bool,
}

#[derive(Debug)]
struct Rotation {
    skills: Vec<Skill>,
}

trait Rotations<T> {
    fn get_rotation(input: &T, cfg: &Config) -> Rotation;
}

fn calculate_haste_coef(cfg: &Config) -> f32 {
    if cfg.has_enchanter {
        return (100.0 - cfg.skill_haste_percent) / 100.0;
    }
    1.0
}

impl Rotations<Class> for Rotation {
    fn get_rotation(input: &Class, cfg: &Config) -> Rotation {
        match input {
            Class::Warrior => Rotation { skills: vec![] },
            Class::Warlock => Rotation { skills: vec![] },
            Class::Enchanter => Rotation {
                skills: vec![
                    Skill {
                        name: "Augmentation".to_string(),
                        key: VK_9,
                        cast_time: 2.0 * calculate_haste_coef(cfg),
                        has_gcd: false,
                        cooldown: 0.0,
                        buff_duration: Some(480.0),
                        is_buff: true,
                    },
                    Skill {
                        name: "Phase Blade".to_string(),
                        key: VK_0,
                        cast_time: 2.0 * calculate_haste_coef(cfg),
                        has_gcd: false,
                        cooldown: 0.0,
                        buff_duration: Some(600.0),
                        is_buff: true,
                    },
                    Skill {
                        name: "Color Shift".to_string(),
                        key: VK_2,
                        cast_time: 1.5 * calculate_haste_coef(cfg),
                        has_gcd: false,
                        cooldown: 30.0,
                        buff_duration: None,
                        is_buff: false,
                    },
                    Skill {
                        name: "Enthrall".to_string(),
                        key: VK_3,
                        cast_time: 3.0 * calculate_haste_coef(cfg),
                        has_gcd: false,
                        cooldown: 12.0,
                        buff_duration: None,
                        is_buff: false,
                    },
                    Skill {
                        name: "Mind Blitz".to_string(),
                        key: VK_4,
                        cast_time: 2.5 * calculate_haste_coef(cfg),
                        has_gcd: false,
                        cooldown: 0.0,
                        buff_duration: None,
                        is_buff: false,
                    },
                ],
            },
        }
    }
}
