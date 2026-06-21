use super::*;

#[test]
fn project_root_is_valid_path() {
    let root = project_root();
    assert!(root.is_absolute() || root.starts_with("."));
}

#[test]
fn project_root_is_parent_of_binary() {
    let exe = std::env::current_exe().unwrap();
    let exe_parent = exe.parent().unwrap().to_path_buf();
    assert_eq!(project_root(), exe_parent);
}

#[test]
fn advice_output_dir_is_current_working_directory() {
    let dir = advice_output_dir();
    let cwd = std::env::current_dir().unwrap();
    assert_eq!(dir, cwd);
}

#[test]
fn project_root_and_advice_output_dir_may_differ() {
    let root = project_root();
    let cwd = advice_output_dir();
    if root != cwd {
        assert_ne!(root, cwd);
    }
}
