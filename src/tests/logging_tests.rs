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

#[test]
fn rotate_log_shifts_existing_files() {
    let dir = std::env::temp_dir().join("sts_copilot_rotate_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join("test.log"), "session 0").unwrap();
    fs::write(dir.join("test.log.1"), "session 1").unwrap();

    rotate_log(&dir, "test.log");

    assert!(!dir.join("test.log").exists());
    assert_eq!(
        fs::read_to_string(dir.join("test.log.1")).unwrap(),
        "session 0"
    );
    assert_eq!(
        fs::read_to_string(dir.join("test.log.2")).unwrap(),
        "session 1"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rotate_log_drops_oldest_when_all_slots_full() {
    let dir = std::env::temp_dir().join("sts_copilot_rotate_full_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join("test.log"), "current").unwrap();
    fs::write(dir.join("test.log.1"), "prev").unwrap();
    fs::write(dir.join("test.log.2"), "oldest").unwrap();

    rotate_log(&dir, "test.log");

    assert!(!dir.join("test.log").exists());
    assert_eq!(
        fs::read_to_string(dir.join("test.log.1")).unwrap(),
        "current"
    );
    assert_eq!(fs::read_to_string(dir.join("test.log.2")).unwrap(), "prev");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rotate_log_no_existing_files_is_noop() {
    let dir = std::env::temp_dir().join("sts_copilot_rotate_empty_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    rotate_log(&dir, "test.log");

    assert!(!dir.join("test.log").exists());
    assert!(!dir.join("test.log.1").exists());
    assert!(!dir.join("test.log.2").exists());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn log_raw_input_to_appends_lines() {
    let dir = std::env::temp_dir().join("sts_copilot_raw_input_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    log_raw_input_to(&dir, "line 1");
    log_raw_input_to(&dir, "line 2");

    let path = dir.join("comm-mod-raw.log");
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "line 1\nline 2\n");

    let _ = fs::remove_dir_all(&dir);
}
