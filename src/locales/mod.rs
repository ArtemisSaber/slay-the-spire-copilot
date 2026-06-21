use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DangerLocale {
    pub fatal_damage: String,
    pub hp_critical: String,
    pub wrath_stance: String,
    pub no_block: String,
    pub danger_prefix: String,
    pub danger_reason_separator: String,
    pub danger_with_reasons: String,
    pub caution: String,
    pub safe: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusLocale {
    pub character: String,
    pub floor: String,
    pub hp: String,
    pub block: String,
    pub energy: String,
    pub gold: String,
    pub powers: String,
    pub relics: String,
    pub potions: String,
    pub damage_total: String,
    pub need_block: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonsterLocale {
    pub section_header: String,
    pub hp_line: String,
    pub killable: String,
    pub intent: String,
    pub damage: String,
    pub multi_hit: String,
    pub no_damage: String,
    pub block: String,
    pub powers: String,
    pub scaling: String,
    pub energy_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CardLocale {
    pub format: String,
    pub with_desc: String,
    pub hand_header: String,
    pub deck_header: String,
    pub type_group: String,
    pub card_count_multi: String,
    pub pile_empty: String,
    pub pile: String,
    pub pile_draw: String,
    pub pile_discard: String,
    pub pile_exhaust: String,
    pub cost_suffix: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SectionLocale {
    pub relics: String,
    pub potions: String,
    pub current_state: String,
    pub task: String,
    pub card_reward: String,
    pub boss_relic: String,
    pub event: String,
    pub options: String,
    pub upgrade_targets: String,
    pub hand_cards: String,
    pub routes: String,
    pub next_nodes: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapPositionLocale {
    pub only: String,
    pub left: String,
    pub right: String,
    pub middle: String,
    pub leftmost: String,
    pub rightmost: String,
    pub from_left: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskLocale {
    pub combat_entry: String,
    pub card_reward: String,
    pub boss_card_reward: String,
    pub rest: String,
    pub boss_relic: String,
    pub event_choice: String,
    pub map_suggestion: String,
    pub map_crossroad: String,
    pub generic: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WarningLocale {
    pub no_block: String,
    pub wrath_stance: String,
    pub boss_card_hp_note: String,
    pub boss_relic_hp_note: String,
    pub low_hp_rest: String,
    pub high_hp_smith: String,
    pub skip: String,
    pub event_id: String,
    pub event_unreadable: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParserLocale {
    pub recommendation: String,
    pub reason: String,
    pub risk: String,
    pub commentary: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FallbackLocale {
    pub llm_error: String,
    pub event_unreadable_choice: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SystemPromptLocale {
    pub card_reward: String,
    pub boss_card_reward: String,
    pub rest: String,
    pub combat_entry: String,
    pub boss_relic: String,
    pub event_choice: String,
    pub map_suggestion: String,
    pub map_crossroad: String,
    pub generic: String,
    pub postmortem: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FewShotExampleLocale {
    pub card_reward: String,
    pub boss_card_reward: String,
    pub rest: String,
    pub combat_entry: String,
    pub boss_relic: String,
    pub event_choice: String,
    pub map_suggestion: String,
    pub map_crossroad: String,
    pub generic: String,
    pub postmortem: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostmortemLocale {
    pub report_title: String,
    pub section_overview: String,
    pub section_decisions: String,
    pub section_rewards: String,
    pub ai_prompt: String,
    pub ai_requirements: String,
    pub ai_req1: String,
    pub ai_req2: String,
    pub ai_req3: String,
    pub ai_req4: String,
    pub machine_summary: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct I18nLocale {
    pub class_ironclad: String,
    pub class_silent: String,
    pub class_defect: String,
    pub class_watcher: String,
    pub type_attack: String,
    pub type_skill: String,
    pub type_power: String,
    pub type_curse: String,
    pub type_status: String,
    pub intent_attack: String,
    pub intent_attack_buff: String,
    pub intent_attack_debuff: String,
    pub intent_attack_defend: String,
    pub intent_buff: String,
    pub intent_debuff: String,
    pub intent_strong_debuff: String,
    pub intent_debug: String,
    pub intent_defend: String,
    pub intent_defend_debuff: String,
    pub intent_defend_buff: String,
    pub intent_escape: String,
    pub intent_magic: String,
    pub intent_none: String,
    pub intent_sleep: String,
    pub intent_stun: String,
    pub intent_unknown: String,
    pub rest_rest: String,
    pub rest_smith: String,
    pub rest_toke: String,
    pub rest_dig: String,
    pub rest_lift: String,
    pub rest_recall: String,
    pub rest_girya: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Locale {
    pub danger: DangerLocale,
    pub status: StatusLocale,
    pub monster: MonsterLocale,
    pub card: CardLocale,
    pub sections: SectionLocale,
    pub map_position: MapPositionLocale,
    pub tasks: TaskLocale,
    pub warnings: WarningLocale,
    pub format_footer: String,
    pub parser: ParserLocale,
    pub fallback: FallbackLocale,
    pub system_prompts: SystemPromptLocale,
    pub few_shot_examples: FewShotExampleLocale,
    pub postmortem: PostmortemLocale,
    pub i18n: I18nLocale,
    pub language_name: String,
}

impl Locale {
    pub fn load(lang: &str) -> Self {
        let json = match lang {
            "zh" => include_str!("zh.json"),
            "en" => include_str!("en.json"),
            "ja" => include_str!("ja.json"),
            "ko" => include_str!("ko.json"),
            _ => include_str!("en.json"),
        };
        serde_json::from_str(json).expect("failed to parse locale JSON")
    }
}

pub fn lang_to_locale_key(detected: &str) -> &str {
    match detected {
        "zhs" | "zht" | "zh" | "zhcn" | "zhtw" | "schinese" | "tchinese" | "chinesesimplified"
        | "chinesetraditional" => "zh",
        "jpn" | "ja" | "jp" | "japanese" => "ja",
        "kor" | "ko" | "kr" | "korean" | "koreana" => "ko",
        _ => "en",
    }
}
