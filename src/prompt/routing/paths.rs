use super::models::{PathCounts, PathDescription, PathMetrics, ShopTiming};
use crate::state::MapCoord;
use std::collections::HashMap;

pub fn enumerate_paths(start_x: i64, start_y: i64, nodes: &[MapCoord]) -> Vec<Vec<MapCoord>> {
    let node_map: HashMap<(i64, i64), &MapCoord> = nodes.iter().map(|n| ((n.x, n.y), n)).collect();

    let start = node_map.get(&(start_x, start_y));

    let Some(start_node) = start else {
        return vec![];
    };

    let mut paths = Vec::new();
    let mut stack = vec![(vec![(*start_node).clone()], *start_node)];

    while let Some((path, node)) = stack.pop() {
        if node.children.is_empty() {
            paths.push(path);
            continue;
        }
        let mut found = false;
        for (cx, cy) in node.children.iter().rev() {
            if let Some(child) = node_map.get(&(*cx, *cy)) {
                if path.iter().any(|n| n.x == *cx && n.y == *cy) {
                    continue;
                }
                found = true;
                let mut new_path = path.clone();
                new_path.push((*child).clone());
                stack.push((new_path, child));
            }
        }
        if !found {
            paths.push(path);
        }
    }

    paths.sort_by(|a, b| {
        for (na, nb) in a.iter().zip(b.iter()) {
            let cx = na.x.cmp(&nb.x);
            if cx != std::cmp::Ordering::Equal {
                return cx;
            }
            let cy = na.y.cmp(&nb.y);
            if cy != std::cmp::Ordering::Equal {
                return cy;
            }
        }
        a.len().cmp(&b.len())
    });

    paths
}

pub fn summarize_path(path: &[MapCoord]) -> String {
    let counts = count_path_nodes(path);
    let mut parts = Vec::new();
    if counts.monsters > 0 {
        parts.push(format!("Monsters:{}", counts.monsters));
    }
    if counts.elites > 0 {
        parts.push(format!("Elites:{}", counts.elites));
    }
    if counts.events > 0 {
        parts.push(format!("Events:{}", counts.events));
    }
    if counts.shops > 0 {
        parts.push(format!("Shops:{}", counts.shops));
    }
    if counts.rests > 0 {
        parts.push(format!("Rests:{}", counts.rests));
    }
    if counts.treasures > 0 {
        parts.push(format!("Treasures:{}", counts.treasures));
    }
    parts.join("  ")
}

pub(crate) fn act_from_floor(floor: i64) -> u8 {
    match floor {
        1..=16 => 1,
        17..=33 => 2,
        34..=50 => 3,
        _ => 1,
    }
}

fn risk_modifier(symbol: &str, act: u8) -> f64 {
    match symbol {
        "E" => 10.0,
        "M" => match act {
            1 => 2.0,
            2 => 4.0,
            3 => 3.0,
            _ => 2.0,
        },
        "?" => match act {
            1 => 1.0,
            2 => 2.0,
            3 => 1.5,
            _ => 1.0,
        },
        _ => 0.0,
    }
}

fn count_path_nodes(path: &[MapCoord]) -> PathCounts {
    let mut counts = PathCounts::default();
    for n in path {
        match n.symbol.as_str() {
            "M" => counts.monsters += 1,
            "E" => counts.elites += 1,
            "?" => counts.events += 1,
            "$" => counts.shops += 1,
            "R" => counts.rests += 1,
            "T" => counts.treasures += 1,
            _ => {}
        }
    }
    counts
}

fn segment_bounds_between_rests(path: &[MapCoord]) -> Vec<(usize, usize)> {
    let r_indices: Vec<usize> = path
        .iter()
        .enumerate()
        .filter(|(_, n)| n.symbol == "R")
        .map(|(i, _)| i)
        .collect();

    let mut bounds = Vec::new();
    let mut start = 0;
    for rest in r_indices {
        if start < rest {
            bounds.push((start, rest));
        }
        start = rest + 1;
    }
    if start < path.len() {
        bounds.push((start, path.len()));
    }
    bounds
}

fn path_metrics(path: &[MapCoord], floor: i64) -> PathMetrics {
    let act = act_from_floor(floor);
    let counts = count_path_nodes(path);

    let rest_before_first_elite = path
        .iter()
        .position(|n| n.symbol == "E")
        .map(|ei| path[..ei].iter().any(|n| n.symbol == "R"))
        .unwrap_or(false);

    let mut double_elite_without_rest = false;
    let mut max_elite_to_rest_risk = 0.0;
    for (start, end) in segment_bounds_between_rests(path) {
        let seg = &path[start..end];
        let elite_count = seg.iter().filter(|n| n.symbol == "E").count();
        if elite_count >= 2 {
            double_elite_without_rest = true;
        }

        for (j, n) in seg.iter().enumerate() {
            if n.symbol == "E" {
                let gap: f64 = seg[j..].iter().map(|m| risk_modifier(&m.symbol, act)).sum();
                if gap > max_elite_to_rest_risk {
                    max_elite_to_rest_risk = gap;
                }
            }
        }
    }

    let shop_timing = path
        .iter()
        .position(|n| n.symbol == "$")
        .map(|si| {
            let pos = si as f64 / path.len().max(1) as f64;
            if pos < 0.33 {
                ShopTiming::Early
            } else if pos < 0.66 {
                ShopTiming::Mid
            } else {
                ShopTiming::Late
            }
        })
        .unwrap_or(ShopTiming::None);

    PathMetrics {
        counts,
        shop_timing,
        rest_before_first_elite,
        double_elite_without_rest,
        max_elite_to_rest_risk,
    }
}

pub fn describe_path(path: &[MapCoord], floor: i64) -> PathDescription {
    let metrics = path_metrics(path, floor);
    let counts = summarize_path(path);
    let route_chain = path
        .iter()
        .map(|n| n.symbol.as_str())
        .collect::<Vec<_>>()
        .join("→");

    let mut annotations = Vec::new();

    if metrics.rest_before_first_elite {
        annotations.push("✓ Rest before first Elite".to_string());
    }

    if metrics.double_elite_without_rest {
        annotations.push("⚠ Double Elite — no Rest between".to_string());
    }

    if metrics.max_elite_to_rest_risk > 15.0 {
        annotations.push(format!(
            "⚠ max E→R gap: {:.0} risk",
            metrics.max_elite_to_rest_risk
        ));
    }

    match metrics.shop_timing {
        ShopTiming::Early | ShopTiming::Mid | ShopTiming::Late => {
            annotations.push(format!("$ Shop ({})", metrics.shop_timing.label()));
        }
        ShopTiming::None => annotations.push("✗ No shop".to_string()),
    }

    PathDescription {
        counts,
        route_chain,
        annotations,
        metrics,
    }
}

pub struct RootPaths {
    pub root: MapCoord,
    pub paths: Vec<Vec<MapCoord>>,
}

pub fn enumerate_paths_from_roots(nodes: &[MapCoord]) -> Vec<RootPaths> {
    let mut result = Vec::new();
    let roots: Vec<&MapCoord> = nodes.iter().filter(|n| n.y == 0).collect();

    for root in roots {
        let paths = enumerate_paths(root.x, root.y, nodes);
        result.push(RootPaths {
            root: (*root).clone(),
            paths,
        });
    }

    result
}
