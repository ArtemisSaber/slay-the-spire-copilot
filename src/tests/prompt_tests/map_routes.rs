use super::*;

#[test]
fn enumerate_paths_single_root_no_branches() {
    let nodes = vec![
        make_node("M", 0, 0, vec![(0, 1)]),
        make_node("R", 0, 1, vec![(0, 2)]),
        make_node("?", 0, 2, vec![]),
    ];
    let paths = enumerate_paths(0, 0, &nodes);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 3);
    assert_eq!(paths[0][0].symbol, "M");
    assert_eq!(paths[0][1].symbol, "R");
    assert_eq!(paths[0][2].symbol, "?");
}

#[test]
fn enumerate_paths_multiple_branches() {
    let nodes = vec![
        make_node("M", 0, 0, vec![(0, 1), (1, 1)]),
        make_node("R", 0, 1, vec![(0, 2)]),
        make_node("?", 1, 1, vec![(1, 2)]),
        make_node("$", 0, 2, vec![]),
        make_node("T", 1, 2, vec![]),
    ];
    let paths = enumerate_paths(0, 0, &nodes);
    assert_eq!(paths.len(), 2);
    // Path A: M → R → $
    assert!(
        paths
            .iter()
            .any(|p| p[1].symbol == "R" && p[2].symbol == "$")
    );
    // Path B: M → ? → T
    assert!(
        paths
            .iter()
            .any(|p| p[1].symbol == "?" && p[2].symbol == "T")
    );
}

#[test]
fn enumerate_paths_missing_child_terminates() {
    let nodes = vec![
        make_node("M", 0, 0, vec![(0, 1)]),
        make_node("R", 0, 1, vec![(0, 2)]),
        // (0,2) missing — child points to nonexistent node
    ];
    let paths = enumerate_paths(0, 0, &nodes);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 2); // terminates at R
    assert_eq!(paths[0][1].symbol, "R");
}

#[test]
fn enumerate_paths_start_not_found_returns_empty() {
    let nodes = vec![make_node("M", 0, 0, vec![])];
    let paths = enumerate_paths(99, 99, &nodes);
    assert!(paths.is_empty());
}

#[test]
fn enumerate_paths_from_roots_groups_by_root() {
    let nodes = vec![
        make_node("M", 0, 0, vec![(0, 1)]),
        make_node("R", 0, 1, vec![]),
        make_node("E", 1, 0, vec![(1, 1)]),
        make_node("$", 1, 1, vec![]),
    ];
    let result = enumerate_paths_from_roots(&nodes);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].root.symbol, "M");
    assert_eq!(result[1].root.symbol, "E");
    assert_eq!(result[0].paths.len(), 1);
    assert_eq!(result[1].paths.len(), 1);
}

#[test]
fn enumerate_paths_from_roots_single_root() {
    let nodes = vec![
        make_node("M", 0, 0, vec![(0, 1)]),
        make_node("R", 0, 1, vec![]),
    ];
    let result = enumerate_paths_from_roots(&nodes);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].paths.len(), 1);
}

#[test]
fn enumerate_paths_handles_cycle_without_infinite_loop() {
    let nodes = vec![
        make_node("A", 0, 0, vec![(0, 1)]),
        make_node("B", 0, 1, vec![(0, 2)]),
        make_node("C", 0, 2, vec![(0, 0)]),
    ];
    let paths = enumerate_paths(0, 0, &nodes);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 3);
    assert_eq!(paths[0][0].symbol, "A");
    assert_eq!(paths[0][1].symbol, "B");
    assert_eq!(paths[0][2].symbol, "C");
}

#[test]
fn enumerate_paths_handles_self_loop() {
    let nodes = vec![make_node("A", 0, 0, vec![(0, 0)])];
    let paths = enumerate_paths(0, 0, &nodes);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 1);
    assert_eq!(paths[0][0].symbol, "A");
}

#[test]
fn enumerate_paths_handles_cycle_with_branch() {
    let nodes = vec![
        make_node("A", 0, 0, vec![(0, 1), (1, 1)]),
        make_node("B", 0, 1, vec![(0, 2)]),
        make_node("C", 0, 2, vec![(0, 0)]),
        make_node("D", 1, 1, vec![]),
    ];
    let paths = enumerate_paths(0, 0, &nodes);
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().any(|p| p.len() == 3 && p[2].symbol == "C"));
    assert!(paths.iter().any(|p| p.len() == 2 && p[1].symbol == "D"));
}

#[test]
fn summarize_path_all_types() {
    let path = vec![
        make_node("M", 0, 0, vec![]),
        make_node("E", 1, 0, vec![]),
        make_node("?", 2, 0, vec![]),
        make_node("$", 3, 0, vec![]),
        make_node("R", 4, 0, vec![]),
        make_node("T", 5, 0, vec![]),
        make_node("M", 6, 0, vec![]),
    ];
    let summary = summarize_path(&path);
    assert!(summary.contains("Monsters:2"));
    assert!(summary.contains("Elites:1"));
    assert!(summary.contains("Events:1"));
    assert!(summary.contains("Shops:1"));
    assert!(summary.contains("Rests:1"));
    assert!(summary.contains("Treasures:1"));
}

#[test]
fn summarize_path_empty() {
    let path: Vec<MapCoord> = vec![];
    let summary = summarize_path(&path);
    assert_eq!(summary, "");
}

#[test]
fn summarize_path_only_monsters() {
    let path = vec![
        make_node("M", 0, 0, vec![]),
        make_node("M", 1, 0, vec![]),
        make_node("M", 2, 0, vec![]),
    ];
    let summary = summarize_path(&path);
    assert!(summary.contains("Monsters:3"));
    assert!(!summary.contains("Elites"));
    assert!(!summary.contains("Events"));
}
