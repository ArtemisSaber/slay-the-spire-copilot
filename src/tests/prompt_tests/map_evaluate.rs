use super::*;

#[test]
fn evaluate_path_records_pros_and_cons() {
    let state = NormalizedState {
        floor: Some(1),
        current_hp: Some(62),
        max_hp: Some(75),
        gold: Some(180),
        ..test_state()
    };
    let path = path_of(&["M", "$", "R", "E", "M", "R"]);
    let eval = evaluate_path(&path, &state, false);
    assert!(eval.score > 80.0);
    assert!(eval.pros.iter().any(|p| p.contains("early shop")));
    assert!(
        eval.pros
            .iter()
            .any(|p| p.contains("rest before first elite"))
    );
    assert!(!eval.cons.iter().any(|c| c.contains("double elite")));
}

#[test]
fn evaluate_path_prefers_healthy_gold_route_with_early_shop_and_rest_elite() {
    let state = NormalizedState {
        floor: Some(1),
        current_hp: Some(68),
        max_hp: Some(75),
        gold: Some(180),
        ..test_state()
    };
    let shop_elite = path_of(&["M", "$", "M", "R", "E", "R"]);
    let safe_no_shop = path_of(&["M", "?", "?", "M", "R", "M", "R"]);
    let shop_elite_eval = evaluate_path(&shop_elite, &state, false);
    let safe_no_shop_eval = evaluate_path(&safe_no_shop, &state, false);
    assert!(shop_elite_eval.score > safe_no_shop_eval.score);
}

#[test]
fn evaluate_path_penalizes_double_elite_when_hp_is_low() {
    let state = NormalizedState {
        floor: Some(1),
        current_hp: Some(20),
        max_hp: Some(80),
        gold: Some(50),
        ..test_state()
    };
    let double_elite = path_of(&["E", "M", "E", "M", "R"]);
    let safe_route = path_of(&["M", "?", "R", "M", "R"]);
    let double_elite_eval = evaluate_path(&double_elite, &state, false);
    let safe_route_eval = evaluate_path(&safe_route, &state, false);
    assert!(safe_route_eval.score > double_elite_eval.score);
    assert!(
        double_elite_eval
            .cons
            .iter()
            .any(|c| c.contains("double elite"))
    );
}

#[test]
fn recommendation_features_four_events() {
    let eval = PathEvaluation {
        score: 75.0,
        pros: vec![],
        cons: vec![],
        description: PathDescription {
            route_chain: "M→?→?→?→R".into(),
            counts: String::new(),
            annotations: vec![],
            metrics: PathMetrics {
                counts: PathCounts {
                    monsters: 1,
                    elites: 0,
                    events: 4,
                    shops: 0,
                    rests: 1,
                    treasures: 0,
                },
                shop_timing: ShopTiming::None,
                rest_before_first_elite: false,
                double_elite_without_rest: false,
                max_elite_to_rest_risk: 0.0,
            },
        },
    };
    let features = recommendation_features(&eval);
    assert!(features.contains("4 events"));
}
