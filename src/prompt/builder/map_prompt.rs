use std::collections::HashMap;

use crate::locales::Locale;
use crate::state::{MapCoord, NormalizedState};

use super::super::routing::{enumerate_paths, enumerate_paths_from_roots};
use super::MAP_CANDIDATE_LIMIT;
use super::inventory::build_relics_potions_section;
use super::map_helpers::{
    format_candidate_label, format_evaluation_line, position_label, rank_labeled_paths,
    recommendation_label,
};
use super::status::status_line;

pub(crate) fn build_map_crossroad(
    state: &NormalizedState,
    locale: &Locale,
    shop_visited: bool,
) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: map_crossroad]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
        locale.sections.next_nodes.clone(),
    ];

    let node_map: HashMap<(i64, i64), &MapCoord> = state
        .map_nodes
        .iter()
        .map(|node| ((node.x, node.y), node))
        .collect();
    let current = match (state.map_current_x, state.map_current_y) {
        (Some(current_x), Some(current_y)) => node_map.get(&(current_x, current_y)),
        _ => None,
    };
    let Some(current_node) = current else {
        return lines.join("\n");
    };

    let mut valid_children: Vec<&MapCoord> = current_node
        .children
        .iter()
        .filter_map(|(child_x, child_y)| node_map.get(&(*child_x, *child_y)).copied())
        .collect();
    valid_children.sort_by_key(|node| (node.x, node.y));

    let total = valid_children.len();
    let mut child_candidates: Vec<(String, Vec<MapCoord>)> = Vec::new();
    for (index, child) in valid_children.iter().enumerate() {
        let label = position_label(index, total, locale);
        let paths = enumerate_paths(child.x, child.y, &state.map_nodes);
        let path = if let Some(path) = paths.first() {
            path.clone()
        } else {
            vec![(*child).clone()]
        };
        let type_name = match child.symbol.as_str() {
            "M" => "monster",
            "E" => "elite",
            "?" => "event",
            "$" => "shop",
            "R" => "rest",
            "T" => "treasure",
            _ => &child.symbol,
        };
        child_candidates.push((format!("({label}) — {}({type_name})", child.symbol), path));
    }

    let ranked = rank_labeled_paths(child_candidates, state, shop_visited);
    for (index, candidate) in ranked.iter().take(MAP_CANDIDATE_LIMIT).enumerate() {
        let evaluation = &candidate.evaluation;
        let description = &evaluation.description;
        let annotations: Vec<String> = description
            .annotations
            .iter()
            .map(|annotation| {
                if shop_visited && annotation == "✗ No shop" {
                    "✗ No shop ahead".to_string()
                } else {
                    annotation.clone()
                }
            })
            .collect();

        let mut details = vec![format!("Ahead: {}", description.route_chain)];
        if !annotations.is_empty() {
            details.push(annotations.join("  "));
        }
        details.push(format_evaluation_line(evaluation));

        lines.push(format!("{}:", format_candidate_label(index)));
        lines.push(format!(
            "  Recommendation label: {}",
            recommendation_label(&candidate.label, evaluation)
        ));
        lines.push(format!("  {}", details.join("  ")));
    }

    lines.join("\n")
}

pub(crate) fn build_map_suggestion(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: map_suggestion]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
        locale.sections.routes.clone(),
    ];

    if state.map_first_node_chosen == Some(true) {
        let paths = match (state.map_current_x, state.map_current_y) {
            (Some(x), Some(y)) => enumerate_paths(x, y, &state.map_nodes),
            _ => vec![],
        };
        let total = paths.len();
        let labeled_paths: Vec<(String, Vec<MapCoord>)> = paths
            .into_iter()
            .enumerate()
            .map(|(index, path)| {
                let position = position_label(index, total, locale);
                (format!("Route {} ({position})", index + 1), path)
            })
            .collect();
        let ranked = rank_labeled_paths(labeled_paths, state, false);
        for (index, candidate) in ranked.iter().take(MAP_CANDIDATE_LIMIT).enumerate() {
            let route: Vec<String> = candidate
                .path
                .iter()
                .map(|node| format!("{}({},{})", node.symbol, node.x, node.y))
                .collect();
            let evaluation = &candidate.evaluation;
            let description = &evaluation.description;
            lines.push(format!("{}:", format_candidate_label(index)));
            lines.push(format!(
                "  Recommendation label: {}",
                recommendation_label(&candidate.label, evaluation)
            ));
            lines.push(format!("  {}  [{}]", route.join("→"), description.counts));
            let mut annotation_line = format!("  {}", description.route_chain);
            if !description.annotations.is_empty() {
                annotation_line.push_str(&format!("  {}", description.annotations.join("  ")));
            }
            annotation_line.push_str(&format!("  {}", format_evaluation_line(evaluation)));
            lines.push(annotation_line);
        }
    } else {
        let mut roots = enumerate_paths_from_roots(&state.map_nodes);
        roots.sort_by_key(|root| root.root.x);
        let root_total = roots.len();
        let mut labeled_paths: Vec<(String, Vec<MapCoord>)> = Vec::new();
        for (root_index, root) in roots.iter().enumerate() {
            let position = position_label(root_index, root_total, locale);
            for path in &root.paths {
                labeled_paths.push((
                    format!("Root {} ({position})", root_index + 1),
                    path.clone(),
                ));
            }
        }
        let ranked = rank_labeled_paths(labeled_paths, state, false);
        for (index, candidate) in ranked.iter().take(MAP_CANDIDATE_LIMIT).enumerate() {
            let route: Vec<String> = candidate
                .path
                .iter()
                .map(|node| format!("{}({},{})", node.symbol, node.x, node.y))
                .collect();
            let evaluation = &candidate.evaluation;
            let description = &evaluation.description;
            lines.push(format!("{}:", format_candidate_label(index)));
            lines.push(format!(
                "  Recommendation label: {}",
                recommendation_label(&candidate.label, evaluation)
            ));
            lines.push(format!("  {}  [{}]", route.join("→"), description.counts));
            let mut annotation_line = format!("  {}", description.route_chain);
            if !description.annotations.is_empty() {
                annotation_line.push_str(&format!("  {}", description.annotations.join("  ")));
            }
            annotation_line.push_str(&format!("  {}", format_evaluation_line(evaluation)));
            lines.push(annotation_line);
        }
    }

    lines.join("\n")
}
