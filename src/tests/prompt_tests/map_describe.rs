use super::*;

#[test]
fn describe_path_rest_before_elite_annotated() {
    let path = path_of(&["R", "E", "M", "?"]);
    let desc = describe_path(&path, 1);
    assert!(
        desc.annotations
            .iter()
            .any(|a| a.contains("Rest before first Elite"))
    );
}

#[test]
fn describe_path_no_rest_before_elite_not_annotated() {
    let path = path_of(&["M", "E", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("Rest before")));
}

#[test]
fn describe_path_double_elite_annotated() {
    let path = path_of(&["R", "E", "M", "E", "R"]);
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("Double Elite")));
}

#[test]
fn describe_path_single_elite_no_double_warning() {
    let path = path_of(&["R", "E", "M", "R", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("Double Elite")));
}

#[test]
fn describe_path_risk_gap_exceeds_threshold_act1() {
    let path = path_of(&["E", "M", "M", "M", "R"]); // E(10)+M(2)+M(2)+M(2)=16 > 15
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("E→R gap")));
    assert!(desc.annotations.iter().any(|a| a.contains("16")));
}

#[test]
fn describe_path_risk_gap_below_threshold_act1() {
    let path = path_of(&["E", "M", "M", "R"]); // E(10)+M(2)+M(2)=14 ≤ 15
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("E→R gap")));
}

#[test]
fn describe_path_risk_gap_direct_e_r() {
    let path = path_of(&["E", "R"]); // E(10)=10 ≤ 15
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("E→R gap")));
}

#[test]
fn describe_path_risk_varies_by_act() {
    let path = path_of(&["E", "M", "M", "R"]);
    // Act 1: E(10)+M(2)+M(2)=14 ≤ 15 (silent)
    let desc1 = describe_path(&path, 1);
    assert!(!desc1.annotations.iter().any(|a| a.contains("E→R gap")));
    // Act 2: E(10)+M(4)+M(4)=18 > 15 (warned)
    let desc2 = describe_path(&path, 17);
    assert!(desc2.annotations.iter().any(|a| a.contains("E→R gap")));
    assert!(desc2.annotations.iter().any(|a| a.contains("18")));
}

#[test]
fn describe_path_risk_resets_at_r() {
    // Segment 1: E→M→R gap=12 ≤ 15, Segment 2: E→M→R gap=12 ≤ 15
    let path = path_of(&["E", "M", "R", "E", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("E→R gap")));
}

#[test]
fn describe_path_risk_from_start_to_r() {
    // No R before first E, segment is start→R: M→E→M→R gap from E=12 ≤ 15
    let path = path_of(&["M", "E", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("E→R gap")));
}

#[test]
fn describe_path_no_shop() {
    let path = path_of(&["M", "R", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("No shop")));
}

#[test]
fn describe_path_shop_early() {
    // $ at index 1, path length 9 → position 0.125 < 0.33 = early
    let path = path_of(&["M", "$", "M", "?", "R", "E", "M", "R", "M"]);
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("Shop (early)")));
}

#[test]
fn describe_path_shop_mid() {
    // $ at index 4, path length 9 → position 0.5 < 0.66 = mid
    let path = path_of(&["M", "M", "M", "R", "$", "E", "M", "R", "M"]);
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("Shop (mid)")));
}

#[test]
fn describe_path_shop_late() {
    // $ at index 7, path length 9 → position 0.875 > 0.66 = late
    let path = path_of(&["M", "M", "M", "R", "E", "M", "?", "$", "R"]);
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("Shop (late)")));
}

#[test]
fn describe_path_no_elites_no_warnings() {
    let path = path_of(&["M", "?", "R", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("Elite")));
    assert!(!desc.annotations.iter().any(|a| a.contains("E→R")));
}

#[test]
fn describe_path_counts_includes_all_types() {
    let path = path_of(&["M", "E", "?", "$", "R", "T", "M"]);
    let desc = describe_path(&path, 1);
    assert!(desc.counts.contains("Monsters:2"));
    assert!(desc.counts.contains("Elites:1"));
    assert!(desc.counts.contains("Events:1"));
    assert!(desc.counts.contains("Shops:1"));
    assert!(desc.counts.contains("Rests:1"));
    assert!(desc.counts.contains("Treasures:1"));
}

#[test]
fn describe_path_route_chain_compact() {
    let path = path_of(&["M", "R", "E", "?", "R"]);
    let desc = describe_path(&path, 1);
    assert_eq!(desc.route_chain, "M→R→E→?→R");
}

#[test]
fn describe_path_exposes_ranking_metrics() {
    let path = path_of(&["M", "$", "R", "E", "?", "R"]);
    let desc = describe_path(&path, 1);
    assert_eq!(desc.metrics.counts.monsters, 1);
    assert_eq!(desc.metrics.counts.shops, 1);
    assert_eq!(desc.metrics.counts.elites, 1);
    assert_eq!(desc.metrics.shop_timing, ShopTiming::Early);
    assert!(desc.metrics.rest_before_first_elite);
    assert!(!desc.metrics.double_elite_without_rest);
}

#[test]
fn describe_path_triple_elite_warning() {
    let path = path_of(&["R", "E", "?", "E", "$", "E", "R"]);
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("Double Elite")));
}

#[test]
fn describe_path_rest_before_elite_seen() {
    let path = path_of(&["R", "E", "M"]);
    let desc = describe_path(&path, 1);
    assert!(
        desc.annotations
            .iter()
            .any(|a| a.contains("Rest before first Elite"))
    );
}

#[test]
fn describe_path_event_types() {
    // '?' events are counted
    let path = path_of(&["M", "?", "R"]);
    let desc = describe_path(&path, 1);
    assert!(desc.counts.contains("Events:1"));
}

#[test]
fn describe_path_elite_first_node_no_rest_before() {
    let path = path_of(&["E", "M", "R", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("Rest before")));
}

#[test]
fn describe_path_triple_elite_with_rest_after_first_hides_double() {
    let path = path_of(&["M", "R", "E", "R", "E", "R", "E", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("Double Elite")));
}
