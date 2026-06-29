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

// --- danger_enters_wrath_under_attack ---

fn make_enters_wrath_rule() -> Rule {
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

#[test]
fn wrath_penalty_formula_correct() {
    let rules = make_rules(vec![make_enters_wrath_rule()]);
    let mut vars = HashMap::new();
    vars.insert("enters_wrath".to_string(), 1.0);
    vars.insert("incoming_damage".to_string(), 9.0);
    vars.insert("current_hp".to_string(), 72.0);
    let ctx = make_ctx(vars);

    let results = evaluate(&ctx, &rules);
    assert!(results[0].matched);
    assert_eq!(results[0].score, 9 * -1000 / 72);
}

#[test]
fn wrath_penalty_not_triggered_when_not_entering_wrath() {
    let rules = make_rules(vec![make_enters_wrath_rule()]);
    let mut vars = HashMap::new();
    vars.insert("enters_wrath".to_string(), 0.0);
    vars.insert("incoming_damage".to_string(), 9.0);
    vars.insert("current_hp".to_string(), 72.0);
    let ctx = make_ctx(vars);

    let results = evaluate(&ctx, &rules);
    assert!(!results[0].matched);
    assert_eq!(results[0].score, 0);
}

#[test]
fn wrath_penalty_not_triggered_when_no_incoming_damage() {
    let rules = make_rules(vec![make_enters_wrath_rule()]);
    let mut vars = HashMap::new();
    vars.insert("enters_wrath".to_string(), 1.0);
    vars.insert("incoming_damage".to_string(), 0.0);
    vars.insert("current_hp".to_string(), 72.0);
    let ctx = make_ctx(vars);

    let results = evaluate(&ctx, &rules);
    assert!(!results[0].matched);
    assert_eq!(results[0].score, 0);
}

#[test]
fn wrath_penalty_scales_with_incoming_damage() {
    let rules = make_rules(vec![make_enters_wrath_rule()]);

    let mut vars_low = HashMap::new();
    vars_low.insert("enters_wrath".to_string(), 1.0);
    vars_low.insert("incoming_damage".to_string(), 5.0);
    vars_low.insert("current_hp".to_string(), 72.0);
    let ctx_low = make_ctx(vars_low);

    let mut vars_high = HashMap::new();
    vars_high.insert("enters_wrath".to_string(), 1.0);
    vars_high.insert("incoming_damage".to_string(), 36.0);
    vars_high.insert("current_hp".to_string(), 10.0);
    let ctx_high = make_ctx(vars_high);

    let results_low = evaluate(&ctx_low, &rules);
    let results_high = evaluate(&ctx_high, &rules);
    assert!(results_low[0].matched);
    assert!(results_high[0].matched);
    assert_eq!(results_low[0].score, 5 * -1000 / 72);
    assert_eq!(results_high[0].score, 36 * -1000 / 10);
    assert!(results_high[0].score < results_low[0].score);
}

#[test]
fn wrath_penalty_is_per_card_not_per_target() {
    let rule = make_enters_wrath_rule();
    assert_eq!(rule.category, RuleCategory::PerCard);
}

// --- rank_contexts edge cases ---

#[test]
fn rank_contexts_empty_returns_empty() {
    let rules = make_rules(vec![]);
    let scored = rank_contexts(&[], &rules);
    assert!(scored.is_empty());
}

#[test]
fn mixed_avoid_and_normal_in_rank() {
    let rules = make_rules(vec![
        Rule {
            category: RuleCategory::PerTarget,
            rule_id: "avoid_on_turn_1".into(),
            priority: 1000,
            weight: Weight::MinI64,
            formula: None,
            score_fn: None,
            override_rule: None,
            applies_to: vec!["play_card".into()],
            conditions: vec![Condition::State(crate::ranker::rules::StateCondition {
                state: crate::ranker::rules::StatePredicates {
                    turn_eq: Some(1),
                    ..Default::default()
                },
            })],
        },
        Rule {
            category: RuleCategory::PerTarget,
            rule_id: "normal_score".into(),
            priority: 1000,
            weight: Weight::Value(50),
            formula: None,
            score_fn: None,
            override_rule: None,
            applies_to: vec!["play_card".into()],
            conditions: vec![],
        },
    ]);
    let mut v1 = HashMap::new();
    v1.insert("turn".to_string(), 1.0);
    let mut v2 = HashMap::new();
    v2.insert("turn".to_string(), 2.0);
    let scored = rank_contexts(&[make_ctx(v1), make_ctx(v2)], &rules);
    assert_eq!(scored.len(), 2);
    let avoids: Vec<_> = scored.iter().filter(|s| s.is_avoid).collect();
    assert_eq!(avoids.len(), 1);
    assert_eq!(avoids[0].score, i64::MIN);
    let normals: Vec<_> = scored.iter().filter(|s| !s.is_avoid).collect();
    assert_eq!(normals.len(), 1);
    assert_eq!(normals[0].score, 50);
}

#[test]
fn end_turn_through_rank_contexts() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "end_turn_rule".into(),
        priority: 1000,
        weight: Weight::Value(30),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["end_turn".into()],
        conditions: vec![],
    }]);
    let ctx = end_turn_ctx(HashMap::new());
    let scored = rank_contexts(&[ctx], &rules);
    assert_eq!(scored.len(), 1);
    assert_eq!(scored[0].score, 30);
    assert!(!scored[0].is_avoid);
}

// --- AoE merge edge cases ---

#[test]
fn aoe_merge_with_avoid_score() {
    let monsters = vec![
        MonsterInfo {
            name: "A".into(),
            index: 0,
            current_hp: Some(10),
            max_hp: Some(10),
            can_be_killed: true,
            ..Default::default()
        },
        MonsterInfo {
            name: "B".into(),
            index: 1,
            current_hp: Some(10),
            max_hp: Some(10),
            can_be_killed: false,
            ..Default::default()
        },
    ];
    let contexts = aoe_contexts("tc-1", "Thunderclap", 4, 1, monsters);
    let rules = make_rules(vec![
        per_target_damage_rule("core_damage", 1050, 10),
        Rule {
            category: RuleCategory::PerTarget,
            rule_id: "avoid_killable".into(),
            priority: 800,
            weight: Weight::MinI64,
            formula: None,
            score_fn: None,
            override_rule: None,
            applies_to: vec!["play_card".into()],
            conditions: vec![Condition::Target(crate::ranker::rules::TargetCondition {
                target: crate::ranker::rules::TargetPredicates {
                    can_be_killed: Some(true),
                    ..Default::default()
                },
            })],
        },
    ]);
    let scored = rank_contexts(&contexts, &rules);
    assert_eq!(scored.len(), 1);
    assert!(scored[0].is_avoid);
    assert_eq!(scored[0].score, i64::MIN);
}

#[test]
fn aoe_single_monster_still_merges() {
    let monsters = vec![MonsterInfo {
        name: "Solo".into(),
        index: 0,
        current_hp: Some(10),
        max_hp: Some(10),
        ..Default::default()
    }];
    let contexts = aoe_contexts("tc-1", "Thunderclap", 4, 1, monsters);
    let rules = make_rules(vec![per_target_damage_rule("core_damage", 1050, 10)]);
    let scored = rank_contexts(&contexts, &rules);
    assert_eq!(scored.len(), 1, "single monster AoE should still merge");
    assert_eq!(scored[0].score, 40);
}

#[test]
fn multiple_aoe_cards_in_separate_groups() {
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        index: 0,
        current_hp: Some(10),
        max_hp: Some(10),
        ..Default::default()
    }];
    let contexts_a = aoe_contexts("card_a", "Cleave", 8, 1, monsters.clone());
    let contexts_b = aoe_contexts("card_b", "Whirlwind", 6, 1, monsters);
    let all: Vec<ActionContext> = [contexts_a, contexts_b].concat();
    let rules = make_rules(vec![per_target_damage_rule("core_damage", 1050, 10)]);
    let scored = rank_contexts(&all, &rules);
    assert_eq!(scored.len(), 2, "two different AoE cards stay separate");
}

#[test]
fn aoe_non_playcard_goes_to_standalone() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["end_turn".into()],
        conditions: vec![],
    }]);
    let mut vars = HashMap::new();
    vars.insert("aoe".to_string(), 1.0);
    let ctx = end_turn_ctx(vars);
    let scored = rank_contexts(&[ctx], &rules);
    assert_eq!(scored.len(), 1, "non-PlayCard AoE goes to standalone");
    assert_eq!(scored[0].score, 10);
}

// --- card conditions ---

fn status_card() -> crate::state::CardInfo {
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

#[test]
fn card_type_exact_match_matches() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                card_type: Some("ATTACK".into()),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Strike".into(),
        name: "Strike".into(),
        cost: 1,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: true,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_type_exact_mismatch_fails() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                card_type: Some("ATTACK".into()),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Defend".into(),
        name: "Defend".into(),
        cost: 1,
        card_type: "SKILL".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: false,
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_type_in_match() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                type_in: Some(vec!["ATTACK".into(), "SKILL".into()]),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Strike".into(),
        name: "Strike".into(),
        cost: 1,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: true,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_type_not_in_match() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                type_not_in: Some(vec!["STATUS".into(), "CURSE".into()]),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Strike".into(),
        name: "Strike".into(),
        cost: 1,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: true,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_type_not_in_fails_when_type_is_excluded() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                type_not_in: Some(vec!["STATUS".into(), "CURSE".into()]),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(status_card());
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_cost_eq_matches() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                cost_eq: Some(2),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Carnage".into(),
        name: "Carnage".into(),
        cost: 2,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: false,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_id_exact_match() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                id: Some("Strike_R".into()),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Strike_R".into(),
        name: "Strike".into(),
        cost: 1,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: true,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_id_in_match() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                id_in: Some(vec!["Strike_R".into(), "Strike_G".into()]),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Strike_R".into(),
        name: "Strike".into(),
        cost: 1,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: true,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_ethereal_match() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                ethereal: Some(true),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "AscendersBane".into(),
        name: "Ascender's Bane".into(),
        cost: 0,
        card_type: "CURSE".into(),
        upgraded: false,
        uuid: None,
        description: "Ethereal".into(),
        price: None,
        playable: false,
        has_target: false,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_is_none_fails_card_condition() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                card_type: Some("ATTACK".into()),
                ..Default::default()
            },
        })],
    }]);
    let ctx = make_ctx(HashMap::new());
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

// --- target conditions ---

#[test]
fn target_power_matches() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            power: Some("Thorns".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        monster_powers: vec![crate::state::PowerInfo {
            id: "Thorns".into(),
            name: "Thorns".into(),
            amount: 1,
        }],
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_power_not_matches_when_absent() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            power_not: Some("Thorns".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_any_power_in_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            any_power_in: Some(vec!["Thorns".into(), "Metallicize".into()]),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_powers: vec![crate::state::PowerInfo {
            id: "Thorns".into(),
            name: "Thorns".into(),
            amount: 1,
        }],
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_is_scaling_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_scaling: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        is_scaling: true,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_is_minion_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_minion: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_powers: vec![crate::state::PowerInfo {
            id: "Minion".into(),
            name: "Minion".into(),
            amount: 1,
        }],
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_is_minion_no_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_minion: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_can_be_killed_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            can_be_killed: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        can_be_killed: true,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_intent_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            intent: Some("ATTACK".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        intent: Some("ATTACK".into()),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_intent_not_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            intent_not: Some("ATTACK".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        intent: Some("BUFF".into()),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_is_none_fails_target_condition() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_scaling: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_ctx(HashMap::new());
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

// --- monster condition edge cases ---

#[test]
fn monsters_none_exclude_target_finds_none_other() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            none: Some(crate::ranker::rules::MonsterSubPredicates {
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
            index: 0,
            is_scaling: true,
            ..Default::default()
        },
        MonsterInfo {
            name: "Other".into(),
            index: 1,
            is_scaling: false,
            ..Default::default()
        },
    ];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monsters_none_with_intent_fails_when_monster_has_intent() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            none: Some(crate::ranker::rules::MonsterSubPredicates {
                intent: Some("ATTACK".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        index: 0,
        intent: Some("ATTACK".into()),
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monsters_none_with_intent_matches_when_no_monster_has_intent() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            none: Some(crate::ranker::rules::MonsterSubPredicates {
                intent: Some("ATTACK".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        index: 0,
        intent: Some("BUFF".into()),
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monsters_both_any_and_none_combined() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                is_scaling: Some(true),
                ..Default::default()
            }),
            none: Some(crate::ranker::rules::MonsterSubPredicates {
                intent: Some("ATTACK".into()),
                ..Default::default()
            }),
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![
        MonsterInfo {
            name: "Scaler".into(),
            index: 0,
            is_scaling: true,
            intent: Some("BUFF".into()),
            ..Default::default()
        },
        MonsterInfo {
            name: "Attacker".into(),
            index: 1,
            is_scaling: false,
            intent: Some("BUFF".into()),
            ..Default::default()
        },
    ];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

// --- parsed conditions ---

#[test]
fn parsed_damage_gt_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            damage_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        damage: Some(6),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_damage_gt_no_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            damage_gt: Some(10),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        damage: Some(5),
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_damage_eq_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            damage_eq: Some(6),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        damage: Some(6),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_block_gt_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            block_gt: Some(4),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        block: Some(8),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_self_damage_gt_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            self_damage_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        self_damage: Some(3),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exhausts_cards_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exhausts_cards: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        exhaust_count: 2,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exhausts_cards_no_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exhausts_cards: Some(false),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        exhaust_count: 2,
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exhausts_status_curse_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exhausts_status_curse: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut ctx = make_parsed_ctx(ParsedEffects {
        exhaust_count: 1,
        ..Default::default()
    });
    ctx.card = Some(status_card());
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exhausts_status_curse_no_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exhausts_status_curse: Some(false),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut ctx = make_parsed_ctx(ParsedEffects {
        exhaust_count: 1,
        ..Default::default()
    });
    ctx.card = Some(status_card());
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

// --- state condition no-match branches ---

#[test]
fn state_turn_eq_no_match() {
    let c = Condition::State(crate::ranker::rules::StateCondition {
        state: crate::ranker::rules::StatePredicates {
            turn_eq: Some(2),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("turn".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn state_remaining_energy_gt_no_match() {
    let c = Condition::State(crate::ranker::rules::StateCondition {
        state: crate::ranker::rules::StatePredicates {
            remaining_energy_gt: Some(2),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("remaining_energy".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn state_incoming_damage_gt_no_match() {
    let c = Condition::State(crate::ranker::rules::StateCondition {
        state: crate::ranker::rules::StatePredicates {
            incoming_damage_gt: Some(10),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("incoming_damage".to_string(), 5.0);
    let ctx = make_ctx(vars);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

// --- player relic conditions ---

#[test]
fn player_relic_matches() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            relic: Some("Chemical X".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("player_relic_Chemical X".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn player_relic_not_matches() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            relic_not: Some("Snecko Eye".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_ctx(HashMap::new());
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn player_relic_not_fails_when_present() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            relic_not: Some("Snecko Eye".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("player_relic_Snecko Eye".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

// --- score formula evaluation edge cases ---

#[test]
fn score_fn_dispatch_path_used() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test_score_fn".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: Some("hp_cost_penalty".into()),
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let mut vars = HashMap::new();
    vars.insert("self_damage".to_string(), 12.0);
    vars.insert("current_hp".to_string(), 60.0);
    let ctx = make_ctx(vars);
    let results = evaluate(&ctx, &rules);
    assert!(results[0].matched);
    assert_eq!(results[0].score, -58);
}

#[test]
fn formula_eval_failure_returns_zero_and_matches() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "bad_formula".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("max(@x)".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = make_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    assert!(results[0].matched);
    assert_eq!(results[0].score, 0);
}

#[test]
fn weight_only_no_formula_no_score_fn() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "weight_only".into(),
        priority: 1000,
        weight: Weight::Value(42),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = make_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    assert!(results[0].matched);
    assert_eq!(results[0].score, 42);
}

// --- compute condition edge cases ---

#[test]
fn compute_condition_zero_evaluates_false() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "zcheck".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Compute(crate::ranker::rules::ComputeCondition {
            compute: crate::ranker::rules::ComputePredicates {
                formula: Some("0".into()),
            },
        })],
    }]);
    let ctx = make_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    assert!(!results[0].matched);
    assert_eq!(results[0].score, 0);
}

#[test]
fn compute_condition_formula_error_returns_false() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "bad_compute".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Compute(crate::ranker::rules::ComputeCondition {
            compute: crate::ranker::rules::ComputePredicates {
                formula: Some("max(@x)".into()),
            },
        })],
    }]);
    let ctx = make_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    assert!(!results[0].matched);
    assert_eq!(results[0].score, 0);
}

// --- applies_to mismatch ---

#[test]
fn play_card_rule_does_not_apply_to_end_turn() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "only_play".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = end_turn_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    assert!(!results[0].matched);
    assert_eq!(results[0].score, 0);
}

// --- additional negative branches ---

#[test]
fn card_cost_eq_no_match() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                cost_eq: Some(2),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Strike".into(),
        name: "Strike".into(),
        cost: 1,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: true,
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_power_not_fails_when_present() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            power_not: Some("Thorns".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_powers: vec![crate::state::PowerInfo {
            id: "Thorns".into(),
            name: "Thorns".into(),
            amount: 1,
        }],
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_is_scaling_no_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_scaling: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        is_scaling: false,
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_can_be_killed_no_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            can_be_killed: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        can_be_killed: false,
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_intent_no_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            intent: Some("ATTACK".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        intent: Some("BUFF".into()),
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_intent_not_fails_when_intent_matches() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            intent_not: Some("ATTACK".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        intent: Some("ATTACK".into()),
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_any_power_in_no_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            any_power_in: Some(vec!["Thorns".into(), "Metallicize".into()]),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_powers: vec![crate::state::PowerInfo {
            id: "Strength".into(),
            name: "Strength".into(),
            amount: 3,
        }],
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_damage_eq_no_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            damage_eq: Some(6),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        damage: Some(8),
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exhausts_cards_true_fails_when_count_zero() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exhausts_cards: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        exhaust_count: 0,
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exhausts_cards_false_matches_when_count_zero() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exhausts_cards: Some(false),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        exhaust_count: 0,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

// --- sort behavior ---

#[test]
fn equal_scores_preserve_order() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "equal_score".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("@weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx_a = {
        let mut v = HashMap::new();
        v.insert("tag".to_string(), 1.0);
        make_ctx(v)
    };
    let ctx_b = {
        let mut v = HashMap::new();
        v.insert("tag".to_string(), 2.0);
        make_ctx(v)
    };
    let scored = rank_contexts(&[ctx_a, ctx_b], &rules);
    assert_eq!(scored.len(), 2);
    assert_eq!(scored[0].score, 10);
    assert_eq!(scored[1].score, 10);
}
