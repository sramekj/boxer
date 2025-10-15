#![allow(dead_code)]

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9, VK_A, VK_B, VK_D,
    VK_E, VK_F, VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_I, VK_OEM_MINUS, VK_OEM_PLUS, VK_Q, VK_S,
    VK_T, VK_W, VK_Z,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Key(VIRTUAL_KEY);

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(self.0.0)
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<u16> = Option::deserialize(deserializer)?;
        match opt {
            Some(u) => Ok(Key(VIRTUAL_KEY(u))),
            None => Err(Error::custom("key not found")),
        }
    }
}

pub const INVENTORY: Key = Key(VK_I);
pub const LOOT_INTERACT: Key = Key(VK_T);
pub const DISCARD: Key = Key(VK_B);
pub const WALK_UP: Key = Key(VK_W);
pub const WALK_DOWN: Key = Key(VK_S);
pub const WALK_LEFT: Key = Key(VK_A);
pub const WALK_RIGHT: Key = Key(VK_D);
pub const AUTO_WALK: Key = Key(VK_Z);
pub const AUTO_ATTACK: Key = Key(VK_E);
pub const AUTO_RANGED_ATTACK: Key = Key(VK_Q);
pub const SKILL_BUTTON_1: Key = Key(VK_1);
pub const SKILL_BUTTON_2: Key = Key(VK_2);
pub const SKILL_BUTTON_3: Key = Key(VK_3);
pub const SKILL_BUTTON_4: Key = Key(VK_4);
pub const SKILL_BUTTON_5: Key = Key(VK_5);
pub const SKILL_BUTTON_6: Key = Key(VK_6);
pub const SKILL_BUTTON_7: Key = Key(VK_7);
pub const SKILL_BUTTON_8: Key = Key(VK_8);
pub const SKILL_BUTTON_9: Key = Key(VK_9);
pub const SKILL_BUTTON_10: Key = Key(VK_0);
pub const SKILL_BUTTON_11: Key = Key(VK_OEM_MINUS);
pub const SKILL_BUTTON_12: Key = Key(VK_OEM_PLUS);
pub const HEALTH_POT: Key = Key(VK_F);

impl Key {
    pub fn get_party_keys() -> Vec<Key> {
        vec![Key(VK_F1), Key(VK_F2), Key(VK_F3), Key(VK_F4), Key(VK_F5)]
    }
}

impl From<VIRTUAL_KEY> for Key {
    fn from(value: VIRTUAL_KEY) -> Self {
        Key(value)
    }
}

impl From<Key> for VIRTUAL_KEY {
    fn from(value: Key) -> Self {
        value.0
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::{INVENTORY, Key};

    #[test]
    fn test_serde_for_key() {
        let key = INVENTORY;
        println!("Testing {:?}", key);
        let serialized = serde_json::to_string(&key).unwrap();
        println!("serialized: {}", serialized);
        let deserialized: Key = serde_json::from_str(&serialized).unwrap();
        println!("deserialized: {:?}", deserialized);
        assert_eq!(key, deserialized);
    }
}
