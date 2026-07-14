use serde_json::Value;

use crate::locales::Locale;

use super::parse::{
    extract_cards, extract_event_choice, extract_potion_infos, extract_relic_infos, first_array,
    first_raw_string, first_string, is_readable_text,
};
use super::{CardInfo, MapCoord, PotionInfo, RelicInfo};

pub(super) struct EventFields {
    pub(super) id: Option<String>,
    pub(super) name: Option<String>,
    pub(super) body: Option<String>,
    pub(super) choices: Vec<String>,
}

pub(super) struct ShopFields {
    pub(super) cards: Vec<CardInfo>,
    pub(super) relics: Vec<RelicInfo>,
    pub(super) potions: Vec<PotionInfo>,
    pub(super) purge_available: bool,
    pub(super) purge_cost: Option<i64>,
}

pub(super) struct HandSelectFields {
    pub(super) max_cards: Option<i64>,
    pub(super) can_pick_zero: bool,
    pub(super) selected: Vec<CardInfo>,
}

pub(super) struct GridFields {
    pub(super) cards: Vec<CardInfo>,
    pub(super) selected_cards: Vec<CardInfo>,
    pub(super) for_upgrade: bool,
    pub(super) for_transform: bool,
    pub(super) for_purge: bool,
    pub(super) num_cards: Option<i64>,
}

pub(super) fn extract_event_fields(
    screen_state: Option<&Value>,
    game_state: Option<&Value>,
    locale: &Locale,
) -> EventFields {
    let id = screen_state
        .and_then(|state| first_raw_string(state, &["event_id", "eventId", "id"]))
        .filter(|text| is_readable_text(text));
    let name = screen_state.and_then(|state| first_string(state, &["event_name", "name", "title"]));
    let body = screen_state
        .and_then(|state| first_string(state, &["body", "body_text", "event_text", "description"]));
    let choices = screen_state
        .and_then(|state| first_array(state, &["options", "choices", "buttons"]))
        .or_else(|| game_state.and_then(|state| first_array(state, &["choice_list"])))
        .map(|choices| {
            choices
                .iter()
                .enumerate()
                .map(|(index, choice)| {
                    extract_event_choice(choice).unwrap_or_else(|| {
                        locale
                            .fallback
                            .event_unreadable_choice
                            .replace("{idx}", &(index + 1).to_string())
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    EventFields {
        id,
        name,
        body,
        choices,
    }
}

pub(super) fn extract_shop_fields(screen_state: Option<&Value>) -> ShopFields {
    let cards = screen_state
        .and_then(|state| state.get("cards"))
        .and_then(|value| value.as_array())
        .map(|cards| extract_cards(cards))
        .unwrap_or_default();
    let relics = screen_state
        .and_then(|state| state.get("relics"))
        .and_then(|value| value.as_array())
        .map(|relics| extract_relic_infos(relics))
        .unwrap_or_default();
    let potions = screen_state
        .and_then(|state| state.get("potions"))
        .and_then(|value| value.as_array())
        .map(|potions| extract_potion_infos(potions))
        .unwrap_or_default();
    let purge_available = screen_state
        .and_then(|state| state.get("purge_available"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let purge_cost = screen_state
        .and_then(|state| state.get("purge_cost"))
        .and_then(|value| value.as_i64());

    ShopFields {
        cards,
        relics,
        potions,
        purge_available,
        purge_cost,
    }
}

pub(super) fn extract_map_nodes(game_state: Option<&Value>) -> Vec<MapCoord> {
    game_state
        .and_then(|state| state.get("map"))
        .and_then(|value| value.as_array())
        .map(|nodes| {
            nodes
                .iter()
                .map(|node| MapCoord {
                    symbol: node
                        .get("symbol")
                        .and_then(|value| value.as_str())
                        .unwrap_or("?")
                        .to_string(),
                    x: node.get("x").and_then(|value| value.as_i64()).unwrap_or(0),
                    y: node.get("y").and_then(|value| value.as_i64()).unwrap_or(0),
                    children: node
                        .get("children")
                        .and_then(|value| value.as_array())
                        .map(|children| {
                            children
                                .iter()
                                .filter_map(|child| {
                                    Some((child.get("x")?.as_i64()?, child.get("y")?.as_i64()?))
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn extract_empty_potion_slots(game_state: Option<&Value>) -> usize {
    game_state
        .and_then(|state| state.get("potions"))
        .and_then(|value| value.as_array())
        .map(|potions| {
            potions
                .iter()
                .filter(|potion| {
                    potion
                        .get("id")
                        .and_then(|id| id.as_str())
                        .map(|id| id == "Potion Slot")
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

pub(super) fn extract_hand_select_fields(screen_state: Option<&Value>) -> HandSelectFields {
    let max_cards = screen_state
        .and_then(|state| state.get("max_cards"))
        .and_then(|value| value.as_i64());
    let can_pick_zero = screen_state
        .and_then(|state| state.get("can_pick_zero"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let selected = screen_state
        .and_then(|state| state.get("selected"))
        .and_then(|value| value.as_array())
        .map(|cards| extract_cards(cards))
        .unwrap_or_default();

    HandSelectFields {
        max_cards,
        can_pick_zero,
        selected,
    }
}

pub(super) fn extract_grid_fields(screen_state: Option<&Value>) -> GridFields {
    let cards = screen_state
        .and_then(|state| state.get("cards"))
        .and_then(|value| value.as_array())
        .map(|cards| extract_cards(cards))
        .unwrap_or_default();
    let selected_cards = screen_state
        .and_then(|state| state.get("selected_cards"))
        .and_then(|value| value.as_array())
        .map(|cards| extract_cards(cards))
        .unwrap_or_default();
    let for_upgrade = screen_state
        .and_then(|state| state.get("for_upgrade"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let for_transform = screen_state
        .and_then(|state| state.get("for_transform"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let for_purge = screen_state
        .and_then(|state| state.get("for_purge"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let num_cards = screen_state
        .and_then(|state| state.get("num_cards"))
        .and_then(|value| value.as_i64());

    GridFields {
        cards,
        selected_cards,
        for_upgrade,
        for_transform,
        for_purge,
        num_cards,
    }
}
