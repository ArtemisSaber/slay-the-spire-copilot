use super::critic_causality_support::guardian_session;

#[test]
fn defeat_critic_rejects_incidental_lesson_and_keeps_terminal_guardian_failure() {
    let temp = tempfile::tempdir().unwrap();
    let (mut session, fatal, incidental) = guardian_session(temp.path());
    let fatal_id = fatal.case_id.clone();
    let incidental_id = incidental.case_id.clone();

    let prompt = session
        .build_critic_prompt("deterministic report", "guardian-run")
        .unwrap();
    let appendix: serde_json::Value = serde_json::from_str(
        prompt
            .rsplit_once("LEARNING_CRITIC_ENVELOPE_V2")
            .unwrap()
            .1
            .trim(),
    )
    .unwrap();
    assert_eq!(appendix["primary_case_id"], fatal_id);
    assert_eq!(
        appendix["eligible_cases"][0]["audit_role"],
        "direct_death_transition"
    );
    assert_eq!(
        appendix["eligible_cases"][0]["ranker_evaluation"]["selected_action_evaluation"]["score"],
        -386_745
    );

    let response = serde_json::json!({
        "schema_version": 2,
        "report_markdown": "# Primary cause\n\nThe final attack triggered retaliation and reduced the player to zero.",
        "run_analysis": {
            "outcome": "defeat",
            "primary_case_id": fatal_id,
            "contributing_case_ids": [],
            "explanation": "The selected final attack was immediately followed by player death.",
            "confidence_millis": 900
        },
        "lesson_proposals": [
            incidental_proposal(&incidental_id),
            fatal_proposal(&fatal_id)
        ]
    })
    .to_string();

    let result = session
        .ingest_critic_response(&response, "guardian-run")
        .unwrap();

    assert!(result.response_valid);
    assert_eq!(result.accepted_lessons, 1);
    assert_eq!(result.rejected_lessons, 1);
    assert_eq!(session.snapshot().lessons.len(), 1);
    let lesson = &session.snapshot().lessons[0];
    assert_eq!(lesson.source_case_ids, [fatal.case_id]);
    assert_eq!(
        lesson.outcome_code,
        crate::learning::lesson::OutcomeCode::CombatDeath
    );
    assert!(lesson.guidance.text.contains("Sharp Hide"));
}

fn incidental_proposal(case_id: &str) -> serde_json::Value {
    serde_json::json!({
        "scope": scope(),
        "trigger": {
            "turn_buckets": ["turn4_plus"],
            "block_threat_buckets": ["no_incoming"],
            "required_card_ids": ["Defend_R"],
            "required_enemy_power_ids": [],
            "required_ranker_tags": ["excessive"]
        },
        "action_pattern": {
            "kind": "play_card",
            "card_types": [],
            "card_ids": ["Defend_R"],
            "potion_ids": []
        },
        "outcome_code": "potion_preserved",
        "guidance": {"kind": "avoid", "text": "Avoid this incidental action."},
        "rationale": "This action occurred earlier but did not cause damage.",
        "source_case_ids": [case_id],
        "confidence_millis": 700
    })
}

fn fatal_proposal(case_id: &str) -> serde_json::Value {
    serde_json::json!({
        "scope": scope(),
        "trigger": {
            "turn_buckets": ["turn4_plus"],
            "block_threat_buckets": ["lethal"],
            "required_card_ids": ["Perfected Strike"],
            "required_enemy_power_ids": ["Sharp Hide"],
            "required_ranker_tags": ["retaliation"]
        },
        "action_pattern": {
            "kind": "play_card",
            "card_types": ["ATTACK"],
            "card_ids": ["Perfected Strike"],
            "potion_ids": []
        },
        "outcome_code": "combat_death",
        "guidance": {
            "kind": "avoid",
            "text": "Avoid an attack into Sharp Hide when its retaliation can reduce the player to zero."
        },
        "rationale": "The selected attack was immediately followed by zero player HP and combat defeat.",
        "source_case_ids": [case_id],
        "confidence_millis": 900
    })
}

fn scope() -> serde_json::Value {
    serde_json::json!({
        "character": "IRONCLAD",
        "objective": "act3_victory",
        "ascension_bands": ["a0"],
        "encounter_ids": ["TheGuardian"]
    })
}
