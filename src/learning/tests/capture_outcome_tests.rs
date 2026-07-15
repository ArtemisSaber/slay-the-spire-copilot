use crate::autoplay::action::AutoPlayAction;
use crate::learning::action::SemanticAction;
use crate::learning::config::{MemoryConfig, MemoryMode};
use crate::learning::session::{ExecutedPlan, LearningSession, SessionProvenance};
use crate::learning::snapshot::KnowledgeSnapshot;
use crate::learning::store::KnowledgeStore;
use crate::learning::telemetry::{DecisionSource, RecordedRankedAction};
use crate::state::{MonsterInfo, NormalizedState, PotionInfo, ScreenType};

fn combat_state(turn: i64) -> NormalizedState {
    NormalizedState {
        screen_type: Some(ScreenType::None),
        character: Some("IRONCLAD".into()),
        seed: Some(42),
        ascension_level: Some(20),
        floor: Some(8),
        current_hp: Some(40),
        max_hp: Some(80),
        energy: Some(3),
        block: Some(0),
        turn_number: Some(turn),
        monsters: vec![MonsterInfo {
            monster_id: Some("GremlinNob".into()),
            index: 0,
            current_hp: Some(20),
            max_hp: Some(100),
            ..MonsterInfo::default()
        }],
        potions: vec![PotionInfo {
            id: Some("FirePotion".into()),
            slot: 0,
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

fn plan(action: AutoPlayAction, semantic: SemanticAction) -> ExecutedPlan {
    ExecutedPlan {
        action,
        source: DecisionSource::Llm,
        selected_action_id: "current".into(),
        available_semantic_actions: vec![semantic],
        ranked_suggestions: vec![],
        memory_ids_used: vec![],
    }
}

fn session(root: &std::path::Path) -> LearningSession {
    let config = MemoryConfig {
        mode: MemoryMode::Collect,
        mod_profile_sha256: Some("mods".into()),
        mod_profile_approved: true,
        ..MemoryConfig::default()
    };
    LearningSession::new(
        config,
        KnowledgeStore::new(root),
        KnowledgeSnapshot::build(vec![], &[]).unwrap(),
        SessionProvenance {
            locale: "en".into(),
            model_profile_sha256: "model".into(),
            rules_sha256: "rules".into(),
            synthetic_input: false,
        },
    )
}

#[test]
fn combat_outcomes_include_terminal_turn_and_all_autoplay_potions_used() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = session(temp.path());

    let first = combat_state(1);
    session.observe(&first);
    let prepared = session.prepare(&first, &[]).unwrap();
    let end = plan(AutoPlayAction::End, SemanticAction::EndTurn);
    session.record_executed("run", &first, &end, &prepared);

    let second = combat_state(2);
    session.observe(&second);
    let prepared = session.prepare(&second, &[]).unwrap();
    let potion = SemanticAction::UsePotion {
        potion_id: "FirePotion".into(),
        target_monster_id: Some("GremlinNob".into()),
    };
    let drink = plan(
        AutoPlayAction::Drink {
            slot_index: 0,
            target_index: Some(0),
        },
        potion,
    );
    session.record_executed("run", &second, &drink, &prepared);

    let mut terminal = second;
    terminal.screen_type = Some(ScreenType::GameOver);
    terminal.monsters.clear();
    terminal.current_hp = Some(35);
    terminal.floor = Some(51);
    session.observe(&terminal);
    session.finalize("game_over").unwrap();

    assert_eq!(session.snapshot().cases.len(), 2);
    assert!(session.snapshot().cases.iter().all(|case| {
        case.outcome.combat_turns == Some(2)
            && case.outcome.potions_used == ["FirePotion".to_string()]
    }));
}

#[test]
fn recording_caps_telemetry_and_retains_the_selected_semantic_action() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = session(temp.path());
    let state = combat_state(1);
    session.observe(&state);
    let prepared = session.prepare(&state, &[]).unwrap();
    let mut executed = plan(AutoPlayAction::End, SemanticAction::EndTurn);
    executed.available_semantic_actions = (0..33)
        .map(|index| SemanticAction::PlayCard {
            card_id: format!("Card{index}"),
            upgraded: false,
            target_monster_id: None,
        })
        .collect();
    executed.ranked_suggestions = (0..65)
        .map(|score| RecordedRankedAction {
            semantic_action: SemanticAction::EndTurn,
            score,
            tags: vec![],
        })
        .collect();

    let event = session
        .record_executed("run", &state, &executed, &prepared)
        .unwrap();

    assert_eq!(event["available_actions"].as_array().unwrap().len(), 32);
    assert!(
        event["available_actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| action["semantic_action"]["kind"] == "end_turn")
    );
    assert_eq!(event["ranked_suggestions"].as_array().unwrap().len(), 64);
}
