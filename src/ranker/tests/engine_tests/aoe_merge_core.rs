use super::*;

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
