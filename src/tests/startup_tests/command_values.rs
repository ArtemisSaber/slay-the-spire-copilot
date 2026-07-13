use super::*;

#[test]
fn command_parser_unquoted_path() {
    assert_eq!(
        parse_command_value(r"C:\Users\Howard Lee\bin\slay-the-spire-copilot.exe"),
        Some(r"C:\Users\Howard Lee\bin\slay-the-spire-copilot.exe".to_string())
    );
}

#[test]
fn command_parser_preserves_trailing_args_for_config_check() {
    assert_eq!(
        parse_command_value(
            r"C:\Program Files\Slay Copilot\slay-the-spire-copilot.exe --stdin-test"
        ),
        Some(r"C:\Program Files\Slay Copilot\slay-the-spire-copilot.exe --stdin-test".to_string())
    );
}

#[test]
fn command_parser_values_with_args_not_fully_quoted_are_passed_through() {
    let result = parse_command_value(
        r#""C:\Program Files\Slay Copilot\slay-the-spire-copilot.exe" --stdin-test"#,
    );
    assert!(result.is_some_and(|v| v.contains("stdin-test")));
}

#[test]
fn command_parser_strips_single_quotes() {
    assert_eq!(
        parse_command_value(r"'/usr/bin/copilot'"),
        Some("/usr/bin/copilot".to_string())
    );
}

#[test]
fn command_parser_unescapes_java_properties_path() {
    assert_eq!(
        parse_command_value(
            r#""G\:\\Barracuda\\Game files\\Slay the Spire\\slay the spire copilot\\slay-the-spire-copilot.exe""#
        ),
        Some(
            r"G:\Barracuda\Game files\Slay the Spire\slay the spire copilot\slay-the-spire-copilot.exe"
                .to_string()
        )
    );
}

#[test]
fn command_parser_unescapes_unquoted_java_properties_path() {
    assert_eq!(
        parse_command_value(
            r"G\:\\Barracuda\\Game files\\Slay the Spire\\slay-the-spire-copilot.exe"
        ),
        Some(r"G:\Barracuda\Game files\Slay the Spire\slay-the-spire-copilot.exe".to_string())
    );
}

#[test]
fn command_parser_empty_returns_none() {
    assert_eq!(parse_command_value(""), None);
    assert_eq!(parse_command_value("  "), None);
    assert_eq!(parse_command_value("\"\""), None);
}

#[test]
fn format_command_value_doubles_backslashes_for_properties() {
    assert_eq!(format_command_value("/usr/bin/copilot"), "/usr/bin/copilot");
}

#[test]
fn format_command_value_doubles_windows_path_backslashes() {
    assert_eq!(
        format_command_value(r"G:\foo\bar\copilot.exe"),
        r"G:\\foo\\bar\\copilot.exe"
    );
}

#[test]
fn format_command_value_quotes_paths_with_spaces_and_doubles_backslashes() {
    assert_eq!(
        format_command_value(r"G:\Program Files\Copilot\copilot.exe"),
        r#""G:\\Program Files\\Copilot\\copilot.exe""#
    );
}

#[test]
fn format_command_value_preserves_already_quoted_value() {
    assert_eq!(
        format_command_value(r#""G:\foo\bar.exe""#),
        r#""G:\\foo\\bar.exe""#
    );
}

#[test]
fn parse_command_value_unescapes_colon_and_equals() {
    assert_eq!(
        parse_command_value(r"path\\:with\\:colons"),
        Some("path:with:colons".to_string())
    );
    assert_eq!(
        parse_command_value(r"key\\=value"),
        Some("key=value".to_string())
    );
}

#[test]
fn parse_command_value_handles_mixed_escapes() {
    assert_eq!(
        parse_command_value(r"C\:\\foo\\:bar\\=baz.exe"),
        Some(r"C:\foo:bar=baz.exe".to_string())
    );
}

#[test]
fn parse_command_value_empty_after_unescape_returns_none() {
    assert_eq!(parse_command_value("\"   \""), None);
    assert_eq!(parse_command_value("'   '"), None);
}

#[test]
fn format_command_value_no_whitespace_no_quotes() {
    assert_eq!(format_command_value("/simple/path"), "/simple/path");
}

#[test]
fn format_command_value_already_double_quoted() {
    let result = format_command_value("\"/usr/bin/copilot\"");
    assert_eq!(result, "\"/usr/bin/copilot\"");
}

#[test]
fn format_command_value_single_quote_not_rewrapped() {
    let result = format_command_value("'/usr/bin/copilot'");
    assert_eq!(result, "'/usr/bin/copilot'");
}
