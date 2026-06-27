pub mod context;
pub mod engine;
pub mod formula;
pub mod parser;
pub mod predicates;
pub mod rules;

use std::sync::LazyLock;

use crate::state::NormalizedState;

use context::ActionContext;
use engine::ScoredAction;
use rules::RuleSet;

static RULES: LazyLock<RuleSet> = LazyLock::new(|| {
    let path = crate::logging::project_root().join("rules.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                "cannot read {}: {e}, using embedded rules.json",
                path.display()
            );
            return serde_json::from_str(include_str!("rules.json"))
                .expect("embedded rules.json is corrupt");
        }
    };
    let mut rule_set: RuleSet = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("invalid {}: {e}", path.display()));

    validate_score_fns(&mut rule_set);
    rule_set
});

pub fn rank(state: &NormalizedState) -> Vec<ScoredAction> {
    let contexts = ActionContext::build_all(state);
    engine::rank_contexts(&contexts, &RULES)
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
