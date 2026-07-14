use serde_json::Value;
use sha2::{Digest, Sha256};

use super::super::{CardInfo, RelicInfo};

pub(super) fn insert_opt_str<T: std::fmt::Display>(
    map: &mut serde_json::Map<String, Value>,
    key: &str,
    value: &Option<T>,
) {
    if let Some(value) = value {
        map.insert(key.to_string(), Value::String(value.to_string()));
    }
}

pub(super) fn insert_opt_i64(
    map: &mut serde_json::Map<String, Value>,
    key: &str,
    value: Option<i64>,
) {
    if let Some(value) = value {
        map.insert(key.to_string(), Value::Number(value.into()));
    }
}

pub(super) fn insert_opt_bool(
    map: &mut serde_json::Map<String, Value>,
    key: &str,
    value: Option<bool>,
) {
    if let Some(value) = value {
        map.insert(key.to_string(), Value::Bool(value));
    }
}

pub(super) fn insert_bool_if_true(
    map: &mut serde_json::Map<String, Value>,
    key: &str,
    value: bool,
) {
    if value {
        map.insert(key.to_string(), Value::Bool(true));
    }
}

pub(super) fn sorted_string_array(items: &[String]) -> Value {
    let mut sorted = items.to_vec();
    sorted.sort();
    Value::Array(sorted.into_iter().map(Value::String).collect())
}

pub(super) fn sorted_id_only_array(items: &[CardInfo]) -> Value {
    let mut sorted = items.to_vec();
    sorted.sort_by(|left, right| left.id.cmp(&right.id));
    Value::Array(
        sorted
            .iter()
            .map(|card| {
                let mut map = serde_json::Map::new();
                map.insert("id".to_string(), Value::String(card.id.clone()));
                Value::Object(map)
            })
            .collect(),
    )
}

pub(super) fn sorted_relic_name_desc_array(items: &[RelicInfo]) -> Value {
    let mut sorted = items.to_vec();
    sorted.sort_by(|left, right| left.name.cmp(&right.name));
    Value::Array(
        sorted
            .into_iter()
            .map(|relic| {
                let mut map = serde_json::Map::new();
                map.insert("name".to_string(), Value::String(relic.name.clone()));
                map.insert(
                    "description".to_string(),
                    Value::String(relic.description.clone()),
                );
                Value::Object(map)
            })
            .collect(),
    )
}

pub(in crate::state) fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
