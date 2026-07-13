use super::*;

#[test]
fn ensure_config_returns_true_when_valid_config_matches() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let home = tempfile::tempdir().unwrap();
    let localappdata = home.path().join("LocalAppData");
    let config_dir = home
        .path()
        .join(".config")
        .join("ModTheSpire")
        .join("CommunicationModCJK");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.properties");
    let exe = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    std::fs::write(
        &config_path,
        format!(
            "command={}\nrunAtGameStart=true\n",
            format_command_value(&exe)
        ),
    )
    .unwrap();

    let old_home = std::env::var("HOME").ok();
    let old_localappdata = std::env::var("LOCALAPPDATA").ok();
    let old_lang = std::env::var("SLAY_THE_SPIRE_LANGUAGE").ok();
    unsafe { std::env::set_var("HOME", home.path().to_str().unwrap()) };
    unsafe { std::env::set_var("LOCALAPPDATA", localappdata.to_str().unwrap()) };
    unsafe { std::env::set_var("SLAY_THE_SPIRE_LANGUAGE", "ENG") };

    let result = ensure_config();

    if let Some(h) = old_home {
        unsafe { std::env::set_var("HOME", h) };
    } else {
        unsafe { std::env::remove_var("HOME") };
    }
    if let Some(v) = old_localappdata {
        unsafe { std::env::set_var("LOCALAPPDATA", v) };
    } else {
        unsafe { std::env::remove_var("LOCALAPPDATA") };
    }
    if let Some(l) = old_lang {
        unsafe { std::env::set_var("SLAY_THE_SPIRE_LANGUAGE", l) };
    } else {
        unsafe { std::env::remove_var("SLAY_THE_SPIRE_LANGUAGE") };
    }

    assert!(result);
}

#[test]
fn ensure_config_cjk_warning_on_standard_mod() {
    let _env_guard = ENV_LOCK.lock().unwrap();
    let home = tempfile::tempdir().unwrap();
    let localappdata = home.path().join("LocalAppData");
    let config_dir = home
        .path()
        .join(".config")
        .join("ModTheSpire")
        .join("CommunicationMod");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.properties");
    let exe = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    std::fs::write(
        &config_path,
        format!(
            "command={}\nrunAtGameStart=true\n",
            format_command_value(&exe)
        ),
    )
    .unwrap();

    let old_home = std::env::var("HOME").ok();
    let old_localappdata = std::env::var("LOCALAPPDATA").ok();
    let old_lang = std::env::var("SLAY_THE_SPIRE_LANGUAGE").ok();
    unsafe { std::env::set_var("HOME", home.path().to_str().unwrap()) };
    unsafe { std::env::set_var("LOCALAPPDATA", localappdata.to_str().unwrap()) };
    unsafe { std::env::set_var("SLAY_THE_SPIRE_LANGUAGE", "ZHS") };

    let result = ensure_config();

    if let Some(h) = old_home {
        unsafe { std::env::set_var("HOME", h) };
    } else {
        unsafe { std::env::remove_var("HOME") };
    }
    if let Some(v) = old_localappdata {
        unsafe { std::env::set_var("LOCALAPPDATA", v) };
    } else {
        unsafe { std::env::remove_var("LOCALAPPDATA") };
    }
    if let Some(l) = old_lang {
        unsafe { std::env::set_var("SLAY_THE_SPIRE_LANGUAGE", l) };
    } else {
        unsafe { std::env::remove_var("SLAY_THE_SPIRE_LANGUAGE") };
    }

    assert!(!result);
}
