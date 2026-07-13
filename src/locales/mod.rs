#![allow(
    dead_code,
    reason = "WIP: effect_parser locale fields will be used by kill_scan in subsequent phases"
)]

mod basic;
mod content;

pub use basic::*;
pub use content::*;

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Locale {
    pub danger: DangerLocale,
    pub status: StatusLocale,
    pub monster: MonsterLocale,
    pub card: CardLocale,
    pub sections: SectionLocale,
    pub map_position: MapPositionLocale,
    pub warnings: WarningLocale,
    pub combat_types: CombatTypesLocale,
    pub parser: ParserLocale,
    pub fallback: FallbackLocale,
    pub system_prompts: SystemPromptLocale,
    pub few_shot_examples: FewShotExampleLocale,
    pub postmortem: PostmortemLocale,
    pub i18n: I18nLocale,
    pub effect_parser: EffectParserLocale,
    pub language_name: String,
    #[serde(default)]
    pub lang_code: String,
    pub unified_preamble: String,
    #[serde(default)]
    pub relic_counter_cycles: HashMap<String, String>,
}

impl Locale {
    pub fn load(lang: &str) -> Self {
        let (json, code) = match lang {
            "zh" => (include_str!("zh.json"), "zh"),
            "en" => (include_str!("en.json"), "en"),
            "ja" => (include_str!("ja.json"), "ja"),
            "ko" => (include_str!("ko.json"), "ko"),
            _ => (include_str!("en.json"), "en"),
        };
        let mut locale: Self = serde_json::from_str(json).expect("failed to parse locale JSON");
        locale.lang_code = code.to_string();
        locale
    }
}

pub fn lang_to_locale_key(detected: &str) -> &str {
    let detected = detected.trim().to_ascii_lowercase();
    match detected.as_str() {
        "zhs" | "zht" | "zh" | "zhcn" | "zhtw" | "schinese" | "tchinese" | "chinesesimplified"
        | "chinesetraditional" => "zh",
        "jpn" | "ja" | "jp" | "japanese" => "ja",
        "kor" | "ko" | "kr" | "korean" | "koreana" => "ko",
        _ => "en",
    }
}

#[cfg(test)]
#[path = "../tests/locale_tests.rs"]
mod tests;
