use crate::i18n::I18n;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

macro_rules! i18n_name {
    ($item:expr, $i18n:expr, $method:ident) => {{
        let id = $item.get("id").and_then(|v| v.as_str());
        let name = $item.get("name").and_then(|v| v.as_str());
        id.and_then(|i| $i18n.$method(i))
            .or(name)
            .unwrap_or("?")
            .to_string()
    }};
}

#[derive(Debug, Clone, Serialize)]
pub struct CardInfo {
    pub id: String,
    pub name: String,
    pub cost: i64,
    pub card_type: String,
    pub upgraded: bool,
    pub uuid: Option<String>,
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
    pub name: String,
    pub amount: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DangerLevel {
    Safe,
    Caution,
    Danger,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct NormalizedState {
    pub screen_type: Option<String>,
    pub room_type: Option<String>,
    pub character: Option<String>,
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
    pub relics: Vec<String>,
    pub potions: Vec<String>,
    pub deck_names: Vec<String>,
    pub incoming_damage: i64,
    pub rest_options: Vec<String>,
    pub danger: DangerFlags,

    pub hand_cards: Vec<CardInfo>,
    pub draw_pile: Vec<CardInfo>,
    pub discard_pile: Vec<CardInfo>,
    pub exhaust_cards: Vec<CardInfo>,
    pub master_cards: Vec<CardInfo>,
}

fn extract_cards(arr: &[Value]) -> Vec<CardInfo> {
    arr.iter().map(|c| CardInfo::from_json(c)).collect()
}

fn extract_powers(arr: &[Value], i18n: &I18n) -> Vec<PowerInfo> {
    arr.iter()
        .map(|p| PowerInfo {
            name: i18n_name!(p, i18n, power),
            amount: p.get("amount").and_then(|n| n.as_i64()).unwrap_or(0),
        })
        .collect()
}

fn extract_card_names(arr: &[Value], i18n: &I18n) -> Vec<String> {
    arr.iter().map(|c| i18n_name!(c, i18n, card)).collect()
}

impl NormalizedState {
    pub fn from_raw(raw: &Value, i18n: &I18n) -> Self {
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
            .map(|arr| extract_powers(arr, i18n))
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
                            name: i18n_name!(m, i18n, monster),
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
                                .map(|arr| extract_powers(arr, i18n))
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

        let card_reward_choices: Vec<CardInfo> = gs
            .and_then(|g| g.get("screen_state"))
            .and_then(|s| s.get("cards"))
            .and_then(|v| v.as_array())
            .map(|arr| extract_cards(arr))
            .unwrap_or_default();

        let rest_options: Vec<String> = gs
            .and_then(|g| g.get("screen_state"))
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

        let mut relics: Vec<String> = gs
            .and_then(|g| g.get("relics"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(|r| i18n_name!(r, i18n, relic)).collect())
            .unwrap_or_default();

        let mut potions: Vec<String> = gs
            .and_then(|g| g.get("potions"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter(|p| {
                        p.get("id")
                            .and_then(|id| id.as_str())
                            .map(|id| id != "Potion Slot")
                            .unwrap_or(true)
                    })
                    .map(|p| i18n_name!(p, i18n, potion))
                    .collect()
            })
            .unwrap_or_default();

        let mut deck_names: Vec<String> = gs
            .and_then(|g| g.get("deck"))
            .and_then(|v| v.as_array())
            .map(|arr| extract_card_names(arr, i18n))
            .unwrap_or_default();

        relics.sort();
        potions.sort();
        deck_names.sort();

        NormalizedState {
            screen_type,
            room_type,
            character,
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
            relics,
            potions,
            deck_names,
            incoming_damage,
            rest_options,
            danger,
            hand_cards,
            draw_pile,
            discard_pile,
            exhaust_cards,
            master_cards,
        }
    }

    pub fn stable_hash(&self) -> String {
        let json_value = self.to_stable_value();
        let json_bytes = serde_json::to_string(&json_value).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json_bytes.as_bytes());
        hex::encode(hasher.finalize())
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

        let mut sorted_relics = self.relics.clone();
        sorted_relics.sort();
        map.insert(
            "relics".to_string(),
            Value::Array(sorted_relics.into_iter().map(Value::String).collect()),
        );

        let mut sorted_potions = self.potions.clone();
        sorted_potions.sort();
        map.insert(
            "potions".to_string(),
            Value::Array(sorted_potions.into_iter().map(Value::String).collect()),
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

        Value::Object(map)
    }
}

#[cfg(test)]
#[path = "tests/state_tests.rs"]
mod tests;
