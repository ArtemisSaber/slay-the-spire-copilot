use std::collections::{BTreeMap, HashSet};

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const FACT_REVIEW_VERSION: u32 = 4;
const MAX_REFS: usize = 8;
const DETERMINISTIC_REPORT_REF: &str = "deterministic_report";

const AUTHORITATIVE_GAME_FACTS: &[(&str, &str)] = &[
    (
        "game:block_resets_before_next_turn",
        "Block does not carry into the next player turn unless a Block-retain effect applies.",
    ),
    (
        "game:no_normal_or_elite_heal",
        "Normal and elite combat victories do not automatically heal the player.",
    ),
    (
        "game:rest_heals_thirty_percent",
        "Rest heals 30 percent of maximum HP.",
    ),
    ("game:smith_upgrades_one_card", "Smith upgrades one card."),
    (
        "game:elite_rewards",
        "Elite combats award a relic and more gold than normal combats.",
    ),
    (
        "game:vulnerable_multiplier",
        "Vulnerable causes the affected target to take 1.5 times damage.",
    ),
    (
        "game:wrath_modifiers",
        "Wrath doubles damage dealt and damage received.",
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FactReviewVerdict {
    Approve,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum FactClaimStatus {
    Supported,
    Plausible,
    Contradicted,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FactClaimCheck {
    path: String,
    status: FactClaimStatus,
    refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FactReview {
    schema_version: u32,
    pub(crate) verdict: FactReviewVerdict,
    feedback: Option<String>,
    pub(crate) checks: Vec<FactClaimCheck>,
}

pub(crate) fn authoritative_game_fact_catalog() -> Vec<Value> {
    AUTHORITATIVE_GAME_FACTS
        .iter()
        .map(|(reference, fact)| json!({ "reference": reference, "fact": fact }))
        .collect()
}

pub(crate) fn parse_fact_review(
    response: &str,
    allowed_decision_ids: &HashSet<String>,
    required_claims: &BTreeMap<String, String>,
) -> anyhow::Result<FactReview> {
    let review: FactReview = serde_json::from_str(response.trim())
        .context("fact reviewer returned invalid JSON or schema")?;
    if review.schema_version != FACT_REVIEW_VERSION {
        bail!("unsupported fact review schema version");
    }
    if required_claims.is_empty() || review.checks.len() != required_claims.len() {
        bail!("fact review must check every required claim exactly once");
    }
    let mut checked_paths = HashSet::new();
    for check in &review.checks {
        if !required_claims.contains_key(&check.path) {
            bail!("fact review checked an unknown claim path");
        }
        if !checked_paths.insert(check.path.as_str()) {
            bail!("fact review must check each required claim exactly once");
        }
        validate_check(check, allowed_decision_ids)?;
        if is_observed_chain(&check.path) && check.status != FactClaimStatus::Supported {
            bail!("observed chains require support from supplied run evidence");
        }
    }
    match review.verdict {
        FactReviewVerdict::Approve
            if review.feedback.is_none()
                && review.checks.iter().all(|check| {
                    matches!(
                        check.status,
                        FactClaimStatus::Supported | FactClaimStatus::Plausible
                    )
                }) => {}
        FactReviewVerdict::Reject
            if review
                .feedback
                .as_deref()
                .is_some_and(|feedback| safe_text(feedback, 2_048))
                && review.checks.iter().any(|check| {
                    matches!(
                        check.status,
                        FactClaimStatus::Contradicted | FactClaimStatus::Unsupported
                    )
                }) => {}
        FactReviewVerdict::Approve => {
            bail!("approval requires every claim to be supported or grounded and plausible")
        }
        FactReviewVerdict::Reject => {
            bail!("rejection requires feedback and an unsupported or contradicted claim")
        }
    }
    Ok(review)
}

fn validate_check(
    check: &FactClaimCheck,
    allowed_decision_ids: &HashSet<String>,
) -> anyhow::Result<()> {
    if !safe_text(&check.path, 128) || check.refs.len() > MAX_REFS {
        bail!("fact review claim check is malformed");
    }
    if matches!(
        check.status,
        FactClaimStatus::Supported | FactClaimStatus::Plausible | FactClaimStatus::Contradicted
    ) && check.refs.is_empty()
    {
        bail!("grounded statuses require at least one reference");
    }
    let mut unique = HashSet::new();
    for reference in &check.refs {
        if !safe_text(reference, 256) || !unique.insert(reference.as_str()) {
            bail!("fact review reference is malformed or duplicated");
        }
        if !valid_reference(reference, allowed_decision_ids) {
            bail!("fact review cited unknown reference {reference}");
        }
    }
    Ok(())
}

fn is_observed_chain(path: &str) -> bool {
    path.starts_with("lesson.evidence[") && path.ends_with("].observed_chain")
}

fn valid_reference(reference: &str, allowed_decision_ids: &HashSet<String>) -> bool {
    reference == DETERMINISTIC_REPORT_REF
        || allowed_decision_ids.contains(reference)
        || AUTHORITATIVE_GAME_FACTS
            .iter()
            .any(|(game_fact_ref, _)| *game_fact_ref == reference)
}

fn safe_text(value: &str, maximum: usize) -> bool {
    let lower = value.to_ascii_lowercase();
    !value.trim().is_empty()
        && value.len() <= maximum
        && !value
            .chars()
            .any(|character| character.is_control() && character != '\n' && character != '\t')
        && !["```", "<script", "http://", "https://"]
            .iter()
            .any(|needle| lower.contains(needle))
}
