use crate::configuration::class_config::{AutoAttack, ClassConfig};
use crate::configuration::hex_hwnd;
use crate::simulation::loot::LootQuality;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::{env, fs};
use windows::Win32::Foundation::HWND;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short = 'd', long)]
    pub debug: bool,
    #[arg(short = 'm', long = "debug-mouse")]
    pub debug_mouse: bool,
    #[arg(short = 'l', long = "debug-line")]
    pub debug_line: bool,
    #[arg(long = "debug-interval", default_value = "1000")]
    pub debug_interval_ms: u64,
    #[arg(long = "debug-sim", default_value = "false")]
    pub debug_sim: bool,
    #[arg(short = 'c', long = "debug-checker", default_value = "false")]
    pub debug_checker: bool,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum Class {
    Enchanter,
    Warlock,
    Warrior,
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WindowConfig {
    pub title: Option<String>,
    #[serde(default, with = "hex_hwnd")]
    pub hwnd: Option<HWND>,
    pub window_width: i32,
    pub window_height: i32,
    pub position_x: i32,
    pub position_y: i32,
    pub active: bool,
    pub master: bool,
    pub class_config: ClassConfig,
}

unsafe impl Send for WindowConfig {}
unsafe impl Sync for WindowConfig {}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub skill_haste_percent: f32,
    pub frenzy_haste_percent: f32,
    pub sync_interval_ms: u64,
    pub cast_leeway_ms: u64,
    pub start_offset_ms: u64,
    pub windows: Vec<WindowConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            skill_haste_percent: 39.0,
            frenzy_haste_percent: 41.9,
            sync_interval_ms: 500,
            cast_leeway_ms: 0,
            start_offset_ms: 100,
            windows: vec![
                WindowConfig {
                    title: Some("[#] [Steam1] Nevergrind [#]".into()),
                    hwnd: None,
                    window_width: 1280,
                    window_height: 720,
                    position_x: 2560,
                    position_y: 0,
                    active: true,
                    master: false,
                    class_config: ClassConfig::new(
                        Class::Enchanter,
                        Some(vec!["Clarity".to_string()]),
                        None,
                        Some(vec![("Enthrall".to_string(), 42.9)]),
                        None,
                        vec![
                            LootQuality::Socketed,
                            LootQuality::Rare,
                            LootQuality::Epic,
                            LootQuality::Set,
                            LootQuality::Legendary,
                            LootQuality::Rune,
                        ],
                        AutoAttack::Primary,
                    ),
                },
                WindowConfig {
                    title: Some("[#] [Steam2] Nevergrind [#]".into()),
                    hwnd: None,
                    window_width: 1280,
                    window_height: 720,
                    position_x: 1280,
                    position_y: 0,
                    active: true,
                    master: false,
                    class_config: ClassConfig::new(
                        Class::Warlock,
                        None,
                        Some(vec!["Engulfing Darkness".to_string()]),
                        Some(vec![("Engulfing Darkness".to_string(), 87.9)]),
                        None,
                        vec![
                            LootQuality::Socketed,
                            LootQuality::Rare,
                            LootQuality::Epic,
                            LootQuality::Set,
                            LootQuality::Legendary,
                            LootQuality::Rune,
                        ],
                        AutoAttack::Primary,
                    ),
                },
                WindowConfig {
                    title: Some("Nevergrind".into()),
                    hwnd: None,
                    window_width: 1280,
                    window_height: 720,
                    position_x: 0,
                    position_y: 0,
                    active: true,
                    master: true,
                    class_config: ClassConfig::new(
                        Class::Warrior,
                        None,
                        Some(vec![
                            "Frenzy".to_string(),
                            "Bulwark".to_string(),
                            "Double Throw".to_string(),
                        ]),
                        None,
                        None,
                        vec![
                            LootQuality::Socketed,
                            LootQuality::Rare,
                            LootQuality::Epic,
                            LootQuality::Set,
                            LootQuality::Legendary,
                            LootQuality::Rune,
                        ],
                        AutoAttack::Primary,
                    ),
                },
            ],
        }
    }
}

const CFG_FILENAME: &str = "config.toml";
fn get_config_path(filename: &str) -> Option<PathBuf> {
    let exe_path = env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    Some(exe_dir.join(filename))
}
pub fn load_config() -> Config {
    let config_path = get_config_path(CFG_FILENAME).expect("Failed to determine config file path");
    if !Path::exists(config_path.as_path()) {
        let cfg = Config::default();
        fs::write(
            config_path.as_path(),
            toml::to_string(&cfg).expect("Could not serialize configuration"),
        )
        .expect("Could not write configuration");
        cfg
    } else {
        let toml = fs::read_to_string(config_path.as_path()).expect("Could not read configuration");
        let cfg: Config = toml::from_str(&toml).expect("Could not deserialize configuration");
        cfg
    }
}
