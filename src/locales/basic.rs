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
    pub max_hp: String,
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
    pub card_reward: String,
    pub boss_relic: String,
    pub event: String,
    pub options: String,
    pub upgrade_targets: String,
    pub hand_cards: String,
    pub routes: String,
    pub next_nodes: String,
    pub combat_profile: String,
    pub turn_status: String,
    pub shop: String,
    pub shop_cards: String,
    pub shop_relics: String,
    pub shop_potions: String,
    pub shop_purge: String,
    pub hand_select: String,
    pub grid_select: String,
    pub hand_select_available: String,
    pub selected_cards: String,
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
pub struct WarningLocale {
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
pub struct CombatTypesLocale {
    pub header: String,
    pub type_line: String,
    pub primary: String,
    pub secondary: String,
    pub trade: String,
    pub power_play: String,
    pub priority: String,
    pub type_normal: String,
    pub type_elite: String,
    pub type_boss: String,
    pub goal_normal: String,
    pub goal_elite: String,
    pub goal_boss: String,
    pub sub_normal: String,
    pub sub_elite: String,
    pub sub_boss: String,
    pub trade_normal: String,
    pub trade_elite: String,
    pub trade_boss: String,
    pub power_normal_high: String,
    pub power_normal_low: String,
    pub power_elite: String,
    pub power_boss: String,
    pub prio_scaling: String,
    pub prio_punish: String,
    pub prio_killable: String,
    pub prio_default: String,
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
