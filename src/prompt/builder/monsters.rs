use std::collections::HashMap;
use std::sync::OnceLock;

use crate::locales::Locale;
use crate::state::{MonsterInfo, NormalizedState};

use super::cards::format_card;

type PowerDescMap = HashMap<String, HashMap<String, String>>;

fn power_descs() -> &'static PowerDescMap {
    static DESCRIPTIONS: OnceLock<PowerDescMap> = OnceLock::new();
    DESCRIPTIONS.get_or_init(|| {
        serde_json::from_str(include_str!("../../i18n/powers.json")).unwrap_or_default()
    })
}

fn power_desc_line(id: &str, amount: i64, locale: &Locale) -> Option<String> {
    let entry = power_descs().get(id)?;
    let template = entry.get(locale.lang_code.as_str())?;
    Some(format!(
        "[{}]",
        template.replace("{amount}", &amount.to_string())
    ))
}

pub(crate) fn format_monster(monster: &MonsterInfo, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![format!("[{}] {}", monster.index, monster.name)];

    if let (Some(current_hp), Some(max_hp)) = (monster.current_hp, monster.max_hp) {
        let mut hp_line = locale
            .monster
            .hp_line
            .replace("{cur}", &current_hp.to_string())
            .replace("{max}", &max_hp.to_string());
        if monster.can_be_killed {
            hp_line.push_str(&locale.monster.killable);
        }
        lines.push(hp_line);
    }

    if let Some(intent) = &monster.intent {
        let intent_name = match intent.as_str() {
            "ATTACK" => &locale.i18n.intent_attack,
            "ATTACK_BUFF" => &locale.i18n.intent_attack_buff,
            "ATTACK_DEBUFF" => &locale.i18n.intent_attack_debuff,
            "ATTACK_DEFEND" => &locale.i18n.intent_attack_defend,
            "BUFF" => &locale.i18n.intent_buff,
            "DEBUFF" => &locale.i18n.intent_debuff,
            "STRONG_DEBUFF" => &locale.i18n.intent_strong_debuff,
            "DEBUG" => &locale.i18n.intent_debug,
            "DEFEND" => &locale.i18n.intent_defend,
            "DEFEND_DEBUFF" => &locale.i18n.intent_defend_debuff,
            "DEFEND_BUFF" => &locale.i18n.intent_defend_buff,
            "ESCAPE" => &locale.i18n.intent_escape,
            "MAGIC" => &locale.i18n.intent_magic,
            "NONE" => &locale.i18n.intent_none,
            "SLEEP" => &locale.i18n.intent_sleep,
            "STUN" => &locale.i18n.intent_stun,
            "UNKNOWN" => &locale.i18n.intent_unknown,
            _ => intent.as_str(),
        };
        lines.push(locale.monster.intent.replace("{intent}", intent_name));
    }

    if let Some(damage) = monster.damage {
        let mut damage_line = locale.monster.damage.replace("{dmg}", &damage.to_string());
        if let Some(hits) = monster.hits
            && hits > 1
        {
            damage_line.push_str(
                &locale
                    .monster
                    .multi_hit
                    .replace("{hits}", &hits.to_string()),
            );
        }
        lines.push(damage_line);
    } else {
        lines.push(locale.monster.no_damage.clone());
    }

    if let Some(block) = monster.block
        && block > 0
    {
        lines.push(locale.monster.block.replace("{blk}", &block.to_string()));
    }

    if !monster.monster_powers.is_empty() {
        let powers: Vec<String> = monster
            .monster_powers
            .iter()
            .map(|power| {
                let base = format!("{}({})", power.name, power.amount);
                if let Some(description) = power_desc_line(&power.id, power.amount, locale) {
                    format!("{base}{description}")
                } else {
                    base
                }
            })
            .collect();
        let mut line = locale.monster.powers.replace("{list}", &powers.join(" "));
        if monster.is_scaling {
            line.push_str(&locale.monster.scaling);
        }
        lines.push(line);
    }

    lines.join("\n")
}

pub(crate) fn build_monsters_section(state: &NormalizedState, locale: &Locale) -> String {
    if state.monsters.is_empty() {
        return String::new();
    }

    let mut lines = vec![
        locale
            .monster
            .section_header
            .replace("{count}", &state.monsters.len().to_string()),
    ];
    for monster in &state.monsters {
        lines.push(format_monster(monster, locale));
        lines.push(String::new());
    }
    lines.join("\n")
}

pub(crate) fn build_hand_section(state: &NormalizedState, locale: &Locale) -> String {
    if state.hand_cards.is_empty() {
        return String::new();
    }

    let mut lines = vec![
        locale
            .card
            .hand_header
            .replace("{count}", &state.hand_cards.len().to_string()),
    ];
    for card in &state.hand_cards {
        lines.push(format!("  {}", format_card(card, locale)));
    }
    lines.push(String::new());
    lines.join("\n")
}
