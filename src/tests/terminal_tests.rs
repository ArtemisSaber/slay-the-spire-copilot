use super::learning_status_with_config;
use super::menu::{HomeAction, choose_run, prompt_home};
use super::review::{ReviewableRun, list_reviewable_runs, review_run_with_provider};
use crate::learning::action::SemanticAction;
use crate::learning::case::{CaseDraft, CaseOutcome, CaseProvenance, seed_hash};
use crate::learning::descriptor::{
    AscensionBand, BlockThreatBucket, CountBucket, MonsterDescriptor, RatioBucket,
    SituationDescriptor, TurnBucket,
};
use crate::learning::eligibility::RunObjective;
use crate::learning::store::KnowledgeStore;
use crate::learning::telemetry::DecisionSource;
use std::collections::HashMap;
use std::io::Cursor;

#[test]
fn terminal_home_exposes_review_without_a_new_command() {
    let mut input = Cursor::new("1\n");
    let mut output = Vec::new();

    let action = prompt_home(&mut input, &mut output, "Learning: on").unwrap();

    assert_eq!(action, HomeAction::Review);
    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("Terminal control center"));
    assert!(rendered.contains("Review a completed run"));
    assert!(rendered.contains("standard input/output"));
}

#[test]
fn terminal_run_picker_uses_the_visible_number() {
    let runs = vec![
        ReviewableRun {
            run_id: "newer-run".into(),
            case_count: 9,
            lesson_count: 0,
        },
        ReviewableRun {
            run_id: "older-run".into(),
            case_count: 4,
            lesson_count: 1,
        },
    ];
    let mut input = Cursor::new("0\n2\n");
    let mut output = Vec::new();

    let selected = choose_run(&mut input, &mut output, &runs).unwrap();

    assert_eq!(selected.as_deref(), Some("older-run"));
    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("Please choose a listed run or b."));
    assert!(rendered.contains("2) older-run — 4 cases, 1 lesson"));
}

#[test]
fn terminal_status_does_not_create_storage_while_learning_is_off() {
    let temp = tempfile::tempdir().unwrap();
    let config = crate::config::Config::from_map(&HashMap::from([("MEMORY_MODE", "off")]));

    let status = learning_status_with_config(temp.path(), &config).unwrap();

    assert!(status.contains("Learning: off"));
    assert!(!temp.path().join("learning").exists());
}

#[tokio::test]
async fn terminal_review_turns_an_existing_run_into_a_lesson() {
    let temp = tempfile::tempdir().unwrap();
    let run_id = "2026-07-15_17-50_Ironclad_A0";
    let run_dir = temp.path().join("runs").join(run_id);
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::write(
        run_dir.join("events.jsonl"),
        format!(
            "{{\"schema_version\":1,\"event\":\"run_started\",\"run_id\":\"{run_id}\",\"ts_ms\":1}}\n{{\"schema_version\":1,\"event\":\"run_ended\",\"run_id\":\"{run_id}\",\"reason\":\"game_over\",\"ts_ms\":2}}\n"
        ),
    )
    .unwrap();
    let store = KnowledgeStore::new(temp.path().join("learning/knowledge"));
    store.append_cases(&[decision_case(run_id)]).unwrap();
    let config = crate::config::Config::from_map(&HashMap::from([
        ("LLM_PROVIDER", "mock"),
        ("MEMORY_MODE", "collect"),
    ]));

    let result = review_run_with_provider(
        temp.path(),
        &config,
        &crate::llm::LlmProvider::Mock,
        &crate::locales::Locale::load("en"),
        "en",
        run_id,
    )
    .await
    .unwrap();

    assert!(result.response_valid);
    assert_eq!(result.accepted_lessons, 1);
    assert_eq!(result.rejected_lessons, 0);
    assert!(result.report_path.is_file());
    assert!(
        !std::fs::read_to_string(result.report_path)
            .unwrap()
            .trim_start()
            .starts_with('{')
    );
    let runs = list_reviewable_runs(temp.path()).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].run_id, run_id);
    assert_eq!(runs[0].case_count, 1);
    assert_eq!(runs[0].lesson_count, 1);
}

fn decision_case(run_id: &str) -> crate::learning::case::DecisionCase {
    let situation = SituationDescriptor {
        descriptor_version: 1,
        ranker_tag_schema_version: 1,
        character: "IRONCLAD".into(),
        objective: RunObjective::Act3Victory,
        ascension_band: AscensionBand::A0,
        act: 1,
        encounter_ids: vec!["GremlinNob".into()],
        alive_monsters: vec![MonsterDescriptor {
            monster_id: "GremlinNob".into(),
            hp_ratio_bucket: RatioBucket::P61_80,
            block_ratio_bucket: RatioBucket::Zero,
            intent: "ATTACK".into(),
            incoming_hits_bucket: CountBucket::One,
            power_ids: vec!["Enrage".into()],
        }],
        turn_bucket: TurnBucket::Turn2,
        hp_ratio_bucket: RatioBucket::P41_60,
        energy_bucket: CountBucket::Two,
        block_threat_bucket: BlockThreatBucket::Danger,
        stance: None,
        player_power_ids: vec![],
        playable_cards: vec![],
        relic_ids: vec!["Burning Blood".into()],
        ranker_tags: vec!["damage".into()],
    };
    CaseDraft {
        run_id: run_id.into(),
        decision_id: format!("{run_id}:8:2:1"),
        seed_hash: seed_hash(7, "mods"),
        situation,
        ascension_level: Some(0),
        selected_action: SemanticAction::EndTurn,
        decision_source: DecisionSource::Llm,
        available_semantic_actions: vec![SemanticAction::EndTurn],
        ranked_suggestions: vec![],
        retrieved_memory_ids: vec![],
        memory_ids_used: vec![],
    }
    .finalize(
        CaseOutcome {
            command_succeeded: true,
            player_hp_before_action: None,
            player_hp_after_action: None,
            action_hp_lost: None,
            player_died_after_action: None,
            alive_monsters_after_action: None,
            turn_hp_lost: Some(2),
            combat_completed: true,
            combat_won: Some(true),
            combat_hp_lost: Some(5),
            combat_turns: Some(2),
            potions_used: vec![],
            run_completed: true,
            run_victory: Some(true),
            final_floor: Some(51),
        },
        CaseProvenance {
            app_version: "0.2.0".into(),
            prompt_schema_version: 1,
            rules_sha256: "rules".into(),
            model_profile_sha256: "model".into(),
            compatibility_sha256: "mods".into(),
            knowledge_snapshot_id: None,
        },
    )
    .unwrap()
}
