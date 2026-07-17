use crate::locales::Locale;
use crate::state::NormalizedState;

#[derive(Clone, Copy)]
pub enum Effort {
    Fast,
    Medium,
    Heavy,
}

impl Effort {
    pub fn from_screen_type(screen_type: &str, in_combat: bool) -> Self {
        if in_combat {
            return Effort::Fast;
        }

        match screen_type {
            "CARD_REWARD" | "BOSS_REWARD" | "MAP" => Effort::Heavy,
            "NONE" | "HAND_SELECT" => Effort::Fast,
            "GRID" => Effort::Medium,
            _ => Effort::Medium,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Effort::Fast => "fast",
            Effort::Medium => "medium",
            Effort::Heavy => "heavy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdviceScenario {
    CardReward,
    BossCardReward,
    BossRelic,
    Rest,
    EventChoice,
    Shop,
    CombatEntry,
    MapSuggestion,
    MapCrossroad,
    Generic,
    Postmortem,
}

impl AdviceScenario {
    pub fn from_state(state: &NormalizedState) -> Self {
        match state
            .screen_type
            .as_ref()
            .map(|screen_type| screen_type.as_str())
        {
            Some("CARD_REWARD") if state.is_boss_card_reward() => AdviceScenario::BossCardReward,
            Some("CARD_REWARD") => AdviceScenario::CardReward,
            Some("BOSS_REWARD") => AdviceScenario::BossRelic,
            Some("REST") => AdviceScenario::Rest,
            Some("EVENT") => AdviceScenario::EventChoice,
            Some("SHOP_SCREEN") => AdviceScenario::Shop,
            Some("MAP") if state.map_first_node_chosen == Some(true) => {
                AdviceScenario::MapCrossroad
            }
            Some("MAP") => AdviceScenario::MapSuggestion,
            _ if state.has_active_monsters() => AdviceScenario::CombatEntry,
            _ => AdviceScenario::Generic,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            AdviceScenario::CardReward => "card_reward",
            AdviceScenario::BossCardReward => "boss_card_reward",
            AdviceScenario::BossRelic => "boss_relic",
            AdviceScenario::Rest => "rest",
            AdviceScenario::EventChoice => "event_choice",
            AdviceScenario::Shop => "shop",
            AdviceScenario::CombatEntry => "combat_entry",
            AdviceScenario::MapSuggestion => "map_suggestion",
            AdviceScenario::MapCrossroad => "map_crossroad",
            AdviceScenario::Generic => "generic",
            AdviceScenario::Postmortem => "postmortem",
        }
    }

    pub fn system_prompt(self, locale: &Locale) -> String {
        if matches!(self, AdviceScenario::Postmortem) {
            return format!(
                "{}\n\n{}",
                &locale.system_prompts.postmortem,
                self.few_shot_example(locale)
            );
        }
        super::prompts::unified_system_prompt(locale)
    }

    pub(crate) fn few_shot_example(self, locale: &Locale) -> &str {
        match self {
            AdviceScenario::CardReward => &locale.few_shot_examples.card_reward,
            AdviceScenario::BossCardReward => &locale.few_shot_examples.boss_card_reward,
            AdviceScenario::BossRelic => &locale.few_shot_examples.boss_relic,
            AdviceScenario::Rest => &locale.few_shot_examples.rest,
            AdviceScenario::EventChoice => &locale.few_shot_examples.event_choice,
            AdviceScenario::Shop => &locale.few_shot_examples.shop,
            AdviceScenario::CombatEntry => &locale.few_shot_examples.combat_entry,
            AdviceScenario::MapSuggestion => &locale.few_shot_examples.map_suggestion,
            AdviceScenario::MapCrossroad => &locale.few_shot_examples.map_crossroad,
            AdviceScenario::Generic => &locale.few_shot_examples.generic,
            AdviceScenario::Postmortem => &locale.few_shot_examples.postmortem,
        }
    }
}
