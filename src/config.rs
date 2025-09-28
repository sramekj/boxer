pub mod hex_hwnd;

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
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
    #[serde(with = "hex_hwnd")]
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

pub fn load_config<P: AsRef<Path>>(path: P) -> Config {
    if !Path::exists(path.as_ref()) {
        let cfg = Config::default();
        fs::write(
            path,
            toml::to_string(&cfg).expect("Could not serialize configuration"),
        )
        .expect("Could not write configuration");
        cfg
    } else {
        let toml = fs::read_to_string(path).expect("Could not read configuration");
        let cfg: Config = toml::from_str(&toml).expect("Could not deserialize configuration");
        cfg
    }
}
