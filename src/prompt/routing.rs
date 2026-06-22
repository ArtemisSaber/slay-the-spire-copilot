use crate::state::{MapCoord, NormalizedState};
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PathCounts {
    pub monsters: usize,
    pub elites: usize,
    pub events: usize,
    pub shops: usize,
    pub rests: usize,
    pub treasures: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ShopTiming {
    #[default]
    None,
    Early,
    Mid,
    Late,
}

impl ShopTiming {
    pub(crate) fn label(self) -> &'static str {
        match self {
            ShopTiming::None => "none",
            ShopTiming::Early => "early",
            ShopTiming::Mid => "mid",
            ShopTiming::Late => "late",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathMetrics {
    pub counts: PathCounts,
    pub shop_timing: ShopTiming,
    pub rest_before_first_elite: bool,
    pub double_elite_without_rest: bool,
    pub max_elite_to_rest_risk: f64,
}

pub struct PathDescription {
    pub counts: String,
    pub route_chain: String,
    pub annotations: Vec<String>,
    pub metrics: PathMetrics,
}

pub struct PathEvaluation {
    pub description: PathDescription,
    pub score: f64,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

fn act_from_floor(floor: i64) -> u8 {
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

fn hp_ratio(state: &NormalizedState) -> f64 {
    match (state.current_hp, state.max_hp) {
        (Some(cur), Some(max)) if max > 0 => cur as f64 / max as f64,
        _ => 0.75,
    }
}

fn score_shop(timing: ShopTiming, shop_count: usize, gold: i64, shop_visited: bool) -> f64 {
    let base = match gold {
        g if g >= 180 => match timing {
            ShopTiming::Early => 12.0,
            ShopTiming::Mid => 8.0,
            ShopTiming::Late => 4.0,
            ShopTiming::None => -5.0,
        },
        g if g >= 100 => match timing {
            ShopTiming::Early => 9.0,
            ShopTiming::Mid => 6.0,
            ShopTiming::Late => 3.0,
            ShopTiming::None if shop_visited => -1.0,
            ShopTiming::None => -3.0,
        },
        g if g >= 60 => match timing {
            ShopTiming::Early => 4.0,
            ShopTiming::Mid => 3.0,
            ShopTiming::Late => 1.0,
            ShopTiming::None => 0.0,
        },
        _ => match timing {
            ShopTiming::None => 1.0,
            ShopTiming::Late => 0.0,
            ShopTiming::Early | ShopTiming::Mid => -1.0,
        },
    };

    let extra_shop_bonus = shop_count.saturating_sub(1) as f64
        * if gold >= 250 {
            2.0
        } else if gold >= 120 {
            0.5
        } else {
            -1.0
        };

    base + extra_shop_bonus
}

pub fn evaluate_path(
    path: &[MapCoord],
    state: &NormalizedState,
    shop_visited: bool,
) -> PathEvaluation {
    let floor = state.floor.unwrap_or(1);
    let description = describe_path(path, floor);
    let metrics = description.metrics;
    let counts = metrics.counts;
    let hp = hp_ratio(state);
    let gold = state.gold.unwrap_or(0);
    let act = act_from_floor(floor);

    let mut score = 50.0;
    let elite_value = match act {
        1 => 12.0,
        2 => 10.0,
        3 => 8.0,
        _ => 10.0,
    } + if hp >= 0.75 {
        3.0
    } else if hp < 0.45 {
        -8.0
    } else {
        0.0
    };

    score += counts.elites as f64 * elite_value;
    score += counts.treasures as f64 * 5.0;
    score += counts.events as f64 * if hp < 0.45 { 3.0 } else { 2.0 };
    score += counts.rests as f64
        * if hp < 0.45 {
            6.0
        } else if hp < 0.7 {
            4.0
        } else {
            2.0
        };
    score += counts.monsters as f64 * if hp < 0.45 { -1.5 } else { 0.8 };
    score += score_shop(metrics.shop_timing, counts.shops, gold, shop_visited);

    if metrics.rest_before_first_elite {
        score += if hp < 0.6 { 10.0 } else { 6.0 };
    } else if counts.elites > 0 && hp < 0.6 {
        score -= 8.0;
    }

    if metrics.double_elite_without_rest {
        score -= if hp < 0.6 { 30.0 } else { 20.0 };
    }

    let risk_tolerance = if hp >= 0.75 {
        24.0
    } else if hp >= 0.55 {
        18.0
    } else {
        12.0
    };
    if metrics.max_elite_to_rest_risk > risk_tolerance {
        score -= (metrics.max_elite_to_rest_risk - risk_tolerance) * 1.2;
    }

    if counts.elites > 0 && counts.rests == 0 {
        score -= if hp < 0.6 { 14.0 } else { 6.0 };
    }

    let mut pros = Vec::new();
    let mut cons = Vec::new();

    if counts.elites > 0 {
        pros.push(format!(
            "{} elite reward{}",
            counts.elites,
            plural(counts.elites)
        ));
    }
    if metrics.rest_before_first_elite {
        pros.push("rest before first elite".to_string());
    }
    match metrics.shop_timing {
        ShopTiming::Early | ShopTiming::Mid | ShopTiming::Late => {
            pros.push(format!("{} shop", metrics.shop_timing.label()));
        }
        ShopTiming::None if !shop_visited && gold >= 100 => {
            cons.push("no shop for current gold".to_string());
        }
        ShopTiming::None if !shop_visited => cons.push("no shop".to_string()),
        ShopTiming::None => cons.push("no shop ahead".to_string()),
    }
    if counts.events >= 3 {
        pros.push(format!("{} events", counts.events));
    }
    if counts.rests >= 2 {
        pros.push(format!("{} rests", counts.rests));
    }
    if counts.treasures > 0 {
        pros.push("treasure".to_string());
    }

    if metrics.double_elite_without_rest {
        cons.push("double elite without rest".to_string());
    }
    if metrics.max_elite_to_rest_risk > 15.0 {
        cons.push(format!(
            "high E->R risk {:.0}",
            metrics.max_elite_to_rest_risk
        ));
    }
    if counts.elites > 0 && !metrics.rest_before_first_elite {
        cons.push("no rest before first elite".to_string());
    }
    if hp < 0.45 && counts.monsters >= 6 {
        cons.push("many hallway fights at low HP".to_string());
    }

    PathEvaluation {
        description,
        score,
        pros,
        cons,
    }
}

pub(crate) fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
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
