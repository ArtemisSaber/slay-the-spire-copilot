use std::io::{self, BufRead, Write};

use super::review::ReviewableRun;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HomeAction {
    Review,
    Settings,
    Refresh,
    Exit,
}

pub(crate) fn prompt_home(
    input: &mut impl BufRead,
    output: &mut impl Write,
    status: &str,
) -> io::Result<HomeAction> {
    writeln!(output)?;
    writeln!(
        output,
        "Slay the Spire AI Copilot — Terminal control center"
    )?;
    writeln!(
        output,
        "CommunicationMod continues through standard input/output automatically."
    )?;
    writeln!(output)?;
    writeln!(output, "{status}")?;
    writeln!(output)?;
    writeln!(output, "  1) Review a completed run")?;
    writeln!(output, "  2) Settings")?;
    writeln!(output, "  3) Refresh status")?;
    writeln!(output, "  4) Exit")?;
    loop {
        let value = prompt_line(input, output, "Selection [1]")?;
        match value.to_ascii_lowercase().as_str() {
            "" | "1" | "review" => return Ok(HomeAction::Review),
            "2" | "settings" | "setup" => return Ok(HomeAction::Settings),
            "3" | "refresh" => return Ok(HomeAction::Refresh),
            "4" | "q" | "quit" | "exit" => return Ok(HomeAction::Exit),
            _ => writeln!(output, "Please choose 1, 2, 3, or 4.")?,
        }
    }
}

pub(crate) fn choose_run(
    input: &mut impl BufRead,
    output: &mut impl Write,
    runs: &[ReviewableRun],
) -> io::Result<Option<String>> {
    if runs.is_empty() {
        writeln!(
            output,
            "No completed runs with saved decision cases were found."
        )?;
        return Ok(None);
    }
    writeln!(output)?;
    writeln!(output, "Completed runs with saved experience:")?;
    for (index, run) in runs.iter().enumerate() {
        let case_label = if run.case_count == 1 { "case" } else { "cases" };
        let lesson_label = if run.lesson_count == 1 {
            "lesson"
        } else {
            "lessons"
        };
        writeln!(
            output,
            "  {}) {} — {} {}, {} {}",
            index + 1,
            run.run_id,
            run.case_count,
            case_label,
            run.lesson_count,
            lesson_label
        )?;
    }
    loop {
        let value = prompt_line(input, output, "Run [1, or b to go back]")?;
        if matches!(value.to_ascii_lowercase().as_str(), "b" | "back" | "q") {
            return Ok(None);
        }
        let index = if value.is_empty() {
            0
        } else if let Ok(number) = value.parse::<usize>() {
            number.checked_sub(1).unwrap_or(runs.len())
        } else {
            runs.len()
        };
        if let Some(run) = runs.get(index) {
            return Ok(Some(run.run_id.clone()));
        }
        writeln!(output, "Please choose a listed run or b.")?;
    }
}

pub(crate) fn confirm_review(
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> io::Result<bool> {
    loop {
        let value = prompt_line(
            input,
            output,
            "Use one LLM request to review this run? [Y/n]",
        )?;
        match value.to_ascii_lowercase().as_str() {
            "" | "y" | "yes" => return Ok(true),
            "n" | "no" | "q" | "cancel" => return Ok(false),
            _ => writeln!(output, "Please answer y or n.")?,
        }
    }
}

fn prompt_line(
    input: &mut impl BufRead,
    output: &mut impl Write,
    label: &str,
) -> io::Result<String> {
    write!(output, "{label}: ")?;
    output.flush()?;
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok("q".to_string());
    }
    Ok(line.trim().to_string())
}
