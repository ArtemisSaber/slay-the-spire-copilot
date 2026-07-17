use super::support::{decision_case, decision_case_for, situation};
use crate::learning::config::{MemoryConfig, MemoryMode};
use crate::learning::descriptor::AscensionBand;
use crate::learning::session::{LearningSession, SessionProvenance};
use crate::learning::snapshot::KnowledgeSnapshot;
use crate::learning::store::KnowledgeStore;

fn session(temp: &std::path::Path) -> (LearningSession, String, String) {
    let case = decision_case("run-a", 1);
    let case_id = case.case_id.clone();
    let decision_id = case.decision_id.clone();
    let snapshot = KnowledgeSnapshot::build(vec![case.clone()], &[]).unwrap();
    let store = KnowledgeStore::new(temp);
    store.append_cases(&[case]).unwrap();
    (
        LearningSession::new(
            MemoryConfig {
                mode: MemoryMode::Collect,
                ..MemoryConfig::default()
            },
            store,
            snapshot,
            provenance(),
        ),
        case_id,
        decision_id,
    )
}

fn provenance() -> SessionProvenance {
    SessionProvenance {
        locale: "en".into(),
        model_profile_sha256: "model".into(),
        compatibility_sha256: "mods".into(),
        rules_sha256: "rules".into(),
        synthetic_input: false,
    }
}

fn strategic_response(decision_id: &str) -> String {
    serde_json::json!({
        "schema_version": 3,
        "report_markdown": "# Review\n\nA detailed postmortem for the completed run.",
        "result": "lesson",
        "lesson": {
            "text": "Prioritize the enemy whose continued presence creates the most pressure.",
            "applies_when": "Multiple targets are alive and target selection affects later turns.",
            "expected_effect": "This may reduce incoming pressure and preserve setup time.",
            "evidence": [{
                "run_id": "run-a",
                "decision_ids": [decision_id],
                "observed_chain": "The cited decision was followed by the recorded run outcome."
            }],
            "uncertainty": "The unchosen target was not played, so its outcome is unknown.",
            "confidence_millis": 700
        },
        "rejected_lesson_analysis": null
    })
    .to_string()
}

#[test]
fn critic_prompt_is_bounded_and_contains_only_the_completed_run() {
    let temp = tempfile::tempdir().unwrap();
    let (session, case_id, decision_id) = session(temp.path());

    let prompt = session.build_critic_prompt("base report", "run-a").unwrap();

    assert!(prompt.contains("LEARNING_CRITIC_ENVELOPE_V3"));
    assert!(prompt.contains("\"mode\": \"initial\""));
    assert!(prompt.contains(&case_id));
    assert!(prompt.contains(&decision_id));
    assert!(prompt.contains("run_evidence"));
    assert!(prompt.contains("do not assume the final action"));
    assert!(prompt.contains("Generate exactly one reusable strategic hypothesis"));
    assert!(!prompt.contains("no_lesson"));
    assert!(!prompt.contains("seed_hash"));
    assert!(prompt.len() <= 40_000);
    assert!(session.build_critic_prompt("base", "other-run").is_none());
}

#[test]
fn critic_omits_an_indivisible_case_over_the_absolute_prompt_budget() {
    let temp = tempfile::tempdir().unwrap();
    let mut oversized = situation();
    oversized.alive_monsters[0].power_ids = (0..200)
        .map(|index| format!("power-{index}-{}", "x".repeat(300)))
        .collect();
    let case = decision_case_for("run-a", 1, oversized, true, &[], "mods");
    let session = LearningSession::new(
        MemoryConfig {
            mode: MemoryMode::Collect,
            ..MemoryConfig::default()
        },
        KnowledgeStore::new(temp.path()),
        KnowledgeSnapshot::build(vec![case], &[]).unwrap(),
        provenance(),
    );

    assert!(session.build_critic_prompt("base", "run-a").is_none());
}

#[test]
fn completed_run_review_uses_exact_ascension_persisted_with_cases() {
    let temp = tempfile::tempdir().unwrap();
    let mut mid_ascension = situation();
    mid_ascension.ascension_band = AscensionBand::A10_16;
    let case = decision_case_for("run-mid", 4, mid_ascension, true, &[], "mods");
    assert_eq!(case.ascension_level, Some(10));
    let store = KnowledgeStore::new(temp.path());
    store.append_cases(std::slice::from_ref(&case)).unwrap();
    let session = LearningSession::new(
        MemoryConfig {
            mode: MemoryMode::Collect,
            ..MemoryConfig::default()
        },
        store,
        KnowledgeSnapshot::build(vec![case], &[]).unwrap(),
        provenance(),
    );

    let prompt = session.build_critic_prompt("report", "run-mid").unwrap();
    let appendix: serde_json::Value = serde_json::from_str(
        prompt
            .rsplit_once("LEARNING_CRITIC_ENVELOPE_V3")
            .unwrap()
            .1
            .trim(),
    )
    .unwrap();

    assert_eq!(appendix["origin_benchmark"]["ascension_level"], 10);
}

#[test]
fn critic_accepts_one_strategic_hypothesis_with_trusted_benchmark() {
    let temp = tempfile::tempdir().unwrap();
    let (mut session, case_id, decision_id) = session(temp.path());

    let result = session
        .ingest_critic_response(&strategic_response(&decision_id), "run-a")
        .unwrap();

    assert!(result.response_valid);
    assert_eq!(result.accepted_lessons, 1);
    assert_eq!(result.rejected_lessons, 0);
    let lesson = &session.snapshot().lessons[0];
    assert_eq!(lesson.schema_version, 2);
    assert_eq!(lesson.source_case_ids, [case_id]);
    assert!(lesson.strategy.is_some());
    let benchmark = &lesson.lifecycle.as_ref().unwrap().benchmark;
    assert_eq!(benchmark.origin_run_id, "run-a");
    assert_eq!(benchmark.ascension_level, 20);
    assert_eq!(benchmark.final_floor, 51);
    assert!(benchmark.victory);
    assert!(temp.path().join("lessons.jsonl").is_file());
}

#[test]
fn unknown_decision_rejects_the_lesson_but_keeps_the_report() {
    let temp = tempfile::tempdir().unwrap();
    let (mut session, _, _) = session(temp.path());

    let result = session
        .ingest_critic_response(&strategic_response("run-a:8:2:unknown"), "run-a")
        .unwrap();

    assert!(result.response_valid);
    assert_eq!(result.accepted_lessons, 0);
    assert_eq!(result.rejected_lessons, 1);
    assert!(result.report_markdown.starts_with("# Review"));
    assert!(session.snapshot().lessons.is_empty());
}

#[test]
fn old_multi_proposal_envelope_cannot_change_knowledge() {
    let temp = tempfile::tempdir().unwrap();
    let (mut session, _, _) = session(temp.path());
    let old = serde_json::json!({
        "schema_version": 2,
        "report_markdown": "Review",
        "run_analysis": {},
        "lesson_proposals": []
    })
    .to_string();

    let result = session.ingest_critic_response(&old, "run-a").unwrap();

    assert!(!result.response_valid);
    assert!(session.snapshot().lessons.is_empty());
}

#[test]
fn malformed_critic_output_cannot_change_knowledge() {
    let temp = tempfile::tempdir().unwrap();
    let (mut session, _, _) = session(temp.path());

    let result = session
        .ingest_critic_response("ordinary markdown fallback", "run-a")
        .unwrap();

    assert_eq!(result.report_markdown, "ordinary markdown fallback");
    assert!(!result.response_valid);
    assert_eq!(result.accepted_lessons, 0);
    assert!(session.snapshot().lessons.is_empty());
}
