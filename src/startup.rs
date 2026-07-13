mod config;
mod language;
mod messages;
mod repair;

pub use language::detect_game_language;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedGameLanguage {
    pub value: String,
    pub source: String,
}

/// Returns true if the CommunicationMod config is properly configured and points to
/// this binary. If not, attempts to fix the config (auto-fix or interactive prompt).
pub fn ensure_config() -> bool {
    let paths = config::communication_mod_config_paths();
    let detected_language = language::detect_game_language();

    if let Some(config_path) = config::find_config_matching_current_exe(&paths) {
        if let Some(language) = &detected_language
            && language::cjk_mod_required_for_language(config_path, Some(&language.value))
        {
            let cjk_config_path = paths
                .iter()
                .find(|path| config::config_path_uses_cjk_mod(path))
                .unwrap_or(config_path);
            messages::show_cjk_mod_language_message_to(
                &mut std::io::stdout().lock(),
                language,
                config_path,
                cjk_config_path,
            );
            tracing::warn!(
                language = language.value,
                source = language.source,
                config = %config_path.display(),
                "Communication Mod CJK is required for detected game language"
            );
            return false;
        }
        tracing::info!("CommunicationMod config found and configured correctly");
        return true;
    }

    let exe = config::current_exe_string();
    if repair::try_fix_config(&paths, &exe) {
        return false; // config was fixed but need CommunicationMod restart
    }

    false
}

#[cfg(test)]
use {
    config::*,
    language::{
        cjk_mod_required_for_language, extract_vdf_value, language_needs_cjk_mod,
        push_slay_the_spire_language_path_for_steam_root,
        push_steam_appmanifest_path_for_steam_root, read_gameplay_settings_language,
        read_steam_appmanifest_language, slay_the_spire_language_paths, steam_appmanifest_paths,
    },
    messages::*,
    repair::*,
};

#[cfg(test)]
#[path = "tests/startup_tests.rs"]
mod tests;
