use std::cmp::Ordering;
use std::collections::HashMap;

use serde_json::{Value, json};

use crate::locales::Locale;
use crate::prompt::builder::{MAP_CANDIDATE_LIMIT, position_label};
use crate::prompt::routing::{
    PathEvaluation, enumerate_paths, enumerate_paths_from_roots, evaluate_path,
};
use crate::state::{MapCoord, NormalizedState};

use super::projection::map_node_value;

struct RankedPath {
    choice_index: usize,
    position: String,
    next_node: MapCoord,
    path: Vec<MapCoord>,
    evaluation: PathEvaluation,
}

pub(super) fn map_context(state: &NormalizedState, locale: &Locale, shop_visited: bool) -> Value {
    let route_options = if state.map_first_node_chosen == Some(true) {
        crossroad_routes(state, locale, shop_visited)
    } else {
        initial_routes(state, locale, shop_visited)
    };
    json!({
        "first_node_chosen": state.map_first_node_chosen,
        "current": {
            "x": state.map_current_x,
            "y": state.map_current_y,
        },
        "nodes": state.map_nodes.iter().map(map_node_value).collect::<Vec<_>>(),
        "route_options": route_options,
    })
}

fn crossroad_routes(state: &NormalizedState, locale: &Locale, shop_visited: bool) -> Vec<Value> {
    let node_map: HashMap<_, _> = state
        .map_nodes
        .iter()
        .map(|node| ((node.x, node.y), node))
        .collect();
    let Some(current) = state
        .map_current_x
        .zip(state.map_current_y)
        .and_then(|coordinate| node_map.get(&coordinate).copied())
    else {
        return vec![];
    };
    let mut children: Vec<_> = current
        .children
        .iter()
        .filter_map(|coordinate| node_map.get(coordinate).copied())
        .collect();
    children.sort_by_key(|node| (node.x, node.y));

    let total = children.len();
    let mut ranked = Vec::new();
    for (choice_index, child) in children.into_iter().enumerate() {
        let paths = enumerate_paths(child.x, child.y, &state.map_nodes);
        let (path, evaluation) = best_path(paths, child, state, shop_visited);
        ranked.push(RankedPath {
            choice_index,
            position: position_label(choice_index, total, locale),
            next_node: child.clone(),
            path,
            evaluation,
        });
    }
    rank_and_project(ranked)
}

fn initial_routes(state: &NormalizedState, locale: &Locale, shop_visited: bool) -> Vec<Value> {
    let mut roots = enumerate_paths_from_roots(&state.map_nodes);
    roots.sort_by_key(|root| (root.root.x, root.root.y));
    let total = roots.len();
    let mut ranked = Vec::new();
    for (choice_index, root) in roots.into_iter().enumerate() {
        let position = position_label(choice_index, total, locale);
        for path in root.paths {
            ranked.push(RankedPath {
                choice_index,
                position: position.clone(),
                next_node: root.root.clone(),
                evaluation: evaluate_path(&path, state, shop_visited),
                path,
            });
        }
    }
    rank_and_project(ranked)
}

fn best_path(
    paths: Vec<Vec<MapCoord>>,
    fallback: &MapCoord,
    state: &NormalizedState,
    shop_visited: bool,
) -> (Vec<MapCoord>, PathEvaluation) {
    let paths = if paths.is_empty() {
        vec![vec![fallback.clone()]]
    } else {
        paths
    };
    paths
        .into_iter()
        .map(|path| {
            let evaluation = evaluate_path(&path, state, shop_visited);
            (path, evaluation)
        })
        .max_by(|left, right| compare_score(&left.1, &right.1))
        .expect("at least one map path must exist")
}

fn rank_and_project(mut ranked: Vec<RankedPath>) -> Vec<Value> {
    ranked.sort_by(|left, right| {
        compare_score(&right.evaluation, &left.evaluation)
            .then_with(|| left.choice_index.cmp(&right.choice_index))
    });
    ranked
        .into_iter()
        .take(MAP_CANDIDATE_LIMIT)
        .enumerate()
        .map(|(rank, path)| route_value(rank, path))
        .collect()
}

fn compare_score(left: &PathEvaluation, right: &PathEvaluation) -> Ordering {
    left.score
        .partial_cmp(&right.score)
        .unwrap_or(Ordering::Equal)
}

fn route_value(rank: usize, path: RankedPath) -> Value {
    let metrics = path.evaluation.description.metrics;
    let counts = metrics.counts;
    json!({
        "rank": rank + 1,
        "choice_index": path.choice_index,
        "position": path.position,
        "next_node": map_node_value(&path.next_node),
        "score": path.evaluation.score,
        "pros": path.evaluation.pros,
        "cons": path.evaluation.cons,
        "route": path.evaluation.description.route_chain,
        "counts": {
            "monsters": counts.monsters,
            "elites": counts.elites,
            "events": counts.events,
            "shops": counts.shops,
            "rests": counts.rests,
            "treasures": counts.treasures,
        },
        "metrics": {
            "shop_timing": metrics.shop_timing.label(),
            "rest_before_first_elite": metrics.rest_before_first_elite,
            "double_elite_without_rest": metrics.double_elite_without_rest,
            "max_elite_to_rest_risk": metrics.max_elite_to_rest_risk,
        },
        "nodes": path.path.iter().map(map_node_value).collect::<Vec<_>>(),
    })
}
