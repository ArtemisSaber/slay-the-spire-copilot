use super::*;

#[test]
fn scenario_resolver_detects_boss_card_reward_floors() {
    for floor in [16, 33, 50] {
        let state = NormalizedState {
            screen_type: Some(ScreenType::CardReward),
            floor: Some(floor),
            ..test_state()
        };
        assert_eq!(
            AdviceScenario::from_state(&state),
            AdviceScenario::BossCardReward
        );
    }
}

#[test]
fn scenario_resolver_detects_ordinary_card_reward() {
    let state = NormalizedState {
        screen_type: Some(ScreenType::CardReward),
        floor: Some(14),
        ..test_state()
    };
    assert_eq!(
        AdviceScenario::from_state(&state),
        AdviceScenario::CardReward
    );
}

#[test]
fn scenario_resolver_detects_rest() {
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        ..test_state()
    };
    assert_eq!(AdviceScenario::from_state(&state), AdviceScenario::Rest);
}

#[test]
fn scenario_resolver_detects_boss_relic() {
    let state = NormalizedState {
        screen_type: Some(ScreenType::BossReward),
        boss_relic_choices: vec![RelicInfo {
            id: "Runic Dome".into(),
            name: "符文圆顶".into(),
            description: String::new(),
            counter: None,
            price: None,
        }],
        ..test_state()
    };
    assert_eq!(
        AdviceScenario::from_state(&state),
        AdviceScenario::BossRelic
    );
}

#[test]
fn scenario_resolver_detects_event_choice() {
    let state = NormalizedState {
        screen_type: Some(ScreenType::Event),
        event_choices: vec!["获得遗物".into()],
        ..test_state()
    };
    assert_eq!(
        AdviceScenario::from_state(&state),
        AdviceScenario::EventChoice
    );
}

#[test]
fn scenario_resolver_detects_combat_entry() {
    let state = NormalizedState {
        monsters: vec![monster()],
        ..test_state()
    };
    assert_eq!(
        AdviceScenario::from_state(&state),
        AdviceScenario::CombatEntry
    );
}

#[test]
fn scenario_resolver_defaults_to_generic() {
    assert_eq!(
        AdviceScenario::from_state(&test_state()),
        AdviceScenario::Generic
    );
}

#[test]
fn scenario_resolver_detects_map_suggestion() {
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        map_first_node_chosen: Some(false),
        ..test_state()
    };
    assert_eq!(
        AdviceScenario::from_state(&state),
        AdviceScenario::MapSuggestion
    );
}

#[test]
fn scenario_resolver_detects_map_crossroad() {
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        map_first_node_chosen: Some(true),
        ..test_state()
    };
    assert_eq!(
        AdviceScenario::from_state(&state),
        AdviceScenario::MapCrossroad
    );
}

#[test]
fn scenario_resolver_detects_shop() {
    let state = NormalizedState {
        screen_type: Some(ScreenType::ShopScreen),
        ..test_state()
    };
    assert_eq!(AdviceScenario::from_state(&state), AdviceScenario::Shop);
}
