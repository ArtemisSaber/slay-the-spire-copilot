use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::locales::Locale;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardInfo {
    pub id: String,
    pub name: String,
    pub cost: i64,
    pub card_type: String,
    #[serde(default)]
    pub upgraded: bool,
    pub uuid: Option<String>,
    pub description: String,
    pub price: Option<i64>,
    #[serde(default = "default_true")]
    pub playable: bool,
    #[serde(default)]
    pub has_target: bool,
}

fn default_true() -> bool {
    true
}

impl CardInfo {
    fn from_json(c: &Value) -> Self {
        CardInfo {
            id: c
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            name: c
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            cost: c.get("cost").and_then(|n| n.as_i64()).unwrap_or(0),
            card_type: c
                .get("type")
                .and_then(|n| n.as_str())
                .unwrap_or("?")
                .to_string(),
            upgraded: c
                .get("upgrades")
                .and_then(|n| n.as_i64())
                .map(|u| u > 0)
                .unwrap_or(false),
            uuid: c
                .get("uuid")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string()),
            description: c
                .get("description")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string(),
            price: c.get("price").and_then(|n| n.as_i64()),
            playable: c
                .get("is_playable")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            has_target: c
                .get("has_target")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonsterInfo {
    pub name: String,
    #[serde(default)]
    pub monster_id: Option<String>,
    pub index: usize,
    pub current_hp: Option<i64>,
    pub max_hp: Option<i64>,
    pub block: Option<i64>,
    pub intent: Option<String>,
    pub damage: Option<i64>,
    pub hits: Option<i64>,
    pub monster_powers: Vec<PowerInfo>,
    pub can_be_killed: bool,
    pub is_scaling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerInfo {
    pub id: String,
    pub name: String,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrbInfo {
    pub id: String,
    #[serde(default)]
    pub amount: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum DangerLevel {
    #[default]
    Safe,
    Caution,
    Danger,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DangerFlags {
    pub hp_critical: bool,
    pub incoming_lethal: bool,
    pub no_block_against_hit: bool,
    pub any_monster_attacking: bool,
    pub wrath_stance: bool,
    pub level: DangerLevel,
}

impl DangerFlags {
    pub fn compute(
        current_hp: Option<i64>,
        max_hp: Option<i64>,
        block: Option<i64>,
        incoming_damage: i64,
        monsters: &[MonsterInfo],
        powers: &[PowerInfo],
    ) -> Self {
        let hp = current_hp.unwrap_or(0);
        let max_hp_val = max_hp.unwrap_or(1);
        let block_val = block.unwrap_or(0);

        let hp_critical = max_hp_val > 0 && (hp as f64 / max_hp_val as f64) < 0.3;
        let incoming_lethal = incoming_damage > hp + block_val;
        let no_block_against_hit = block_val == 0 && incoming_damage > 0;
        let low_hp = max_hp_val > 0 && (hp as f64 / max_hp_val as f64) < 0.6;
        let any_monster_attacking = monsters.iter().any(|m| m.intent.as_deref() != Some("NONE"));
        let wrath_stance = powers.iter().any(|p| p.name == "Wrath" && p.amount > 0);

        let level = if hp_critical || incoming_lethal {
            DangerLevel::Danger
        } else if low_hp || no_block_against_hit || any_monster_attacking {
            DangerLevel::Caution
        } else {
            DangerLevel::Safe
        };

        DangerFlags {
            hp_critical,
            incoming_lethal,
            no_block_against_hit,
            any_monster_attacking,
            wrath_stance,
            level,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelicInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub counter: Option<i64>,
    pub price: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PotionInfo {
    #[serde(default)]
    pub slot: usize,
    pub name: String,
    pub description: String,
    pub price: Option<i64>,
    #[serde(default)]
    pub can_use: bool,
    #[serde(default)]
    pub can_discard: bool,
    #[serde(default)]
    pub requires_target: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapCoord {
    pub symbol: String,
    pub x: i64,
    pub y: i64,
    pub children: Vec<(i64, i64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct NormalizedState {
    pub screen_type: Option<String>,
    pub room_type: Option<String>,
    pub character: Option<String>,
    pub seed: Option<i64>,
    pub ascension_level: Option<i64>,
    pub floor: Option<i64>,
    pub current_hp: Option<i64>,
    pub max_hp: Option<i64>,
    pub gold: Option<i64>,
    pub energy: Option<i64>,
    pub block: Option<i64>,
    pub powers: Vec<PowerInfo>,
    pub hand: Vec<CardInfo>,
    pub monsters: Vec<MonsterInfo>,
    pub card_reward_choices: Vec<CardInfo>,
    pub boss_relic_choices: Vec<RelicInfo>,
    pub event_id: Option<String>,
    pub event_name: Option<String>,
    pub event_body: Option<String>,
    pub event_choices: Vec<String>,
    pub relics: Vec<RelicInfo>,
    pub potions: Vec<PotionInfo>,
    pub deck_names: Vec<String>,
    pub incoming_damage: i64,
    pub rest_options: Vec<String>,
    pub danger: DangerFlags,
    pub skip_available: bool,

    pub shop_cards: Vec<CardInfo>,
    pub shop_relics: Vec<RelicInfo>,
    pub shop_potions: Vec<PotionInfo>,
    pub purge_available: bool,
    pub purge_cost: Option<i64>,

    pub turn_number: Option<i64>,
    pub orbs: Vec<OrbInfo>,
    pub stance: Option<String>,

    pub hand_cards: Vec<CardInfo>,
    pub draw_pile: Vec<CardInfo>,
    pub discard_pile: Vec<CardInfo>,
    pub exhaust_cards: Vec<CardInfo>,
    pub master_cards: Vec<CardInfo>,
    pub map_nodes: Vec<MapCoord>,
    pub map_first_node_chosen: Option<bool>,
    pub map_current_x: Option<i64>,
    pub map_current_y: Option<i64>,

    // HAND_SELECT
    pub hand_select_max_cards: Option<i64>,
    pub hand_select_can_pick_zero: bool,
    pub hand_select_selected: Vec<CardInfo>,
    pub current_action: Option<String>,

    // card_in_play (card that triggered the hand selection, e.g. Burning Pact)
    pub card_in_play: Option<CardInfo>,

    // GRID
    pub grid_cards: Vec<CardInfo>,
    #[serde(default)]
    pub grid_selected_cards: Vec<CardInfo>,
    pub grid_for_upgrade: bool,
    pub grid_for_transform: bool,
    pub grid_for_purge: bool,
    pub grid_num_cards: Option<i64>,

    // count of empty potion slots ("Potion Slot" entries in raw potions array)
    pub empty_potion_slots: usize,
}

fn detect_stance_from_powers(powers: &[PowerInfo]) -> Option<String> {
    for p in powers {
        if p.amount <= 0 {
            continue;
        }
        match p.id.as_str() {
            "Wrath" => return Some("Wrath".to_string()),
            "Calm" => return Some("Calm".to_string()),
            "Divinity" => return Some("Divinity".to_string()),
            _ => {}
        }
    }
    None
}

fn extract_cards(arr: &[Value]) -> Vec<CardInfo> {
    arr.iter().map(CardInfo::from_json).collect()
}

fn extract_powers(arr: &[Value]) -> Vec<PowerInfo> {
    arr.iter()
        .map(|p| PowerInfo {
            id: p
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            name: p
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            amount: p.get("amount").and_then(|n| n.as_i64()).unwrap_or(0),
        })
        .collect()
}

fn extract_card_names(arr: &[Value]) -> Vec<String> {
    arr.iter()
        .map(|c| {
            c.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string()
        })
        .collect()
}

fn first_array<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Vec<Value>> {
    keys.iter()
        .find_map(|key| value.get(key).and_then(|v| v.as_array()))
}

fn first_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| is_readable_text(s))
    })
}

fn first_raw_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    })
}

fn is_readable_text(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    let total = trimmed.chars().filter(|c| !c.is_whitespace()).count();
    if total == 0 {
        return false;
    }

    let question_marks = trimmed.chars().filter(|c| *c == '?').count();
    if question_marks >= 2 && question_marks * 3 >= total {
        return false;
    }

    trimmed
        .chars()
        .any(|c| c.is_alphabetic() || ('\u{4e00}'..='\u{9fff}').contains(&c))
}

fn extract_relic_infos(arr: &[Value]) -> Vec<RelicInfo> {
    arr.iter()
        .map(|r| match r {
            Value::String(name) => RelicInfo {
                id: String::new(),
                name: name.clone(),
                description: String::new(),
                counter: None,
                price: None,
            },
            Value::Object(_) => RelicInfo {
                id: r
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string(),
                name: r
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string(),
                description: r
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                counter: r
                    .get("counter")
                    .and_then(|v| v.as_i64())
                    .filter(|&c| c >= 0),
                price: r.get("price").and_then(|v| v.as_i64()),
            },
            _ => RelicInfo {
                id: String::new(),
                name: "?".to_string(),
                description: String::new(),
                counter: None,
                price: None,
            },
        })
        .collect()
}

fn extract_potion_infos(arr: &[Value]) -> Vec<PotionInfo> {
    arr.iter()
        .enumerate()
        .filter(|(_, p)| {
            p.get("id")
                .and_then(|id| id.as_str())
                .map(|id| id != "Potion Slot")
                .unwrap_or(true)
        })
        .map(|(i, p)| PotionInfo {
            slot: i,
            name: p
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            description: p
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            price: p.get("price").and_then(|v| v.as_i64()),
            can_use: p.get("can_use").and_then(|v| v.as_bool()).unwrap_or(false),
            can_discard: p
                .get("can_discard")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            requires_target: p
                .get("requires_target")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
        .collect()
}

fn extract_event_choice(option: &Value) -> Option<String> {
    match option {
        Value::String(text) => {
            let text = text.trim();
            is_readable_text(text).then(|| text.to_string())
        }
        Value::Object(_) => first_string(
            option,
            &[
                "text",
                "description",
                "choice_text",
                "button_text",
                "label",
                "name",
            ],
        ),
        _ => None,
    }
}

struct EventFields {
    id: Option<String>,
    name: Option<String>,
    body: Option<String>,
    choices: Vec<String>,
}

struct ShopFields {
    cards: Vec<CardInfo>,
    relics: Vec<RelicInfo>,
    potions: Vec<PotionInfo>,
    purge_available: bool,
    purge_cost: Option<i64>,
}

struct HandSelectFields {
    max_cards: Option<i64>,
    can_pick_zero: bool,
    selected: Vec<CardInfo>,
}

struct GridFields {
    cards: Vec<CardInfo>,
    selected_cards: Vec<CardInfo>,
    for_upgrade: bool,
    for_transform: bool,
    for_purge: bool,
    num_cards: Option<i64>,
}

fn extract_orbs(player: Option<&Value>) -> Vec<OrbInfo> {
    player
        .and_then(|p| p.get("orbs"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|o| OrbInfo {
                    id: o
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    amount: o.get("amount").and_then(|v| v.as_i64()).unwrap_or(0),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn extract_monsters(combat: Option<&Value>, total_hand_atk: i64) -> Vec<MonsterInfo> {
    combat
        .and_then(|c| c.get("monsters"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .enumerate()
                .filter(|(_, m)| !m.get("is_gone").and_then(|g| g.as_bool()).unwrap_or(false))
                .map(|(idx, m)| {
                    let hp = m.get("current_hp").and_then(|n| n.as_i64());
                    let is_scaling = m
                        .get("powers")
                        .and_then(|v| v.as_array())
                        .map(|parr| {
                            parr.iter().any(|p| {
                                let id = p.get("id").and_then(|i| i.as_str()).unwrap_or("");
                                id == "Strength"
                                    || id == "Regeneration"
                                    || id == "Metallicize"
                                    || id == "Plated Armor"
                            })
                        })
                        .unwrap_or(false);

                    let can_be_killed = hp.map(|h| h <= total_hand_atk).unwrap_or(false);

                    MonsterInfo {
                        name: m
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?")
                            .to_string(),
                        monster_id: m.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        index: idx,
                        current_hp: hp,
                        max_hp: m.get("max_hp").and_then(|n| n.as_i64()),
                        block: m.get("block").and_then(|n| n.as_i64()),
                        intent: m
                            .get("intent")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string()),
                        damage: m.get("move_adjusted_damage").and_then(|n| n.as_i64()),
                        hits: m.get("move_hits").and_then(|n| n.as_i64()),
                        monster_powers: m
                            .get("powers")
                            .and_then(|v| v.as_array())
                            .map(|arr| extract_powers(arr))
                            .unwrap_or_default(),
                        can_be_killed,
                        is_scaling,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn extract_event_fields(ss: Option<&Value>, gs: Option<&Value>, locale: &Locale) -> EventFields {
    let id = ss
        .and_then(|s| first_raw_string(s, &["event_id", "eventId", "id"]))
        .filter(|s| is_readable_text(s));

    let name = ss.and_then(|s| first_string(s, &["event_name", "name", "title"]));

    let body =
        ss.and_then(|s| first_string(s, &["body", "body_text", "event_text", "description"]));

    let choices: Vec<String> = ss
        .and_then(|s| first_array(s, &["options", "choices", "buttons"]))
        .or_else(|| gs.and_then(|g| first_array(g, &["choice_list"])))
        .map(|arr| {
            arr.iter()
                .enumerate()
                .map(|(idx, choice)| {
                    extract_event_choice(choice).unwrap_or_else(|| {
                        locale
                            .fallback
                            .event_unreadable_choice
                            .replace("{idx}", &(idx + 1).to_string())
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    EventFields {
        id,
        name,
        body,
        choices,
    }
}

fn extract_shop_fields(ss: Option<&Value>) -> ShopFields {
    let cards: Vec<CardInfo> = ss
        .and_then(|s| s.get("cards"))
        .and_then(|v| v.as_array())
        .map(|arr| extract_cards(arr))
        .unwrap_or_default();

    let relics: Vec<RelicInfo> = ss
        .and_then(|s| s.get("relics"))
        .and_then(|v| v.as_array())
        .map(|arr| extract_relic_infos(arr))
        .unwrap_or_default();

    let potions: Vec<PotionInfo> = ss
        .and_then(|s| s.get("potions"))
        .and_then(|v| v.as_array())
        .map(|arr| extract_potion_infos(arr))
        .unwrap_or_default();

    let purge_available = ss
        .and_then(|s| s.get("purge_available"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let purge_cost = ss
        .and_then(|s| s.get("purge_cost"))
        .and_then(|v| v.as_i64());

    ShopFields {
        cards,
        relics,
        potions,
        purge_available,
        purge_cost,
    }
}

fn extract_map_nodes(gs: Option<&Value>) -> Vec<MapCoord> {
    gs.and_then(|g| g.get("map"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|n| MapCoord {
                    symbol: n
                        .get("symbol")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?")
                        .to_string(),
                    x: n.get("x").and_then(|v| v.as_i64()).unwrap_or(0),
                    y: n.get("y").and_then(|v| v.as_i64()).unwrap_or(0),
                    children: n
                        .get("children")
                        .and_then(|v| v.as_array())
                        .map(|children| {
                            children
                                .iter()
                                .filter_map(|c| {
                                    Some((c.get("x")?.as_i64()?, c.get("y")?.as_i64()?))
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn extract_empty_potion_slots(gs: Option<&Value>) -> usize {
    gs.and_then(|g| g.get("potions"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|p| {
                    p.get("id")
                        .and_then(|id| id.as_str())
                        .map(|id| id == "Potion Slot")
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

fn extract_hand_select_fields(ss: Option<&Value>) -> HandSelectFields {
    let max_cards = ss.and_then(|s| s.get("max_cards")).and_then(|v| v.as_i64());

    let can_pick_zero = ss
        .and_then(|s| s.get("can_pick_zero"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let selected: Vec<CardInfo> = ss
        .and_then(|s| s.get("selected"))
        .and_then(|v| v.as_array())
        .map(|arr| extract_cards(arr))
        .unwrap_or_default();

    HandSelectFields {
        max_cards,
        can_pick_zero,
        selected,
    }
}

fn extract_grid_fields(ss: Option<&Value>) -> GridFields {
    let cards: Vec<CardInfo> = ss
        .and_then(|s| s.get("cards"))
        .and_then(|v| v.as_array())
        .map(|arr| extract_cards(arr))
        .unwrap_or_default();

    let selected_cards: Vec<CardInfo> = ss
        .and_then(|s| s.get("selected_cards"))
        .and_then(|v| v.as_array())
        .map(|arr| extract_cards(arr))
        .unwrap_or_default();

    let for_upgrade = ss
        .and_then(|s| s.get("for_upgrade"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let for_transform = ss
        .and_then(|s| s.get("for_transform"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let for_purge = ss
        .and_then(|s| s.get("for_purge"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let num_cards = ss.and_then(|s| s.get("num_cards")).and_then(|v| v.as_i64());

    GridFields {
        cards,
        selected_cards,
        for_upgrade,
        for_transform,
        for_purge,
        num_cards,
    }
}

fn insert_opt_str(map: &mut serde_json::Map<String, Value>, key: &str, val: &Option<String>) {
    if let Some(v) = val {
        map.insert(key.to_string(), Value::String(v.clone()));
    }
}

fn insert_opt_i64(map: &mut serde_json::Map<String, Value>, key: &str, val: Option<i64>) {
    if let Some(v) = val {
        map.insert(key.to_string(), Value::Number(v.into()));
    }
}

fn insert_opt_bool(map: &mut serde_json::Map<String, Value>, key: &str, val: Option<bool>) {
    if let Some(v) = val {
        map.insert(key.to_string(), Value::Bool(v));
    }
}

fn insert_bool_if_true(map: &mut serde_json::Map<String, Value>, key: &str, val: bool) {
    if val {
        map.insert(key.to_string(), Value::Bool(true));
    }
}

fn sorted_string_array(arr: &[String]) -> Value {
    let mut sorted = arr.to_vec();
    sorted.sort();
    Value::Array(sorted.into_iter().map(Value::String).collect())
}

fn sorted_id_only_array(arr: &[CardInfo]) -> Value {
    let mut sorted = arr.to_vec();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    Value::Array(
        sorted
            .iter()
            .map(|c| {
                let mut cm = serde_json::Map::new();
                cm.insert("id".to_string(), Value::String(c.id.clone()));
                Value::Object(cm)
            })
            .collect(),
    )
}

fn sorted_relic_name_desc_array(arr: &[RelicInfo]) -> Value {
    let mut sorted = arr.to_vec();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    Value::Array(
        sorted
            .into_iter()
            .map(|r| {
                let mut rm = serde_json::Map::new();
                rm.insert("name".to_string(), Value::String(r.name.clone()));
                rm.insert(
                    "description".to_string(),
                    Value::String(r.description.clone()),
                );
                Value::Object(rm)
            })
            .collect(),
    )
}

impl NormalizedState {
    pub fn from_raw(raw: &Value, locale: &Locale) -> Self {
        let gs = raw.get("game_state");

        let screen_type = gs
            .and_then(|g| g.get("screen_type"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let room_type = gs
            .and_then(|g| g.get("room_type"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let character = gs
            .and_then(|g| g.get("class"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let seed = gs.and_then(|g| g.get("seed")).and_then(|v| v.as_i64());
        let ascension_level = gs
            .and_then(|g| g.get("ascension_level"))
            .and_then(|v| v.as_i64());
        let floor = gs.and_then(|g| g.get("floor")).and_then(|v| v.as_i64());
        let current_hp = gs
            .and_then(|g| g.get("current_hp"))
            .and_then(|v| v.as_i64());
        let max_hp = gs.and_then(|g| g.get("max_hp")).and_then(|v| v.as_i64());
        let gold = gs.and_then(|g| g.get("gold")).and_then(|v| v.as_i64());

        let combat = gs.and_then(|g| g.get("combat_state"));
        let player = combat.and_then(|c| c.get("player"));

        let energy = player
            .and_then(|p| p.get("energy"))
            .and_then(|v| v.as_i64());
        let block = player.and_then(|p| p.get("block")).and_then(|v| v.as_i64());

        let powers: Vec<PowerInfo> = player
            .and_then(|p| p.get("powers"))
            .and_then(|v| v.as_array())
            .map(|arr| extract_powers(arr))
            .unwrap_or_default();

        let turn_number = combat.and_then(|c| c.get("turn")).and_then(|v| v.as_i64());

        let orbs = extract_orbs(player);
        let stance = detect_stance_from_powers(&powers);

        let hand: Vec<CardInfo> = combat
            .and_then(|c| c.get("hand"))
            .and_then(|v| v.as_array())
            .map(|arr| extract_cards(arr))
            .unwrap_or_default();

        let hand_cards = hand.clone();

        let draw_pile: Vec<CardInfo> = combat
            .and_then(|c| c.get("draw_pile"))
            .and_then(|v| v.as_array())
            .map(|arr| extract_cards(arr))
            .unwrap_or_default();

        let discard_pile: Vec<CardInfo> = combat
            .and_then(|c| c.get("discard_pile"))
            .and_then(|v| v.as_array())
            .map(|arr| extract_cards(arr))
            .unwrap_or_default();

        let exhaust_cards: Vec<CardInfo> = combat
            .and_then(|c| c.get("exhaust_pile"))
            .and_then(|v| v.as_array())
            .map(|arr| extract_cards(arr))
            .unwrap_or_default();

        let total_hand_atk: i64 = hand_cards
            .iter()
            .filter(|c| c.card_type == "ATTACK")
            .map(|c| c.cost.min(1) * 6) // rough estimate: 6 dmg per attack
            .sum();

        let monsters = extract_monsters(combat, total_hand_atk);

        let incoming_damage = monsters
            .iter()
            .filter(|m| m.intent.as_deref() != Some("NONE"))
            .filter_map(|m| m.damage)
            .filter(|&d| d > 0)
            .sum();

        let danger = DangerFlags::compute(
            current_hp,
            max_hp,
            block,
            incoming_damage,
            &monsters,
            &powers,
        );

        let screen_state = gs.and_then(|g| g.get("screen_state"));

        let map_first_node_chosen = screen_state
            .and_then(|s| s.get("first_node_chosen"))
            .and_then(|v| v.as_bool());

        let map_current_x = screen_state
            .and_then(|s| s.get("current_node"))
            .and_then(|n| n.get("x"))
            .and_then(|v| v.as_i64());

        let map_current_y = screen_state
            .and_then(|s| s.get("current_node"))
            .and_then(|n| n.get("y"))
            .and_then(|v| v.as_i64());

        let skip_available = screen_state
            .and_then(|s| s.get("skip_available"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let card_reward_choices: Vec<CardInfo> = screen_state
            .and_then(|s| s.get("cards"))
            .and_then(|v| v.as_array())
            .map(|arr| extract_cards(arr))
            .unwrap_or_default();

        let boss_relic_choices: Vec<RelicInfo> = screen_state
            .and_then(|s| first_array(s, &["relics", "boss_relics", "relic_options"]))
            .map(|arr| extract_relic_infos(arr))
            .unwrap_or_default();

        let event = extract_event_fields(screen_state, gs, locale);

        let rest_options: Vec<String> = screen_state
            .and_then(|s| s.get("rest_options"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|o| o.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let shop = extract_shop_fields(screen_state);

        let purge_available = shop.purge_available;
        let purge_cost = shop.purge_cost;

        let master_cards: Vec<CardInfo> = gs
            .and_then(|g| g.get("deck"))
            .and_then(|v| v.as_array())
            .map(|arr| extract_cards(arr))
            .unwrap_or_default();

        let map_nodes = extract_map_nodes(gs);

        let mut relics: Vec<RelicInfo> = gs
            .and_then(|g| g.get("relics"))
            .and_then(|v| v.as_array())
            .map(|arr| extract_relic_infos(arr))
            .unwrap_or_default();

        let mut potions: Vec<PotionInfo> = gs
            .and_then(|g| g.get("potions"))
            .and_then(|v| v.as_array())
            .map(|arr| extract_potion_infos(arr))
            .unwrap_or_default();

        let empty_potion_slots = extract_empty_potion_slots(gs);

        let hand_select = extract_hand_select_fields(screen_state);

        let current_action = gs
            .and_then(|g| g.get("current_action"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let card_in_play = combat
            .and_then(|c| c.get("card_in_play"))
            .map(CardInfo::from_json);

        let grid = extract_grid_fields(screen_state);

        let mut deck_names: Vec<String> = gs
            .and_then(|g| g.get("deck"))
            .and_then(|v| v.as_array())
            .map(|arr| extract_card_names(arr))
            .unwrap_or_default();

        relics.sort_by(|a, b| a.name.cmp(&b.name));
        potions.sort_by(|a, b| a.name.cmp(&b.name));
        deck_names.sort();

        NormalizedState {
            screen_type,
            room_type,
            character,
            seed,
            ascension_level,
            floor,
            current_hp,
            max_hp,
            gold,
            energy,
            block,
            powers,
            hand,
            monsters,
            card_reward_choices,
            boss_relic_choices,
            event_id: event.id,
            event_name: event.name,
            event_body: event.body,
            event_choices: event.choices,
            relics,
            potions,
            deck_names,
            incoming_damage,
            rest_options,
            danger,
            skip_available,
            shop_cards: shop.cards,
            shop_relics: shop.relics,
            shop_potions: shop.potions,
            purge_available,
            purge_cost,
            turn_number,
            orbs,
            stance,
            hand_cards,
            draw_pile,
            discard_pile,
            exhaust_cards,
            master_cards,
            map_nodes,
            map_first_node_chosen,
            map_current_x,
            map_current_y,
            hand_select_max_cards: hand_select.max_cards,
            hand_select_can_pick_zero: hand_select.can_pick_zero,
            hand_select_selected: hand_select.selected,
            current_action,
            card_in_play,
            grid_cards: grid.cards,
            grid_selected_cards: grid.selected_cards,
            grid_for_upgrade: grid.for_upgrade,
            grid_for_transform: grid.for_transform,
            grid_for_purge: grid.for_purge,
            grid_num_cards: grid.num_cards,
            empty_potion_slots,
        }
    }

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

    pub fn is_boss_card_reward(&self) -> bool {
        self.screen_type.as_deref() == Some("CARD_REWARD")
            && matches!(self.floor, Some(16 | 33 | 50))
    }

    fn to_stable_value(&self) -> Value {
        let mut map = serde_json::Map::new();

        insert_opt_str(&mut map, "screen_type", &self.screen_type);
        insert_opt_str(&mut map, "room_type", &self.room_type);
        insert_opt_str(&mut map, "character", &self.character);
        insert_opt_i64(&mut map, "floor", self.floor);
        insert_opt_i64(&mut map, "current_hp", self.current_hp);
        insert_opt_i64(&mut map, "max_hp", self.max_hp);
        insert_opt_i64(&mut map, "gold", self.gold);
        insert_opt_i64(&mut map, "energy", self.energy);
        insert_opt_i64(&mut map, "block", self.block);
        map.insert(
            "incoming_damage".to_string(),
            Value::Number(self.incoming_damage.into()),
        );

        let mut sorted_powers = self.powers.clone();
        sorted_powers.sort_by(|a, b| a.name.cmp(&b.name));
        map.insert(
            "powers".to_string(),
            Value::Array(
                sorted_powers
                    .iter()
                    .map(|p| {
                        let mut pm = serde_json::Map::new();
                        pm.insert("name".to_string(), Value::String(p.name.clone()));
                        pm.insert("amount".to_string(), Value::Number(p.amount.into()));
                        Value::Object(pm)
                    })
                    .collect(),
            ),
        );

        let mut sorted_hand = self.hand.clone();
        sorted_hand.sort_by(|a, b| a.id.cmp(&b.id));
        map.insert(
            "hand".to_string(),
            Value::Array(
                sorted_hand
                    .iter()
                    .map(|c| {
                        let mut cm = serde_json::Map::new();
                        cm.insert("id".to_string(), Value::String(c.id.clone()));
                        cm.insert("cost".to_string(), Value::Number(c.cost.into()));
                        cm.insert("type".to_string(), Value::String(c.card_type.clone()));
                        cm.insert("upgraded".to_string(), Value::Bool(c.upgraded));
                        Value::Object(cm)
                    })
                    .collect(),
            ),
        );

        let mut sorted_monsters = self.monsters.clone();
        sorted_monsters.sort_by(|a, b| a.name.cmp(&b.name));
        map.insert(
            "monsters".to_string(),
            Value::Array(
                sorted_monsters
                    .iter()
                    .map(|m| {
                        let mut mm = serde_json::Map::new();
                        mm.insert("name".to_string(), Value::String(m.name.clone()));
                        insert_opt_i64(&mut mm, "current_hp", m.current_hp);
                        insert_opt_i64(&mut mm, "max_hp", m.max_hp);
                        insert_opt_str(&mut mm, "intent", &m.intent);
                        insert_opt_i64(&mut mm, "damage", m.damage);
                        Value::Object(mm)
                    })
                    .collect(),
            ),
        );

        map.insert(
            "card_reward_choices".to_string(),
            sorted_id_only_array(&self.card_reward_choices),
        );
        map.insert(
            "boss_relic_choices".to_string(),
            sorted_relic_name_desc_array(&self.boss_relic_choices),
        );

        insert_opt_str(&mut map, "event_name", &self.event_name);
        insert_opt_str(&mut map, "event_id", &self.event_id);
        insert_opt_str(&mut map, "event_body", &self.event_body);
        map.insert(
            "event_choices".to_string(),
            sorted_string_array(&self.event_choices),
        );

        map.insert(
            "relics".to_string(),
            sorted_relic_name_desc_array(&self.relics),
        );

        let mut sorted_potions = self.potions.clone();
        sorted_potions.sort_by(|a, b| a.name.cmp(&b.name));
        map.insert(
            "potions".to_string(),
            Value::Array(
                sorted_potions
                    .into_iter()
                    .map(|p| {
                        let mut pm = serde_json::Map::new();
                        pm.insert("name".to_string(), Value::String(p.name.clone()));
                        pm.insert(
                            "description".to_string(),
                            Value::String(p.description.clone()),
                        );
                        Value::Object(pm)
                    })
                    .collect(),
            ),
        );

        map.insert(
            "deck_names".to_string(),
            sorted_string_array(&self.deck_names),
        );
        map.insert(
            "rest_options".to_string(),
            sorted_string_array(&self.rest_options),
        );

        map.insert(
            "skip_available".to_string(),
            Value::Bool(self.skip_available),
        );

        let mut sorted_shop_cards = self.shop_cards.clone();
        sorted_shop_cards.sort_by(|a, b| a.id.cmp(&b.id));
        map.insert(
            "shop_cards".to_string(),
            Value::Array(
                sorted_shop_cards
                    .into_iter()
                    .map(|c| {
                        let mut cm = serde_json::Map::new();
                        cm.insert("id".to_string(), Value::String(c.id.clone()));
                        cm.insert("cost".to_string(), Value::Number(c.cost.into()));
                        cm.insert(
                            "price".to_string(),
                            Value::Number(c.price.unwrap_or(0).into()),
                        );
                        Value::Object(cm)
                    })
                    .collect(),
            ),
        );

        let mut sorted_shop_relics = self.shop_relics.clone();
        sorted_shop_relics.sort_by(|a, b| a.name.cmp(&b.name));
        map.insert(
            "shop_relics".to_string(),
            Value::Array(
                sorted_shop_relics
                    .into_iter()
                    .map(|r| {
                        let mut rm = serde_json::Map::new();
                        rm.insert("name".to_string(), Value::String(r.name.clone()));
                        rm.insert(
                            "price".to_string(),
                            Value::Number(r.price.unwrap_or(0).into()),
                        );
                        Value::Object(rm)
                    })
                    .collect(),
            ),
        );

        if self.purge_available {
            map.insert("purge_available".to_string(), Value::Bool(true));
            insert_opt_i64(&mut map, "purge_cost", self.purge_cost);
        }

        insert_opt_bool(
            &mut map,
            "map_first_node_chosen",
            self.map_first_node_chosen,
        );
        insert_opt_i64(&mut map, "map_current_x", self.map_current_x);
        insert_opt_i64(&mut map, "map_current_y", self.map_current_y);

        insert_opt_i64(
            &mut map,
            "hand_select_max_cards",
            self.hand_select_max_cards,
        );
        insert_bool_if_true(
            &mut map,
            "hand_select_can_pick_zero",
            self.hand_select_can_pick_zero,
        );
        if !self.hand_select_selected.is_empty() {
            map.insert(
                "hand_select_selected".to_string(),
                sorted_id_only_array(&self.hand_select_selected),
            );
        }
        insert_opt_str(&mut map, "current_action", &self.current_action);
        if let Some(ref c) = self.card_in_play {
            let mut cm = serde_json::Map::new();
            cm.insert("id".to_string(), Value::String(c.id.clone()));
            cm.insert("cost".to_string(), Value::Number(c.cost.into()));
            cm.insert("type".to_string(), Value::String(c.card_type.clone()));
            cm.insert("upgraded".to_string(), Value::Bool(c.upgraded));
            map.insert("card_in_play".to_string(), Value::Object(cm));
        }
        if !self.grid_cards.is_empty() {
            map.insert(
                "grid_cards".to_string(),
                sorted_id_only_array(&self.grid_cards),
            );
        }
        if !self.grid_selected_cards.is_empty() {
            map.insert(
                "grid_selected_cards".to_string(),
                sorted_id_only_array(&self.grid_selected_cards),
            );
        }
        insert_bool_if_true(&mut map, "grid_for_upgrade", self.grid_for_upgrade);
        insert_bool_if_true(&mut map, "grid_for_transform", self.grid_for_transform);
        insert_bool_if_true(&mut map, "grid_for_purge", self.grid_for_purge);
        insert_opt_i64(&mut map, "grid_num_cards", self.grid_num_cards);
        if self.empty_potion_slots > 0 {
            map.insert(
                "empty_potion_slots".to_string(),
                Value::Number(self.empty_potion_slots.into()),
            );
        }

        Value::Object(map)
    }
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
#[path = "tests/state_tests.rs"]
mod tests;
