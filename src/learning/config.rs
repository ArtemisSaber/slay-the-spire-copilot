use serde::{Deserialize, Serialize};

const DEFAULT_MAX_ITEMS: usize = 3;
const DEFAULT_MAX_CONTEXT_BYTES: usize = 2_048;
const DEFAULT_CASE_MIN_SIMILARITY: u16 = 800;
const DEFAULT_LESSON_MIN_SIMILARITY: u16 = 700;
const DEFAULT_PROPOSED_MIN_SIMILARITY: u16 = 850;
const DEFAULT_MAX_CASES_PER_RUN: usize = 500;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryMode {
    #[default]
    Off,
    Collect,
    Shadow,
    On,
}

impl MemoryMode {
    fn parse(value: Option<String>) -> Self {
        match value.as_deref().map(str::trim).map(str::to_ascii_lowercase) {
            Some(value) if value == "collect" => Self::Collect,
            Some(value) if value == "shadow" => Self::Shadow,
            Some(value) if value == "on" => Self::On,
            _ => Self::Off,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryConfig {
    pub mode: MemoryMode,
    pub max_items: usize,
    pub max_context_bytes: usize,
    pub case_min_similarity: u16,
    pub lesson_min_similarity: u16,
    pub proposed_lesson_min_similarity: u16,
    pub max_cases_per_run: usize,
    pub mod_profile_sha256: Option<String>,
    pub mod_profile_approved: bool,
    pub debug_card_ids: Vec<String>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            mode: MemoryMode::Off,
            max_items: DEFAULT_MAX_ITEMS,
            max_context_bytes: DEFAULT_MAX_CONTEXT_BYTES,
            case_min_similarity: DEFAULT_CASE_MIN_SIMILARITY,
            lesson_min_similarity: DEFAULT_LESSON_MIN_SIMILARITY,
            proposed_lesson_min_similarity: DEFAULT_PROPOSED_MIN_SIMILARITY,
            max_cases_per_run: DEFAULT_MAX_CASES_PER_RUN,
            mod_profile_sha256: None,
            mod_profile_approved: false,
            debug_card_ids: vec![],
        }
    }
}

impl MemoryConfig {
    pub fn from_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            mode: MemoryMode::parse(lookup("MEMORY_MODE")),
            max_items: bounded(&mut lookup, "MEMORY_MAX_ITEMS", 1, 10, DEFAULT_MAX_ITEMS),
            max_context_bytes: bounded(
                &mut lookup,
                "MEMORY_MAX_CONTEXT_BYTES",
                256,
                16_384,
                DEFAULT_MAX_CONTEXT_BYTES,
            ),
            case_min_similarity: bounded(
                &mut lookup,
                "MEMORY_CASE_MIN_SIMILARITY",
                0,
                1_000,
                DEFAULT_CASE_MIN_SIMILARITY,
            ),
            lesson_min_similarity: bounded(
                &mut lookup,
                "MEMORY_LESSON_MIN_SIMILARITY",
                0,
                1_000,
                DEFAULT_LESSON_MIN_SIMILARITY,
            ),
            proposed_lesson_min_similarity: bounded(
                &mut lookup,
                "MEMORY_PROPOSED_LESSON_MIN_SIMILARITY",
                0,
                1_000,
                DEFAULT_PROPOSED_MIN_SIMILARITY,
            ),
            max_cases_per_run: bounded(
                &mut lookup,
                "MEMORY_MAX_CASES_PER_RUN",
                1,
                5_000,
                DEFAULT_MAX_CASES_PER_RUN,
            ),
            mod_profile_sha256: non_empty(lookup("MEMORY_MOD_PROFILE_SHA256")),
            mod_profile_approved: truthy(lookup("MEMORY_MOD_PROFILE_APPROVED")),
            debug_card_ids: comma_list(lookup("MEMORY_DEBUG_CARD_IDS")),
        }
    }

    pub fn captures(&self) -> bool {
        self.mode != MemoryMode::Off
    }

    pub fn retrieves(&self) -> bool {
        matches!(self.mode, MemoryMode::Shadow | MemoryMode::On)
    }

    pub fn injects(&self) -> bool {
        self.mode == MemoryMode::On
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && value.len() <= 256)
}

fn truthy(value: Option<String>) -> bool {
    value.is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

fn comma_list(value: Option<String>) -> Vec<String> {
    let mut values: Vec<_> = value
        .iter()
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.len() <= 256)
        .map(str::to_string)
        .collect();
    values.sort();
    values.dedup();
    values
}

fn bounded<T>(
    lookup: &mut impl FnMut(&str) -> Option<String>,
    key: &str,
    min: T,
    max: T,
    default: T,
) -> T
where
    T: Copy + PartialOrd + std::str::FromStr,
{
    lookup(key)
        .and_then(|value| value.trim().parse().ok())
        .filter(|value| *value >= min && *value <= max)
        .unwrap_or(default)
}
