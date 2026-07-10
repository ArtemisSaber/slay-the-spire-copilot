use super::*;

#[test]
fn compute_condition_zero_evaluates_false() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "zcheck".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Compute(crate::ranker::rules::ComputeCondition {
            compute: crate::ranker::rules::ComputePredicates {
                formula: Some("0".into()),
            },
        })],
    }]);
    let ctx = make_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    assert!(!results[0].matched);
    assert_eq!(results[0].score, 0);
}
