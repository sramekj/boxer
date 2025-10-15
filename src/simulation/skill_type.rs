use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum SkillType {
    Attack,
    Buff,
    Debuff,
}
