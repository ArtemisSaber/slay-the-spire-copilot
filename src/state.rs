use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize)]
pub struct CardInfo {
    pub name: String,
    pub cost: i64,
    pub card_type: String,
    pub upgraded: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonsterInfo {
    pub name: String,
    pub current_hp: Option<i64>,
    pub max_hp: Option<i64>,
    pub intent: Option<String>,
    pub damage: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct NormalizedState {
    pub screen_type: Option<String>,
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
    pub card_reward_choices: Vec<String>,
    pub relics: Vec<String>,
    pub potions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PowerInfo {
    pub name: String,
    pub amount: i64,
}

impl NormalizedState {
    pub fn from_raw(raw: &Value) -> Self {
        let gs = raw.get("game_state");

        let screen_type = gs
            .and_then(|g| g.get("screen_type"))
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

        let powers = player
            .and_then(|p| p.get("powers"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|p| PowerInfo {
                        name: p
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("?")
                            .to_string(),
                        amount: p.get("amount").and_then(|n| n.as_i64()).unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let hand = combat
            .and_then(|c| c.get("hand"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|c| CardInfo {
                        name: c
                            .get("name")
                            .and_then(|n| n.as_str())
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
                    })
                    .collect()
            })
            .unwrap_or_default();

        let monsters = combat
            .and_then(|c| c.get("monsters"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter(|m| !m.get("is_gone").and_then(|g| g.as_bool()).unwrap_or(false))
                    .map(|m| MonsterInfo {
                        name: m
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("?")
                            .to_string(),
                        current_hp: m.get("current_hp").and_then(|n| n.as_i64()),
                        max_hp: m.get("max_hp").and_then(|n| n.as_i64()),
                        intent: m
                            .get("intent")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string()),
                        damage: m.get("move_adjusted_damage").and_then(|n| n.as_i64()),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let card_reward_choices = gs
            .and_then(|g| g.get("screen_state"))
            .and_then(|s| s.get("cards"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|c| {
                        c.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("?")
                            .to_string()
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut relics: Vec<String> = gs
            .and_then(|g| g.get("relics"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|r| {
                        r.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("?")
                            .to_string()
                    })
                    .collect()
            })
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
                    .map(|p| {
                        p.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("?")
                            .to_string()
                    })
                    .collect()
            })
            .unwrap_or_default();

        relics.sort();
        potions.sort();

        NormalizedState {
            screen_type,
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
        sorted_hand.sort_by(|a, b| a.name.cmp(&b.name));
        let hand_arr: Vec<Value> = sorted_hand
            .iter()
            .map(|c| {
                let mut cm = serde_json::Map::new();
                cm.insert("name".to_string(), Value::String(c.name.clone()));
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
        sorted_choices.sort();
        map.insert(
            "card_reward_choices".to_string(),
            Value::Array(sorted_choices.into_iter().map(Value::String).collect()),
        );

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

        Value::Object(map)
    }
}
