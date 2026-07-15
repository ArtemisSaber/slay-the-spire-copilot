#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOptions {
    pub skip_startup_check: bool,
    pub force_mock_provider: bool,
    pub setup_only: bool,
    pub postmortem_path: Option<String>,
    pub postmortem_plain: bool,
    pub knowledge_args: Option<Vec<String>>,
}

impl RuntimeOptions {
    pub fn from_env_and_args() -> Self {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let skip_env = std::env::var("SKIP_COMM_CONFIG").ok();
        runtime_options_from(args.iter().map(|s| s.as_str()), skip_env.as_deref())
    }

    pub fn opens_terminal_home(&self, stdin_is_terminal: bool) -> bool {
        stdin_is_terminal
            && !self.skip_startup_check
            && !self.force_mock_provider
            && !self.setup_only
            && self.postmortem_path.is_none()
            && self.knowledge_args.is_none()
    }
}

pub fn runtime_options_from<'a>(
    args: impl IntoIterator<Item = &'a str>,
    skip_comm_config: Option<&str>,
) -> RuntimeOptions {
    let mut skip_startup_check = matches!(skip_comm_config, Some("1" | "true" | "yes"));
    let mut force_mock_provider = false;
    let mut setup_only = false;
    let mut postmortem_path = None;
    let mut postmortem_plain = false;
    let mut knowledge_args = None;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg {
            "--no-startup-check" => skip_startup_check = true,
            "--stdin-test" => {
                skip_startup_check = true;
                force_mock_provider = true;
            }
            "setup" | "configure" => {
                setup_only = true;
                skip_startup_check = true;
            }
            "postmortem" => {
                for next in iter.by_ref() {
                    if next == "--plain" {
                        postmortem_plain = true;
                    } else {
                        postmortem_path = Some(next.to_string());
                        break;
                    }
                }
            }
            "knowledge" | "learning" => {
                skip_startup_check = true;
                knowledge_args = Some(iter.map(str::to_string).collect());
                break;
            }
            _ => {}
        }
    }

    RuntimeOptions {
        skip_startup_check,
        force_mock_provider,
        setup_only,
        postmortem_path,
        postmortem_plain,
        knowledge_args,
    }
}

pub fn is_in_game(raw: &serde_json::Value) -> bool {
    raw.get("in_game")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

pub fn is_error(raw: &serde_json::Value) -> bool {
    raw.get("error").is_some()
}

pub fn is_game_over_state(raw: &serde_json::Value) -> bool {
    raw.pointer("/game_state/screen_type")
        .and_then(|v| v.as_str())
        .is_some_and(|screen| screen == "GAME_OVER")
}

pub fn should_end_run(raw: &serde_json::Value, has_seen_game_state: bool) -> bool {
    is_game_over_state(raw) || (has_seen_game_state && !is_in_game(raw))
}

pub fn run_end_reason(raw: &serde_json::Value, has_seen_game_state: bool) -> Option<&'static str> {
    if is_game_over_state(raw) {
        Some("game_over")
    } else if has_seen_game_state && !is_in_game(raw) {
        Some("left_game")
    } else {
        None
    }
}

pub fn has_monsters(raw: &serde_json::Value) -> bool {
    raw.pointer("/game_state/combat_state/monsters")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .any(|m| !m.get("is_gone").and_then(|g| g.as_bool()).unwrap_or(false))
        })
        .unwrap_or(false)
}
