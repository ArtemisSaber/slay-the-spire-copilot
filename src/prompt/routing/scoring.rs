use super::models::{PathDescription, ShopTiming};
use super::paths::{act_from_floor, describe_path};
use crate::state::{MapCoord, NormalizedState};

/// HP fraction at/above which the player is considered healthy (full risk tolerance).
const HP_RATIO_HEALTHY: f64 = 0.75;
/// HP fraction below which the player is in danger (elite/event/rest scoring shifts defensive).
const HP_RATIO_DANGER: f64 = 0.45;
/// HP fraction below which mid-rest sites are valued higher.
const HP_RATIO_REST_MID: f64 = 0.7;
/// HP fraction below which pre-elite rest is heavily preferred.
const HP_RATIO_PRE_ELITE: f64 = 0.6;
/// HP fraction at/above which mid-tier risk tolerance applies.
const HP_RATIO_RISK_MID: f64 = 0.55;

/// Gold at/above which a shop is strongly valuable (early/mid/late scoring tier).
const GOLD_THRESHOLD_HIGH: i64 = 180;
/// Gold at/above which a shop is moderately valuable.
const GOLD_THRESHOLD_MID: i64 = 100;
/// Gold at/above which a shop is marginally valuable.
const GOLD_THRESHOLD_LOW: i64 = 60;
/// Gold at/above which extra shops get a large bonus.
const GOLD_THRESHOLD_VERY_HIGH: i64 = 250;
/// Gold at/above which extra shops get a small bonus.
const GOLD_THRESHOLD_BONUS: i64 = 120;

/// Starting score for every path before additive adjustments.
const BASE_PATH_SCORE: f64 = 50.0;

pub struct PathEvaluation {
    pub description: PathDescription,
    pub score: f64,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

fn hp_ratio(state: &NormalizedState) -> f64 {
    match (state.current_hp, state.max_hp) {
        (Some(cur), Some(max)) if max > 0 => cur as f64 / max as f64,
        _ => HP_RATIO_HEALTHY,
    }
}

fn score_shop(timing: ShopTiming, shop_count: usize, gold: i64, shop_visited: bool) -> f64 {
    let base = match gold {
        g if g >= GOLD_THRESHOLD_HIGH => match timing {
            ShopTiming::Early => 12.0,
            ShopTiming::Mid => 8.0,
            ShopTiming::Late => 4.0,
            ShopTiming::None => -5.0,
        },
        g if g >= GOLD_THRESHOLD_MID => match timing {
            ShopTiming::Early => 9.0,
            ShopTiming::Mid => 6.0,
            ShopTiming::Late => 3.0,
            ShopTiming::None if shop_visited => -1.0,
            ShopTiming::None => -3.0,
        },
        g if g >= GOLD_THRESHOLD_LOW => match timing {
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
        * if gold >= GOLD_THRESHOLD_VERY_HIGH {
            2.0
        } else if gold >= GOLD_THRESHOLD_BONUS {
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

    let mut score = BASE_PATH_SCORE;
    let elite_value = match act {
        1 => 12.0,
        2 => 10.0,
        3 => 8.0,
        _ => 10.0,
    } + if hp >= HP_RATIO_HEALTHY {
        3.0
    } else if hp < HP_RATIO_DANGER {
        -8.0
    } else {
        0.0
    };

    score += counts.elites as f64 * elite_value;
    score += counts.treasures as f64 * 5.0;
    score += counts.events as f64 * if hp < HP_RATIO_DANGER { 3.0 } else { 2.0 };
    score += counts.rests as f64
        * if hp < HP_RATIO_DANGER {
            6.0
        } else if hp < HP_RATIO_REST_MID {
            4.0
        } else {
            2.0
        };
    score += counts.monsters as f64 * if hp < HP_RATIO_DANGER { -1.5 } else { 0.8 };
    score += score_shop(metrics.shop_timing, counts.shops, gold, shop_visited);

    if metrics.rest_before_first_elite {
        score += if hp < HP_RATIO_PRE_ELITE { 10.0 } else { 6.0 };
    } else if counts.elites > 0 && hp < HP_RATIO_PRE_ELITE {
        score -= 8.0;
    }

    if metrics.double_elite_without_rest {
        score -= if hp < HP_RATIO_PRE_ELITE { 30.0 } else { 20.0 };
    }

    let risk_tolerance = if hp >= HP_RATIO_HEALTHY {
        24.0
    } else if hp >= HP_RATIO_RISK_MID {
        18.0
    } else {
        12.0
    };
    if metrics.max_elite_to_rest_risk > risk_tolerance {
        score -= (metrics.max_elite_to_rest_risk - risk_tolerance) * 1.2;
    }

    if counts.elites > 0 && counts.rests == 0 {
        score -= if hp < HP_RATIO_PRE_ELITE { 14.0 } else { 6.0 };
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
