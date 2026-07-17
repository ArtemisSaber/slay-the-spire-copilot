mod menu;
mod review;

use std::io::{self, BufRead, Write};
use std::path::Path;

pub(crate) async fn run(project_root: &Path) {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    if let Err(error) = run_with_io(project_root, &mut input, &mut output).await {
        let _ = writeln!(output, "Terminal control center failed: {error}");
    }
}

async fn run_with_io(
    project_root: &Path,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> io::Result<()> {
    let env_path = crate::setup_wizard::env_path(project_root);
    if crate::setup_wizard::run_setup_with_io(&env_path, input, output, false)? {
        reload_env(&env_path);
    }
    loop {
        let status = learning_status(project_root);
        match menu::prompt_home(input, output, &status)? {
            menu::HomeAction::Review => review_interactively(project_root, input, output).await?,
            menu::HomeAction::Settings => {
                if crate::setup_wizard::run_setup_with_io(&env_path, input, output, true)? {
                    reload_env(&env_path);
                }
            }
            menu::HomeAction::Refresh => {}
            menu::HomeAction::Exit => {
                writeln!(output, "Goodbye.")?;
                return Ok(());
            }
        }
    }
}

async fn review_interactively(
    project_root: &Path,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> io::Result<()> {
    if !crate::config::Config::from_env().memory.captures() {
        writeln!(
            output,
            "Local learning is off. Enable it in Settings before reviewing runs."
        )?;
        return Ok(());
    }
    let runs = match review::list_reviewable_runs(project_root) {
        Ok(runs) => runs,
        Err(error) => {
            writeln!(output, "Could not load completed runs: {error}")?;
            return Ok(());
        }
    };
    let Some(run_id) = menu::choose_run(input, output, &runs)? else {
        return Ok(());
    };
    if !menu::confirm_review(input, output)? {
        writeln!(output, "Review cancelled.")?;
        return Ok(());
    }
    writeln!(output, "Reviewing {run_id}...")?;
    output.flush()?;
    match review::review_run(project_root, &run_id).await {
        Ok(result) if !result.response_valid => writeln!(
            output,
            "Lesson generation failed after {} API call(s). The fail-safe saved no lesson. Report: {}",
            result.api_calls,
            result.report_path.display()
        )?,
        Ok(result)
            if result.outcome == crate::learning::deliberation::DeliberationOutcome::ReportOnly =>
        {
            writeln!(
                output,
                "Review complete. The report was refreshed without requesting a duplicate lesson. Report: {}",
                result.report_path.display()
            )?
        }
        Ok(result) if result.accepted_lessons == 0 => writeln!(
            output,
            "Review complete. No lesson was approved after {} API call(s); the fail-safe saved none. Report: {}",
            result.api_calls,
            result.report_path.display()
        )?,
        Ok(result) => writeln!(
            output,
            "Review complete. Saved {} lesson(s); rejected {}. Report: {}",
            result.accepted_lessons,
            result.rejected_lessons,
            result.report_path.display()
        )?,
        Err(error) => writeln!(output, "Review failed: {error:#}")?,
    }
    Ok(())
}

fn learning_status(project_root: &Path) -> String {
    learning_status_with_config(project_root, &crate::config::Config::from_env())
        .unwrap_or_else(|error| format!("Learning status unavailable: {error:#}"))
}

fn learning_status_with_config(
    project_root: &Path,
    config: &crate::config::Config,
) -> anyhow::Result<String> {
    let bundle = crate::learning::bundle::embedded_bundle()
        .map_err(|error| anyhow::anyhow!("embedded knowledge bundle invalid: {error:?}"))?;
    let store = crate::learning::store::KnowledgeStore::new(
        project_root.join("learning").join("knowledge"),
    );
    let snapshot = match store.load_snapshot() {
        Ok(snapshot) => snapshot,
        Err(_) if config.memory.captures() => store.rebuild(std::slice::from_ref(&bundle))?,
        Err(_) => crate::learning::snapshot::KnowledgeSnapshot::build(vec![], &[bundle])
            .map_err(|error| anyhow::anyhow!("knowledge fallback failed: {error:?}"))?,
    };
    let last_run = store
        .read_last_run_eligibility()?
        .as_ref()
        .map(crate::learning::status::LastRunStatus::from);
    Ok(crate::learning::status::render_human(
        &crate::learning::status::LearningStatus {
            mode: config.memory.mode,
            case_count: snapshot.cases.len(),
            lesson_count: snapshot.lessons.len(),
            last_run,
        },
    ))
}

fn reload_env(path: &Path) {
    if let Err(error) = dotenvy::from_path_override(path) {
        tracing::warn!("failed to reload terminal settings: {error}");
    }
}

#[cfg(test)]
#[path = "tests/terminal_tests.rs"]
mod tests;
