use super::*;

#[test]
fn top_ranked_returns_all_legal_non_avoided_actions() {
    let state = combat_state_with_cards(
        3,
        vec![strike_card("s1", 1), defend_card(), limit_break_card()],
    );
    let context = top_ranked_context(&state).expect("should produce ranked context");
    let rs = context["ranked_suggestions"]
        .as_array()
        .expect("should be array");

    assert!(
        rs.iter()
            .any(|e| e["action"]["PlayCard"]["card_name"].as_str() == Some("Strike")),
        "legal targeted cards should be included"
    );
    assert!(
        rs.iter()
            .any(|e| e["action"]["PlayCard"]["card_name"].as_str() == Some("Defend")),
        "legal lower-ranked cards should still be included"
    );
    assert!(
        rs.iter()
            .any(|e| e["action"] == serde_json::json!("EndTurn")),
        "legal EndTurn action should still be included"
    );
    assert!(
        rs.iter()
            .all(|e| e["action"]["PlayCard"]["card_name"].as_str() != Some("Limit Break")),
        "avoided actions should be filtered out of ranked suggestions"
    );
}

#[test]
fn top_ranked_returns_flat_entries_with_score_and_tags() {
    let state = combat_state_with_cards(3, vec![strike_card("s1", 1)]);
    let context = top_ranked_context(&state).expect("should produce ranked context");
    let rs = context["ranked_suggestions"]
        .as_array()
        .expect("should be array");
    assert!(!rs.is_empty(), "should have ranked actions");
    let first = &rs[0];
    assert!(first["score"].is_i64(), "should have numeric score");
    assert!(first["tags"].is_array(), "should have tags array");
    assert!(
        first["tags"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("damage")),
        "Strike should have damage tag"
    );
}

#[test]
fn derive_tags_marks_retaliatory_damage() {
    let tags = derive_tags(&[crate::ranker::engine::RuleResult {
        rule_id: "danger_retaliatory_damage".to_string(),
        score: -1,
        matched: true,
    }]);

    assert_eq!(tags, vec!["retaliation"]);
}

#[test]
fn top_ranked_context_warns_when_retaliation_is_lethal() {
    let state: NormalizedState = serde_json::from_value(serde_json::json!({
        "screen_type": "NONE",
        "energy": 1,
        "block": 0,
        "current_hp": 3,
        "max_hp": 80,
        "incoming_damage": 6,
        "hand": [{
            "id": "Strike_R",
            "name": "Strike",
            "cost": 1,
            "card_type": "ATTACK",
            "uuid": "strike-1",
            "description": "Deal 6 damage.",
            "has_target": true,
            "playable": true
        }],
        "monsters": [{
            "index": 0,
            "name": "The Guardian",
            "current_hp": 5,
            "max_hp": 240,
            "block": 0,
            "intent": "ATTACK",
            "damage": 6,
            "hits": 1,
            "monster_powers": [{
                "id": "Sharp Hide",
                "name": "Sharp Hide",
                "amount": 3
            }],
            "can_be_killed": true,
            "is_scaling": false
        }]
    }))
    .expect("test state should deserialize");

    let context = top_ranked_context(&state).expect("combat should be ranked");
    let attack = context["ranked_suggestions"]
        .as_array()
        .expect("ranked_suggestions should be an array")
        .iter()
        .find(|entry| entry["action"]["PlayCard"]["card_id"] == "strike-1")
        .expect("attack should be present in ranked suggestions");

    assert!(
        attack["tags"]
            .as_array()
            .expect("tags should be an array")
            .iter()
            .any(|tag| tag == "retaliation")
    );
    assert!(
        attack["score"]
            .as_i64()
            .expect("score should be an integer")
            < -100_000,
        "lethal retaliation must dominate the attack's lethal reward"
    );
}
