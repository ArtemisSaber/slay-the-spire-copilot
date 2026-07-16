use serde_json::{Value, json};

use super::contract::AnalysisOutcome;
use crate::learning::audit::{AuditCase, decision_sequence, ranker_regret};

pub(super) fn critic_case_summary(audit: &AuditCase<'_>) -> Value {
    let case = audit.case;
    let best = case.ranked_suggestions.first();
    let selected_rank = case
        .ranked_suggestions
        .iter()
        .find(|ranked| ranked.semantic_action == case.selected_action);
    let available_potion_actions: Vec<_> = case
        .available_semantic_actions
        .iter()
        .filter(|action| {
            matches!(
                action,
                crate::learning::action::SemanticAction::UsePotion { .. }
            )
        })
        .collect();
    json!({
        "case_id": case.case_id,
        "decision_id": case.decision_id,
        "decision_sequence": decision_sequence(case),
        "audit_role": audit.role.as_str(),
        "situation": case.situation,
        "selected_action": case.selected_action,
        "decision_source": case.decision_source,
        "available_semantic_actions": case.available_semantic_actions,
        "available_potion_actions": available_potion_actions,
        "ranker_evaluation": {
            "best_action": best,
            "selected_action_evaluation": selected_rank,
            "selected_differs_from_best": best.is_some_and(|ranked| ranked.semantic_action != case.selected_action),
            "score_regret": ranker_regret(case),
        },
        "immediate_observation": {
            "player_hp_before_action": case.outcome.player_hp_before_action,
            "player_hp_after_action": case.outcome.player_hp_after_action,
            "action_hp_lost": case.outcome.action_hp_lost,
            "player_died_after_action": case.outcome.player_died_after_action,
            "alive_monsters_after_action": case.outcome.alive_monsters_after_action,
        },
        "later_outcome": {
            "turn_hp_lost": case.outcome.turn_hp_lost,
            "combat_completed": case.outcome.combat_completed,
            "combat_won": case.outcome.combat_won,
            "combat_hp_lost": case.outcome.combat_hp_lost,
            "combat_turns": case.outcome.combat_turns,
            "potions_used": case.outcome.potions_used,
            "run_victory": case.outcome.run_victory,
            "final_floor": case.outcome.final_floor,
        },
        "retrieved_memory_ids": case.retrieved_memory_ids,
        "memory_ids_used": case.memory_ids_used,
    })
}

pub(super) fn critic_appendix(
    cases: &[Value],
    outcome: AnalysisOutcome,
    primary_case_id: &str,
) -> String {
    serde_json::to_string_pretty(&json!({
        "instruction": [
            "Return the required JSON envelope and put all human-readable Markdown in report_markdown.",
            "Start a defeat report with the observed terminal mechanism, then list earlier contributing decisions.",
            "The supplied primary_case_id is deterministic: do not replace it with an incidental observation.",
            "For a defeat, every proposed lesson must cite primary_case_id, use outcome_code combat_death, and describe the selected fatal action pattern.",
            "For a victory, every proposed lesson must cite primary_case_id, use outcome_code combat_win, and describe the selected terminal action pattern.",
            "Do not claim an unselected action was proven better; label it as a ranker-identified or computed alternative.",
            "Do not infer potion availability from Potion Belt or potions_used. Only available_potion_actions proves a potion action was available.",
            "Do not claim unused energy carries to a later turn unless an explicit state feature says so.",
            "If the primary failure cannot support a narrow lesson, return no lesson instead of an unrelated lesson."
        ],
        "run_outcome": outcome.as_str(),
        "primary_case_id": primary_case_id,
        "output_contract": {
            "schema_version": super::ENVELOPE_VERSION,
            "report_markdown": "localized Markdown beginning with the primary outcome mechanism",
            "run_analysis": {
                "outcome": outcome.as_str(),
                "primary_case_id": primary_case_id,
                "contributing_case_ids": ["optional supplied causal-candidate case_id"],
                "explanation": "bounded factual explanation grounded in the cited observations",
                "confidence_millis": "integer 0..1000"
            },
            "lesson_proposals": [{
                "scope": {
                    "character": "copy the exact cited situation character",
                    "objective": "act3_victory|act4_victory",
                    "ascension_bands": ["a0|a1_9|a10_16|a17_19|a20"],
                    "encounter_ids": ["copy the exact cited situation encounter_ids"]
                },
                "trigger": {
                    "turn_buckets": ["turn1|turn2|turn3|turn4_plus"],
                    "block_threat_buckets": ["no_incoming|fully_covered|chip|danger|lethal"],
                    "required_card_ids": ["optional exact card_id from the cited situation"],
                    "required_enemy_power_ids": ["optional exact enemy power ID from the cited situation"],
                    "required_ranker_tags": ["optional tag on the cited selected action"]
                },
                "action_pattern": {
                    "kind": "play_card|use_potion|end_turn",
                    "card_types": ["empty unless kind is play_card"],
                    "card_ids": ["empty unless kind is play_card"],
                    "potion_ids": ["empty unless kind is use_potion"]
                },
                "outcome_code": "combat_death|combat_win|high_combat_hp_loss|low_combat_hp_loss|turn_damage_taken|combat_completed_quickly",
                "guidance": {"kind": "caution|consider|avoid|prefer", "text": "observational guidance"},
                "rationale": "bounded factual rationale",
                "source_case_ids": [primary_case_id],
                "confidence_millis": "integer 600..1000"
            }]
        },
        "eligible_cases": cases,
    }))
    .expect("critic appendix contains serializable values")
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
