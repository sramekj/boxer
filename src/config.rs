pub mod hex_hwnd;

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{env, fs};
use windows::Win32::Foundation::HWND;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long)]
    pub debug: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Rotation {
    Enchanter,
    Warlock,
    Warrior,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WindowConfig {
    pub title: Option<String>,
    #[serde(default, with = "hex_hwnd")]
    pub hwnd: Option<HWND>,
    pub rotation: Rotation,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub number_of_windows: u32,
    pub window_width: u32,
    pub window_height: u32,
    pub windows: Vec<WindowConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            number_of_windows: 1,
            window_width: 1280,
            window_height: 720,
            windows: vec![WindowConfig {
                title: Some("[#] Nevergrind [#]".into()),
                hwnd: None,
                rotation: Rotation::Enchanter,
            }],
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
