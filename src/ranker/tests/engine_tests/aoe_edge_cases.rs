use super::*;

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
