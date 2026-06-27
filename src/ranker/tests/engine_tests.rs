use std::collections::HashMap;

use crate::ranker::context::{ActionContext, ActionType};
use crate::ranker::engine::{evaluate, rank_contexts};
use crate::ranker::parser::ParsedEffects;
use crate::ranker::rules::{Condition, Rule, RuleSet, Weight};
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
        rule_id: "test_flat".into(),
        priority: 1000,
        weight: Weight::Value(-100),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        per_target: false,
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
        rule_id: "test_formula".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("@damage * @hits * @weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        per_target: false,
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
            rule_id: "override_me".into(),
            priority: 2000,
            weight: Weight::Value(10),
            formula: Some("@damage * @weight".into()),
            score_fn: None,
            override_rule: None,
            applies_to: vec!["play_card".into()],
            per_target: false,
            conditions: vec![],
        },
        Rule {
            rule_id: "the_override".into(),
            priority: 1000,
            weight: Weight::Value(5),
            formula: Some("@damage * @weight".into()),
            score_fn: None,
            override_rule: Some("override_me".into()),
            applies_to: vec!["play_card".into()],
            per_target: false,
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
        rule_id: "min".into(),
        priority: 1000,
        weight: Weight::MinI64,
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        per_target: false,
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
    let rules = make_rules(vec![]);
    let ctx1 = make_ctx(HashMap::new());
    let ctx2 = end_turn_ctx(HashMap::new());

    let scored = rank_contexts(&[ctx1, ctx2], &rules);
    assert_eq!(scored.len(), 2);
    assert_eq!(scored[0].score, 0);
    assert_eq!(scored[1].score, 0);
}

// --- parsed condition predicates ---

fn rule_with_condition(cond: Condition) -> Rule {
    Rule {
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        per_target: false,
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
fn per_target_sums_across_targets() {
    let rules = make_rules(vec![Rule {
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("@weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        per_target: true,
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
    assert_eq!(results[0].score, 20);
    assert!(results[0].matched);
}

#[test]
fn per_target_no_targets_returns_zero() {
    let rules = make_rules(vec![Rule {
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("@weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        per_target: true,
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
    assert_eq!(results[0].score, 0);
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
