use std::collections::HashSet;

use super::StrategicLessonProposal;
use crate::learning::case::DecisionCase;
use crate::learning::descriptor::AscensionBand;
use crate::learning::lesson::LessonError;

pub(super) fn validate_proposal(
    proposal: &mut StrategicLessonProposal,
    cases: &[DecisionCase],
) -> Result<(), LessonError> {
    validate_static_fields(proposal)?;
    proposal.source_case_ids.sort();
    proposal.source_case_ids.dedup();
    let cited = cited_cases(&proposal.source_case_ids, cases)?;
    validate_cited_cases(proposal, &cited)?;
    validate_evidence(proposal, &cited)
}

fn validate_static_fields(proposal: &StrategicLessonProposal) -> Result<(), LessonError> {
    let benchmark = &proposal.benchmark;
    let identifiers = [
        proposal.language.as_str(),
        proposal.critic_model_profile_sha256.as_str(),
        benchmark.origin_run_id.as_str(),
        benchmark.character.as_str(),
        benchmark.compatibility_sha256.as_str(),
    ];
    let texts = [
        &proposal.strategy.text,
        &proposal.strategy.applies_when,
        &proposal.strategy.expected_effect,
        &proposal.strategy.uncertainty,
    ];
    let exact_band = AscensionBand::from_level(benchmark.ascension_level);
    let lineage_valid = match &proposal.parent_lesson_id {
        None => proposal.generation == 1,
        Some(parent) => proposal.generation > 1 && valid_identifier(parent),
    };
    if proposal.confidence_millis > 1_000
        || benchmark.final_floor < 0
        || !lineage_valid
        || identifiers.into_iter().any(|id| !valid_identifier(id))
        || proposal.scope.character != benchmark.character
        || proposal.scope.objective != benchmark.objective
        || exact_band.is_none_or(|band| !proposal.scope.ascension_bands.contains(&band))
    {
        return Err(LessonError::InvalidLifecycle);
    }
    if texts.iter().any(|text| !safe_text(text, 1_024))
        || proposal.strategy.evidence.is_empty()
        || proposal.strategy.evidence.len() > 3
    {
        return Err(LessonError::UnsafeText);
    }
    Ok(())
}

fn cited_cases<'a>(
    source_ids: &[String],
    cases: &'a [DecisionCase],
) -> Result<Vec<&'a DecisionCase>, LessonError> {
    if source_ids.is_empty() {
        return Err(LessonError::InvalidLifecycle);
    }
    source_ids
        .iter()
        .map(|id| {
            cases
                .iter()
                .find(|case| &case.case_id == id)
                .ok_or(LessonError::UnknownSourceCase)
        })
        .collect()
}

fn validate_cited_cases(
    proposal: &StrategicLessonProposal,
    cited: &[&DecisionCase],
) -> Result<(), LessonError> {
    let benchmark = &proposal.benchmark;
    let exact_band = AscensionBand::from_level(benchmark.ascension_level)
        .ok_or(LessonError::InvalidLifecycle)?;
    let invalid = cited.iter().any(|case| {
        case.situation.character != benchmark.character
            || case.situation.objective != benchmark.objective
            || case.situation.ascension_band != exact_band
            || case.provenance.compatibility_sha256 != benchmark.compatibility_sha256
            || case
                .ascension_level
                .is_some_and(|level| level != benchmark.ascension_level)
            || (!proposal.scope.encounter_ids.is_empty()
                && case.situation.encounter_ids != proposal.scope.encounter_ids)
    });
    let current = cited
        .iter()
        .filter(|case| case.run_id == benchmark.origin_run_id)
        .all(|case| {
            case.outcome.final_floor == Some(benchmark.final_floor)
                && case.outcome.run_victory == Some(benchmark.victory)
        });
    if invalid
        || !current
        || cited
            .iter()
            .all(|case| case.run_id != benchmark.origin_run_id)
    {
        return Err(LessonError::InvalidLifecycle);
    }
    Ok(())
}

fn validate_evidence(
    proposal: &StrategicLessonProposal,
    cited: &[&DecisionCase],
) -> Result<(), LessonError> {
    let mut evidence_case_ids = HashSet::new();
    let mut decision_ids = HashSet::new();
    for evidence in &proposal.strategy.evidence {
        if !valid_identifier(&evidence.run_id)
            || !safe_text(&evidence.observed_chain, 1_024)
            || evidence.decision_ids.is_empty()
            || evidence.decision_ids.len() > 12
        {
            return Err(LessonError::UnsafeText);
        }
        for decision_id in &evidence.decision_ids {
            if !valid_identifier(decision_id) || !decision_ids.insert(decision_id.as_str()) {
                return Err(LessonError::InvalidLifecycle);
            }
            let case = cited
                .iter()
                .find(|case| case.run_id == evidence.run_id && case.decision_id == *decision_id)
                .ok_or(LessonError::UnknownSourceCase)?;
            evidence_case_ids.insert(case.case_id.as_str());
        }
    }
    if evidence_case_ids.len() != cited.len()
        || cited
            .iter()
            .any(|case| !evidence_case_ids.contains(case.case_id.as_str()))
    {
        return Err(LessonError::InvalidLifecycle);
    }
    Ok(())
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
