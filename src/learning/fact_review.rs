use std::collections::{BTreeMap, HashSet};

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const FACT_REVIEW_VERSION: u32 = 2;
const MAX_CITATIONS: usize = 8;

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
    Contradicted,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum FactSource {
    AuthoritativeGameFact,
    DeterministicReport,
    RunEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FactCitation {
    source: FactSource,
    reference: String,
    fact: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FactClaimCheck {
    path: String,
    claim: String,
    status: FactClaimStatus,
    citations: Vec<FactCitation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FactReview {
    schema_version: u32,
    pub(crate) verdict: FactReviewVerdict,
    feedback: Option<String>,
    pub(crate) claim_checks: Vec<FactClaimCheck>,
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
    deterministic_report: &str,
) -> anyhow::Result<FactReview> {
    let review: FactReview = serde_json::from_str(response.trim())
        .context("fact reviewer returned invalid JSON or schema")?;
    if review.schema_version != FACT_REVIEW_VERSION {
        bail!("unsupported fact review schema version");
    }
    if required_claims.is_empty() || review.claim_checks.len() != required_claims.len() {
        bail!("fact review must check every required claim exactly once");
    }
    let mut checked_paths = HashSet::new();
    for check in &review.claim_checks {
        let Some(required_claim) = required_claims.get(&check.path) else {
            bail!("fact review checked an unknown claim path");
        };
        if !checked_paths.insert(check.path.as_str()) || &check.claim != required_claim {
            bail!("fact review must copy each required claim exactly once");
        }
        validate_check(check, allowed_decision_ids, deterministic_report)?;
    }
    match review.verdict {
        FactReviewVerdict::Approve
            if review.feedback.is_none()
                && review
                    .claim_checks
                    .iter()
                    .all(|check| check.status == FactClaimStatus::Supported) => {}
        FactReviewVerdict::Reject
            if review
                .feedback
                .as_deref()
                .is_some_and(|feedback| safe_text(feedback, 2_048))
                && review
                    .claim_checks
                    .iter()
                    .any(|check| check.status != FactClaimStatus::Supported) => {}
        FactReviewVerdict::Approve => {
            bail!("approval requires positive support for every required claim")
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
    deterministic_report: &str,
) -> anyhow::Result<()> {
    if !safe_text(&check.path, 128) || !safe_text(&check.claim, 2_048) {
        bail!("fact review claim check is malformed");
    }
    match check.status {
        FactClaimStatus::Unsupported if check.citations.is_empty() => return Ok(()),
        FactClaimStatus::Supported | FactClaimStatus::Contradicted
            if (1..=MAX_CITATIONS).contains(&check.citations.len()) => {}
        _ => bail!(
            "supported and contradicted claims require cited facts; unsupported claims do not"
        ),
    }
    for citation in &check.citations {
        validate_citation(citation, allowed_decision_ids, deterministic_report)?;
    }
    Ok(())
}

fn validate_citation(
    citation: &FactCitation,
    allowed_decision_ids: &HashSet<String>,
    deterministic_report: &str,
) -> anyhow::Result<()> {
    if !safe_text(&citation.reference, 256) || !safe_text(&citation.fact, 2_048) {
        bail!("fact review citation is malformed");
    }
    let valid = match citation.source {
        FactSource::AuthoritativeGameFact => AUTHORITATIVE_GAME_FACTS
            .iter()
            .any(|(reference, fact)| *reference == citation.reference && *fact == citation.fact),
        FactSource::DeterministicReport => {
            citation.reference == "deterministic_report"
                && deterministic_report.contains(&citation.fact)
        }
        FactSource::RunEvidence => allowed_decision_ids.contains(&citation.reference),
    };
    if !valid {
        let label = match citation.source {
            FactSource::RunEvidence => "unknown run evidence reference",
            FactSource::AuthoritativeGameFact => "unknown authoritative game fact",
            FactSource::DeterministicReport => "fact is absent from the deterministic report",
        };
        bail!("{label}");
    }
    Ok(())
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
