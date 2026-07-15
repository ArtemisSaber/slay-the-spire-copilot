use super::support::{decision_case, decision_case_for, lesson_for, situation};
use crate::learning::config::{MemoryConfig, MemoryMode};
use crate::learning::session::{LearningSession, SessionProvenance};
use crate::learning::snapshot::KnowledgeSnapshot;
use crate::learning::store::KnowledgeStore;

fn session(temp: &std::path::Path) -> (LearningSession, String) {
    let case = decision_case("run-a", 1);
    let case_id = case.case_id.clone();
    let snapshot = KnowledgeSnapshot::build(vec![case.clone()], &[]).unwrap();
    let store = KnowledgeStore::new(temp);
    store.append_cases(&[case]).unwrap();
    let config = MemoryConfig {
        mode: MemoryMode::Collect,
        mod_profile_sha256: Some("mods".into()),
        mod_profile_approved: true,
        ..MemoryConfig::default()
    };
    (
        LearningSession::new(
            config,
            store,
            snapshot,
            SessionProvenance {
                locale: "en".into(),
                model_profile_sha256: "model".into(),
                rules_sha256: "rules".into(),
                synthetic_input: false,
            },
        ),
        case_id,
    )
}

#[test]
fn critic_prompt_is_bounded_and_contains_only_cases_from_the_completed_run() {
    let temp = tempfile::tempdir().unwrap();
    let (session, case_id) = session(temp.path());

    let prompt = session.build_critic_prompt("base report", "run-a").unwrap();

    assert!(prompt.contains("LEARNING_CRITIC_ENVELOPE_V1"));
    assert!(prompt.contains(&case_id));
    assert!(!prompt.contains("seed_hash"));
    assert!(prompt.len() <= 20_000);
    assert!(session.build_critic_prompt("base", "other-run").is_none());
}

#[test]
fn critic_omits_an_indivisible_case_that_exceeds_the_absolute_prompt_budget() {
    let temp = tempfile::tempdir().unwrap();
    let mut oversized = situation();
    oversized.ranker_tags = (0..100)
        .map(|index| format!("tag-{index}-{}", "x".repeat(200)))
        .collect();
    let case = decision_case_for("run-a", 1, oversized, true, &[], "mods");
    let snapshot = KnowledgeSnapshot::build(vec![case], &[]).unwrap();
    let config = MemoryConfig {
        mode: MemoryMode::Collect,
        mod_profile_sha256: Some("mods".into()),
        mod_profile_approved: true,
        ..MemoryConfig::default()
    };
    let session = LearningSession::new(
        config,
        KnowledgeStore::new(temp.path()),
        snapshot,
        SessionProvenance {
            locale: "en".into(),
            model_profile_sha256: "model".into(),
            rules_sha256: "rules".into(),
            synthetic_input: false,
        },
    );

    assert!(session.build_critic_prompt("base", "run-a").is_none());
}

#[test]
fn critic_accepts_only_structured_lessons_citing_supplied_cases() {
    let temp = tempfile::tempdir().unwrap();
    let (mut session, case_id) = session(temp.path());
    let case = session.snapshot().cases[0].clone();
    let lesson = lesson_for(&case);
    let response = serde_json::json!({
        "schema_version": 1,
        "report_markdown": "# Review\n\nA sufficiently detailed postmortem report for this completed run.",
        "lesson_proposals": [{
            "scope": lesson.scope,
            "trigger": lesson.trigger,
            "action_pattern": lesson.action_pattern,
            "outcome_code": lesson.outcome_code,
            "guidance": lesson.guidance,
            "rationale": lesson.rationale,
            "source_case_ids": [case_id],
            "confidence_millis": 700
        }]
    })
    .to_string();

    let result = session.ingest_critic_response(&response, "run-a").unwrap();

    assert_eq!(result.accepted_lessons, 1);
    assert_eq!(result.rejected_lessons, 0);
    assert_eq!(session.snapshot().lessons.len(), 1);
    assert!(temp.path().join("lessons.jsonl").is_file());
}

#[test]
fn one_malformed_proposal_does_not_discard_valid_siblings() {
    let temp = tempfile::tempdir().unwrap();
    let (mut session, case_id) = session(temp.path());
    let case = session.snapshot().cases[0].clone();
    let lesson = lesson_for(&case);
    let response = serde_json::json!({
        "schema_version": 1,
        "report_markdown": "Review",
        "lesson_proposals": [
            {"scope": "not-an-object"},
            {
                "scope": lesson.scope,
                "trigger": lesson.trigger,
                "action_pattern": lesson.action_pattern,
                "outcome_code": lesson.outcome_code,
                "guidance": lesson.guidance,
                "rationale": lesson.rationale,
                "source_case_ids": [case_id],
                "confidence_millis": 700
            }
        ]
    })
    .to_string();

    let result = session.ingest_critic_response(&response, "run-a").unwrap();

    assert_eq!(result.accepted_lessons, 1);
    assert_eq!(result.rejected_lessons, 1);
}

#[test]
fn malformed_or_uncited_critic_output_cannot_change_knowledge() {
    let temp = tempfile::tempdir().unwrap();
    let (mut session, _) = session(temp.path());

    let result = session
        .ingest_critic_response("ordinary markdown fallback", "run-a")
        .unwrap();

    assert_eq!(result.report_markdown, "ordinary markdown fallback");
    assert_eq!(result.accepted_lessons, 0);
    assert!(session.snapshot().lessons.is_empty());
}
