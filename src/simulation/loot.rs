#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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
