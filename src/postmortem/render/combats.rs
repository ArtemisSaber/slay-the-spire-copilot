use super::super::aggregation::JournalSummary;
use crate::locales::PostmortemLocale;
use std::collections::HashMap;

pub(super) fn append_combat_history(
    report: &mut Vec<String>,
    summary: &JournalSummary,
    pm: &PostmortemLocale,
) {
    if summary.combat_records.is_empty() {
        return;
    }

    report.push(String::new());
    report.push(pm.section_monsters.clone());
    report.push(
        pm.label_combats_summary
            .replace("{n}", &summary.combat_records.len().to_string())
            .replace("{list}", &summary.global_monster_names.join(", ")),
    );
    report.push(
        pm.label_combat_type_count
            .replace("{normal}", &summary.normal_count.to_string())
            .replace("{elite}", &summary.elite_count.to_string())
            .replace("{boss}", &summary.boss_count.to_string()),
    );

    for (index, combat) in summary.combat_records.iter().enumerate() {
        if let (Some(start), Some(end)) = (combat.start_hp, combat.end_hp) {
            let delta = end - start;
            let hp_change = if delta >= 0 {
                format!("+{delta}")
            } else {
                delta.to_string()
            };
            let label = match combat.room_type.as_str() {
                "MonsterRoomElite" => &pm.label_combat_elite,
                "MonsterRoomBoss" => &pm.label_combat_boss,
                _ => &pm.label_combat_hp,
            };
            report.push(
                (if combat.is_fatal {
                    format!("💀 {label}")
                } else {
                    label.clone()
                })
                .replace("{n}", &(index + 1).to_string())
                .replace("{start}", &start.to_string())
                .replace("{end}", &end.to_string())
                .replace("{delta}", &hp_change)
                .replace("{monsters}", &monster_list(&combat.monsters)),
            );
        }
    }
}

fn monster_list(monsters: &[String]) -> String {
    if monsters.is_empty() {
        return String::new();
    }

    let mut counts: HashMap<&str, usize> = HashMap::new();
    for monster in monsters {
        *counts.entry(monster.as_str()).or_insert(0) += 1;
    }
    let mut entries: Vec<String> = counts
        .into_iter()
        .map(|(name, count)| {
            if count > 1 {
                format!("{name}×{count}")
            } else {
                name.to_string()
            }
        })
        .collect();
    entries.sort();
    entries.join("、")
}
