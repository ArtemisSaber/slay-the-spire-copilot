use super::critic_causality_support::guardian_session;
use crate::learning::lesson::{ActionKind, GuidanceKind, OutcomeCode};

#[test]
fn defeat_critic_can_form_a_multi_decision_strategy_without_terminal_action_overfit() {
    let temp = tempfile::tempdir().unwrap();
    let (mut session, fatal, incidental) = guardian_session(temp.path());

    let prompt = session
        .build_critic_prompt("deterministic report", "guardian-run")
        .unwrap();
    let appendix: serde_json::Value = serde_json::from_str(
        prompt
            .rsplit_once("LEARNING_CRITIC_ENVELOPE_V3")
            .unwrap()
            .1
            .trim(),
    )
    .unwrap();
    let evidence = appendix["run_evidence"]["guardian-run"].as_array().unwrap();
    assert_eq!(evidence.len(), 2);
    assert!(
        evidence
            .iter()
            .any(|case| { case["decision"]["selected_evaluation"]["score"] == -386_745 })
    );
    assert!(
        appendix["instruction"]
            .as_array()
            .unwrap()
            .iter()
            .any(|line| line.as_str().unwrap().contains("multi-decision"))
    );

    let response = serde_json::json!({
        "schema_version": 3,
        "report_markdown": "# Strategic cause\n\nThe run reached the retaliation phase without enough margin to stabilize.",
        "result": "lesson",
        "lesson": {
            "text": "Build a survivability margin before committing to a retaliation-sensitive damage sequence.",
            "applies_when": "A setup phase is followed by attacks that can trigger enemy retaliation.",
            "expected_effect": "This may leave enough HP or block to bootstrap the damage sequence safely.",
            "evidence": [{
                "run_id": "guardian-run",
                "decision_ids": [incidental.decision_id, fatal.decision_id],
                "observed_chain": "An earlier low-pressure decision preceded a later attack that was followed by zero HP."
            }],
            "uncertainty": "The alternative setup and defense sequence was not played, so survival is not guaranteed.",
            "confidence_millis": 760
        },
        "rejected_lesson_analysis": null
    })
    .to_string();

    let result = session
        .ingest_critic_response(&response, "guardian-run")
        .unwrap();

    assert!(result.response_valid);
    assert_eq!(result.accepted_lessons, 1);
    let lesson = &session.snapshot().lessons[0];
    assert_eq!(lesson.source_case_ids.len(), 2);
    assert_eq!(lesson.action_pattern.kind, ActionKind::StrategicPolicy);
    assert_eq!(lesson.outcome_code, OutcomeCode::RunProgression);
    assert_eq!(lesson.guidance.kind, GuidanceKind::Experimental);
    assert!(
        lesson
            .strategy
            .as_ref()
            .unwrap()
            .text
            .contains("survivability")
    );
    let benchmark = &lesson.lifecycle.as_ref().unwrap().benchmark;
    assert_eq!(benchmark.final_floor, 16);
    assert!(!benchmark.victory);
}
