use std::collections::HashSet;

use crate::learning::case::DecisionCase;
use crate::learning::descriptor::AscensionBand;
use crate::learning::lesson::{
    ActionKind, GuidanceKind, Lesson, LessonStatus, OutcomeCode, SupportStats,
};

pub(super) fn verify_structure(lesson: &Lesson) -> bool {
    let (Some(strategy), Some(lifecycle)) = (&lesson.strategy, &lesson.lifecycle) else {
        return false;
    };
    let benchmark = &lifecycle.benchmark;
    let exact_band = AscensionBand::from_level(benchmark.ascension_level);
    let trailing_degraded = lifecycle
        .recent_trials
        .iter()
        .rev()
        .take_while(|trial| trial.degraded)
        .count() as u8;
    let lineage_valid = match &lifecycle.parent_lesson_id {
        None => lifecycle.generation == 1,
        Some(parent) => lifecycle.generation > 1 && valid_identifier(parent),
    };
    lesson.schema_version == 2
        && valid_carriers(lesson)
        && valid_identifier(&lesson.language)
        && valid_identifier(&lesson.critic.model_profile_sha256)
        && lesson.critic.confidence_millis <= 1_000
        && !lesson.source_case_ids.is_empty()
        && unique_identifiers(&lesson.source_case_ids)
        && lesson.scope.character == benchmark.character
        && lesson.scope.objective == benchmark.objective
        && exact_band.is_some_and(|band| lesson.scope.ascension_bands.contains(&band))
        && benchmark.final_floor >= 0
        && valid_identifier(&benchmark.origin_run_id)
        && valid_identifier(&benchmark.character)
        && valid_identifier(&benchmark.compatibility_sha256)
        && lineage_valid
        && lifecycle.recent_trials.len() <= 2
        && lifecycle.qualifying_runs >= lifecycle.recent_trials.len() as u32
        && lifecycle.consecutive_degraded_runs == trailing_degraded
        && (trailing_degraded < 2 || lesson.status == LessonStatus::Retired)
        && (!lifecycle.regeneration_resolved || lesson.status == LessonStatus::Retired)
        && valid_strategy(strategy)
        && valid_trials(lesson)
}

pub(super) fn verify_sources(lesson: &Lesson, cases: &[DecisionCase]) -> bool {
    let (Some(strategy), Some(lifecycle)) = (&lesson.strategy, &lesson.lifecycle) else {
        return false;
    };
    let cited: Vec<_> = lesson
        .source_case_ids
        .iter()
        .filter_map(|id| cases.iter().find(|case| &case.case_id == id))
        .collect();
    if cited.len() != lesson.source_case_ids.len() {
        return false;
    }
    let benchmark = &lifecycle.benchmark;
    let Some(exact_band) = AscensionBand::from_level(benchmark.ascension_level) else {
        return false;
    };
    let facts_match = cited.iter().all(|case| {
        case.situation.character == benchmark.character
            && case.situation.objective == benchmark.objective
            && case.situation.ascension_band == exact_band
            && case.provenance.compatibility_sha256 == benchmark.compatibility_sha256
            && case
                .ascension_level
                .is_none_or(|level| level == benchmark.ascension_level)
            && (lesson.scope.encounter_ids.is_empty()
                || case.situation.encounter_ids == lesson.scope.encounter_ids)
    });
    let current_matches = cited
        .iter()
        .filter(|case| case.run_id == benchmark.origin_run_id)
        .any(|case| {
            case.outcome.final_floor == Some(benchmark.final_floor)
                && case.outcome.run_victory == Some(benchmark.victory)
        });
    let mut evidence_ids = HashSet::new();
    let evidence_matches = strategy.evidence.iter().all(|evidence| {
        evidence.decision_ids.iter().all(|decision_id| {
            cited
                .iter()
                .find(|case| case.run_id == evidence.run_id && case.decision_id == *decision_id)
                .is_some_and(|case| evidence_ids.insert(case.case_id.as_str()))
        })
    });
    facts_match
        && current_matches
        && evidence_matches
        && evidence_ids.len() == cited.len()
        && cited
            .iter()
            .all(|case| evidence_ids.contains(case.case_id.as_str()))
}

fn valid_carriers(lesson: &Lesson) -> bool {
    let strategy = lesson.strategy.as_ref().expect("checked by caller");
    lesson.trigger.turn_buckets.is_empty()
        && lesson.trigger.block_threat_buckets.is_empty()
        && lesson.trigger.required_card_ids.is_empty()
        && lesson.trigger.required_enemy_power_ids.is_empty()
        && lesson.trigger.required_ranker_tags.is_empty()
        && lesson.action_pattern.kind == ActionKind::StrategicPolicy
        && lesson.action_pattern.card_types.is_empty()
        && lesson.action_pattern.card_ids.is_empty()
        && lesson.action_pattern.potion_ids.is_empty()
        && lesson.outcome_code == OutcomeCode::RunProgression
        && lesson.guidance.kind == GuidanceKind::Experimental
        && lesson.guidance.text == strategy.text
        && lesson.rationale == strategy.expected_effect
        && lesson.support == SupportStats::default()
}

fn valid_strategy(strategy: &super::StrategicHypothesis) -> bool {
    let texts = [
        &strategy.text,
        &strategy.applies_when,
        &strategy.expected_effect,
        &strategy.uncertainty,
    ];
    let mut decisions = HashSet::new();
    texts.iter().all(|text| safe_text(text, 1_024))
        && !strategy.evidence.is_empty()
        && strategy.evidence.len() <= 3
        && strategy.evidence.iter().all(|evidence| {
            valid_identifier(&evidence.run_id)
                && safe_text(&evidence.observed_chain, 1_024)
                && !evidence.decision_ids.is_empty()
                && evidence.decision_ids.len() <= 12
                && evidence
                    .decision_ids
                    .iter()
                    .all(|id| valid_identifier(id) && decisions.insert(id.as_str()))
        })
}

fn valid_trials(lesson: &Lesson) -> bool {
    let lifecycle = lesson.lifecycle.as_ref().expect("checked by caller");
    let benchmark = &lifecycle.benchmark;
    let mut run_ids = HashSet::new();
    lifecycle.recent_trials.iter().all(|trial| {
        valid_identifier(&trial.run_id)
            && run_ids.insert(trial.run_id.as_str())
            && trial.run_id != benchmark.origin_run_id
            && trial.final_floor >= 0
            && trial.degraded
                == degraded(
                    benchmark.victory,
                    benchmark.final_floor,
                    trial.victory,
                    trial.final_floor,
                )
    })
}

fn degraded(base_victory: bool, base_floor: i64, victory: bool, floor: i64) -> bool {
    if base_victory {
        !victory
    } else {
        !victory && floor < base_floor
    }
}

fn unique_identifiers(values: &[String]) -> bool {
    let mut unique = HashSet::new();
    values
        .iter()
        .all(|value| valid_identifier(value) && unique.insert(value))
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.len() <= 256
        && !value.chars().any(char::is_control)
}

fn safe_text(value: &str, maximum: usize) -> bool {
    let lower = value.to_ascii_lowercase();
    !value.trim().is_empty()
        && value.len() <= maximum
        && !value
            .chars()
            .any(|character| character.is_control() && character != '\n')
        && !["```", "<script", "http://", "https://"]
            .iter()
            .any(|needle| lower.contains(needle))
}
