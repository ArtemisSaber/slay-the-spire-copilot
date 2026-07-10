use super::*;

#[test]
fn position_label_uses_readable_ordinals() {
    assert_eq!(position_label(2, 5, &Locale::load("en")), "3rd from left");
    assert_eq!(position_label(2, 5, &Locale::load("zh")), "左起第3个");
    assert_eq!(position_label(2, 5, &Locale::load("ja")), "左から3番目");
    assert_eq!(position_label(2, 5, &Locale::load("ko")), "왼쪽에서 3번째");
}

#[test]
fn position_label_only_one() {
    let locale = test_locale();
    assert_eq!(position_label(0, 1, &locale), "唯一");
}

#[test]
fn position_label_left_of_two() {
    let locale = test_locale();
    assert_eq!(position_label(0, 2, &locale), "左侧");
}

#[test]
fn position_label_right_of_two() {
    let locale = test_locale();
    assert_eq!(position_label(1, 2, &locale), "右侧");
}

#[test]
fn position_label_left_of_three() {
    let locale = test_locale();
    assert_eq!(position_label(0, 3, &locale), "左侧");
}

#[test]
fn position_label_middle_of_three() {
    let locale = test_locale();
    assert_eq!(position_label(1, 3, &locale), "中间");
}

#[test]
fn position_label_right_of_three() {
    let locale = test_locale();
    assert_eq!(position_label(2, 3, &locale), "右侧");
}

#[test]
fn position_label_leftmost_of_many() {
    let locale = test_locale();
    assert_eq!(position_label(0, 6, &locale), "最左侧");
}

#[test]
fn position_label_rightmost_of_many() {
    let locale = test_locale();
    assert_eq!(position_label(5, 6, &locale), "最右侧");
}

#[test]
fn position_label_middle_of_many_falls_back_to_from_left() {
    let locale = test_locale();
    let label = position_label(2, 6, &locale);
    assert!(label.contains("第3"));
}

#[test]
fn format_evaluation_line_with_pros_and_cons() {
    let eval = make_path_evaluation(
        85.0,
        vec!["early shop", "rest before elite"],
        vec!["double elite"],
    );
    let line = format_evaluation_line(&eval);
    assert!(line.contains("Score:85"));
    assert!(line.contains("+early shop"));
    assert!(line.contains("-double elite"));
}

#[test]
fn format_evaluation_line_no_pros_no_cons() {
    let eval = make_path_evaluation(50.0, vec![], vec![]);
    let line = format_evaluation_line(&eval);
    assert_eq!(line, "Score:50");
}

#[test]
fn candidate_label_format() {
    assert_eq!(format_candidate_label(0), "Candidate 1");
    assert_eq!(format_candidate_label(4), "Candidate 5");
}

#[test]
fn recommendation_label_basic() {
    let eval = PathEvaluation {
        score: 90.0,
        pros: vec![],
        cons: vec![],
        description: PathDescription {
            route_chain: "M→$→R→E→R".into(),
            counts: String::new(),
            annotations: vec![],
            metrics: PathMetrics {
                counts: PathCounts {
                    monsters: 1,
                    elites: 1,
                    events: 0,
                    shops: 1,
                    rests: 2,
                    treasures: 0,
                },
                shop_timing: ShopTiming::Early,
                rest_before_first_elite: true,
                double_elite_without_rest: false,
                max_elite_to_rest_risk: 0.0,
            },
        },
    };
    let label = recommendation_label("Route 1 (左侧)", &eval);
    assert!(label.starts_with("Route 1 (左侧)"));
    assert!(label.contains("M→$→R→E→R"));
    assert!(label.contains("early shop"));
    assert!(label.contains("rest before elite"));
}

#[test]
fn rank_labeled_paths_sorts_by_score_descending() {
    let state = NormalizedState {
        floor: Some(1),
        current_hp: Some(68),
        max_hp: Some(75),
        gold: Some(99),
        ..test_state()
    };
    let paths = vec![
        ("Route 1".to_string(), path_of(&["M", "R", "M", "R"])),
        ("Route 2".to_string(), path_of(&["M", "$", "R", "E", "R"])),
    ];
    let ranked = rank_labeled_paths(paths, &state, false);
    assert_eq!(ranked.len(), 2);
}

#[test]
fn ordinal_basic() {
    assert_eq!(ordinal(1), "1st");
    assert_eq!(ordinal(2), "2nd");
    assert_eq!(ordinal(3), "3rd");
    assert_eq!(ordinal(4), "4th");
    assert_eq!(ordinal(11), "11th");
    assert_eq!(ordinal(12), "12th");
    assert_eq!(ordinal(13), "13th");
    assert_eq!(ordinal(21), "21st");
    assert_eq!(ordinal(22), "22nd");
    assert_eq!(ordinal(23), "23rd");
    assert_eq!(ordinal(101), "101st");
}

#[test]
fn from_left_label_basic() {
    let locale = test_locale();
    assert_eq!(from_left_label(1, &locale), "左起第1个");
    assert_eq!(from_left_label(7, &locale), "左起第7个");
}
