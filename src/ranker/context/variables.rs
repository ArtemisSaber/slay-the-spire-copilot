use std::collections::HashMap;

use crate::ranker::parser::ParsedEffects;
use crate::state::{CardInfo, MonsterInfo, NormalizedState};

use super::builder::ContextInputs;

pub(super) fn base_vars(
    inputs: &ContextInputs,
    state: &NormalizedState,
    remaining_energy: i64,
) -> HashMap<String, f64> {
    let mut vars = HashMap::new();
    vars.insert("current_block".to_string(), inputs.block as f64);
    vars.insert("current_hp".to_string(), inputs.hp as f64);
    vars.insert("incoming_damage".to_string(), inputs.incoming as f64);
    vars.insert("retaliatory_damage".to_string(), 0.0);
    vars.insert(
        "incoming_lethal".to_string(),
        if inputs.incoming >= inputs.hp + inputs.block {
            1.0
        } else {
            0.0
        },
    );
    vars.insert("monster_count".to_string(), inputs.monster_count);
    vars.insert(
        "monsters_total_hp_plus_block".to_string(),
        inputs.monsters_total,
    );
    vars.insert("useful_cards_in_hand".to_string(), inputs.useful_count);
    vars.insert("remaining_energy".to_string(), remaining_energy as f64);

    if let Some(turn) = state.turn_number {
        vars.insert("turn".to_string(), turn as f64);
    }
    for power in &state.powers {
        vars.insert(format!("player_power_{}", power.id), 1.0);
        if power.id == "Focus" {
            vars.insert("focus".to_string(), power.amount as f64);
        }
        if power.id == "Strength" {
            vars.insert("player_str_amount".to_string(), power.amount as f64);
        }
    }
    if let Some(stance) = &state.stance {
        vars.insert(format!("player_stance_{stance}"), 1.0);
    } else {
        for power in &state.powers {
            if power.amount > 0 && matches!(power.id.as_str(), "Wrath" | "Calm" | "Divinity") {
                vars.insert(format!("player_stance_{}", power.id), 1.0);
            }
        }
    }
    for relic in &state.relics {
        vars.insert(format!("player_relic_{}", relic.id), 1.0);
    }
    if state.relics.iter().any(|relic| relic.id == "Chemical X") {
        vars.insert("chemical_x_bonus".to_string(), 2.0);
    }
    vars
}

pub(super) fn fill_parsed_vars(vars: &mut HashMap<String, f64>, parsed: &ParsedEffects) {
    insert_optional(vars, "damage", parsed.damage);
    if let Some(damage) = parsed.damage {
        vars.insert("total_damage".to_string(), (damage * parsed.hits) as f64);
    }
    vars.insert("hits".to_string(), parsed.hits as f64);
    insert_optional(vars, "block", parsed.block);
    insert_optional(vars, "heal", parsed.heal);
    insert_optional(vars, "draw", parsed.draw);
    insert_optional(vars, "self_damage", parsed.self_damage);
    insert_optional(vars, "str_gain", parsed.str_gain);
    insert_optional(vars, "dex_gain", parsed.dex_gain);
    insert_optional(vars, "poison", parsed.poison);
    insert_optional(vars, "vulnerable", parsed.vulnerable);
    insert_optional(vars, "weak", parsed.weak);
    insert_optional(vars, "energy_gain", parsed.energy_gain);
    insert_optional(vars, "str_loss", parsed.str_loss);
    vars.insert(
        "str_loss_temp".to_string(),
        parsed.str_loss_temp as u8 as f64,
    );
    insert_optional(vars, "mantra", parsed.mantra);
    insert_optional(vars, "focus_gain", parsed.focus_gain);
    vars.insert("exhaust_count".to_string(), parsed.exhaust_count as f64);
    vars.insert("ethereal".to_string(), parsed.ethereal as u8 as f64);
    if let Some(orb_type) = &parsed.channel_orb {
        vars.insert(format!("channel_orb_{orb_type}"), 1.0);
    }
    if let Some(orb_type) = &parsed.evoke_orb {
        vars.insert(format!("evoke_orb_{orb_type}"), 1.0);
    }
    if parsed.orb_slot_expand > 0 {
        vars.insert("expand_count".to_string(), parsed.orb_slot_expand as f64);
    }
    vars.insert("exits_stance".to_string(), parsed.exits_stance as u8 as f64);
    vars.insert("enters_wrath".to_string(), parsed.enters_wrath as u8 as f64);
    vars.insert("enters_calm".to_string(), parsed.enters_calm as u8 as f64);
}

pub(super) fn fill_target_vars(vars: &mut HashMap<String, f64>, monster: &MonsterInfo) {
    vars.insert(
        "target_damage".to_string(),
        monster.damage.unwrap_or(0) as f64,
    );
    vars.insert("target_hits".to_string(), monster.hits.unwrap_or(0) as f64);
    if let Some(thorns) = monster
        .monster_powers
        .iter()
        .find(|power| power.id == "Thorns")
    {
        vars.insert("thorns".to_string(), thorns.amount as f64);
    }
    fill_retaliatory_damage(vars, std::slice::from_ref(monster));
}

pub(super) fn fill_retaliatory_damage(vars: &mut HashMap<String, f64>, monsters: &[MonsterInfo]) {
    let damage: i64 = monsters
        .iter()
        .flat_map(|monster| &monster.monster_powers)
        .filter(|power| matches!(power.id.as_str(), "Thorns" | "Sharp Hide"))
        .map(|power| power.amount.max(0))
        .sum();
    vars.insert("retaliatory_damage".to_string(), damage as f64);
}

pub(super) fn fill_card_meta_vars(
    vars: &mut HashMap<String, f64>,
    card: &CardInfo,
    state: &NormalizedState,
) {
    vars.insert("card_base_score".to_string(), card_base_score(card) as f64);
    if is_aoe(card) {
        vars.insert("aoe".to_string(), 1.0);
    }
    if !card.has_target && card.card_type == "ATTACK" && !is_aoe(card) {
        vars.insert("random_target".to_string(), 1.0);
    }
    vars.insert("free_count".to_string(), count_free_cards(state));
    vars.insert(
        "pile_size".to_string(),
        state.draw_pile.len().max(state.discard_pile.len()).max(1) as f64,
    );
}

pub(super) fn is_aoe(card: &CardInfo) -> bool {
    card.description.contains("所有")
        || card.description.to_lowercase().contains("all enemies")
        || card
            .description
            .to_lowercase()
            .contains("all other enemies")
}

fn insert_optional(vars: &mut HashMap<String, f64>, key: &str, value: Option<i64>) {
    if let Some(value) = value {
        vars.insert(key.to_string(), value as f64);
    }
}

fn card_base_score(card: &CardInfo) -> i64 {
    if card.card_type == "STATUS" || card.card_type == "CURSE" {
        -100
    } else {
        -10 * card.cost
    }
}

fn count_free_cards(state: &NormalizedState) -> f64 {
    let pile = if state.draw_pile.is_empty() {
        &state.discard_pile
    } else {
        &state.draw_pile
    };
    pile.iter()
        .filter(|card| card.cost == 0 && card.card_type != "STATUS" && card.card_type != "CURSE")
        .count() as f64
}
