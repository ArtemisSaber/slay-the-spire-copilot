use super::ActionCandidate;

const ACTION_REFERENCE_PREFIX: char = 'A';

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
