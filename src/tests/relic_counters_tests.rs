use super::*;
use crate::test_utils::test_locale;

#[test]
fn uses_remaining_replaces_first_integer() {
    let locale = test_locale();
    let result = rewrite_relic_description(
        "NeowsBlessing",
        2,
        "接下来 3 场战斗中的敌人将只有 1 点生命。",
        locale,
    );
    assert_eq!(result, "接下来 2 场战斗中的敌人将只有 1 点生命。");
}

#[test]
fn uses_remaining_no_digits_returns_unchanged() {
    let locale = test_locale();
    let result = rewrite_relic_description(
        "Girya",
        2,
        "你现在能在休息处获得 力量 。（最多3次）",
        locale,
    );
    assert_eq!(result, "你现在能在休息处获得 力量 。（最多2次）");
}

#[test]
fn cyclic_computes_remaining_from_max_minus_counter() {
    let locale = test_locale();
    let result = rewrite_relic_description("Happy Flower", 2, "每 3 个回合，获得 。", locale);
    assert_eq!(result, "1 回合后获得 [E]");
}

#[test]
fn cyclic_at_max_returns_zero_remaining() {
    let locale = test_locale();
    let result = rewrite_relic_description(
        "Incense Burner",
        6,
        "每 6 回合，获得 1 层 无实体 。",
        locale,
    );
    assert_eq!(result, "0 回合后获得 1 层 无实体");
}

#[test]
fn unknown_relic_returns_original_description() {
    let locale = test_locale();
    let result =
        rewrite_relic_description("Burning Blood", -1, "在战斗结束时，回复 6 点生命。", locale);
    assert_eq!(result, "在战斗结束时，回复 6 点生命。");
}

#[test]
fn cyclic_stone_calendar_at_turn_3() {
    let locale = test_locale();
    let result = rewrite_relic_description(
        "StoneCalendar",
        3,
        "在第 7 回合结束时，对所有敌人造成 52 点伤害。",
        locale,
    );
    assert_eq!(result, "4 回合后对全体敌人造成伤害");
}

#[test]
fn cyclic_pen_nib_at_7_attacks() {
    let locale = test_locale();
    let result =
        rewrite_relic_description("Pen Nib", 7, "每打出 10 张 攻击牌 ，造成双倍伤害。", locale);
    assert_eq!(result, "3 次攻击后造成双倍伤害");
}
