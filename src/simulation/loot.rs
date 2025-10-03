#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum LootQuality {
    Normal,
    Magic,
    Rare,
    Epic,
    Legendary,
    Rune,
    Unknown,
}
