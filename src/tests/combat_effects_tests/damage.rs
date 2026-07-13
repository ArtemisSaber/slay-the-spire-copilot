use super::*;

#[test]
fn extract_first_integer_no_digits() {
    assert_eq!(extract_first_integer::<i16>("no numbers here"), None);
}

#[test]
fn extract_first_integer_with_digits() {
    assert_eq!(extract_first_integer::<i16>("deal 42 damage"), Some(42));
}

#[test]
fn parse_hits_suffix_en_2_times() {
    assert_eq!(parse_hits_suffix("2 times"), Some(HitCount::Fixed(2)));
}

#[test]
fn parse_hits_suffix_en_twice() {
    assert_eq!(
        parse_hits_suffix("deal damage twice"),
        Some(HitCount::Fixed(2))
    );
}

#[test]
fn parse_hits_suffix_en_x_plus() {
    assert_eq!(parse_hits_suffix("X+2"), Some(HitCount::XPlus(2)));
}

#[test]
fn parse_hits_suffix_en_x_plus_space() {
    assert_eq!(parse_hits_suffix("X + 3"), Some(HitCount::XPlus(3)));
}

#[test]
fn parse_hits_suffix_en_x_times() {
    assert_eq!(parse_hits_suffix("X times"), Some(HitCount::XTimes));
}

#[test]
fn parse_hits_suffix_en_lowercase_x_times() {
    assert_eq!(parse_hits_suffix("x times"), Some(HitCount::XTimes));
}

#[test]
fn parse_hits_suffix_en_fixed_times() {
    assert_eq!(parse_hits_suffix("3 times"), Some(HitCount::Fixed(3)));
}

#[test]
fn parse_hits_suffix_x_plus_no_digits() {
    assert_eq!(parse_hits_suffix("X+abc"), None);
}

#[test]
fn parse_hits_suffix_times_no_digit_before() {
    assert_eq!(parse_hits_suffix(" times "), None);
}

#[test]
fn parse_hits_suffix_no_pattern() {
    assert_eq!(parse_hits_suffix("just some text"), None);
}

#[test]
fn parse_damage_multiplier_hits_invalid_desc() {
    assert_eq!(parse_damage_multiplier_hits(""), None);
    assert_eq!(parse_damage_multiplier_hits("  "), None);
}

#[test]
fn parse_damage_multiplier_hits_zero_damage() {
    assert_eq!(parse_damage_multiplier_hits("Deal 0 damage."), None);
}

#[test]
fn parse_damage_multiplier_hits_overflow_damage() {
    assert_eq!(parse_damage_multiplier_hits("Deal 1001 damage."), None);
}

#[test]
fn parse_damage_multiplier_hits_en_strike() {
    assert_eq!(
        parse_damage_multiplier_hits("Deal 6 damage."),
        Some((6, HitCount::Fixed(1)))
    );
}

#[test]
fn parse_damage_multiplier_hits_en_multi_hit() {
    assert_eq!(
        parse_damage_multiplier_hits("Deal 3 damage 4 times. Exhaust."),
        Some((3, HitCount::Fixed(4)))
    );
}

#[test]
fn parse_target_type_en_aoe() {
    assert_eq!(
        parse_target_type("deal damage to all enemies", &en_locale()),
        TargetType::AoE
    );
}

#[test]
fn parse_target_type_en_aoe_caps() {
    assert_eq!(
        parse_target_type("deal damage to ALL enemies", &en_locale()),
        TargetType::AoE
    );
}

#[test]
fn parse_target_type_en_random() {
    assert_eq!(
        parse_target_type("deal damage to a random enemy", &en_locale()),
        TargetType::RandomTarget
    );
}

#[test]
fn parse_target_type_en_targeted() {
    assert_eq!(
        parse_target_type("deal damage", &en_locale()),
        TargetType::Targeted
    );
}

#[test]
fn parse_target_type_ja_aoe() {
    assert_eq!(
        parse_target_type("すべての敵にダメージを与える", &ja_locale()),
        TargetType::AoE
    );
}

#[test]
fn parse_target_type_ko_random() {
    assert_eq!(
        parse_target_type("무작위 적에게 피해를 줍니다", &ko_locale()),
        TargetType::RandomTarget
    );
}

#[test]
fn parse_damage_non_attack() {
    assert_eq!(parse_damage("Deal 6 damage", "SKILL", &en_locale()), None);
    assert_eq!(parse_damage("Deal 6 damage", "POWER", &en_locale()), None);
    assert_eq!(parse_damage("Deal 6 damage", "CURSE", &en_locale()), None);
}

#[test]
fn parse_damage_non_attack_zh() {
    assert_eq!(parse_damage("造成 6 点伤害。", "SKILL", &zh_locale()), None);
}
