use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum LootQuality {
    Normal,
    Socketed,
    Magic,
    Rare,
    Set,
    Epic,
    Legendary,
    Unknown,
}
