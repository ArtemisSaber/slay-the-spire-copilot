#[cfg(test)]
mod tests {
    #[test]
    fn malformed_json_does_not_panic() {
        let inputs = vec!["", "{invalid", "not json at all", r#"{"in_game": true"#];

        for input in inputs {
            let result: Result<serde_json::Value, _> = serde_json::from_str(input);
            assert!(result.is_err(), "expected error for input: {input}");
        }
    }

    #[test]
    fn malformed_fixture_lines_do_not_panic() {
        let content = std::fs::read_to_string("tests/fixtures/malformed-input.txt").unwrap();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let result: Result<serde_json::Value, _> = serde_json::from_str(trimmed);
            assert!(
                result.is_err() || (result.is_ok() && trimmed == "null"),
                "line should either fail or be null: {trimmed}"
            );
        }
    }
}
