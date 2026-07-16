use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::locales::Locale;
use crate::prompt::builder::clean_description;
use crate::relic_counters::rewrite_relic_description;
use crate::state::{CardInfo, MapCoord, MonsterInfo, PotionInfo, PowerInfo, RelicInfo};

pub(super) fn card_value(card: &CardInfo, locale: &Locale) -> Value {
    json!({
        "id": card.id,
        "name": card.name,
        "upgraded": card.upgraded,
        "cost": card.cost,
        "type": card.card_type,
        "description": clean_description(&card.description, locale),
        "price": card.price,
        "playable": card.playable,
        "target_required": card.has_target,
    })
}

pub(super) fn indexed_card_value(
    card: &CardInfo,
    index_name: &str,
    index: usize,
    locale: &Locale,
) -> Value {
    with_index(card_value(card, locale), index_name, index)
}

pub(super) fn card_values(cards: &[CardInfo], index_name: &str, locale: &Locale) -> Vec<Value> {
    cards
        .iter()
        .enumerate()
        .map(|(index, card)| indexed_card_value(card, index_name, index, locale))
        .collect()
}

pub(super) fn deck_value(cards: &[CardInfo], names: &[String], locale: &Locale) -> Vec<Value> {
    if !cards.is_empty() {
        return card_values(cards, "deck_index", locale);
    }
    names
        .iter()
        .enumerate()
        .map(|(index, name)| json!({"deck_index": index, "name": name}))
        .collect()
}

pub(super) fn power_values(powers: &[PowerInfo]) -> Vec<Value> {
    powers
        .iter()
        .map(|power| {
            json!({
                "id": power.id,
                "name": power.name,
                "amount": power.amount,
            })
        })
        .collect()
}

pub(super) fn relic_value(relic: &RelicInfo, locale: &Locale) -> Value {
    let description = relic.counter.map_or_else(
        || relic.description.clone(),
        |counter| rewrite_relic_description(&relic.id, counter, &relic.description, locale),
    );
    json!({
        "id": relic.id,
        "name": relic.name,
        "description": clean_description(&description, locale),
        "counter": relic.counter,
        "price": relic.price,
    })
}

pub(super) fn potion_value(potion: &PotionInfo, locale: &Locale) -> Value {
    json!({
        "id": potion.id,
        "slot": potion.slot,
        "name": potion.name,
        "description": clean_description(&potion.description, locale),
        "price": potion.price,
        "can_use": potion.can_use,
        "can_discard": potion.can_discard,
        "target_required": potion.requires_target,
    })
}

pub(super) fn monster_value(monster: &MonsterInfo) -> Value {
    let damage_per_hit = monster.damage;
    let hits = monster.hits.unwrap_or(1).max(0);
    let total_damage = if monster.intent.as_deref() == Some("NONE") {
        0
    } else {
        damage_per_hit.unwrap_or(0).max(0).saturating_mul(hits)
    };
    json!({
        "id": monster.monster_id,
        "name": monster.name,
        "index": monster.index,
        "current_hp": monster.current_hp,
        "max_hp": monster.max_hp,
        "block": monster.block,
        "intent": monster.intent,
        "damage_per_hit": damage_per_hit,
        "hits": hits,
        "total_damage": total_damage,
        "powers": power_values(&monster.monster_powers),
        "can_be_killed": monster.can_be_killed,
        "is_scaling": monster.is_scaling,
    })
}

pub(super) fn pile_value(cards: &[CardInfo], locale: &Locale) -> Vec<Value> {
    type CardKey = (String, String, bool, i64, String, String);
    let mut counts: BTreeMap<CardKey, usize> = BTreeMap::new();
    for card in cards {
        let key = (
            card.id.clone(),
            card.name.clone(),
            card.upgraded,
            card.cost,
            card.card_type.clone(),
            clean_description(&card.description, locale),
        );
        *counts.entry(key).or_default() += 1;
    }
    counts
        .into_iter()
        .map(
            |((id, name, upgraded, cost, card_type, description), count)| {
                json!({
                    "id": id,
                    "name": name,
                    "upgraded": upgraded,
                    "cost": cost,
                    "type": card_type,
                    "description": description,
                    "count": count,
                })
            },
        )
        .collect()
}

pub(super) fn map_node_value(node: &MapCoord) -> Value {
    json!({
        "symbol": node.symbol,
        "x": node.x,
        "y": node.y,
        "children": node.children.iter().map(|(x, y)| {
            json!({"x": x, "y": y})
        }).collect::<Vec<_>>(),
    })
}

pub(super) fn with_index(mut value: Value, name: &str, index: usize) -> Value {
    value
        .as_object_mut()
        .expect("projected value must be an object")
        .insert(name.to_string(), json!(index));
    value
}
