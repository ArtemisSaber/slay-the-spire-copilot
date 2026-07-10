use super::*;

#[test]
fn target_intent_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            intent: Some("ATTACK".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        intent: Some("ATTACK".into()),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_intent_not_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            intent_not: Some("ATTACK".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        intent: Some("BUFF".into()),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_intent_no_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            intent: Some("ATTACK".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        intent: Some("BUFF".into()),
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_intent_not_fails_when_intent_matches() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            intent_not: Some("ATTACK".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        intent: Some("ATTACK".into()),
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}
