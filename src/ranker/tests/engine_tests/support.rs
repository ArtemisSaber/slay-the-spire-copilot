use super::*;

pub(super) fn aoe_contexts(
    card_id: &str,
    card_name: &str,
    damage: i64,
    cost: i64,
    monsters: Vec<MonsterInfo>,
) -> Vec<ActionContext> {
    let mut contexts = Vec::new();
    let mut vars_base = HashMap::new();
    vars_base.insert("current_energy".to_string(), 3.0);
    vars_base.insert("remaining_energy".to_string(), (3 - cost) as f64);
    vars_base.insert("cost".to_string(), cost as f64);
    vars_base.insert("current_hp".to_string(), 60.0);
    vars_base.insert("incoming_damage".to_string(), 5.0);
    vars_base.insert("monster_count".to_string(), monsters.len() as f64);
    vars_base.insert("card_base_score".to_string(), (-10 * cost) as f64);
    vars_base.insert("weight".to_string(), 1.0);

    for monster in &monsters {
        let mut vars = vars_base.clone();
        vars.insert("damage".to_string(), damage as f64);
        vars.insert("hits".to_string(), 1.0);
        vars.insert("total_damage".to_string(), damage as f64);
        vars.insert("aoe".to_string(), 1.0);
        vars.insert(
            "monsters_total_hp_plus_block".to_string(),
            monsters
                .iter()
                .map(|m| m.current_hp.unwrap_or(0) as f64)
                .sum(),
        );

        contexts.push(ActionContext {
            action_type: ActionType::PlayCard {
                card_id: card_id.into(),
                card_name: card_name.into(),
            },
            card: None,
            target_index: Some(monster.index),
            target: Some(monster.clone()),
            monsters: monsters.clone(),
            parsed: crate::ranker::parser::ParsedEffects {
                damage: Some(damage),
                hits: 1,
                ..Default::default()
            },
            vars,
        });
    }
    contexts
}

pub(super) fn per_target_damage_rule(rule_id: &str, priority: i64, weight: i64) -> Rule {
    Rule {
        category: RuleCategory::PerTarget,
        rule_id: rule_id.into(),
        priority,
        weight: Weight::Value(weight),
        formula: Some("@damage * @hits * @weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }
}

pub(super) fn per_card_power_rule(rule_id: &str, priority: i64, weight: i64) -> Rule {
    Rule {
        category: RuleCategory::PerCard,
        rule_id: rule_id.into(),
        priority,
        weight: Weight::Value(weight),
        formula: Some("@weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }
}

pub(super) fn per_card_cost_rule(rule_id: &str, priority: i64, weight: i64) -> Rule {
    Rule {
        category: RuleCategory::PerCard,
        rule_id: rule_id.into(),
        priority,
        weight: Weight::Value(weight),
        formula: Some("@cost * @weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }
}

pub(super) fn make_parsed_ctx(parsed: ParsedEffects) -> ActionContext {
    ActionContext {
        action_type: ActionType::PlayCard {
            card_id: "test-1".into(),
            card_name: "Test".into(),
        },
        card: None,
        target_index: None,
        target: None,
        monsters: vec![],
        parsed,
        vars: HashMap::new(),
    }
}

pub(super) fn status_card() -> crate::state::CardInfo {
    crate::state::CardInfo {
        id: "Burn".into(),
        name: "Burn".into(),
        cost: 0,
        card_type: "STATUS".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: false,
    }
}

pub(super) fn make_enters_wrath_rule() -> Rule {
    Rule {
        category: RuleCategory::PerCard,
        rule_id: "danger_enters_wrath_under_attack".into(),
        priority: 1102,
        weight: Weight::Value(-1000),
        formula: Some("@incoming_damage * @weight / max(@current_hp, 1)".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![
            Condition::Compute(crate::ranker::rules::ComputeCondition {
                compute: crate::ranker::rules::ComputePredicates {
                    formula: Some("@enters_wrath == 1".into()),
                },
            }),
            Condition::State(crate::ranker::rules::StateCondition {
                state: crate::ranker::rules::StatePredicates {
                    incoming_damage_gt: Some(0),
                    ..Default::default()
                },
            }),
        ],
    }
}

pub(super) fn rule_with_condition(cond: Condition) -> Rule {
    Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![cond],
    }
}

pub(super) fn target_ctx(target: MonsterInfo) -> ActionContext {
    ActionContext {
        action_type: ActionType::PlayCard {
            card_id: "test".into(),
            card_name: "Test".into(),
        },
        card: None,
        target_index: Some(0),
        target: Some(target),
        monsters: vec![],
        parsed: ParsedEffects::default(),
        vars: HashMap::new(),
    }
}

pub(super) fn monsters_ctx(monsters: Vec<MonsterInfo>, target_idx: usize) -> ActionContext {
    let target = monsters.get(target_idx).cloned();
    ActionContext {
        action_type: ActionType::PlayCard {
            card_id: "test".into(),
            card_name: "Test".into(),
        },
        card: None,
        target_index: Some(target_idx),
        target,
        monsters,
        parsed: ParsedEffects::default(),
        vars: HashMap::new(),
    }
}
