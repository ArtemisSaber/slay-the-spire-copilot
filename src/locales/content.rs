use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SystemPromptLocale {
    pub card_reward: String,
    pub boss_card_reward: String,
    pub rest: String,
    pub combat_entry: String,
    pub boss_relic: String,
    pub event_choice: String,
    pub shop: String,
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
    pub shop: String,
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
    pub section_relics: String,
    pub section_potions: String,
    pub section_deck: String,
    pub section_monsters: String,
    pub run_section: String,
    pub label_started: String,
    pub label_ended: String,
    pub label_malformed: String,
    pub label_character: String,
    pub label_ascension: String,
    pub label_floor: String,
    pub label_hp: String,
    pub label_gold: String,
    pub label_deck_count: String,
    pub label_relics_count: String,
    pub label_picked: String,
    pub label_skipped: String,
    pub label_combats_summary: String,
    pub label_combat_hp: String,
    pub label_combat_elite: String,
    pub label_combat_boss: String,
    pub label_death: String,
    pub label_victory: String,
    pub label_combat_type_count: String,
    pub section_machine: String,
    pub ai_prompt: String,
    pub ai_requirements: String,
    pub ai_req1: String,
    pub ai_req2: String,
    pub ai_req3: String,
    pub ai_req4: String,
    pub machine_summary: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EffectParserLocale {
    pub damage_keyword: String,
    pub aoe_keywords: Vec<String>,
    pub random_keywords: Vec<String>,
    pub vulnerable_keyword: String,
    pub strength_gain_keyword: String,
    pub strength_lose_keyword: String,
    pub enter_wrath_keywords: Vec<String>,
    pub enter_calm_keywords: Vec<String>,
    pub exit_stance_keywords: Vec<String>,
    pub enter_divinity_keywords: Vec<String>,
    pub mantra_keyword: String,
    pub execute_keywords: Vec<String>,
    pub exhaust_keyword: String,
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
    pub grid_upgrade: String,
    pub grid_transform: String,
    pub grid_purge: String,
    pub grid_other: String,
}

impl I18nLocale {
    pub fn character_display_name<'a>(&'a self, class: &'a str) -> &'a str {
        match class {
            "IRONCLAD" => &self.class_ironclad,
            "THE_SILENT" => &self.class_silent,
            "DEFECT" => &self.class_defect,
            "WATCHER" => &self.class_watcher,
            _ => class,
        }
    }
}
