use anyhow::{anyhow, bail};
use serde::Deserialize;

use super::{CardComparisonCase, CardComparisonDecision, CardComparisonVerdict};

#[derive(Debug, Deserialize)]
struct CardComparisonResponse {
    schema_version: u32,
    verdict: String,
    #[serde(default)]
    preferred_ref: Option<String>,
    #[serde(default)]
    memory_ids_used: Vec<String>,
}

pub(in crate::autoplay::planner) fn parse_card_comparison_response(
    response: &str,
    case: &CardComparisonCase,
    allowed_memory_ids: &[String],
) -> anyhow::Result<CardComparisonDecision> {
    let parsed: CardComparisonResponse = serde_json::from_str(response.trim())
        .map_err(|error| anyhow!("card comparison returned invalid JSON: {error}"))?;
    if parsed.schema_version != 1 {
        bail!(
            "card comparison returned unsupported schema_version {}",
            parsed.schema_version
        );
    }
    let verdict = match (parsed.verdict.as_str(), parsed.preferred_ref.as_deref()) {
        ("prefer", Some(reference)) if reference == case.added_card_ref => {
            CardComparisonVerdict::PreferAddedCard
        }
        ("prefer", Some(reference)) if reference == case.unchanged_ref => {
            CardComparisonVerdict::PreferUnchanged
        }
        ("prefer", Some(reference)) => {
            bail!("card comparison returned unknown preferred_ref {reference}")
        }
        ("prefer", None) => bail!("card comparison omitted preferred_ref for prefer verdict"),
        ("indifferent", None) => CardComparisonVerdict::Indifferent,
        ("uncertain", None) => CardComparisonVerdict::Uncertain,
        ("indifferent" | "uncertain", Some(_)) => {
            bail!("card comparison supplied preferred_ref without a prefer verdict")
        }
        (verdict, _) => bail!("card comparison returned unsupported verdict {verdict}"),
    };
    let mut memory_ids_used: Vec<_> = parsed
        .memory_ids_used
        .into_iter()
        .filter(|id| allowed_memory_ids.contains(id))
        .collect();
    memory_ids_used.sort();
    memory_ids_used.dedup();
    Ok(CardComparisonDecision {
        verdict,
        memory_ids_used,
    })
}
