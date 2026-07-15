use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunKind {
    Legitimate,
    DebugFlow,
    Synthetic,
    Restored,
    Incomplete,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunOutcome {
    Victory,
    Defeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunObjective {
    Act3Victory,
    Act4Victory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IneligibilityReason {
    DebugAscension,
    InvalidAscension,
    MissingTerminalOutcome,
    MissingCharacter,
    MissingSeed,
    MissingObjective,
    MissingAppVersion,
    MissingModelProfile,
    MissingPromptSchema,
    MissingLocale,
    SyntheticInput,
    StdinTest,
    StateRestore,
    UndoUsed,
    InconsistentTelemetry,
    AlreadyCommitted,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunFacts {
    pub ascension_level: Option<i64>,
    pub terminal_outcome: Option<RunOutcome>,
    pub character: Option<String>,
    pub seed: Option<i64>,
    pub objective: Option<RunObjective>,
    pub app_version: Option<String>,
    pub model_profile_sha256: Option<String>,
    pub prompt_schema_version: Option<u32>,
    pub locale: Option<String>,
    pub synthetic_input: bool,
    pub stdin_test: bool,
    pub used_restore: bool,
    pub used_undo: bool,
    pub telemetry_consistent: bool,
    pub already_committed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eligibility {
    pub run_kind: RunKind,
    pub knowledge_eligible: bool,
    pub reasons: Vec<IneligibilityReason>,
}

pub fn evaluate(facts: &RunFacts) -> Eligibility {
    let mut reasons = Vec::new();
    check_ascension(facts, &mut reasons);
    required(facts, &mut reasons);
    contamination(facts, &mut reasons);

    let run_kind = classify(facts, &reasons);
    Eligibility {
        knowledge_eligible: run_kind == RunKind::Legitimate && reasons.is_empty(),
        run_kind,
        reasons,
    }
}

fn check_ascension(facts: &RunFacts, reasons: &mut Vec<IneligibilityReason>) {
    match facts.ascension_level {
        Some(-15) => reasons.push(IneligibilityReason::DebugAscension),
        Some(0..=20) => {}
        _ => reasons.push(IneligibilityReason::InvalidAscension),
    }
}

fn required(facts: &RunFacts, reasons: &mut Vec<IneligibilityReason>) {
    let checks = [
        (
            facts.terminal_outcome.is_none(),
            IneligibilityReason::MissingTerminalOutcome,
        ),
        (
            facts.character.is_none(),
            IneligibilityReason::MissingCharacter,
        ),
        (facts.seed.is_none(), IneligibilityReason::MissingSeed),
        (
            facts.objective.is_none(),
            IneligibilityReason::MissingObjective,
        ),
        (
            facts.app_version.is_none(),
            IneligibilityReason::MissingAppVersion,
        ),
        (
            facts.model_profile_sha256.is_none(),
            IneligibilityReason::MissingModelProfile,
        ),
        (
            facts.prompt_schema_version.is_none(),
            IneligibilityReason::MissingPromptSchema,
        ),
        (facts.locale.is_none(), IneligibilityReason::MissingLocale),
        (
            !facts.telemetry_consistent,
            IneligibilityReason::InconsistentTelemetry,
        ),
        (
            facts.already_committed,
            IneligibilityReason::AlreadyCommitted,
        ),
    ];
    reasons.extend(
        checks
            .into_iter()
            .filter_map(|(failed, reason)| failed.then_some(reason)),
    );
}

fn contamination(facts: &RunFacts, reasons: &mut Vec<IneligibilityReason>) {
    let checks = [
        (facts.synthetic_input, IneligibilityReason::SyntheticInput),
        (facts.stdin_test, IneligibilityReason::StdinTest),
        (facts.used_restore, IneligibilityReason::StateRestore),
        (facts.used_undo, IneligibilityReason::UndoUsed),
    ];
    reasons.extend(
        checks
            .into_iter()
            .filter_map(|(used, reason)| used.then_some(reason)),
    );
}

fn classify(facts: &RunFacts, reasons: &[IneligibilityReason]) -> RunKind {
    if facts.ascension_level == Some(-15) {
        RunKind::DebugFlow
    } else if facts.synthetic_input || facts.stdin_test {
        RunKind::Synthetic
    } else if facts.used_restore || facts.used_undo {
        RunKind::Restored
    } else if facts.terminal_outcome.is_none() {
        RunKind::Incomplete
    } else if reasons.is_empty() {
        RunKind::Legitimate
    } else {
        RunKind::Unknown
    }
}
