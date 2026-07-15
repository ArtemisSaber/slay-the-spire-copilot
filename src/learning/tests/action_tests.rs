use crate::autoplay::action::{ActionCandidate, AutoPlayAction};
use crate::learning::action::SemanticAction;
use crate::state::{CardInfo, MonsterInfo, NormalizedState, PotionInfo};

fn state() -> NormalizedState {
    NormalizedState {
        hand: vec![CardInfo {
            id: "Bash".into(),
            name: "Bash".into(),
            cost: 2,
            card_type: "ATTACK".into(),
            upgraded: true,
            uuid: Some("run-local".into()),
            description: String::new(),
            price: None,
            playable: true,
            has_target: true,
        }],
        monsters: vec![MonsterInfo {
            monster_id: Some("GremlinNob".into()),
            index: 4,
            ..MonsterInfo::default()
        }],
        potions: vec![PotionInfo {
            id: Some("Fire Potion".into()),
            slot: 2,
            name: "Fire Potion".into(),
            description: String::new(),
            price: None,
            can_use: true,
            can_discard: true,
            requires_target: true,
        }],
        ..NormalizedState::default()
    }
}

#[test]
fn maps_execution_local_card_data_to_stable_meaning() {
    let semantic = SemanticAction::from_execution(
        &AutoPlayAction::Play {
            hand_index: 0,
            target_index: Some(4),
        },
        &state(),
    );

    assert_eq!(
        semantic,
        Some(SemanticAction::PlayCard {
            card_id: "Bash".into(),
            upgraded: true,
            target_monster_id: Some("GremlinNob".into()),
        })
    );
}

#[test]
fn maps_potions_by_id_and_end_turn_without_slots_or_indices() {
    assert_eq!(
        SemanticAction::from_execution(
            &AutoPlayAction::Drink {
                slot_index: 2,
                target_index: Some(4),
            },
            &state(),
        ),
        Some(SemanticAction::UsePotion {
            potion_id: "Fire Potion".into(),
            target_monster_id: Some("GremlinNob".into()),
        })
    );
    assert_eq!(
        SemanticAction::from_execution(&AutoPlayAction::End, &state()),
        Some(SemanticAction::EndTurn)
    );
}

#[test]
fn refuses_cross_run_mapping_when_a_stable_id_is_missing() {
    let mut state = state();
    state.monsters[0].monster_id = None;
    assert!(
        SemanticAction::from_execution(
            &AutoPlayAction::Play {
                hand_index: 0,
                target_index: Some(4),
            },
            &state,
        )
        .is_none()
    );

    state.monsters[0].monster_id = Some("GremlinNob".into());
    state.potions[0].id = None;
    assert!(
        SemanticAction::from_execution(
            &AutoPlayAction::Drink {
                slot_index: 2,
                target_index: Some(4),
            },
            &state,
        )
        .is_none()
    );
}

#[test]
fn non_combat_actions_have_no_semantic_combat_identity() {
    for action in [
        AutoPlayAction::Choose(0),
        AutoPlayAction::Skip,
        AutoPlayAction::Proceed,
        AutoPlayAction::Leave,
    ] {
        assert!(SemanticAction::from_execution(&action, &state()).is_none());
    }
}

#[test]
fn available_actions_expand_targets_and_remove_execution_local_ids() {
    let state = state();
    let candidates = vec![
        ActionCandidate {
            kind: "play".into(),
            action_id: "combat:play:run-local".into(),
            label: "Play Bash".into(),
            target_required: Some(true),
        },
        ActionCandidate {
            kind: "drink".into(),
            action_id: "combat:potion:2".into(),
            label: "Drink Fire Potion".into(),
            target_required: Some(true),
        },
        ActionCandidate {
            kind: "end".into(),
            action_id: "combat:end".into(),
            label: "End turn".into(),
            target_required: None,
        },
    ];

    let actions = crate::learning::action::available_semantic_actions(&candidates, &state);

    assert_eq!(
        actions,
        vec![
            SemanticAction::PlayCard {
                card_id: "Bash".into(),
                upgraded: true,
                target_monster_id: Some("GremlinNob".into()),
            },
            SemanticAction::UsePotion {
                potion_id: "Fire Potion".into(),
                target_monster_id: Some("GremlinNob".into()),
            },
            SemanticAction::EndTurn,
        ]
    );
}

#[test]
fn available_actions_fail_closed_when_target_ids_are_missing() {
    let mut state = state();
    state.monsters[0].monster_id = None;
    let candidates = vec![ActionCandidate {
        kind: "play".into(),
        action_id: "combat:play:run-local".into(),
        label: "Play Bash".into(),
        target_required: Some(true),
    }];

    assert!(crate::learning::action::available_semantic_actions(&candidates, &state).is_empty());
}

#[test]
fn duplicate_monster_ids_are_ambiguous_for_cross_run_targets() {
    let mut state = state();
    let mut duplicate = state.monsters[0].clone();
    duplicate.index = 5;
    state.monsters.push(duplicate);

    assert!(
        SemanticAction::from_execution(
            &AutoPlayAction::Play {
                hand_index: 0,
                target_index: Some(4),
            },
            &state,
        )
        .is_none()
    );
}
