use serde_json::Value;

use crate::locales::Locale;

use super::parse::{
    extract_event_choice, first_array, first_raw_string, first_string, is_readable_text,
};

pub(super) struct EventFields {
    pub(super) id: Option<String>,
    pub(super) name: Option<String>,
    pub(super) body: Option<String>,
    pub(super) choices: Vec<String>,
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
        .and_then(|choices| aligned_screen_event_choices(choices, locale))
        .or_else(|| {
            game_state
                .and_then(|state| first_array(state, &["choice_list"]))
                .map(|choices| event_choice_texts(choices, locale))
        })
        .unwrap_or_default();

    EventFields {
        id,
        name,
        body,
        choices,
    }
}

fn aligned_screen_event_choices(choices: &[Value], locale: &Locale) -> Option<Vec<String>> {
    let mut enabled: Vec<_> = choices
        .iter()
        .filter(|choice| {
            !choice
                .get("disabled")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .map(|choice| {
            let choice_index = choice
                .get("choice_index")
                .and_then(Value::as_u64)
                .and_then(|index| usize::try_from(index).ok());
            (choice_index, choice)
        })
        .collect();

    if enabled.is_empty() {
        return None;
    }

    let indexed_count = enabled
        .iter()
        .filter(|(choice_index, _)| choice_index.is_some())
        .count();
    if indexed_count != 0 && indexed_count != enabled.len() {
        return None;
    }

    if indexed_count == enabled.len() {
        enabled.sort_by_key(|(choice_index, _)| *choice_index);
        if !enabled
            .iter()
            .enumerate()
            .all(|(expected, (actual, _))| *actual == Some(expected))
        {
            return None;
        }
    }

    Some(
        enabled
            .into_iter()
            .enumerate()
            .map(|(aligned_index, (_, choice))| {
                extract_event_choice(choice).unwrap_or_else(|| {
                    locale
                        .fallback
                        .event_unreadable_choice
                        .replace("{idx}", &(aligned_index + 1).to_string())
                })
            })
            .collect(),
    )
}

fn event_choice_texts(choices: &[Value], locale: &Locale) -> Vec<String> {
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
}
