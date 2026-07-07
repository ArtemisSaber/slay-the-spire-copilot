/// Extracts the first contiguous run of ASCII digits from `s` and parses
/// it as `T`. Returns `None` if no digits are found or parsing fails (e.g.
/// overflow). Parse failures are logged via `tracing::warn!`.
pub(crate) fn extract_first_integer<T: std::str::FromStr>(s: &str) -> Option<T>
where
    T::Err: std::fmt::Display,
{
    let digits: String = s
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return None;
    }
    match digits.parse() {
        Ok(val) => Some(val),
        Err(e) => {
            tracing::warn!("integer parse failed in '{s}': {e}");
            None
        }
    }
}

#[cfg(test)]
#[path = "tests/parsing_tests.rs"]
mod tests;
