use std::collections::HashMap;

use crate::state::{CardInfo, MonsterInfo, NormalizedState};

use super::parser::{self, ParsedEffects};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionType {
    PlayCard { card_id: String, card_name: String },
    UsePotion { potion_name: String },
    EndTurn,
}

#[derive(Debug, Clone)]
pub struct ActionContext {
    pub action_type: ActionType,
    pub card: Option<CardInfo>,
    pub target_index: Option<usize>,
    pub target: Option<MonsterInfo>,
    pub monsters: Vec<MonsterInfo>,
    pub parsed: ParsedEffects,
    pub vars: HashMap<String, f64>,
}

impl ActionContext {
    pub fn build_all(state: &NormalizedState) -> Vec<ActionContext> {
        let mut contexts = Vec::new();

        let energy = state.energy.unwrap_or(0);
        let block = state.block.unwrap_or(0);
        let hp = state.current_hp.unwrap_or(0);
        let incoming = state.incoming_damage;
        let monster_count = state.monsters.len() as f64;
        let monsters_total = state
            .monsters
            .iter()
            .map(|m| m.current_hp.unwrap_or(0) + m.block.unwrap_or(0))
            .sum::<i64>() as f64;

        let useful_count = state
            .hand
            .iter()
            .filter(|c| c.playable && c.uuid.is_some())
            .count() as f64;

        let chemical_x = if state.relics.iter().any(|r| r.id == "Chemical X") {
            2.0
        } else {
            0.0
        };

        let hand_cards: Vec<&CardInfo> = state
            .hand
            .iter()
            .filter(|c| c.uuid.is_some() && c.playable)
            .collect();

        for card in hand_cards {
            let mut parsed = parser::parse_description(&card.description);

            let (effective_cost, effective_hits, effective_amount) =
                resolve_x_cost(card, energy, chemical_x);

            let remaining_energy = energy - effective_cost;
            if remaining_energy < 0 {
                continue;
            }

            if let Some(h) = effective_hits {
                parsed.hits = h;
            }
            if let Some(a) = effective_amount {
                apply_x_cost_amount(&mut parsed, a);
            }

            if card.has_target && card.card_type == "ATTACK" {
                for monster in &state.monsters {
                    let mut vars = base_vars(
                        remaining_energy,
                        block,
                        hp,
                        incoming,
                        monster_count,
                        monsters_total,
                        useful_count,
                        state,
                    );
                    vars.insert("cost".to_string(), effective_cost as f64);
                    vars.insert("current_energy".to_string(), energy as f64);
                    vars.insert("remaining_energy".to_string(), remaining_energy as f64);
                    fill_parsed_vars(&mut vars, &parsed);
                    fill_target_vars(&mut vars, monster);
                    fill_card_meta_vars(&mut vars, card, state);

                    contexts.push(ActionContext {
                        action_type: ActionType::PlayCard {
                            card_id: card.uuid.clone().unwrap_or_else(|| card.id.clone()),
                            card_name: card.name.clone(),
                        },
                        card: Some(card.clone()),
                        target_index: Some(monster.index),
                        target: Some(monster.clone()),
                        monsters: state.monsters.clone(),
                        parsed: parsed.clone(),
                        vars,
                    });
                }
            } else {
                let mut vars = base_vars(
                    remaining_energy,
                    block,
                    hp,
                    incoming,
                    monster_count,
                    monsters_total,
                    useful_count,
                    state,
                );
                vars.insert("cost".to_string(), effective_cost as f64);
                vars.insert("current_energy".to_string(), energy as f64);
                vars.insert("remaining_energy".to_string(), remaining_energy as f64);
                fill_parsed_vars(&mut vars, &parsed);
                fill_card_meta_vars(&mut vars, card, state);

                contexts.push(ActionContext {
                    action_type: ActionType::PlayCard {
                        card_id: card.uuid.clone().unwrap_or_else(|| card.id.clone()),
                        card_name: card.name.clone(),
                    },
                    card: Some(card.clone()),
                    target_index: None,
                    target: None,
                    monsters: state.monsters.clone(),
                    parsed: parsed.clone(),
                    vars,
                });
            }
        }

        for potion in &state.potions {
            if !potion.can_use {
                continue;
            }
            let parsed = parser::parse_description(&potion.description);

            if potion.requires_target {
                for monster in &state.monsters {
                    let mut vars = base_vars(
                        energy,
                        block,
                        hp,
                        incoming,
                        monster_count,
                        monsters_total,
                        useful_count,
                        state,
                    );
                    vars.insert("cost".to_string(), 0.0);
                    vars.insert("current_energy".to_string(), energy as f64);
                    vars.insert("remaining_energy".to_string(), energy as f64);
                    fill_parsed_vars(&mut vars, &parsed);
                    fill_target_vars(&mut vars, monster);

                    contexts.push(ActionContext {
                        action_type: ActionType::UsePotion {
                            potion_name: potion.name.clone(),
                        },
                        card: None,
                        target_index: Some(monster.index),
                        target: Some(monster.clone()),
                        monsters: state.monsters.clone(),
                        parsed: parsed.clone(),
                        vars,
                    });
                }
            } else {
                let mut vars = base_vars(
                    energy,
                    block,
                    hp,
                    incoming,
                    monster_count,
                    monsters_total,
                    useful_count,
                    state,
                );
                vars.insert("cost".to_string(), 0.0);
                vars.insert("current_energy".to_string(), energy as f64);
                vars.insert("remaining_energy".to_string(), energy as f64);
                fill_parsed_vars(&mut vars, &parsed);

                contexts.push(ActionContext {
                    action_type: ActionType::UsePotion {
                        potion_name: potion.name.clone(),
                    },
                    card: None,
                    target_index: None,
                    target: None,
                    monsters: state.monsters.clone(),
                    parsed: parsed.clone(),
                    vars,
                });
            }
        }

        let mut end_vars = base_vars(
            energy,
            block,
            hp,
            incoming,
            monster_count,
            monsters_total,
            useful_count,
            state,
        );
        end_vars.insert("cost".to_string(), 0.0);
        end_vars.insert("current_energy".to_string(), energy as f64);
        end_vars.insert("remaining_energy".to_string(), energy as f64);
        fill_parsed_vars(&mut end_vars, &ParsedEffects::default());

        contexts.push(ActionContext {
            action_type: ActionType::EndTurn,
            card: None,
            target_index: None,
            target: None,
            monsters: state.monsters.clone(),
            parsed: ParsedEffects::default(),
            vars: end_vars,
        });

        contexts
    }
}

fn resolve_x_cost(
    card: &CardInfo,
    energy: i64,
    chemical_x: f64,
) -> (i64, Option<i64>, Option<i64>) {
    if card.cost != -1 {
        return (card.cost, None, None);
    }
    let spent = energy as f64 + chemical_x;
    if card.card_type == "ATTACK" {
        (energy, Some(spent as i64), None)
    } else {
        (energy, None, Some(spent as i64))
    }
}

fn apply_x_cost_amount(parsed: &mut ParsedEffects, amount: i64) {
    if parsed.damage.is_none() || parsed.damage == Some(0) {
        parsed.damage = Some(amount);
    }
    if parsed.block.is_none() || parsed.block == Some(0) {
        parsed.block = Some(amount);
    }
    if parsed.str_loss.is_none() || parsed.str_loss == Some(0) {
        parsed.str_loss = Some(amount);
    }
    if parsed.weak.is_none() || parsed.weak == Some(0) {
        parsed.weak = Some(amount);
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "vars builder needs all combat state fields"
)]
fn base_vars(
    remaining: i64,
    block: i64,
    hp: i64,
    incoming: i64,
    monster_count: f64,
    monsters_total: f64,
    useful_count: f64,
    state: &NormalizedState,
) -> HashMap<String, f64> {
    let mut vars = HashMap::new();
    vars.insert("current_block".to_string(), block as f64);
    vars.insert("current_hp".to_string(), hp as f64);
    vars.insert("incoming_damage".to_string(), incoming as f64);
    vars.insert(
        "incoming_lethal".to_string(),
        if incoming >= hp + block { 1.0 } else { 0.0 },
    );
    vars.insert("monster_count".to_string(), monster_count);
    vars.insert("monsters_total_hp_plus_block".to_string(), monsters_total);
    vars.insert("useful_cards_in_hand".to_string(), useful_count);
    vars.insert("remaining_energy".to_string(), remaining as f64);

    if let Some(t) = state.turn_number {
        vars.insert("turn".to_string(), t as f64);
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

    if let Some(ref stance) = state.stance {
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

    if state.relics.iter().any(|r| r.id == "Chemical X") {
        vars.insert("chemical_x_bonus".to_string(), 2.0);
    }

    vars
}

fn fill_parsed_vars(vars: &mut HashMap<String, f64>, parsed: &ParsedEffects) {
    if let Some(d) = parsed.damage {
        vars.insert("damage".to_string(), d as f64);
        vars.insert("total_damage".to_string(), (d * parsed.hits) as f64);
    }
    vars.insert("hits".to_string(), parsed.hits as f64);

    if let Some(b) = parsed.block {
        vars.insert("block".to_string(), b as f64);
    }
    if let Some(h) = parsed.heal {
        vars.insert("heal".to_string(), h as f64);
    }
    if let Some(d) = parsed.draw {
        vars.insert("draw".to_string(), d as f64);
    }
    if let Some(sd) = parsed.self_damage {
        vars.insert("self_damage".to_string(), sd as f64);
    }
    if let Some(s) = parsed.str_gain {
        vars.insert("str_gain".to_string(), s as f64);
    }
    if let Some(d) = parsed.dex_gain {
        vars.insert("dex_gain".to_string(), d as f64);
    }
    if let Some(p) = parsed.poison {
        vars.insert("poison".to_string(), p as f64);
    }
    if let Some(v) = parsed.vulnerable {
        vars.insert("vulnerable".to_string(), v as f64);
    }
    if let Some(w) = parsed.weak {
        vars.insert("weak".to_string(), w as f64);
    }
    if let Some(e) = parsed.energy_gain {
        vars.insert("energy_gain".to_string(), e as f64);
    }
    if let Some(s) = parsed.str_loss {
        vars.insert("str_loss".to_string(), s as f64);
    }
    vars.insert(
        "str_loss_temp".to_string(),
        if parsed.str_loss_temp { 1.0 } else { 0.0 },
    );
    if let Some(m) = parsed.mantra {
        vars.insert("mantra".to_string(), m as f64);
    }
    if let Some(f) = parsed.focus_gain {
        vars.insert("focus_gain".to_string(), f as f64);
    }
    vars.insert("exhaust_count".to_string(), parsed.exhaust_count as f64);
    vars.insert(
        "ethereal".to_string(),
        if parsed.ethereal { 1.0 } else { 0.0 },
    );
    if let Some(ref orb_type) = parsed.channel_orb {
        vars.insert(format!("channel_orb_{orb_type}"), 1.0);
    }
    if let Some(ref orb_type) = parsed.evoke_orb {
        vars.insert(format!("evoke_orb_{orb_type}"), 1.0);
    }
    if parsed.orb_slot_expand > 0 {
        vars.insert("expand_count".to_string(), parsed.orb_slot_expand as f64);
    }
    vars.insert(
        "exits_stance".to_string(),
        if parsed.exits_stance { 1.0 } else { 0.0 },
    );
    vars.insert(
        "enters_wrath".to_string(),
        if parsed.enters_wrath { 1.0 } else { 0.0 },
    );
    vars.insert(
        "enters_calm".to_string(),
        if parsed.enters_calm { 1.0 } else { 0.0 },
    );
}

fn fill_target_vars(vars: &mut HashMap<String, f64>, monster: &MonsterInfo) {
    vars.insert(
        "target_damage".to_string(),
        monster.damage.unwrap_or(0) as f64,
    );
    vars.insert("target_hits".to_string(), monster.hits.unwrap_or(0) as f64);
    if let Some(thorns) = monster.monster_powers.iter().find(|p| p.id == "Thorns") {
        vars.insert("thorns".to_string(), thorns.amount as f64);
    }
}

fn fill_card_meta_vars(vars: &mut HashMap<String, f64>, card: &CardInfo, state: &NormalizedState) {
    let card_base_score = compute_card_base_score(card);
    vars.insert("card_base_score".to_string(), card_base_score as f64);

    let is_aoe = card.description.contains("所有")
        || card.description.to_lowercase().contains("all enemies")
        || card
            .description
            .to_lowercase()
            .contains("all other enemies");
    if is_aoe {
        vars.insert("aoe".to_string(), 1.0);
    }

    let is_random = !card.has_target && card.card_type == "ATTACK" && !is_aoe;
    if is_random {
        vars.insert("random_target".to_string(), 1.0);
    }

    let free_count = count_free_cards(state);
    let pile_size = state.draw_pile.len().max(state.discard_pile.len()).max(1) as f64;
    vars.insert("free_count".to_string(), free_count);
    vars.insert("pile_size".to_string(), pile_size);
}

fn compute_card_base_score(card: &CardInfo) -> i64 {
    if card.card_type == "STATUS" || card.card_type == "CURSE" {
        return -100;
    }
    -10 * card.cost
}

fn count_free_cards(state: &NormalizedState) -> f64 {
    let pile = if state.draw_pile.is_empty() {
        &state.discard_pile
    } else {
        &state.draw_pile
    };
    pile.iter()
        .filter(|c| c.cost == 0 && c.card_type != "STATUS" && c.card_type != "CURSE")
        .count() as f64
}

#[cfg(test)]
#[path = "tests/context_tests.rs"]
mod tests;
