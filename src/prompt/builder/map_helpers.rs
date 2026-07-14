use std::cmp::Ordering;

use crate::locales::Locale;
use crate::state::{MapCoord, NormalizedState};

use super::super::routing::{PathEvaluation, ShopTiming, evaluate_path, plural};

pub(crate) fn position_label(index: usize, total: usize, locale: &Locale) -> String {
    match (index, total) {
        (0, 1) => &locale.map_position.only,
        (0, 2) => &locale.map_position.left,
        (1, 2) => &locale.map_position.right,
        (0, 3) => &locale.map_position.left,
        (1, 3) => &locale.map_position.middle,
        (2, 3) => &locale.map_position.right,
        (0, _) => &locale.map_position.leftmost,
        _ if index + 1 == total => &locale.map_position.rightmost,
        _ => return from_left_label(index + 1, locale),
    }
    .to_string()
}

pub(crate) fn from_left_label(number: usize, locale: &Locale) -> String {
    locale
        .map_position
        .from_left
        .replace("{ordinal}", &ordinal(number))
        .replace("{n}", &number.to_string())
}

pub(crate) fn ordinal(number: usize) -> String {
    let suffix = if (11..=13).contains(&(number % 100)) {
        "th"
    } else {
        match number % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    };
    format!("{number}{suffix}")
}

pub(crate) struct LabeledPath {
    pub(super) label: String,
    pub(super) path: Vec<MapCoord>,
    pub(super) evaluation: PathEvaluation,
}

pub(crate) fn rank_labeled_paths(
    paths: Vec<(String, Vec<MapCoord>)>,
    state: &NormalizedState,
    shop_visited: bool,
) -> Vec<LabeledPath> {
    let mut ranked: Vec<LabeledPath> = paths
        .into_iter()
        .map(|(label, path)| LabeledPath {
            evaluation: evaluate_path(&path, state, shop_visited),
            label,
            path,
        })
        .collect();

    ranked.sort_by(|left, right| {
        right
            .evaluation
            .score
            .partial_cmp(&left.evaluation.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.label.cmp(&right.label))
    });
    ranked
}

pub(crate) fn format_candidate_label(index: usize) -> String {
    format!("Candidate {}", index + 1)
}

pub(crate) fn recommendation_features(evaluation: &PathEvaluation) -> String {
    let metrics = evaluation.description.metrics;
    let counts = metrics.counts;
    let mut parts = Vec::new();

    match metrics.shop_timing {
        ShopTiming::Early | ShopTiming::Mid | ShopTiming::Late => {
            parts.push(format!("{} shop", metrics.shop_timing.label()));
        }
        ShopTiming::None => {}
    }
    if metrics.rest_before_first_elite {
        parts.push("rest before elite".to_string());
    }
    if counts.elites > 0 {
        parts.push(format!("{} elite{}", counts.elites, plural(counts.elites)));
    }
    if counts.rests >= 2 {
        parts.push(format!("{} rests", counts.rests));
    }
    if counts.events >= 3 {
        parts.push(format!("{} events", counts.events));
    }

    if parts.is_empty() {
        "safe route".to_string()
    } else {
        parts.join(", ")
    }
}

pub(crate) fn compact_route_chain(chain: &str) -> String {
    let parts: Vec<&str> = chain.split('→').collect();
    if parts.len() <= 10 {
        return chain.to_string();
    }

    format!(
        "{}→…→{}",
        parts[..5].join("→"),
        parts[parts.len() - 4..].join("→")
    )
}

pub(crate) fn recommendation_label(base_label: &str, evaluation: &PathEvaluation) -> String {
    format!(
        "{} — {} — {}",
        base_label,
        compact_route_chain(&evaluation.description.route_chain),
        recommendation_features(evaluation)
    )
}

pub(crate) fn format_evaluation_line(evaluation: &PathEvaluation) -> String {
    let mut parts = vec![format!("Score:{:.0}", evaluation.score)];
    if !evaluation.pros.is_empty() {
        parts.push(format!("+{}", evaluation.pros.join(", ")));
    }
    if !evaluation.cons.is_empty() {
        parts.push(format!("-{}", evaluation.cons.join(", ")));
    }
    parts.join("  ")
}
