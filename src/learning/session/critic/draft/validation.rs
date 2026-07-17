use super::super::contract::{CandidateLesson, CriticEnvelope, safe_text};
use super::super::mode::CriticMode;
use crate::learning::lesson::LessonScope;

pub(super) fn regeneration_is_valid(envelope: &CriticEnvelope, mode: CriticMode<'_>) -> bool {
    match mode {
        CriticMode::Regenerate(parent) => {
            envelope
                .rejected_lesson_analysis
                .as_deref()
                .is_some_and(|analysis| safe_text(analysis, 1_024))
                && envelope.lesson.as_ref().is_some_and(|candidate| {
                    parent.strategy.as_ref().is_none_or(|strategy| {
                        candidate_text(candidate) != normalize(&strategy.text)
                    })
                })
        }
        CriticMode::Initial | CriticMode::ReportOnly => envelope.rejected_lesson_analysis.is_none(),
    }
}

pub(super) fn strategic_scope(
    cited: &[&crate::learning::case::DecisionCase],
    benchmark: &crate::learning::lesson::LessonBenchmark,
) -> Option<LessonScope> {
    let first = cited
        .iter()
        .find(|case| case.run_id == benchmark.origin_run_id)?;
    let encounters = if cited
        .iter()
        .all(|case| case.situation.encounter_ids == first.situation.encounter_ids)
    {
        first.situation.encounter_ids.clone()
    } else {
        vec![]
    };
    Some(LessonScope {
        character: benchmark.character.clone(),
        objective: benchmark.objective,
        ascension_bands: vec![first.situation.ascension_band],
        encounter_ids: encounters,
    })
}

fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn candidate_text(candidate: &CandidateLesson) -> String {
    normalize(candidate.text())
}
