use crate::locales::Locale;
use crate::state::NormalizedState;

use super::cards::{clean_description, compact_pile};
use super::monsters::{build_hand_section, build_monsters_section};
use super::status::{combat_profile_line, turn_status_line};

pub(crate) fn build_combat(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec!["[mode: combat]\n".to_string()];
    lines.push(locale.combat_types.header.clone());

    let room = state.room_type.as_ref().map(|room_type| room_type.as_str());
    let type_name = match room {
        Some("MonsterRoomElite") => &locale.combat_types.type_elite,
        Some("MonsterRoomBoss") => &locale.combat_types.type_boss,
        _ => &locale.combat_types.type_normal,
    };
    lines.push(locale.combat_types.type_line.replace("{type}", type_name));

    let goal = match room {
        Some("MonsterRoomElite") => &locale.combat_types.goal_elite,
        Some("MonsterRoomBoss") => &locale.combat_types.goal_boss,
        _ => &locale.combat_types.goal_normal,
    };
    lines.push(locale.combat_types.primary.replace("{goal}", goal));

    let sub = match room {
        Some("MonsterRoomElite") => &locale.combat_types.sub_elite,
        Some("MonsterRoomBoss") => &locale.combat_types.sub_boss,
        _ => &locale.combat_types.sub_normal,
    };
    lines.push(locale.combat_types.secondary.replace("{sub}", sub));

    let trade = match room {
        Some("MonsterRoomElite") => &locale.combat_types.trade_elite,
        Some("MonsterRoomBoss") => &locale.combat_types.trade_boss,
        _ => &locale.combat_types.trade_normal,
    };
    lines.push(locale.combat_types.trade.replace("{advice}", trade));

    let max_hp = state.max_hp.unwrap_or(75);
    let power = match room {
        Some("MonsterRoomElite") => &locale.combat_types.power_elite,
        Some("MonsterRoomBoss") => &locale.combat_types.power_boss,
        _ if state.incoming_damage > max_hp / 5 => &locale.combat_types.power_normal_high,
        _ => &locale.combat_types.power_normal_low,
    };
    lines.push(locale.combat_types.power_play.replace("{advice}", power));

    let priority: Vec<String> = state
        .monsters
        .iter()
        .map(|monster| {
            if monster.is_scaling {
                locale
                    .combat_types
                    .prio_scaling
                    .replace("{name}", &monster.name)
            } else if monster.monster_powers.iter().any(|power| {
                matches!(
                    power.id.as_str(),
                    "Enrage" | "Thorns" | "Curiosity" | "Anger"
                )
            }) {
                locale
                    .combat_types
                    .prio_punish
                    .replace("{name}", &monster.name)
            } else if monster.can_be_killed {
                locale
                    .combat_types
                    .prio_killable
                    .replace("{name}", &monster.name)
            } else {
                locale
                    .combat_types
                    .prio_default
                    .replace("{name}", &monster.name)
            }
        })
        .collect();
    lines.push(
        locale
            .combat_types
            .priority
            .replace("{list}", &priority.join("；")),
    );
    lines.push(String::new());

    lines.push(locale.sections.combat_profile.clone());
    lines.push(combat_profile_line(state, locale));
    lines.push(String::new());
    lines.push(locale.sections.turn_status.clone());
    lines.push(turn_status_line(state, locale));
    lines.push(String::new());

    if !state.potions.is_empty() {
        lines.push(locale.sections.potions.clone());
        for potion in &state.potions {
            lines.push(format!(
                "{}：{}",
                potion.name,
                clean_description(&potion.description, locale)
            ));
        }
        lines.push(String::new());
    }

    if state.danger.wrath_stance {
        lines.push(locale.warnings.wrath_stance.clone());
    }
    if !state.monsters.is_empty() {
        lines.push(build_monsters_section(state, locale));
    }
    if !state.hand_cards.is_empty() {
        lines.push(build_hand_section(state, locale));
    }
    lines.push(compact_pile(
        &locale.card.pile_draw,
        &state.draw_pile,
        locale,
    ));
    lines.push(compact_pile(
        &locale.card.pile_discard,
        &state.discard_pile,
        locale,
    ));
    if !state.exhaust_cards.is_empty() {
        lines.push(compact_pile(
            &locale.card.pile_exhaust,
            &state.exhaust_cards,
            locale,
        ));
    }

    lines.join("\n")
}
