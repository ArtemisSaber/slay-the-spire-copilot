use super::*;

#[test]
fn player_power_matches() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            power: Some("Barricade".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("player_power_Barricade".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn player_power_not_matches_when_absent() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            power_not: Some("Barricade".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_ctx(HashMap::new());
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn player_power_not_fails_when_present() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            power_not: Some("Barricade".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("player_power_Barricade".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn player_stance_matches() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            stance: Some("Wrath".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("player_stance_Wrath".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn player_stance_not_matches() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            stance_not: Some("Divinity".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_ctx(HashMap::new());
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn player_relic_matches() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            relic: Some("Chemical X".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("player_relic_Chemical X".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn player_relic_not_matches() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            relic_not: Some("Snecko Eye".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_ctx(HashMap::new());
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn player_relic_not_fails_when_present() {
    let c = Condition::Player(crate::ranker::rules::PlayerCondition {
        player: crate::ranker::rules::PlayerPredicates {
            relic_not: Some("Snecko Eye".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("player_relic_Snecko Eye".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}
