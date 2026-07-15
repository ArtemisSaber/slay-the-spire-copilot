use serde::Serialize;

use crate::learning::config::MemoryMode;
use crate::learning::eligibility::{Eligibility, IneligibilityReason, RunKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LearningStatus {
    pub mode: MemoryMode,
    pub case_count: usize,
    pub lesson_count: usize,
    pub last_run: Option<LastRunStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LastRunStatus {
    pub accepted: bool,
    pub run_kind: RunKind,
    pub reasons: Vec<IneligibilityReason>,
}

impl From<&Eligibility> for LastRunStatus {
    fn from(eligibility: &Eligibility) -> Self {
        Self {
            accepted: eligibility.knowledge_eligible,
            run_kind: eligibility.run_kind,
            reasons: eligibility.reasons.clone(),
        }
    }
}

pub fn render_human(status: &LearningStatus) -> String {
    let mode = match status.mode {
        MemoryMode::Off => "off",
        MemoryMode::Collect => "collecting locally; past experience does not influence play",
        MemoryMode::Shadow => "checking past experience; it does not influence play",
        MemoryMode::On => "on; relevant past experience can influence play",
    };
    let mut lines = vec![
        format!("Learning: {mode}"),
        format!(
            "Saved experience: {} cases, {} lessons",
            status.case_count, status.lesson_count
        ),
    ];
    let Some(last_run) = &status.last_run else {
        lines.push("Last run: none recorded yet".to_string());
        return lines.join("\n");
    };
    let result = if last_run.accepted {
        "learned"
    } else {
        "not learned"
    };
    lines.push(format!(
        "Last run: {result} ({})",
        run_kind_label(last_run.run_kind)
    ));
    if !last_run.reasons.is_empty() {
        lines.push(format!(
            "Reason: {}",
            last_run
                .reasons
                .iter()
                .map(|reason| reason_label(*reason))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    lines.join("\n")
}

fn run_kind_label(kind: RunKind) -> &'static str {
    match kind {
        RunKind::Legitimate => "normal completed run",
        RunKind::DebugFlow => "debug run",
        RunKind::Synthetic => "test input",
        RunKind::Restored => "restored or undone run",
        RunKind::Incomplete => "incomplete run",
        RunKind::Unknown => "unclassified run",
    }
}

fn reason_label(reason: IneligibilityReason) -> &'static str {
    match reason {
        IneligibilityReason::DebugAscension => "debug Ascension (-15)",
        IneligibilityReason::InvalidAscension => "invalid Ascension level",
        IneligibilityReason::MissingTerminalOutcome => "run did not reach a recorded ending",
        IneligibilityReason::MissingCharacter => "character was not recorded",
        IneligibilityReason::MissingSeed => "run seed was not recorded",
        IneligibilityReason::MissingObjective => "run objective was not recorded",
        IneligibilityReason::MissingAppVersion => "app version was not recorded",
        IneligibilityReason::MissingModelProfile => "model setup was not recorded",
        IneligibilityReason::MissingPromptSchema => "prompt version was not recorded",
        IneligibilityReason::MissingLocale => "language was not recorded",
        IneligibilityReason::SyntheticInput => "synthetic test input",
        IneligibilityReason::StdinTest => "stdin test mode",
        IneligibilityReason::StateRestore => "game state was restored",
        IneligibilityReason::UndoUsed => "an undo was detected",
        IneligibilityReason::InconsistentTelemetry => "decision history was incomplete",
        IneligibilityReason::AlreadyCommitted => "run was already learned",
    }
}
