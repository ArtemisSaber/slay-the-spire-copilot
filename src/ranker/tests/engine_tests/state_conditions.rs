use super::*;

#[test]
fn state_turn_eq_matches() {
    let c = Condition::State(crate::ranker::rules::StateCondition {
        state: crate::ranker::rules::StatePredicates {
            turn_eq: Some(1),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("turn".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn state_remaining_energy_gt_matches() {
    let c = Condition::State(crate::ranker::rules::StateCondition {
        state: crate::ranker::rules::StatePredicates {
            remaining_energy_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("remaining_energy".to_string(), 2.0);
    let ctx = make_ctx(vars);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn state_incoming_damage_gt_matches() {
    let c = Condition::State(crate::ranker::rules::StateCondition {
        state: crate::ranker::rules::StatePredicates {
            incoming_damage_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("incoming_damage".to_string(), 5.0);
    let ctx = make_ctx(vars);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn state_turn_eq_no_match() {
    let c = Condition::State(crate::ranker::rules::StateCondition {
        state: crate::ranker::rules::StatePredicates {
            turn_eq: Some(2),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("turn".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn state_remaining_energy_gt_no_match() {
    let c = Condition::State(crate::ranker::rules::StateCondition {
        state: crate::ranker::rules::StatePredicates {
            remaining_energy_gt: Some(2),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("remaining_energy".to_string(), 1.0);
    let ctx = make_ctx(vars);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn state_incoming_damage_gt_no_match() {
    let c = Condition::State(crate::ranker::rules::StateCondition {
        state: crate::ranker::rules::StatePredicates {
            incoming_damage_gt: Some(10),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut vars = HashMap::new();
    vars.insert("incoming_damage".to_string(), 5.0);
    let ctx = make_ctx(vars);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}
