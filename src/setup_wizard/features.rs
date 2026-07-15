use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use super::prompts::prompt_yes_no;
use super::types::{FeatureSetup, LearningSetup, parse_env_bool};

pub(crate) fn prompt_feature_setup(
    input: &mut impl BufRead,
    output: &mut impl Write,
    existing: &HashMap<String, String>,
) -> io::Result<FeatureSetup> {
    writeln!(output)?;
    writeln!(output, "Auto-play and learning")?;
    writeln!(
        output,
        "Learning stores compact local experience from completed normal runs."
    )?;

    let auto_default =
        parse_env_bool(existing.get("AUTO_PLAY").map(String::as_str)).unwrap_or(false);
    let auto_play = prompt_yes_no(input, output, "Enable automatic play?", auto_default)?;
    if !auto_play {
        writeln!(
            output,
            "Learning is off because it records decisions made during auto-play."
        )?;
        return Ok(FeatureSetup::new(false, LearningSetup::Off));
    }

    let existing_learning =
        LearningSetup::from_env(existing.get("MEMORY_MODE").map(String::as_str));
    let learn_default = existing_learning
        .map(|mode| mode != LearningSetup::Off)
        .unwrap_or(true);
    let learn = prompt_yes_no(
        input,
        output,
        "Learn locally from completed normal auto-play runs?",
        learn_default,
    )?;
    if !learn {
        return Ok(FeatureSetup::new(true, LearningSetup::Off));
    }

    let use_default = existing_learning == Some(LearningSetup::On);
    let use_memory = prompt_yes_no(
        input,
        output,
        "Use past experience to influence future auto-play now?",
        use_default,
    )?;
    let learning = if use_memory {
        LearningSetup::On
    } else if existing_learning == Some(LearningSetup::Shadow) {
        LearningSetup::Shadow
    } else {
        LearningSetup::Collect
    };
    Ok(FeatureSetup::new(true, learning))
}

pub(crate) fn write_feature_summary(
    output: &mut impl Write,
    setup: FeatureSetup,
) -> io::Result<()> {
    let auto = if setup.auto_play {
        "enabled"
    } else {
        "disabled"
    };
    writeln!(output, "Auto-play: {auto}")?;
    let detail = match setup.learning {
        LearningSetup::Off => "off",
        LearningSetup::Collect => "collect - saves eligible runs; does not influence play",
        LearningSetup::Shadow => "shadow - evaluates retrieval; does not influence play",
        LearningSetup::On => "on - saves runs and supplies relevant past experience",
    };
    writeln!(output, "Learning: {detail}")
}
