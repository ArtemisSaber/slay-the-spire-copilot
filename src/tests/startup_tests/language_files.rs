use super::*;

#[test]
fn gameplay_settings_language_is_read_from_sts_preferences() {
    let (_dir, settings) = temp_config(
        r#"{
          "LANGUAGE": "ZHS",
          "Fast Mode": "true"
        }"#,
    );

    assert_eq!(
        read_gameplay_settings_language(&settings),
        Some("ZHS".to_string())
    );
}

#[test]
fn read_gameplay_settings_language_missing_field() {
    let (_dir, path) = temp_config(r#"{"Fast Mode": "true"}"#);
    assert_eq!(read_gameplay_settings_language(&path), None);
}

#[test]
fn read_gameplay_settings_language_empty_value() {
    let (_dir, path) = temp_config(r#"{"LANGUAGE": ""}"#);
    assert_eq!(read_gameplay_settings_language(&path), None);
}

#[test]
fn read_gameplay_settings_language_invalid_json() {
    let (_dir, path) = temp_config("not valid json at all");
    assert_eq!(read_gameplay_settings_language(&path), None);
}

#[test]
fn read_gameplay_settings_language_nonexistent_file() {
    let path = PathBuf::from("/tmp/nonexistent-sts-settings-99999.json");
    assert_eq!(read_gameplay_settings_language(&path), None);
}

#[test]
fn read_gameplay_settings_language_with_whitespace() {
    let (_dir, path) = temp_config(r#"{"LANGUAGE": "  schinese  "}"#);
    assert_eq!(
        read_gameplay_settings_language(&path),
        Some("schinese".to_string())
    );
}

#[test]
fn extract_vdf_value_from_typical_content() {
    let content = "\"AppState\"\n{\n\t\"appid\"\t\t\"646570\"\n\t\"language\"\t\t\"schinese\"\n\t\"name\"\t\t\"Slay the Spire\"\n}";
    assert_eq!(
        extract_vdf_value(content, "language"),
        Some("schinese".to_string())
    );
    assert_eq!(
        extract_vdf_value(content, "appid"),
        Some("646570".to_string())
    );
    assert_eq!(
        extract_vdf_value(content, "name"),
        Some("Slay the Spire".to_string())
    );
}

#[test]
fn extract_vdf_value_empty_content() {
    assert_eq!(extract_vdf_value("", "key"), None);
}

#[test]
fn extract_vdf_value_key_not_found() {
    assert_eq!(
        extract_vdf_value("\"other\"\t\t\"value\"", "language"),
        None
    );
}

#[test]
fn extract_vdf_value_no_value_after_key() {
    assert_eq!(extract_vdf_value("\"language\"", "language"), None);
}

#[test]
fn extract_vdf_value_case_insensitive_key() {
    let content = "\"Language\"\t\t\"schinese\"";
    assert_eq!(
        extract_vdf_value(content, "language"),
        Some("schinese".to_string())
    );
    assert_eq!(
        extract_vdf_value(content, "LANGUAGE"),
        Some("schinese".to_string())
    );
    assert_eq!(
        extract_vdf_value(content, "Language"),
        Some("schinese".to_string())
    );
}

#[test]
fn extract_vdf_value_trims_value_whitespace() {
    let content = "  \"appid\"  \t  \"  646570  \"  ";
    assert_eq!(
        extract_vdf_value(content, "appid"),
        Some("646570".to_string())
    );
}

#[test]
fn extract_vdf_value_multiline_finds_correct_key() {
    let content =
        "\"appid\"\t\t\"646570\"\n\"language\"\t\t\"english\"\n\"installdir\"\t\t\"SlayTheSpire\"";
    assert_eq!(
        extract_vdf_value(content, "language"),
        Some("english".to_string())
    );
    assert_eq!(
        extract_vdf_value(content, "installdir"),
        Some("SlayTheSpire".to_string())
    );
}

#[test]
fn extract_vdf_value_skips_lines_without_parts() {
    let content = "{\n\t\"language\"\t\t\"schinese\"\n}";
    assert_eq!(
        extract_vdf_value(content, "language"),
        Some("schinese".to_string())
    );
}

#[test]
fn read_steam_appmanifest_language_from_sample() {
    let (_dir, path) =
        temp_config("\"AppState\"\n{\n\t\"appid\"\t\t\"646570\"\n\t\"language\"\t\t\"english\"\n}");
    assert_eq!(
        read_steam_appmanifest_language(&path),
        Some("english".to_string())
    );
}

#[test]
fn read_steam_appmanifest_language_empty_value() {
    let (_dir, path) = temp_config("\"language\"\t\t\"\"");
    assert_eq!(read_steam_appmanifest_language(&path), None);
}

#[test]
fn read_steam_appmanifest_language_nonexistent_file() {
    let path = PathBuf::from("/tmp/nonexistent-appmanifest-99999.acf");
    assert_eq!(read_steam_appmanifest_language(&path), None);
}

#[test]
fn read_steam_appmanifest_language_no_language_key() {
    let (_dir, path) = temp_config("\"appid\"\t\t\"646570\"\n\"name\"\t\t\"Slay the Spire\"");
    assert_eq!(read_steam_appmanifest_language(&path), None);
}

#[test]
fn read_steam_appmanifest_language_whitespace_only_value() {
    let (_dir, path) = temp_config("\"language\"\t\t\"   \"");
    assert_eq!(read_steam_appmanifest_language(&path), None);
}
