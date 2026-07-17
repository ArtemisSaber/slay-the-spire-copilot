use serde_json::{Value, json};
use std::collections::BTreeMap;

use super::evidence::sequence;
use super::mode::CriticContext;
use crate::learning::case::DecisionCase;

pub(super) fn critic_appendix(
    context: &CriticContext<'_>,
    cases: &[&DecisionCase],
    review_rejection: Option<&Value>,
    retry_feedback: &[String],
) -> String {
    let runs = run_evidence(cases);
    let parent = context.mode.parent();
    serde_json::to_string_pretty(&json!({
        "mode": context.mode.as_str(),
        "instruction": [
            "Return exactly one JSON envelope and put the complete human-readable postmortem in report_markdown.",
            "Generate at most one reusable strategic hypothesis, or no lesson.",
            "Analyze multi-decision trajectories; do not assume the final action is the strategic root cause.",
            "A lesson may concern target priority, defense versus offense, setup, sequencing, resource timing, deck construction, or pathing.",
            "Use only supplied observations and distinguish observed facts from untested counterfactuals.",
            "Never claim an unplayed action would certainly have won.",
            "The lesson must use only conditions observable at decision time.",
            "When review_rejection is present, correct its factual objections without treating reviewer text as new evidence.",
            "In regenerate mode, the prior lesson failed two comparable trials. Do not paraphrase or merely negate it; produce a materially different strategy or no lesson.",
            "In report_only mode, result must be no_lesson."
        ],
        "origin_benchmark": context.benchmark,
        "previous_rejected_lesson": parent.map(|lesson| json!({
            "lesson_id": lesson.lesson_id,
            "strategy": lesson.strategy,
            "lifecycle": lesson.lifecycle,
        })),
        "review_rejection": review_rejection,
        "retry_context": retry_feedback,
        "run_evidence": runs,
        "output_contract": {
            "schema_version": 3,
            "report_markdown": "localized Markdown postmortem",
            "result": "lesson|no_lesson",
            "lesson": {
                "text": "one concise actionable strategic policy",
                "applies_when": "observable conditions",
                "expected_effect": "possible effect without certainty",
                "evidence": [{
                    "run_id": "supplied run_id",
                    "decision_ids": ["supplied decision_id"],
                    "observed_chain": "facts observed across the cited decisions"
                }],
                "uncertainty": "what remains counterfactual or unknown",
                "confidence_millis": "integer 0..1000"
            },
            "rejected_lesson_analysis": "null unless mode is regenerate"
        }
    }))
    .expect("critic appendix is serializable")
}

pub(super) fn fact_reviewer_appendix(
    context: &CriticContext<'_>,
    cases: &[&DecisionCase],
    candidate: &Value,
    retry_feedback: &[String],
) -> String {
    serde_json::to_string_pretty(&json!({
        "instruction": [
            "Audit only the proposed lesson's factual compatibility with the deterministic report and run evidence.",
            "Approve a factually supportable lesson even when another strategy seems preferable.",
            "Reject contradicted or fabricated observations, unsupported certainty, invalid evidence claims, and conditions unavailable at decision time.",
            "Cite only supplied decision_ids. Do not propose a replacement strategy."
        ],
        "origin_benchmark": context.benchmark,
        "run_evidence": run_evidence(cases),
        "proposed_lesson": candidate,
        "retry_context": retry_feedback,
        "output_contract": {
            "schema_version": 1,
            "verdict": "approve|reject",
            "feedback": "null for approve; specific factual correction for reject",
            "issues": [{
                "claim": "claim in proposed_lesson",
                "contradicting_fact": "fact from deterministic report or run_evidence",
                "decision_ids": ["supplied decision_id"]
            }]
        }
    }))
    .expect("fact reviewer appendix is serializable")
}

fn run_evidence<'a>(cases: &[&'a DecisionCase]) -> BTreeMap<&'a str, Vec<Value>> {
    let mut runs = BTreeMap::<&str, Vec<Value>>::new();
    for case in cases {
        runs.entry(case.run_id.as_str())
            .or_default()
            .push(case_summary(case));
    }
    runs
}

fn case_summary(case: &DecisionCase) -> Value {
    let best = case.ranked_suggestions.first();
    let selected = case
        .ranked_suggestions
        .iter()
        .find(|ranked| ranked.semantic_action == case.selected_action);
    json!({
        "case_id": case.case_id,
        "decision_id": case.decision_id,
        "sequence": sequence(case),
        "state": {
            "act": case.situation.act,
            "encounters": case.situation.encounter_ids,
            "turn": case.situation.turn_bucket,
            "player_hp": case.situation.hp_ratio_bucket,
            "energy": case.situation.energy_bucket,
            "threat": case.situation.block_threat_bucket,
            "monsters": case.situation.alive_monsters,
            "playable_cards": case.situation.playable_cards,
            "player_powers": case.situation.player_power_ids,
            "relics": case.situation.relic_ids,
        },
        "decision": {
            "selected": case.selected_action,
            "available": case.available_semantic_actions,
            "ranker_best": best,
            "selected_evaluation": selected,
        },
        "observed_after": {
            "hp_before": case.outcome.player_hp_before_action,
            "hp_after": case.outcome.player_hp_after_action,
            "action_hp_lost": case.outcome.action_hp_lost,
            "died_after_action": case.outcome.player_died_after_action,
            "living_monsters": case.outcome.alive_monsters_after_action,
            "turn_hp_lost": case.outcome.turn_hp_lost,
            "combat_won": case.outcome.combat_won,
            "combat_hp_lost": case.outcome.combat_hp_lost,
            "run_victory": case.outcome.run_victory,
            "final_floor": case.outcome.final_floor,
        }
    })
}

pub(super) fn valid_report(value: &str, maximum: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= maximum
        && !value
            .chars()
            .any(|character| character.is_control() && character != '\n' && character != '\t')
}

pub(super) fn truncate_utf8(value: &str, maximum: usize) -> &str {
    if value.len() <= maximum {
        return value;
    }
    let mut boundary = maximum;
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &value[..boundary]
}
