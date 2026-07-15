use super::*;
use crate::learning::action::SemanticAction;

#[test]
fn exposes_stable_ranked_actions_and_aggregate_tags_for_learning() {
    let mut state = combat_state_with_cards(3, vec![strike_card("local-uuid", 1)]);
    state.monsters[0].monster_id = Some("JawWorm".into());

    let ranked = recorded_ranked_actions(&state);
    let strike = ranked.iter().find(|item| {
        matches!(
            &item.semantic_action,
            SemanticAction::PlayCard { card_id, .. } if card_id == "Strike_R"
        )
    });

    assert!(strike.is_some());
    assert!(matches!(
        &strike.unwrap().semantic_action,
        SemanticAction::PlayCard {
            target_monster_id: Some(id),
            ..
        } if id == "JawWorm"
    ));
    let tags = ranker_tags(&state);
    assert!(tags.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(tags.iter().any(|tag| tag == "damage"));
}

#[test]
fn omits_ranked_actions_that_cannot_be_mapped_to_stable_ids() {
    let state = combat_state_with_cards(3, vec![strike_card("local-uuid", 1)]);

    assert!(
        recorded_ranked_actions(&state)
            .iter()
            .all(|item| !matches!(item.semantic_action, SemanticAction::PlayCard { .. }))
    );
}
