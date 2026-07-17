use super::support::{decision_case, strategic_lesson_for};
use crate::autoplay::action::AutoPlayAction;
use crate::learning::action::SemanticAction;
use crate::learning::config::{MemoryConfig, MemoryMode};
use crate::learning::deliberation::{DeliberationOutcome, deliberate_lesson};
use crate::learning::lesson::{LessonEvent, LessonEventKind, LessonStatus};
use crate::learning::session::{ExecutedPlan, LearningSession, SessionProvenance};
use crate::learning::store::KnowledgeStore;
use crate::learning::telemetry::DecisionSource;
use crate::llm::LlmProvider;
use crate::locales::Locale;
use crate::state::{CardInfo, MonsterInfo, NormalizedState, ScreenType};

fn provenance() -> SessionProvenance {
    SessionProvenance {
        locale: "en".into(),
        model_profile_sha256: "model".into(),
        compatibility_sha256: "mods".into(),
        rules_sha256: "rules".into(),
        synthetic_input: false,
    }
}

fn combat_state(seed: i64, floor: i64) -> NormalizedState {
    NormalizedState {
        screen_type: Some(ScreenType::None),
        character: Some("IRONCLAD".into()),
        seed: Some(seed),
        ascension_level: Some(20),
        floor: Some(floor),
        current_hp: Some(40),
        max_hp: Some(80),
        energy: Some(3),
        block: Some(0),
        hand: vec![CardInfo {
            id: "Bash".into(),
            name: "Bash".into(),
            cost: 2,
            card_type: "ATTACK".into(),
            upgraded: false,
            uuid: Some(format!("card-{seed}")),
            description: String::new(),
            price: None,
            playable: true,
            has_target: true,
        }],
        monsters: vec![MonsterInfo {
            monster_id: Some("GremlinNob".into()),
            index: 0,
            current_hp: Some(60),
            max_hp: Some(100),
            intent: Some("ATTACK".into()),
            damage: Some(10),
            hits: Some(1),
            ..MonsterInfo::default()
        }],
        incoming_damage: 10,
        turn_number: Some(1),
        ..NormalizedState::default()
    }
}

fn run_defeat(
    session: &mut LearningSession,
    run_id: &str,
    seed: i64,
    final_floor: i64,
    lesson_id: &str,
) -> crate::learning::session::FinalizeSummary {
    let state = combat_state(seed, final_floor);
    session.observe(&state);
    let prepared = session.prepare(&state, &["damage".into()]).unwrap();
    assert!(prepared.exposed_memory_ids.contains(&lesson_id.to_string()));
    let plan = ExecutedPlan {
        action: AutoPlayAction::End,
        source: DecisionSource::Llm,
        selected_action_id: "combat:end".into(),
        available_semantic_actions: vec![SemanticAction::EndTurn],
        ranked_suggestions: vec![],
        memory_ids_used: vec![lesson_id.into()],
    };
    session
        .record_executed(run_id, &state, &plan, &prepared)
        .unwrap();

    let mut successor = state.clone();
    successor.turn_number = Some(2);
    successor.current_hp = Some(30);
    session.observe(&successor);
    let mut reward = successor;
    reward.screen_type = Some(ScreenType::CombatReward);
    reward.monsters.clear();
    session.observe(&reward);
    let mut game_over = reward;
    game_over.screen_type = Some(ScreenType::GameOver);
    game_over.current_hp = Some(0);
    session.observe(&game_over);
    session.finalize("game_over").unwrap()
}

fn retired_session(root: &std::path::Path) -> (LearningSession, String, String) {
    let source = decision_case("origin-run", 1);
    let lesson = strategic_lesson_for(&source);
    let lesson_id = lesson.lesson_id.clone();
    let store = KnowledgeStore::new(root);
    store.append_cases(&[source]).unwrap();
    store
        .append_lesson_events(&[
            LessonEvent::new(LessonEventKind::Proposed, lesson, None, 1).unwrap()
        ])
        .unwrap();
    let snapshot = store.rebuild(&[]).unwrap();
    let mut session = LearningSession::new(
        MemoryConfig {
            mode: MemoryMode::On,
            proposed_lesson_min_similarity: 0,
            ..MemoryConfig::default()
        },
        store,
        snapshot,
        provenance(),
    );

    let first = run_defeat(&mut session, "trial-one", 2, 40, &lesson_id);
    assert_eq!(first.evaluated_lessons, 1);
    assert_eq!(first.retired_lessons, 0);
    let second = run_defeat(&mut session, "trial-two", 3, 39, &lesson_id);
    assert_eq!(second.evaluated_lessons, 1);
    assert_eq!(second.retired_lessons, 1);
    let decision_id = session
        .snapshot()
        .cases
        .iter()
        .find(|case| case.run_id == "trial-two")
        .unwrap()
        .decision_id
        .clone();
    (session, lesson_id, decision_id)
}

#[test]
fn two_degraded_used_runs_retire_and_regenerate_a_different_lesson() {
    let temp = tempfile::tempdir().unwrap();
    let (mut session, parent_id, decision_id) = retired_session(temp.path());

    let prompt = session.build_critic_prompt("report", "trial-two").unwrap();
    let appendix: serde_json::Value = serde_json::from_str(
        prompt
            .rsplit_once("LEARNING_CRITIC_ENVELOPE_V3")
            .unwrap()
            .1
            .trim(),
    )
    .unwrap();
    assert_eq!(appendix["mode"], "regenerate");
    assert_eq!(appendix["previous_rejected_lesson"]["lesson_id"], parent_id);

    let response = serde_json::json!({
        "schema_version": 3,
        "report_markdown": "# Replacement review\n\nThe prior target-priority lesson failed twice.",
        "result": "lesson",
        "lesson": {
            "text": "Preserve enough HP to establish setup before committing to a long damage sequence.",
            "applies_when": "The deck needs setup turns and current HP cannot absorb an extended race.",
            "expected_effect": "This may improve the chance of reaching the deck's stable sequence.",
            "evidence": [{
                "run_id": "trial-two",
                "decision_ids": [decision_id],
                "observed_chain": "The lesson was used and the comparable run ended below its benchmark."
            }],
            "uncertainty": "The more defensive sequence was not played in the recorded run.",
            "confidence_millis": 720
        },
        "rejected_lesson_analysis": "Two degraded trials show that target priority alone did not preserve progression; this replacement changes the policy dimension to setup survivability."
    })
    .to_string();
    let ingest = session
        .ingest_critic_response(&response, "trial-two")
        .unwrap();

    assert_eq!(ingest.accepted_lessons, 1);
    assert_eq!(session.snapshot().lessons.len(), 2);
    let parent = session
        .snapshot()
        .lessons
        .iter()
        .find(|lesson| lesson.lesson_id == parent_id)
        .unwrap();
    assert_eq!(parent.status, LessonStatus::Retired);
    let child = session
        .snapshot()
        .lessons
        .iter()
        .find(|lesson| lesson.lesson_id != parent_id)
        .unwrap();
    let lifecycle = child.lifecycle.as_ref().unwrap();
    assert_eq!(
        lifecycle.parent_lesson_id.as_deref(),
        Some(parent_id.as_str())
    );
    assert_eq!(lifecycle.generation, 2);
    assert_eq!(lifecycle.benchmark.origin_run_id, "trial-two");
    assert_eq!(lifecycle.benchmark.final_floor, 39);
}

#[tokio::test]
async fn exhausted_regeneration_resolves_without_a_replacement() {
    let temp = tempfile::tempdir().unwrap();
    let (mut session, parent_id, decision_id) = retired_session(temp.path());
    let provider = LlmProvider::scripted(move |system, prompt, _effort| {
        if system.contains("LESSON_FACT_REVIEWER_V4") {
            let mut review: serde_json::Value =
                serde_json::from_str(&crate::llm::mock::mock_lesson_fact_review_response(prompt))
                    .unwrap();
            review["verdict"] = "reject".into();
            review["feedback"] = "The replacement remains unsupported.".into();
            review["checks"][0]["status"] = "unsupported".into();
            review["checks"][0]["refs"] = serde_json::json!([]);
            return Ok(review.to_string());
        }
        Ok(serde_json::json!({
            "schema_version": 3,
            "report_markdown": "# Replacement review\n\nA replacement was attempted.",
            "result": "lesson",
            "lesson": {
                "text": "Preserve HP before extending setup.",
                "applies_when": "The recorded tactical conditions recur.",
                "expected_effect": "This may improve survival.",
                "evidence": [{"run_id": "trial-two", "decision_ids": [decision_id], "observed_chain": "The cited action preceded the recorded defeat."}],
                "uncertainty": "The alternative was not observed.",
                "confidence_millis": 700
            },
            "rejected_lesson_analysis": "The replacement changes the policy dimension."
        }).to_string())
    });

    let result = deliberate_lesson(
        &mut session,
        &provider,
        &Locale::load("en"),
        "base report",
        "trial-two",
    )
    .await
    .unwrap();

    assert_eq!(result.outcome, DeliberationOutcome::BudgetExhausted);
    assert_eq!(result.api_calls, 16);
    assert_eq!(result.ingest.accepted_lessons, 0);
    let parent = session
        .snapshot()
        .lessons
        .iter()
        .find(|lesson| lesson.lesson_id == parent_id)
        .unwrap();
    assert!(parent.lifecycle.as_ref().unwrap().regeneration_resolved);
    assert!(session.build_critic_prompt("report", "trial-two").is_none());
    assert!(session.critic_is_report_only("trial-two"));
}
