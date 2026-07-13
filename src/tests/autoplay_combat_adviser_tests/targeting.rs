use super::*;

#[test]
fn top_ranked_includes_target_resolution() {
    let state = combat_state_with_cards(3, vec![strike_card("s1", 1)]);
    let context = top_ranked_context(&state).expect("should produce ranked context");
    let rs = context["ranked_suggestions"].as_array().unwrap();
    let first = &rs[0];
    let target = &first["target"];
    assert_eq!(target["name"], "Jaw Worm");
    assert_eq!(target["hp"], 20);

    let strike_entries: Vec<_> = rs
        .iter()
        .filter(|e| e["action"]["PlayCard"]["card_name"].as_str() == Some("Strike"))
        .collect();
    assert_eq!(strike_entries.len(), 1);
    assert_eq!(strike_entries[0]["target"]["index"], 0);
}

#[test]
fn top_ranked_excludes_avoided_actions() {
    let state = combat_state_with_cards(
        3,
        vec![serde_json::json!({
            "id": "Limit Break",
            "name": "Limit Break",
            "cost": 1,
            "card_type": "SKILL",
            "uuid": "lb-1",
            "description": "Double your Strength.",
            "has_target": false,
            "playable": true
        })],
    );
    let context = top_ranked_context(&state).expect("should produce ranked context");
    let rs = context["ranked_suggestions"].as_array().unwrap();
    let limit_break_entries: Vec<_> = rs
        .iter()
        .filter(|e| e["action"]["PlayCard"]["card_name"].as_str() == Some("Limit Break"))
        .collect();
    assert!(
        limit_break_entries.is_empty(),
        "Limit Break with 0 Strength should be excluded"
    );
}

#[test]
fn top_ranked_returns_none_for_non_combat() {
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        ..NormalizedState::default()
    };
    assert_eq!(top_ranked_context(&state), None);
}

#[test]
fn aoe_merged_into_single_entry_with_null_target() {
    let state = two_monster_state(3, vec![thunderclap_card()]);
    let context = top_ranked_context(&state).expect("should produce ranked context");
    let rs = context["ranked_suggestions"].as_array().unwrap();
    let aoe_entries: Vec<_> = rs
        .iter()
        .filter(|e| e["action"]["PlayCard"]["card_name"].as_str() == Some("Thunderclap"))
        .collect();
    assert_eq!(
        aoe_entries.len(),
        1,
        "AoE card should produce one merged entry"
    );
    assert!(
        aoe_entries[0]["target"].is_null(),
        "merged AoE entry should have null target"
    );
}

#[test]
fn targeted_keeps_separate_entries_per_monster() {
    let state = two_monster_state(3, vec![strike_card("s1", 1)]);
    let context = top_ranked_context(&state).expect("should produce ranked context");
    let rs = context["ranked_suggestions"].as_array().unwrap();
    let strike_entries: Vec<_> = rs
        .iter()
        .filter(|e| e["action"]["PlayCard"]["card_name"].as_str() == Some("Strike"))
        .collect();
    assert_eq!(
        strike_entries.len(),
        2,
        "targeted Strike vs 2 monsters should produce 2 entries"
    );
    assert_eq!(strike_entries[0]["target"]["index"], 0);
    assert_eq!(strike_entries[1]["target"]["index"], 1);
}
