use crate::config::Class;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ClassConfig {
    pub class: Class,
    pub cast_all_skills: Option<Vec<String>>,
}

impl ClassConfig {
    pub fn new(class: Class, cast_all_skills: Option<Vec<String>>) -> ClassConfig {
        ClassConfig {
            class,
            cast_all_skills,
        }
    }
}
