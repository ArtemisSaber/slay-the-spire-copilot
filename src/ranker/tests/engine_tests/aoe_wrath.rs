use super::*;

#[test]
fn wrath_penalty_formula_correct() {
    let rules = make_rules(vec![make_enters_wrath_rule()]);
    let mut vars = HashMap::new();
    vars.insert("enters_wrath".to_string(), 1.0);
    vars.insert("incoming_damage".to_string(), 9.0);
    vars.insert("current_hp".to_string(), 72.0);
    let ctx = make_ctx(vars);

    let results = evaluate(&ctx, &rules);
    assert!(results[0].matched);
    assert_eq!(results[0].score, 9 * -1000 / 72);
}

#[test]
fn wrath_penalty_not_triggered_when_not_entering_wrath() {
    let rules = make_rules(vec![make_enters_wrath_rule()]);
    let mut vars = HashMap::new();
    vars.insert("enters_wrath".to_string(), 0.0);
    vars.insert("incoming_damage".to_string(), 9.0);
    vars.insert("current_hp".to_string(), 72.0);
    let ctx = make_ctx(vars);

    let results = evaluate(&ctx, &rules);
    assert!(!results[0].matched);
    assert_eq!(results[0].score, 0);
}

#[test]
fn wrath_penalty_not_triggered_when_no_incoming_damage() {
    let rules = make_rules(vec![make_enters_wrath_rule()]);
    let mut vars = HashMap::new();
    vars.insert("enters_wrath".to_string(), 1.0);
    vars.insert("incoming_damage".to_string(), 0.0);
    vars.insert("current_hp".to_string(), 72.0);
    let ctx = make_ctx(vars);

    let results = evaluate(&ctx, &rules);
    assert!(!results[0].matched);
    assert_eq!(results[0].score, 0);
}

#[test]
fn wrath_penalty_scales_with_incoming_damage() {
    let rules = make_rules(vec![make_enters_wrath_rule()]);

    let mut vars_low = HashMap::new();
    vars_low.insert("enters_wrath".to_string(), 1.0);
    vars_low.insert("incoming_damage".to_string(), 5.0);
    vars_low.insert("current_hp".to_string(), 72.0);
    let ctx_low = make_ctx(vars_low);

    let mut vars_high = HashMap::new();
    vars_high.insert("enters_wrath".to_string(), 1.0);
    vars_high.insert("incoming_damage".to_string(), 36.0);
    vars_high.insert("current_hp".to_string(), 10.0);
    let ctx_high = make_ctx(vars_high);

    let results_low = evaluate(&ctx_low, &rules);
    let results_high = evaluate(&ctx_high, &rules);
    assert!(results_low[0].matched);
    assert!(results_high[0].matched);
    assert_eq!(results_low[0].score, 5 * -1000 / 72);
    assert_eq!(results_high[0].score, 36 * -1000 / 10);
    assert!(results_high[0].score < results_low[0].score);
}

#[test]
fn wrath_penalty_is_per_card_not_per_target() {
    let rule = make_enters_wrath_rule();
    assert_eq!(rule.category, RuleCategory::PerCard);
}
