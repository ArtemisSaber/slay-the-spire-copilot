pub mod context;
pub mod engine;
pub mod formula;
pub mod parser;
pub mod predicates;
pub mod rules;

use crate::state::NormalizedState;

use context::ActionContext;
use engine::ScoredAction;
use rules::RuleSet;

pub fn rank(state: &NormalizedState) -> Vec<ScoredAction> {
    let rule_set = match load_rules() {
        Ok(rs) => rs,
        Err(e) => {
            tracing::error!("failed to load rules.json: {e}, ranking everything at 0");
            let contexts = ActionContext::build_all(state);
            return contexts
                .into_iter()
                .map(|ctx| ScoredAction {
                    action_type: ctx.action_type,
                    target_index: ctx.target_index,
                    score: 0,
                    breakdown: vec![],
                    is_avoid: false,
                })
                .collect();
        }
    };

    let rule_set = validate_score_fns(rule_set);

    let contexts = ActionContext::build_all(state);
    engine::rank_contexts(&contexts, &rule_set)
}

fn load_rules() -> Result<RuleSet, String> {
    let path = crate::logging::project_root().join("rules.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                "cannot read {}: {e}, falling back to embedded rules.json",
                path.display()
            );
            return serde_json::from_str(include_str!("rules.json"))
                .map_err(|e| format!("embedded rules.json error: {e}"));
        }
    };
    let rule_set: RuleSet =
        serde_json::from_str(&content).map_err(|e| format!("invalid rules.json: {e}"))?;
    Ok(rule_set)
}

fn validate_score_fns(mut rule_set: RuleSet) -> RuleSet {
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
    rule_set
}
