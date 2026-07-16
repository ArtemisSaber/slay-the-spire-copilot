use serde::Serialize;

use super::ActionCandidate;

const ACTION_REFERENCE_PREFIX: char = 'A';

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PromptActionCandidate {
    #[serde(rename = "ref")]
    pub(crate) action_ref: String,
    pub(crate) kind: String,
    pub(crate) label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) target_required: Option<bool>,
}

pub(crate) fn prompt_action_candidates(
    candidates: &[ActionCandidate],
) -> Vec<PromptActionCandidate> {
    candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| PromptActionCandidate {
            action_ref: action_reference(index),
            kind: candidate.kind.clone(),
            label: candidate.label.clone(),
            target_required: candidate.target_required,
        })
        .collect()
}

pub(crate) fn action_reference(index: usize) -> String {
    format!("{ACTION_REFERENCE_PREFIX}{index}")
}

pub(crate) fn candidate_for_reference<'a>(
    action_ref: &str,
    candidates: &'a [ActionCandidate],
) -> Option<&'a ActionCandidate> {
    let index = action_ref
        .strip_prefix(ACTION_REFERENCE_PREFIX)?
        .parse()
        .ok()?;
    (action_reference(index) == action_ref)
        .then(|| candidates.get(index))
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidates() -> Vec<ActionCandidate> {
        vec![ActionCandidate {
            kind: "play".into(),
            action_id: "combat:play:secret-uuid".into(),
            label: "Play Strike".into(),
            target_required: Some(true),
        }]
    }

    #[test]
    fn prompt_candidates_hide_execution_identifiers() {
        let prompt = prompt_action_candidates(&candidates());

        assert_eq!(prompt[0].action_ref, "A0");
        assert_eq!(prompt[0].kind, "play");
        assert_eq!(prompt[0].label, "Play Strike");
        assert!(
            !serde_json::to_string(&prompt)
                .expect("prompt candidates should serialize")
                .contains("secret-uuid")
        );
    }

    #[test]
    fn references_must_be_canonical_and_in_range() {
        let candidates = candidates();

        assert_eq!(
            candidate_for_reference("A0", &candidates),
            candidates.first()
        );
        assert_eq!(candidate_for_reference("A00", &candidates), None);
        assert_eq!(candidate_for_reference("A1", &candidates), None);
        assert_eq!(candidate_for_reference("0", &candidates), None);
    }
}
