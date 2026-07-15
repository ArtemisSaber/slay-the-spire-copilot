use super::*;

#[test]
fn env_assignment_construction() {
    let a = EnvAssignment {
        key: "TEST",
        value: "val".to_string(),
    };
    assert_eq!(a.key, "TEST");
    assert_eq!(a.value, "val");
}

#[test]
fn env_assignment_equality() {
    let a = EnvAssignment {
        key: "KEY",
        value: "val".to_string(),
    };
    let b = EnvAssignment {
        key: "KEY",
        value: "val".to_string(),
    };
    let c = EnvAssignment {
        key: "KEY",
        value: "other".to_string(),
    };
    let d = EnvAssignment {
        key: "OTHER",
        value: "val".to_string(),
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_ne!(a, d);
}

#[test]
fn env_assignment_debug_format() {
    let a = assignment("LLM_PROVIDER", "mock");
    let dbg = format!("{a:?}");
    assert!(dbg.contains("LLM_PROVIDER"));
    assert!(dbg.contains("mock"));
}

#[test]
fn all_api_presets_have_required_fields() {
    assert!(!API_PRESETS.is_empty());
    for (i, preset) in API_PRESETS.iter().enumerate() {
        assert!(!preset.name.is_empty(), "preset {i}: name empty");
        assert!(
            !preset.description.is_empty(),
            "preset {i}: description empty"
        );
        assert!(!preset.provider.is_empty(), "preset {i}: provider empty");
        assert!(!preset.base_url.is_empty(), "preset {i}: base_url empty");
        assert!(!preset.model.is_empty(), "preset {i}: model empty");
    }
}

#[test]
fn pollinations_free_preset_no_api_key() {
    let preset = &API_PRESETS[0];
    assert_eq!(preset.name, "Pollinations Free");
    assert!(!preset.requires_api_key);
    assert_eq!(preset.disable_fast_thinking, "");
}

#[test]
fn deepseek_preset_disables_fast_thinking() {
    let preset = API_PRESETS
        .iter()
        .find(|p| p.name == "DeepSeek")
        .expect("DeepSeek preset not found");
    assert_eq!(preset.disable_fast_thinking, "true");
    assert_eq!(preset.provider, "openai-compatible");
}

#[test]
fn all_openai_compatible_presets_require_key() {
    for preset in API_PRESETS
        .iter()
        .filter(|p| p.provider == "openai-compatible")
    {
        assert!(preset.requires_api_key);
    }
}

#[test]
fn api_presets_count_is_9() {
    assert_eq!(API_PRESETS.len(), 9);
}

#[test]
fn learning_setup_parses_only_supported_modes() {
    assert_eq!(
        LearningSetup::from_env(Some("collect")),
        Some(LearningSetup::Collect)
    );
    assert_eq!(LearningSetup::from_env(Some("ON")), Some(LearningSetup::On));
    assert_eq!(LearningSetup::from_env(Some("unknown")), None);
    assert_eq!(LearningSetup::from_env(None), None);
}

#[test]
fn setup_boolean_parser_rejects_ambiguous_values() {
    assert_eq!(parse_env_bool(Some("yes")), Some(true));
    assert_eq!(parse_env_bool(Some("off")), Some(false));
    assert_eq!(parse_env_bool(Some("maybe")), None);
}
