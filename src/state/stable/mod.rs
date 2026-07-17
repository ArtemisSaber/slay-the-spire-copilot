mod base;
mod helpers;
mod screens;

use serde_json::Value;

use super::{NormalizedState, ScreenType};
pub(super) use helpers::hash_bytes;

impl NormalizedState {
    pub fn stable_hash(&self) -> String {
        let json_value = self.to_stable_value();
        let json_bytes = serde_json::to_string(&json_value).unwrap_or_default();
        hash_bytes(json_bytes.as_bytes())
    }

    pub fn observation_hash(&self) -> String {
        let json_bytes = serde_json::to_string(self).unwrap_or_default();
        hash_bytes(json_bytes.as_bytes())
    }

    pub fn has_active_monsters(&self) -> bool {
        !self.monsters.is_empty()
    }

    pub fn is_in_combat(&self) -> bool {
        self.room_phase.as_deref() == Some("COMBAT") || self.has_active_monsters()
    }

    pub fn is_boss_card_reward(&self) -> bool {
        self.screen_type.as_ref() == Some(&ScreenType::CardReward)
            && matches!(self.floor, Some(16 | 33 | 50))
    }

    pub(super) fn to_stable_value(&self) -> Value {
        let mut map = serde_json::Map::new();
        base::insert_base_fields(&mut map, self);
        screens::insert_screen_fields(&mut map, self);
        Value::Object(map)
    }
}
