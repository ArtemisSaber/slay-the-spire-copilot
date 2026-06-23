use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::locales::Locale;

#[derive(Debug, Clone, Serialize)]
pub struct CardInfo {
    pub id: String,
    pub name: String,
    pub cost: i64,
    pub card_type: String,
    pub upgraded: bool,
    pub uuid: Option<String>,
    pub description: String,
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
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MonsterInfo {
    pub name: String,
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

#[derive(Debug, Clone, Serialize)]
pub struct PowerInfo {
    pub id: String,
    pub name: String,
    pub amount: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum DangerLevel {
    Safe,
    Caution,
    Danger,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RelicInfo {
    pub name: String,
    pub description: String,
    pub counter: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PotionInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapCoord {
    pub symbol: String,
    pub x: i64,
    pub y: i64,
    pub children: Vec<(i64, i64)>,
}

#[derive(Debug, Clone, Serialize)]
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

    pub hand_cards: Vec<CardInfo>,
    pub draw_pile: Vec<CardInfo>,
    pub discard_pile: Vec<CardInfo>,
    pub exhaust_cards: Vec<CardInfo>,
    pub master_cards: Vec<CardInfo>,
    pub map_nodes: Vec<MapCoord>,
    pub map_first_node_chosen: Option<bool>,
    pub map_current_x: Option<i64>,
    pub map_current_y: Option<i64>,
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
                name: name.clone(),
                description: String::new(),
                counter: None,
            },
            Value::Object(_) => RelicInfo {
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
            },
            _ => RelicInfo {
                name: "?".to_string(),
                description: String::new(),
                counter: None,
            },
        })
        .collect()
}

fn extract_potion_infos(arr: &[Value]) -> Vec<PotionInfo> {
    arr.iter()
        .filter(|p| {
            p.get("id")
                .and_then(|id| id.as_str())
                .map(|id| id != "Potion Slot")
                .unwrap_or(true)
        })
        .map(|p| PotionInfo {
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

        let monster_arr = combat
            .and_then(|c| c.get("monsters"))
            .and_then(|v| v.as_array());

        let monsters: Vec<MonsterInfo> = monster_arr
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
            .unwrap_or_default();

        let incoming_damage = monsters
            .iter()
            .filter(|m| m.intent.as_deref() != Some("NONE"))
            .filter_map(|m| m.damage)
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

        let event_id = screen_state
            .and_then(|s| first_raw_string(s, &["event_id", "eventId", "id"]))
            .filter(|s| is_readable_text(s));

        let event_name =
            screen_state.and_then(|s| first_string(s, &["event_name", "name", "title"]));

        let event_body = screen_state
            .and_then(|s| first_string(s, &["body", "body_text", "event_text", "description"]));

        let event_choices: Vec<String> = screen_state
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

        let rest_options: Vec<String> = screen_state
            .and_then(|s| s.get("rest_options"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|o| o.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let master_cards: Vec<CardInfo> = gs
            .and_then(|g| g.get("deck"))
            .and_then(|v| v.as_array())
            .map(|arr| extract_cards(arr))
            .unwrap_or_default();

        let map_nodes: Vec<MapCoord> = gs
            .and_then(|g| g.get("map"))
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
            .unwrap_or_default();

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
            event_id,
            event_name,
            event_body,
            event_choices,
            relics,
            potions,
            deck_names,
            incoming_damage,
            rest_options,
            danger,
            skip_available,
            hand_cards,
            draw_pile,
            discard_pile,
            exhaust_cards,
            master_cards,
            map_nodes,
            map_first_node_chosen,
            map_current_x,
            map_current_y,
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

        if let Some(ref v) = self.screen_type {
            map.insert("screen_type".to_string(), Value::String(v.clone()));
        }
        if let Some(ref v) = self.room_type {
            map.insert("room_type".to_string(), Value::String(v.clone()));
        }
        if let Some(ref v) = self.character {
            map.insert("character".to_string(), Value::String(v.clone()));
        }
        if let Some(v) = self.floor {
            map.insert("floor".to_string(), Value::Number(v.into()));
        }
        if let Some(v) = self.current_hp {
            map.insert("current_hp".to_string(), Value::Number(v.into()));
        }
        if let Some(v) = self.max_hp {
            map.insert("max_hp".to_string(), Value::Number(v.into()));
        }
        if let Some(v) = self.gold {
            map.insert("gold".to_string(), Value::Number(v.into()));
        }
        if let Some(v) = self.energy {
            map.insert("energy".to_string(), Value::Number(v.into()));
        }
        if let Some(v) = self.block {
            map.insert("block".to_string(), Value::Number(v.into()));
        }

        map.insert(
            "incoming_damage".to_string(),
            Value::Number(self.incoming_damage.into()),
        );

        let mut sorted_powers = self.powers.clone();
        sorted_powers.sort_by(|a, b| a.name.cmp(&b.name));
        let powers_arr: Vec<Value> = sorted_powers
            .iter()
            .map(|p| {
                let mut pm = serde_json::Map::new();
                pm.insert("name".to_string(), Value::String(p.name.clone()));
                pm.insert("amount".to_string(), Value::Number(p.amount.into()));
                Value::Object(pm)
            })
            .collect();
        map.insert("powers".to_string(), Value::Array(powers_arr));

        let mut sorted_hand = self.hand.clone();
        sorted_hand.sort_by(|a, b| a.id.cmp(&b.id));
        let hand_arr: Vec<Value> = sorted_hand
            .iter()
            .map(|c| {
                let mut cm = serde_json::Map::new();
                cm.insert("id".to_string(), Value::String(c.id.clone()));
                cm.insert("cost".to_string(), Value::Number(c.cost.into()));
                cm.insert("type".to_string(), Value::String(c.card_type.clone()));
                cm.insert("upgraded".to_string(), Value::Bool(c.upgraded));
                Value::Object(cm)
            })
            .collect();
        map.insert("hand".to_string(), Value::Array(hand_arr));

        let mut sorted_monsters = self.monsters.clone();
        sorted_monsters.sort_by(|a, b| a.name.cmp(&b.name));
        let monster_arr: Vec<Value> = sorted_monsters
            .iter()
            .map(|m| {
                let mut mm = serde_json::Map::new();
                mm.insert("name".to_string(), Value::String(m.name.clone()));
                if let Some(v) = m.current_hp {
                    mm.insert("current_hp".to_string(), Value::Number(v.into()));
                }
                if let Some(v) = m.max_hp {
                    mm.insert("max_hp".to_string(), Value::Number(v.into()));
                }
                if let Some(ref v) = m.intent {
                    mm.insert("intent".to_string(), Value::String(v.clone()));
                }
                if let Some(v) = m.damage {
                    mm.insert("damage".to_string(), Value::Number(v.into()));
                }
                Value::Object(mm)
            })
            .collect();
        map.insert("monsters".to_string(), Value::Array(monster_arr));

        let mut sorted_choices = self.card_reward_choices.clone();
        sorted_choices.sort_by(|a, b| a.id.cmp(&b.id));
        let choices_arr: Vec<Value> = sorted_choices
            .iter()
            .map(|c| {
                let mut cm = serde_json::Map::new();
                cm.insert("id".to_string(), Value::String(c.id.clone()));
                Value::Object(cm)
            })
            .collect();
        map.insert("card_reward_choices".to_string(), Value::Array(choices_arr));

        let mut sorted_boss_relics = self.boss_relic_choices.clone();
        sorted_boss_relics.sort_by(|a, b| a.name.cmp(&b.name));
        map.insert(
            "boss_relic_choices".to_string(),
            Value::Array(
                sorted_boss_relics
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
            ),
        );

        if let Some(ref v) = self.event_name {
            map.insert("event_name".to_string(), Value::String(v.clone()));
        }
        if let Some(ref v) = self.event_id {
            map.insert("event_id".to_string(), Value::String(v.clone()));
        }
        if let Some(ref v) = self.event_body {
            map.insert("event_body".to_string(), Value::String(v.clone()));
        }
        let mut sorted_event_choices = self.event_choices.clone();
        sorted_event_choices.sort();
        map.insert(
            "event_choices".to_string(),
            Value::Array(
                sorted_event_choices
                    .into_iter()
                    .map(Value::String)
                    .collect(),
            ),
        );

        let mut sorted_relics = self.relics.clone();
        sorted_relics.sort_by(|a, b| a.name.cmp(&b.name));
        map.insert(
            "relics".to_string(),
            Value::Array(
                sorted_relics
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
            ),
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

        let mut sorted_deck = self.deck_names.clone();
        sorted_deck.sort();
        map.insert(
            "deck_names".to_string(),
            Value::Array(sorted_deck.into_iter().map(Value::String).collect()),
        );

        let mut sorted_rest = self.rest_options.clone();
        sorted_rest.sort();
        map.insert(
            "rest_options".to_string(),
            Value::Array(sorted_rest.into_iter().map(Value::String).collect()),
        );

        map.insert(
            "skip_available".to_string(),
            Value::Bool(self.skip_available),
        );

        if let Some(v) = self.map_first_node_chosen {
            map.insert("map_first_node_chosen".to_string(), Value::Bool(v));
        }
        if let Some(v) = self.map_current_x {
            map.insert("map_current_x".to_string(), Value::Number(v.into()));
        }
        if let Some(v) = self.map_current_y {
            map.insert("map_current_y".to_string(), Value::Number(v.into()));
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
