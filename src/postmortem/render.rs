mod combats;
mod inventory;
mod overview;

use super::aggregation::JournalSummary;
use crate::locales::PostmortemLocale;

pub(crate) fn render_report(summary: &JournalSummary, pm: &PostmortemLocale) -> String {
    let mut report = Vec::new();
    report.push(pm.report_title.clone());
    report.push(String::new());
    report.push(pm.run_section.clone());
    overview::append_run_details(&mut report, summary, pm);
    overview::append_overview(&mut report, summary, pm);
    inventory::append_inventory(&mut report, summary.final_state.as_ref(), pm);
    combats::append_combat_history(&mut report, summary, pm);

    if !summary.advice_lines.is_empty() {
        report.push(String::new());
        report.push(pm.section_decisions.clone());
        report.extend(summary.advice_lines.iter().cloned());
    }

    if !summary.reward_lines.is_empty() {
        report.push(String::new());
        report.push(pm.section_rewards.clone());
        report.extend(summary.reward_lines.iter().cloned());
    }

    report.join("\n")
}
