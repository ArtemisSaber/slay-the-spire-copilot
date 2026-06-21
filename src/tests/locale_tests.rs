use super::*;

#[test]
fn uppercase_game_language_codes_map_correctly() {
    assert_eq!(lang_to_locale_key("ZHS"), "zh");
    assert_eq!(lang_to_locale_key("ZHT"), "zh");
    assert_eq!(lang_to_locale_key("SCHINESE"), "zh");
    assert_eq!(lang_to_locale_key("TCHINESE"), "zh");
    assert_eq!(lang_to_locale_key("JPN"), "ja");
    assert_eq!(lang_to_locale_key("JAPANESE"), "ja");
    assert_eq!(lang_to_locale_key("KOR"), "ko");
    assert_eq!(lang_to_locale_key("KOREAN"), "ko");
    assert_eq!(lang_to_locale_key("KOREANA"), "ko");
}

#[test]
fn mixed_case_language_codes_map_correctly() {
    assert_eq!(lang_to_locale_key("Zhs"), "zh");
    assert_eq!(lang_to_locale_key("Jp"), "ja");
    assert_eq!(lang_to_locale_key("Korean"), "ko");
}

#[test]
fn english_and_unknown_languages_map_to_en() {
    assert_eq!(lang_to_locale_key("ENG"), "en");
    assert_eq!(lang_to_locale_key("english"), "en");
    assert_eq!(lang_to_locale_key("french"), "en");
    assert_eq!(lang_to_locale_key("deu"), "en");
    assert_eq!(lang_to_locale_key("rus"), "en");
    assert_eq!(lang_to_locale_key(""), "en");
}

#[test]
fn lowercase_language_codes_still_work() {
    assert_eq!(lang_to_locale_key("zhs"), "zh");
    assert_eq!(lang_to_locale_key("schinese"), "zh");
    assert_eq!(lang_to_locale_key("japanese"), "ja");
    assert_eq!(lang_to_locale_key("korean"), "ko");
}

#[test]
fn whitespace_is_trimmed() {
    assert_eq!(lang_to_locale_key(" ZHS "), "zh");
    assert_eq!(lang_to_locale_key("  english  "), "en");
}
