use super::*;

#[test]
fn extract_first_integer_returns_none_for_no_digits() {
    assert_eq!(extract_first_integer::<i64>("no digits here"), None);
}

#[test]
fn extract_first_integer_returns_value_for_digits() {
    assert_eq!(extract_first_integer::<i64>("deal 10 damage"), Some(10));
}

#[test]
fn extract_first_integer_returns_first_integer_in_string() {
    assert_eq!(extract_first_integer::<i64>("first 3 then 7"), Some(3));
}

#[test]
fn extract_first_integer_works_with_i16() {
    assert_eq!(extract_first_integer::<i16>("cost 5 energy"), Some(5));
}

#[test]
fn extract_first_integer_returns_none_for_empty() {
    assert_eq!(extract_first_integer::<i64>(""), None);
}

#[test]
fn extract_first_integer_extracts_digits_not_minus_sign() {
    assert_eq!(extract_first_integer::<i64>("-5"), Some(5));
}

#[test]
fn extract_first_integer_handles_chinese_text() {
    assert_eq!(extract_first_integer::<i64>("造成 12 点伤害"), Some(12));
}
