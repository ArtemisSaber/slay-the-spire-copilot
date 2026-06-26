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

        let hand_cards: Vec<&CardInfo> = state
            .hand
            .iter()
            .filter(|c| c.uuid.is_some() && c.playable)
            .collect();

        for card in hand_cards {
            let parsed = parser::parse_description(&card.description);
            let remaining_energy = energy - card.cost;
            if remaining_energy < 0 {
                continue;
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
                    );
                    vars.insert("cost".to_string(), card.cost as f64);
                    vars.insert("current_energy".to_string(), energy as f64);
                    vars.insert("remaining_energy".to_string(), remaining_energy as f64);
                    fill_parsed_vars(&mut vars, &parsed);
                    fill_target_vars(&mut vars, monster);
                    fill_monster_vars(&mut vars, &state.monsters, monster.index);

                    contexts.push(ActionContext {
                        action_type: ActionType::PlayCard {
                            card_id: card.uuid.clone().unwrap_or_else(|| card.id.clone()),
                            card_name: card.name.clone(),
                        },
                        card: Some(card.clone()),
                        target_index: Some(monster.index),
                        target: Some(monster.clone()),
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
                );
                vars.insert("cost".to_string(), card.cost as f64);
                vars.insert("current_energy".to_string(), energy as f64);
                vars.insert("remaining_energy".to_string(), remaining_energy as f64);
                fill_parsed_vars(&mut vars, &parsed);
                fill_monster_vars(&mut vars, &state.monsters, usize::MAX);

                contexts.push(ActionContext {
                    action_type: ActionType::PlayCard {
                        card_id: card.uuid.clone().unwrap_or_else(|| card.id.clone()),
                        card_name: card.name.clone(),
                    },
                    card: Some(card.clone()),
                    target_index: None,
                    target: None,
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
        );
        end_vars.insert("cost".to_string(), 0.0);
        end_vars.insert("current_energy".to_string(), energy as f64);
        end_vars.insert("remaining_energy".to_string(), energy as f64);
        fill_parsed_vars(&mut end_vars, &ParsedEffects::default());
        fill_monster_vars(&mut end_vars, &state.monsters, usize::MAX);

        contexts.push(ActionContext {
            action_type: ActionType::EndTurn,
            card: None,
            target_index: None,
            target: None,
            parsed: ParsedEffects::default(),
            vars: end_vars,
        });

        contexts
    }
}

fn base_vars(
    remaining: i64,
    block: i64,
    hp: i64,
    incoming: i64,
    monster_count: f64,
    monsters_total: f64,
    useful_count: f64,
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
    if let Some(sd) = parsed.self_damage {
        vars.insert("self_damage".to_string(), sd as f64);
    }
    vars.insert("exhaust_count".to_string(), parsed.exhaust_count as f64);
}

fn fill_target_vars(vars: &mut HashMap<String, f64>, monster: &MonsterInfo) {
    if let Some(thorns) = monster.monster_powers.iter().find(|p| p.id == "Thorns") {
        vars.insert("thorns".to_string(), thorns.amount as f64);
    }
}

fn fill_monster_vars(vars: &mut HashMap<String, f64>, monsters: &[MonsterInfo], target_idx: usize) {
    let mut has_nob = false;
    for m in monsters {
        if m.name.contains("GremlinNob") || m.name.contains("Nob") {
            has_nob = true;
        }
    }
    if has_nob {
        vars.insert("has_gremlin_nob".to_string(), 1.0);
    }
    let _ = target_idx;
}

#[cfg(test)]
#[path = "tests/context_tests.rs"]
mod tests;
