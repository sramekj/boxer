use crate::configuration::config::Class;
use crate::simulation::skill::Skill;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io::Error;
use std::path::Path;
use std::{fs, io};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rotation {
    pub skills: Vec<Skill>,
}

impl Rotation {
    fn load<P: AsRef<Path> + Debug>(file_path: P) -> io::Result<Self> {
        println!("Loading rotation from {:?}", file_path);
        let file_str = fs::read_to_string(file_path)?;
        let result: io::Result<Self> = serde_json::from_str(&file_str).map_err(Error::other);
        match result {
            Ok(r) => Ok(r),
            Err(e) => {
                eprintln!("{}", e);
                Err(e)
            }
        }
    }

    #[allow(dead_code)]
    fn save<P: AsRef<Path> + Debug>(&self, file_path: P) -> io::Result<()> {
        println!("Saving rotation to {:?}", file_path);
        let serialized_r = serde_json::to_string_pretty(self).map_err(Error::other);
        match serialized_r {
            Ok(serialized) => {
                fs::write(file_path, serialized)?;
            }
            Err(e) => {
                eprintln!("{}", e);
                return Err(e);
            }
        }
        Ok(())
    }

    pub fn save_rotation(class: Class, rotation: &Rotation) {
        let file_name = format!("{}.json", class);
        let folder_name = "rotations/";
        let path = Path::new(folder_name).join(file_name);
        fs::create_dir_all(folder_name).unwrap();
        rotation.save(path).unwrap_or_else(|e| {
            panic!("Saving of {} rotation failed: {}", class, e);
        });
    }

    pub fn load_rotation(class: Class) -> Rotation {
        let file_name = format!("{}.json", class);
        let folder_name = "rotations/";
        let path = Path::new(folder_name).join(file_name);
        Rotation::load(path).unwrap_or_else(|e| {
            panic!("Loading of {} rotation failed: {}", class, e);
        })
    }
}
