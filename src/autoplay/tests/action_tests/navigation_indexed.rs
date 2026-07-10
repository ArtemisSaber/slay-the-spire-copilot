use super::*;

#[test]
fn resolve_indexed_rejects_wrong_kind() {
    let req = ActionRequest {
        kind: "skip".into(),
        action_id: "prefix:0".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_indexed("prefix:", 5, &req), None);
}

#[test]
fn resolve_indexed_rejects_out_of_bounds() {
    let req = ActionRequest {
        kind: "choose".into(),
        action_id: "prefix:5".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_indexed("prefix:", 3, &req), None);
}

#[test]
fn resolve_indexed_rejects_bad_format() {
    let req = ActionRequest {
        kind: "choose".into(),
        action_id: "prefix:abc".into(),
        target_index: None,
    };
    assert_eq!(resolve_requested_indexed("prefix:", 3, &req), None);
}
