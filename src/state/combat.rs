use serde_json::Value;

use super::parse::extract_powers;
use super::{MonsterInfo, OrbInfo, PowerInfo};

pub(super) fn detect_stance_from_powers(powers: &[PowerInfo]) -> Option<String> {
    for power in powers {
        if power.amount <= 0 {
            continue;
        }
        match power.id.as_str() {
            "Wrath" => return Some("Wrath".to_string()),
            "Calm" => return Some("Calm".to_string()),
            "Divinity" => return Some("Divinity".to_string()),
            _ => {}
        }
    }
    None
}

pub(super) fn extract_orbs(player: Option<&Value>) -> Vec<OrbInfo> {
    player
        .and_then(|value| value.get("orbs"))
        .and_then(|value| value.as_array())
        .map(|orbs| {
            orbs.iter()
                .map(|orb| OrbInfo {
                    id: orb
                        .get("id")
                        .and_then(|value| value.as_str())
                        .unwrap_or("")
                        .to_string(),
                    amount: orb
                        .get("amount")
                        .and_then(|value| value.as_i64())
                        .unwrap_or(0),
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn extract_monsters(combat: Option<&Value>, total_hand_atk: i64) -> Vec<MonsterInfo> {
    combat
        .and_then(|value| value.get("monsters"))
        .and_then(|value| value.as_array())
        .map(|monsters| {
            monsters
                .iter()
                .enumerate()
                .filter(|(_, monster)| {
                    !monster
                        .get("is_gone")
                        .and_then(|gone| gone.as_bool())
                        .unwrap_or(false)
                })
                .map(|(index, monster)| {
                    let hp = monster.get("current_hp").and_then(|number| number.as_i64());
                    let is_scaling = monster
                        .get("powers")
                        .and_then(|value| value.as_array())
                        .map(|powers| {
                            powers.iter().any(|power| {
                                matches!(
                                    power.get("id").and_then(|id| id.as_str()).unwrap_or(""),
                                    "Strength" | "Regeneration" | "Metallicize" | "Plated Armor"
                                )
                            })
                        })
                        .unwrap_or(false);
                    let can_be_killed = hp.map(|value| value <= total_hand_atk).unwrap_or(false);

                    MonsterInfo {
                        name: monster
                            .get("name")
                            .and_then(|value| value.as_str())
                            .unwrap_or("?")
                            .to_string(),
                        monster_id: monster
                            .get("id")
                            .and_then(|value| value.as_str())
                            .map(str::to_string),
                        index,
                        current_hp: hp,
                        max_hp: monster.get("max_hp").and_then(|number| number.as_i64()),
                        block: monster.get("block").and_then(|number| number.as_i64()),
                        intent: monster
                            .get("intent")
                            .and_then(|value| value.as_str())
                            .map(str::to_string),
                        damage: monster
                            .get("move_adjusted_damage")
                            .and_then(|number| number.as_i64()),
                        hits: monster.get("move_hits").and_then(|number| number.as_i64()),
                        monster_powers: monster
                            .get("powers")
                            .and_then(|value| value.as_array())
                            .map(|powers| extract_powers(powers))
                            .unwrap_or_default(),
                        can_be_killed,
                        is_scaling,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}
