use std::collections::HashSet;

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};

const FACT_REVIEW_VERSION: u32 = 1;
const MAX_ISSUES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FactReviewVerdict {
    Approve,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FactReviewIssue {
    pub(crate) claim: String,
    pub(crate) contradicting_fact: String,
    pub(crate) decision_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FactReview {
    schema_version: u32,
    pub(crate) verdict: FactReviewVerdict,
    pub(crate) feedback: Option<String>,
    pub(crate) issues: Vec<FactReviewIssue>,
}

pub(crate) fn parse_fact_review(
    response: &str,
    allowed_decision_ids: &HashSet<String>,
) -> anyhow::Result<FactReview> {
    let review: FactReview = serde_json::from_str(response.trim())
        .context("fact reviewer returned invalid JSON or schema")?;
    if review.schema_version != FACT_REVIEW_VERSION {
        bail!("unsupported fact review schema version");
    }
    match review.verdict {
        FactReviewVerdict::Approve if review.feedback.is_none() && review.issues.is_empty() => {}
        FactReviewVerdict::Reject
            if review
                .feedback
                .as_deref()
                .is_some_and(|feedback| safe_text(feedback, 2_048))
                && (1..=MAX_ISSUES).contains(&review.issues.len()) => {}
        FactReviewVerdict::Approve => bail!("approval must not contain feedback or issues"),
        FactReviewVerdict::Reject => bail!("rejection requires safe feedback and 1-8 issues"),
    }
    let mut cited = HashSet::new();
    for issue in &review.issues {
        if !safe_text(&issue.claim, 1_024)
            || !safe_text(&issue.contradicting_fact, 1_024)
            || issue.decision_ids.len() > 12
        {
            bail!("fact review issue is malformed");
        }
        for decision_id in &issue.decision_ids {
            if !allowed_decision_ids.contains(decision_id) {
                bail!("fact review cited unknown decision id {decision_id}");
            }
            if !cited.insert(decision_id) {
                bail!("fact review cited a duplicate decision id");
            }
        }
    }
    Ok(review)
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
