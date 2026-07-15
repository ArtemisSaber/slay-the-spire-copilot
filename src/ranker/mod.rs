pub mod context;
pub mod engine;
pub mod formula;
pub mod parser;
pub mod predicates;
mod profile;
pub mod rules;
pub use profile::active_rules_sha256;

use std::sync::LazyLock;
use std::time::Instant;

use crate::state::NormalizedState;

use context::ActionContext;
use engine::ScoredAction;
use rules::RuleSet;

static RULES: LazyLock<RuleSet> = LazyLock::new(|| {
    let path = crate::logging::project_root().join("rules.json");
    load_rules_from(&path, include_str!("rules.json"))
});

fn load_rules_from(path: &std::path::Path, embedded: &str) -> RuleSet {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                "cannot read {}: {e}, using embedded rules.json",
                path.display()
            );
            return parse_embedded(embedded);
        }
    };
    match serde_json::from_str::<RuleSet>(&content) {
        Ok(mut rule_set) => {
            validate_score_fns(&mut rule_set);
            rule_set
        }
        Err(e) => {
            tracing::warn!("invalid {}: {e}, using embedded rules.json", path.display());
            parse_embedded(embedded)
        }
    }
}

fn parse_embedded(embedded: &str) -> RuleSet {
    let mut rule_set: RuleSet =
        serde_json::from_str(embedded).expect("embedded rules.json is corrupt");
    validate_score_fns(&mut rule_set);
    rule_set
}

pub fn rank(state: &NormalizedState) -> Vec<ScoredAction> {
    let energy = state.energy.unwrap_or(0);
    let hand_count = state.hand.len();
    let potion_count = state.potions.len();
    let monster_count = state.monsters.len();
    tracing::info!(
        "ranker started hand={} energy={} mons={} potions={}",
        hand_count,
        energy,
        monster_count,
        potion_count
    );

    let started = Instant::now();
    let contexts = ActionContext::build_all(state);
    let scored = engine::rank_contexts(&contexts, &RULES);

    let avoided = scored.iter().filter(|s| s.is_avoid).count();
    let top = scored.first();
    let top_score = top.map(|s| s.score).unwrap_or(0);
    let top_action = top.map(|s| &s.action_type);
    tracing::info!(
        "ranker finished contexts={} scored={} avoided={} top_score={} top={:?} duration_ms={}",
        contexts.len(),
        scored.len(),
        avoided,
        top_score,
        top_action,
        started.elapsed().as_millis(),
    );

    scored
}

fn validate_score_fns(rule_set: &mut RuleSet) {
    let valid: std::collections::HashSet<&str> = rule_set
        .available_score_fns
        .iter()
        .map(|s| s.as_str())
        .collect();

    let before = rule_set.rules.len();
    rule_set.rules.retain(|rule| {
        if let Some(ref fn_name) = rule.score_fn
            && !valid.contains(fn_name.as_str())
        {
            tracing::warn!(
                "rule {} has unknown score_fn \"{fn_name}\", discarding rule",
                rule.rule_id
            );
            return false;
        }
        if rule.formula.is_some() && rule.score_fn.is_some() {
            tracing::warn!(
                "rule {} has both formula and score_fn, discarding rule",
                rule.rule_id
            );
            return false;
        }
        true
    });
    let after = rule_set.rules.len();
    if after < before {
        tracing::warn!(
            "discarded {} rules with unknown/invalid score_fns",
            before - after
        );
    }
}

#[cfg(test)]
mod tests {
    use super::context::ActionType;
    use super::rules::{Rule, RuleCategory, Weight};
    use super::*;

    #[test]
    fn validate_score_fns_removes_unknown_and_ambiguous_rules() {
        let mut rule_set = RuleSet {
            version: "1.0".into(),
            available_score_fns: vec!["known_fn".into()],
            rules: vec![
                Rule {
                    rule_id: "valid_formula_rule".into(),
                    priority: 1000,
                    weight: Weight::Value(10),
                    formula: Some("@weight * 1".into()),
                    score_fn: None,
                    override_rule: None,
                    applies_to: vec!["play_card".into()],
                    category: RuleCategory::PerTarget,
                    conditions: vec![],
                },
                Rule {
                    rule_id: "unknown_score_fn".into(),
                    priority: 1000,
                    weight: Weight::Value(10),
                    formula: None,
                    score_fn: Some("unknown_fn".into()),
                    override_rule: None,
                    applies_to: vec!["play_card".into()],
                    category: RuleCategory::PerTarget,
                    conditions: vec![],
                },
                Rule {
                    rule_id: "both_formula_and_fn".into(),
                    priority: 1000,
                    weight: Weight::Value(10),
                    formula: Some("@weight * 1".into()),
                    score_fn: Some("known_fn".into()),
                    override_rule: None,
                    applies_to: vec!["play_card".into()],
                    category: RuleCategory::PerTarget,
                    conditions: vec![],
                },
                Rule {
                    rule_id: "valid_known_fn".into(),
                    priority: 1000,
                    weight: Weight::Value(10),
                    formula: None,
                    score_fn: Some("known_fn".into()),
                    override_rule: None,
                    applies_to: vec!["play_card".into()],
                    category: RuleCategory::PerTarget,
                    conditions: vec![],
                },
            ],
        };

        validate_score_fns(&mut rule_set);

        let ids: Vec<&str> = rule_set.rules.iter().map(|r| r.rule_id.as_str()).collect();
        assert_eq!(ids.len(), 2, "expected 2 valid rules, got: {:?}", ids);
        assert!(ids.contains(&"valid_formula_rule"));
        assert!(ids.contains(&"valid_known_fn"));
    }

    #[test]
    fn rank_returns_single_end_turn_for_default_state() {
        let state = NormalizedState::default();
        let result = rank(&state);
        assert_eq!(
            result.len(),
            1,
            "expected 1 end-turn action, got {}",
            result.len()
        );
        assert!(
            matches!(result[0].action_type, ActionType::EndTurn),
            "expected EndTurn action"
        );
    }

    #[test]
    fn rules_lazy_static_loads_embedded_rules_json() {
        let rules = &*RULES;
        assert_eq!(rules.version, "1.0");
        assert!(
            !rules.rules.is_empty(),
            "embedded rules.json should have rules"
        );
        assert!(
            !rules.available_score_fns.is_empty(),
            "should have available_score_fns"
        );
        assert!(
            rules.rules.iter().any(|r| r.rule_id == "base_cost_penalty"),
            "should include base_cost_penalty rule from embedded rules.json"
        );
    }

    #[test]
    fn load_rules_from_falls_back_on_malformed_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rules.json");
        std::fs::write(&path, "{ this is not valid json }").unwrap();

        let rule_set = load_rules_from(&path, include_str!("rules.json"));

        assert_eq!(rule_set.version, "1.0");
        assert!(!rule_set.rules.is_empty());
        assert!(
            rule_set
                .rules
                .iter()
                .any(|r| r.rule_id == "base_cost_penalty")
        );
    }

    #[test]
    fn load_rules_from_falls_back_on_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");

        let rule_set = load_rules_from(&path, include_str!("rules.json"));

        assert_eq!(rule_set.version, "1.0");
        assert!(!rule_set.rules.is_empty());
    }

    #[test]
    fn load_rules_from_uses_valid_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rules.json");
        let embedded = include_str!("rules.json");
        std::fs::write(&path, embedded).unwrap();

        let rule_set = load_rules_from(&path, embedded);

        assert_eq!(rule_set.version, "1.0");
        assert!(!rule_set.rules.is_empty());
        assert!(
            rule_set
                .rules
                .iter()
                .any(|r| r.rule_id == "base_cost_penalty")
        );
    }
}
