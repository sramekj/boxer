#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum CharState {
    InTown,
    InDungeon,
    Fighting,
    Looting,
    AtShrine,
    Dead,
}
