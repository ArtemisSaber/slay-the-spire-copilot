use super::*;

#[test]
fn postmortem_monsters_without_name_or_index_are_filtered() {
    let mut combat = normalized_fixture("combat-state.json");
    combat["room_type"] = json!("MonsterRoom");
    combat["screen_type"] = json!("NONE");
    combat["monsters"] = json!([
        {"current_hp": 10, "max_hp": 10, "block": 0},
        {"name": "Named", "current_hp": 12, "max_hp": 12, "block": 0}
    ]);
    combat["current_hp"] = json!(75);

    let mut combat_end = combat.clone();
    combat_end["screen_type"] = json!("COMBAT_REWARD");
    combat_end["monsters"] = json!([]);
    combat_end["current_hp"] = json!(60);

    let input = [state_event(combat), state_event(combat_end)].join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();
    assert!(report.contains("Named"));
}

#[test]
fn postmortem_new_monster_mid_combat_is_tracked() {
    let mut combat = normalized_fixture("combat-state.json");
    combat["room_type"] = json!("MonsterRoom");
    combat["screen_type"] = json!("BATTLE");
    combat["monsters"] = json!([
        {"name": "虫", "current_hp": 10, "max_hp": 10, "block": 0, "index": 0}
    ]);
    combat["current_hp"] = json!(75);

    let mut combat_mid = combat.clone();
    combat_mid["monsters"] = json!([
        {"name": "虫", "current_hp": 10, "max_hp": 10, "block": 0, "index": 0},
        {"name": "鼠", "current_hp": 8, "max_hp": 8, "block": 0, "index": 1}
    ]);
    combat_mid["current_hp"] = json!(65);

    let mut combat_end = combat_mid.clone();
    combat_end["screen_type"] = json!("COMBAT_REWARD");
    combat_end["monsters"] = json!([]);
    combat_end["current_hp"] = json!(55);

    let input = [
        state_event(combat),
        state_event(combat_mid),
        state_event(combat_end),
    ]
    .join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();
    assert!(report.contains("虫"));
    assert!(report.contains("鼠"));
}

#[test]
fn postmortem_duplicate_monster_name_is_only_added_once() {
    let mut combat = normalized_fixture("combat-state.json");
    combat["room_type"] = json!("MonsterRoom");
    combat["screen_type"] = json!("NONE");
    combat["monsters"] = json!([
        {"name": "虱虫", "current_hp": 11, "max_hp": 11, "block": 0, "intent": "ATTACK", "damage": 6, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false},
        {"name": "虱虫", "current_hp": 12, "max_hp": 12, "block": 0, "intent": "ATTACK", "damage": 6, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false}
    ]);
    combat["current_hp"] = json!(68);

    let mut combat_end = combat.clone();
    combat_end["screen_type"] = json!("COMBAT_REWARD");
    combat_end["monsters"] = json!([]);
    combat_end["current_hp"] = json!(62);

    let input = [state_event(combat), state_event(combat_end)].join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();
    // "虱虫" should appear exactly once in global list (deduplicated)
    let count = report.match_indices("虱虫").count();
    // Appears in global monster list + combat entry = 2
    assert_eq!(
        count, 2,
        "虱虫 should appear in global list once and in combat label"
    );
}

#[test]
fn postmortem_unrecognized_deck_change_produces_no_reward_line() {
    let reward = normalized_fixture("card-reward-state.json");
    let mut after = reward.clone();
    after["screen_type"] = Value::String("NONE".into());
    after["card_reward_choices"] = Value::Array(vec![]);
    after["master_cards"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"MysteryCard","name":"MysteryCard","cost":1,"card_type":"ATTACK","upgraded":false,"uuid":"unknown"}));

    let input = [state_event(reward), state_event(after)].join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();
    assert!(!report.contains("已选:"));
    assert!(!report.contains("可能跳过"));
}

#[test]
fn postmortem_card_reward_without_choices_creates_no_snapshot() {
    let state = json!({
        "character": "IRONCLAD",
        "current_hp": 75,
        "max_hp": 75,
        "gold": 99,
        "floor": 1,
        "screen_type": "CARD_REWARD",
        "monsters": [],
        "master_cards": [],
        "relics": [],
        "potions": [],
        "deck_names": []
    });
    let report =
        generate_report_from_jsonl(&state_event(state), crate::test_utils::test_locale()).unwrap();
    assert!(!report.contains("## 选牌记录"));
}

#[test]
fn postmortem_multiple_combats_across_types_count_correctly() {
    let base = normalized_fixture("combat-state.json");

    let mut normal_combat = base.clone();
    normal_combat["room_type"] = json!("MonsterRoom");
    normal_combat["screen_type"] = json!("NONE");
    normal_combat["monsters"] = json!([
        {"name": "虱虫", "current_hp": 11, "max_hp": 11, "block": 0, "intent": "ATTACK", "damage": 6, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false}
    ]);
    normal_combat["current_hp"] = json!(68);
    let mut normal_end = normal_combat.clone();
    normal_end["screen_type"] = json!("COMBAT_REWARD");
    normal_end["monsters"] = json!([]);
    normal_end["current_hp"] = json!(62);

    let mut elite_combat = base.clone();
    elite_combat["room_type"] = json!("MonsterRoomElite");
    elite_combat["screen_type"] = json!("NONE");
    elite_combat["monsters"] = json!([
        {"name": "地精大法师", "current_hp": 60, "max_hp": 60, "block": 0, "intent": "ATTACK", "damage": 10, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false}
    ]);
    elite_combat["current_hp"] = json!(62);
    let mut elite_end = elite_combat.clone();
    elite_end["screen_type"] = json!("COMBAT_REWARD");
    elite_end["monsters"] = json!([]);
    elite_end["current_hp"] = json!(40);

    let mut boss_combat = base.clone();
    boss_combat["room_type"] = json!("MonsterRoomBoss");
    boss_combat["screen_type"] = json!("NONE");
    boss_combat["monsters"] = json!([
        {"name": "六火亡魂", "current_hp": 250, "max_hp": 250, "block": 0, "intent": "ATTACK", "damage": 20, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false}
    ]);
    boss_combat["current_hp"] = json!(40);
    let mut boss_end = boss_combat.clone();
    boss_end["screen_type"] = json!("COMBAT_REWARD");
    boss_end["monsters"] = json!([]);
    boss_end["current_hp"] = json!(5);

    let input = [
        state_event(normal_combat),
        state_event(normal_end),
        state_event(elite_combat),
        state_event(elite_end),
        state_event(boss_combat),
        state_event(boss_end),
    ]
    .join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("(普通:1 精英:1 Boss:1)"));
    assert!(report.contains("(精英)"));
    assert!(report.contains("(Boss)"));
}
