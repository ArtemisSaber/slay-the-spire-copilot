use super::super::LearningSession;
use crate::learning::descriptor::AscensionBand;
use crate::learning::lesson::{Lesson, LessonBenchmark};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CriticMode<'a> {
    Initial,
    ReportOnly,
    Regenerate(&'a Lesson),
}

impl<'a> CriticMode<'a> {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::ReportOnly => "report_only",
            Self::Regenerate(_) => "regenerate",
        }
    }

    pub(super) fn parent(&self) -> Option<&'a Lesson> {
        match *self {
            Self::Regenerate(parent) => Some(parent),
            Self::Initial | Self::ReportOnly => None,
        }
    }
}

pub(super) struct CriticContext<'a> {
    pub(super) mode: CriticMode<'a>,
    pub(super) benchmark: LessonBenchmark,
}

pub(super) fn context_for_run<'a>(
    learning: &'a LearningSession,
    run_id: &str,
) -> Option<CriticContext<'a>> {
    let parent = pending_parent(learning, run_id);
    let benchmark = learning
        .last_completed_benchmark
        .as_ref()
        .filter(|benchmark| benchmark.origin_run_id == run_id)
        .cloned()
        .or_else(|| parent.and_then(|lesson| trial_benchmark(lesson, run_id)))
        .or_else(|| legacy_benchmark(learning, run_id))?;
    let already_origin = learning.snapshot.lessons.iter().any(|lesson| {
        lesson
            .lifecycle
            .as_ref()
            .is_some_and(|lifecycle| lifecycle.benchmark.origin_run_id == run_id)
    });
    let already_trial = learning.snapshot.lessons.iter().any(|lesson| {
        lesson.lifecycle.as_ref().is_some_and(|lifecycle| {
            lifecycle
                .recent_trials
                .last()
                .is_some_and(|trial| trial.run_id == run_id)
        })
    });
    let mode = if let Some(parent) = parent {
        CriticMode::Regenerate(parent)
    } else if already_origin || already_trial {
        CriticMode::ReportOnly
    } else {
        CriticMode::Initial
    };
    Some(CriticContext { mode, benchmark })
}

fn pending_parent<'a>(learning: &'a LearningSession, run_id: &str) -> Option<&'a Lesson> {
    learning.snapshot.lessons.iter().find(|lesson| {
        let Some(lifecycle) = lesson.lifecycle.as_ref() else {
            return false;
        };
        lesson.status == crate::learning::lesson::LessonStatus::Retired
            && lifecycle.consecutive_degraded_runs >= 2
            && !lifecycle.regeneration_resolved
            && lifecycle
                .recent_trials
                .last()
                .is_some_and(|trial| trial.run_id == run_id)
            && !learning.snapshot.lessons.iter().any(|candidate| {
                candidate
                    .lifecycle
                    .as_ref()
                    .is_some_and(|candidate_lifecycle| {
                        candidate_lifecycle.parent_lesson_id.as_deref()
                            == Some(lesson.lesson_id.as_str())
                    })
            })
    })
}

fn trial_benchmark(lesson: &Lesson, run_id: &str) -> Option<LessonBenchmark> {
    let lifecycle = lesson.lifecycle.as_ref()?;
    let trial = lifecycle
        .recent_trials
        .iter()
        .find(|trial| trial.run_id == run_id)?;
    Some(LessonBenchmark {
        origin_run_id: trial.run_id.clone(),
        character: lifecycle.benchmark.character.clone(),
        ascension_level: lifecycle.benchmark.ascension_level,
        objective: lifecycle.benchmark.objective,
        final_floor: trial.final_floor,
        victory: trial.victory,
        compatibility_sha256: lifecycle.benchmark.compatibility_sha256.clone(),
    })
}

fn legacy_benchmark(learning: &LearningSession, run_id: &str) -> Option<LessonBenchmark> {
    let case = learning
        .snapshot
        .cases
        .iter()
        .find(|case| case.run_id == run_id)?;
    let ascension_level = case
        .ascension_level
        .or(match case.situation.ascension_band {
            AscensionBand::A0 => Some(0),
            AscensionBand::A20 => Some(20),
            AscensionBand::A1_9 | AscensionBand::A10_16 | AscensionBand::A17_19 => None,
        })?;
    Some(LessonBenchmark {
        origin_run_id: run_id.to_string(),
        character: case.situation.character.clone(),
        ascension_level,
        objective: case.situation.objective,
        final_floor: case.outcome.final_floor?,
        victory: case.outcome.run_victory?,
        compatibility_sha256: case.provenance.compatibility_sha256.clone(),
    })
}
