use std::collections::HashMap;

use crate::ranker::context::{ActionContext, ActionType};
use crate::ranker::engine::{evaluate, rank_contexts};
use crate::ranker::parser::ParsedEffects;
use crate::ranker::rules::{Condition, Rule, RuleCategory, RuleSet, Weight};
use crate::state::MonsterInfo;

fn make_rules(rules: Vec<Rule>) -> RuleSet {
    RuleSet {
        version: "1.0".into(),
        available_score_fns: vec!["hp_cost_penalty".into()],
        rules,
    }
}

fn make_ctx(vars: HashMap<String, f64>) -> ActionContext {
    ActionContext {
        action_type: ActionType::PlayCard {
            card_id: "test-1".into(),
            card_name: "Test".into(),
        },
        card: None,
        target_index: None,
        target: None,
        monsters: vec![],
        parsed: ParsedEffects::default(),
        vars,
    }
}

fn end_turn_ctx(vars: HashMap<String, f64>) -> ActionContext {
    ActionContext {
        action_type: ActionType::EndTurn,
        card: None,
        target_index: None,
        target: None,
        monsters: vec![],
        parsed: ParsedEffects::default(),
        vars,
    }
}

fn make_parsed_ctx(parsed: ParsedEffects) -> ActionContext {
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

// --- existing tests ---

#[test]
fn flat_weight_rule_scores_correctly() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test_flat".into(),
        priority: 1000,
        weight: Weight::Value(-100),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = make_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rule_id, "test_flat");
    assert_eq!(results[0].score, -100);
    assert!(results[0].matched);
}

#[test]
fn formula_rule_uses_weight_and_vars() {
    let mut vars = HashMap::new();
    vars.insert("damage".to_string(), 6.0);
    vars.insert("hits".to_string(), 2.0);

    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test_formula".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("@damage * @hits * @weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = make_ctx(vars);
    let results = evaluate(&ctx, &rules);
    assert_eq!(results[0].rule_id, "test_formula");
    assert_eq!(results[0].score, 120);
    assert!(results[0].matched);
}

#[test]
fn override_suppresses_target_rule() {
    let rules = make_rules(vec![
        Rule {
            category: RuleCategory::PerTarget,
            rule_id: "override_me".into(),
            priority: 2000,
            weight: Weight::Value(10),
            formula: Some("@damage * @weight".into()),
            score_fn: None,
            override_rule: None,
            applies_to: vec!["play_card".into()],
            conditions: vec![],
        },
        Rule {
            category: RuleCategory::PerTarget,
            rule_id: "the_override".into(),
            priority: 1000,
            weight: Weight::Value(5),
            formula: Some("@damage * @weight".into()),
            score_fn: None,
            override_rule: Some("override_me".into()),
            applies_to: vec!["play_card".into()],
            conditions: vec![],
        },
    ]);
    let mut vars = HashMap::new();
    vars.insert("damage".to_string(), 6.0);
    let ctx = make_ctx(vars);

    let results = evaluate(&ctx, &rules);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].rule_id, "the_override");
    assert_eq!(results[0].score, 30);
    assert!(results[0].matched);
    assert_eq!(results[1].rule_id, "override_me");
    assert_eq!(results[1].score, 0);
    assert!(!results[1].matched);
}

#[test]
fn avoid_actions_separated() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "min".into(),
        priority: 1000,
        weight: Weight::MinI64,
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = make_ctx(HashMap::new());
    let scored = rank_contexts(&[ctx], &rules);
    assert_eq!(scored.len(), 1);
    assert!(scored[0].is_avoid);
    assert_eq!(scored[0].score, i64::MIN);
}

#[test]
fn unscored_end_turn_gets_total_zero() {
    let rules = make_rules(vec![]);
    let ctx = end_turn_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    let total: i64 = results.iter().map(|r| r.score).sum();
    assert_eq!(total, 0);
}

#[test]
fn sort_by_score_descending() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "score_by_value".into(),
        priority: 1000,
        weight: Weight::Value(1),
        formula: Some("@value * @weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let mut low_vars = HashMap::new();
    low_vars.insert("value".to_string(), 2.0);
    let mut high_vars = HashMap::new();
    high_vars.insert("value".to_string(), 5.0);
    let low = make_ctx(low_vars);
    let high = make_ctx(high_vars);

    let scored = rank_contexts(&[low, high], &rules);
    assert_eq!(scored.len(), 2);
    assert_eq!(scored[0].score, 5);
    assert_eq!(scored[1].score, 2);
}

// --- parsed condition predicates ---

fn rule_with_condition(cond: Condition) -> Rule {
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

#[test]
fn parsed_heal_gt_matches() {
    let rules = make_rules(vec![rule_with_condition(Condition::Parsed(
        crate::ranker::rules::ParsedCondition {
            parsed: crate::ranker::rules::ParsedPredicates {
                heal_gt: Some(0),
                ..Default::default()
            },
        },
    ))]);
    let ctx = make_parsed_ctx(ParsedEffects {
        heal: Some(5),
        ..Default::default()
    });
    let results = evaluate(&ctx, &rules);
    assert!(results[0].matched);
}

#[test]
fn parsed_draw_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            draw_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        draw: Some(2),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_str_gain_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            str_gain_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        str_gain: Some(3),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_dex_gain_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            dex_gain_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        dex_gain: Some(2),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_poison_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            poison_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        poison: Some(5),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_vulnerable_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            vulnerable_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        vulnerable: Some(2),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_weak_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            weak_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        weak: Some(3),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_energy_gain_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            energy_gain_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        energy_gain: Some(2),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_focus_gain_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            focus_gain_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        focus_gain: Some(2),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_str_loss_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            str_loss_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        str_loss: Some(4),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_str_loss_temp_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            str_loss_temp: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        str_loss: Some(2),
        str_loss_temp: true,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_mantra_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            mantra_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        mantra: Some(3),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_channel_orb_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            channel_orb: Some("Lightning".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        channel_orb: Some("Lightning".into()),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_channel_orb_no_match_wrong_type() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            channel_orb: Some("Frost".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        channel_orb: Some("Lightning".into()),
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_orb_slot_expand_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            orb_slot_expand_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        orb_slot_expand: 1,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exits_stance_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exits_stance: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        exits_stance: true,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

// --- target condition predicates ---

fn target_ctx(target: MonsterInfo) -> ActionContext {
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

#[test]
fn target_is_max_hp_true() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_max_hp: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(20),
        max_hp: Some(20),
        block: Some(0),
        intent: None,
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_is_max_hp_false() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_max_hp: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(10),
        max_hp: Some(20),
        block: Some(0),
        intent: None,
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

// --- monster condition predicates ---

fn monsters_ctx(monsters: Vec<MonsterInfo>, target_idx: usize) -> ActionContext {
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

#[test]
fn monster_is_scaling_any() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                is_scaling: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        is_scaling: true,
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_is_not_minion_any() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                is_not_minion: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        monster_powers: vec![],
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_is_not_minion_any_false_when_minion() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                is_not_minion: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        monster_powers: vec![crate::state::PowerInfo {
            id: "Minion".into(),
            name: "Minion".into(),
            amount: 1,
        }],
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_exclude_target_excludes_target() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                exclude_target: Some(true),
                is_scaling: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![
        MonsterInfo {
            name: "Target".into(),
            monster_id: None,
            index: 0,
            is_scaling: true,
            ..Default::default()
        },
        MonsterInfo {
            name: "Other".into(),
            monster_id: None,
            index: 1,
            is_scaling: false,
            ..Default::default()
        },
    ];
    let ctx = monsters_ctx(monsters, 0);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_exclude_target_finds_other() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                exclude_target: Some(true),
                is_scaling: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![
        MonsterInfo {
            name: "Target".into(),
            monster_id: None,
            index: 0,
            is_scaling: false,
            ..Default::default()
        },
        MonsterInfo {
            name: "Other".into(),
            monster_id: None,
            index: 1,
            is_scaling: true,
            ..Default::default()
        },
    ];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_power_any() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                power: Some("Thorns".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        monster_powers: vec![crate::state::PowerInfo {
            id: "Thorns".into(),
            name: "Thorns".into(),
            amount: 1,
        }],
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_intent_any() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                intent: Some("ATTACK".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        intent: Some("ATTACK".into()),
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_id_exact_match() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                monster_id: Some("GremlinNob".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "Gremlin Nob Leader".into(),
        monster_id: Some("GremlinNob".into()),
        index: 0,
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_id_exact_no_false_match() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                monster_id: Some("GremlinNob".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "Gremlin Nob Leader".into(),
        monster_id: Some("GremlinNobLeader".into()),
        index: 0,
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

// --- per_target ---

#[test]
fn single_context_scores_once_per_context() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("@weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = ActionContext {
        action_type: ActionType::PlayCard {
            card_id: "test".into(),
            card_name: "Test".into(),
        },
        card: None,
        target_index: Some(0),
        target: None,
        monsters: vec![
            MonsterInfo {
                name: "A".into(),
                monster_id: None,
                index: 0,
                ..Default::default()
            },
            MonsterInfo {
                name: "B".into(),
                monster_id: None,
                index: 1,
                ..Default::default()
            },
        ],
        parsed: ParsedEffects::default(),
        vars: HashMap::new(),
    };
    let results = evaluate(&ctx, &rules);
    assert_eq!(results[0].score, 10, "single context scores once");
    assert!(results[0].matched);
}

#[test]
fn context_with_no_monsters_still_scores_once() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("@weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = ActionContext {
        action_type: ActionType::PlayCard {
            card_id: "test".into(),
            card_name: "Test".into(),
        },
        card: None,
        target_index: None,
        target: None,
        monsters: vec![],
        parsed: ParsedEffects::default(),
        vars: HashMap::new(),
    };
    let results = evaluate(&ctx, &rules);
    assert_eq!(
        results[0].score, 10,
        "score applies regardless of monster count"
    );
    assert!(results[0].matched);
}

// --- monster none quantifier ---

#[test]
fn monsters_none_matches_when_no_monster_satisfies() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            none: Some(crate::ranker::rules::MonsterSubPredicates {
                monster_id: Some("AwakenedOne".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "Jaw Worm".into(),
        monster_id: Some("JawWorm".into()),
        index: 0,
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monsters_none_fails_when_monster_satisfies() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            none: Some(crate::ranker::rules::MonsterSubPredicates {
                monster_id: Some("AwakenedOne".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "Awakened One".into(),
        monster_id: Some("AwakenedOne".into()),
        index: 0,
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

// --- player condition ---

#[test]
fn player_power_matches() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            power: Some("Barricade".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("player_power_Barricade".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn player_power_not_matches_when_absent() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            power_not: Some("Barricade".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_ctx(HashMap::new());
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn player_power_not_fails_when_present() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            power_not: Some("Barricade".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("player_power_Barricade".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn player_stance_matches() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            stance: Some("Wrath".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("player_stance_Wrath".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn player_stance_not_matches() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            stance_not: Some("Divinity".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_ctx(HashMap::new());
    assert!(evaluate(&ctx, &rules)[0].matched);
}

// --- state condition ---

#[test]
fn state_turn_eq_matches() {
    let c = Condition::State(crate::ranker::rules::StateCondition {
        state: crate::ranker::rules::StatePredicates {
            turn_eq: Some(1),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("turn".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn state_remaining_energy_gt_matches() {
    let c = Condition::State(crate::ranker::rules::StateCondition {
        state: crate::ranker::rules::StatePredicates {
            remaining_energy_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("remaining_energy".to_string(), 2.0);
    let ctx = make_ctx(vars);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn state_incoming_damage_gt_matches() {
    let c = Condition::State(crate::ranker::rules::StateCondition {
        state: crate::ranker::rules::StatePredicates {
            incoming_damage_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("incoming_damage".to_string(), 5.0);
    let ctx = make_ctx(vars);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn scored_action_propagates_target_index_from_context() {
    let target = MonsterInfo {
        name: "Enemy".into(),
        monster_id: None,
        index: 3,
        current_hp: Some(20),
        max_hp: Some(20),
        block: Some(0),
        intent: None,
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let ctx = ActionContext {
        action_type: ActionType::PlayCard {
            card_id: "uuid-1".into(),
            card_name: "Strike".into(),
        },
        card: None,
        target_index: Some(3),
        target: Some(target),
        monsters: vec![],
        parsed: ParsedEffects::default(),
        vars: HashMap::new(),
    };
    let rule_set = make_rules(vec![]);
    let scored = rank_contexts(&[ctx], &rule_set);

    assert_eq!(scored.len(), 1);
    assert_eq!(scored[0].target_index, Some(3));
}

// --- AoE merge: per-target rules sum, per-card rules applied once ---

fn aoe_contexts(
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

fn per_target_damage_rule(rule_id: &str, priority: i64, weight: i64) -> Rule {
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

fn per_card_power_rule(rule_id: &str, priority: i64, weight: i64) -> Rule {
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

fn per_card_cost_rule(rule_id: &str, priority: i64, weight: i64) -> Rule {
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

#[test]
fn aoe_merge_sums_per_target_rules() {
    let monsters = vec![
        MonsterInfo {
            name: "Slime A".into(),
            monster_id: None,
            index: 0,
            current_hp: Some(10),
            max_hp: Some(10),
            block: Some(0),
            ..Default::default()
        },
        MonsterInfo {
            name: "Slime B".into(),
            monster_id: None,
            index: 1,
            current_hp: Some(10),
            max_hp: Some(10),
            block: Some(0),
            ..Default::default()
        },
    ];
    let contexts = aoe_contexts("tc-1", "Thunderclap", 4, 1, monsters);

    let rules = make_rules(vec![per_target_damage_rule("core_damage", 1050, 10)]);

    let scored = rank_contexts(&contexts, &rules);
    assert_eq!(scored.len(), 1, "AoE contexts should merge into one entry");
    // 2 monsters × (4 dmg × 10 weight) = 80
    assert_eq!(scored[0].score, 80);
    assert!(
        scored[0].target_index.is_none(),
        "merged AoE should have null target"
    );
}

#[test]
fn aoe_merge_applies_per_card_rules_once() {
    let monsters = vec![
        MonsterInfo {
            name: "Slime A".into(),
            monster_id: None,
            index: 0,
            current_hp: Some(10),
            max_hp: Some(10),
            block: Some(0),
            ..Default::default()
        },
        MonsterInfo {
            name: "Slime B".into(),
            monster_id: None,
            index: 1,
            current_hp: Some(10),
            max_hp: Some(10),
            block: Some(0),
            ..Default::default()
        },
    ];
    let contexts = aoe_contexts("tc-1", "Thunderclap", 4, 1, monsters);

    let rules = make_rules(vec![
        per_target_damage_rule("core_damage", 1050, 10), // 40 per monster
        per_card_cost_rule("base_cost_penalty", 1000, -10), // -10 once
    ]);

    let scored = rank_contexts(&contexts, &rules);
    assert_eq!(scored.len(), 1);
    // 2 × 40 damage + (-10 cost once) = 70
    assert_eq!(
        scored[0].score, 70,
        "cost should be applied once, not twice"
    );
}

#[test]
fn aoe_merge_combines_matched_rules() {
    let monsters = vec![
        MonsterInfo {
            name: "Slime A".into(),
            monster_id: None,
            index: 0,
            current_hp: Some(10),
            max_hp: Some(10),
            block: Some(0),
            ..Default::default()
        },
        MonsterInfo {
            name: "Slime B".into(),
            monster_id: None,
            index: 1,
            current_hp: Some(10),
            max_hp: Some(10),
            block: Some(0),
            ..Default::default()
        },
    ];
    let contexts = aoe_contexts("tc-1", "Thunderclap", 4, 1, monsters);

    let rules = make_rules(vec![
        per_target_damage_rule("core_damage", 1050, 10),
        per_card_cost_rule("base_cost_penalty", 1000, -10),
        per_card_power_rule("setup_power", 1070, 30),
    ]);

    let scored = rank_contexts(&contexts, &rules);
    assert_eq!(scored.len(), 1);
    // 2 × 40 (damage) + (-10) (cost) + 30 (power) = 100
    assert_eq!(scored[0].score, 100);
    let breakdown = &scored[0].breakdown;
    assert!(
        breakdown
            .iter()
            .any(|b| b.rule_id == "core_damage" && b.matched)
    );
    assert!(
        breakdown
            .iter()
            .any(|b| b.rule_id == "base_cost_penalty" && b.matched)
    );
    assert!(
        breakdown
            .iter()
            .any(|b| b.rule_id == "setup_power" && b.matched)
    );
}

#[test]
fn targeted_does_not_merge() {
    let monsters = vec![
        MonsterInfo {
            name: "Slime A".into(),
            monster_id: None,
            index: 0,
            current_hp: Some(10),
            max_hp: Some(10),
            block: Some(0),
            ..Default::default()
        },
        MonsterInfo {
            name: "Slime B".into(),
            monster_id: None,
            index: 1,
            current_hp: Some(10),
            max_hp: Some(10),
            block: Some(0),
            ..Default::default()
        },
    ];
    // Build targeted contexts manually (no aoe var, different card IDs or has_target implied)
    let mut contexts = Vec::new();
    for monster in &monsters {
        let mut vars = HashMap::new();
        vars.insert("damage".to_string(), 6.0);
        vars.insert("hits".to_string(), 1.0);
        vars.insert("cost".to_string(), 1.0);
        vars.insert("current_energy".to_string(), 3.0);
        vars.insert("remaining_energy".to_string(), 2.0);
        vars.insert("monster_count".to_string(), monsters.len() as f64);
        vars.insert("card_base_score".to_string(), -10.0);
        vars.insert("weight".to_string(), 1.0);

        contexts.push(ActionContext {
            action_type: ActionType::PlayCard {
                card_id: "s1".into(),
                card_name: "Strike".into(),
            },
            card: None,
            target_index: Some(monster.index),
            target: Some(monster.clone()),
            monsters: monsters.clone(),
            parsed: crate::ranker::parser::ParsedEffects {
                damage: Some(6),
                hits: 1,
                ..Default::default()
            },
            vars,
        });
    }

    let rules = make_rules(vec![
        per_target_damage_rule("core_damage", 1050, 10),
        per_card_cost_rule("base_cost_penalty", 1000, -10),
    ]);

    let scored = rank_contexts(&contexts, &rules);
    assert_eq!(scored.len(), 2, "targeted Strike should NOT merge");
    assert_eq!(scored[0].target_index.unwrap(), 0);
    assert_eq!(scored[1].target_index.unwrap(), 1);
}
