use super::*;

#[test]
fn character_classes_are_translated() {
    assert_eq!(translate_class("IRONCLAD"), "铁甲战士");
    assert_eq!(translate_class("THE_SILENT"), "猎人");
    assert_eq!(translate_class("DEFECT"), "机器人");
    assert_eq!(translate_class("WATCHER"), "观者");
}

#[test]
fn monster_intents_are_translated() {
    assert_eq!(translate_intent("ATTACK"), "攻击");
    assert_eq!(translate_intent("DEFEND"), "防御");
    assert_eq!(translate_intent("BUFF"), "增益");
    assert_eq!(translate_intent("DEBUFF"), "减益");
}

#[test]
fn card_types_are_translated() {
    assert_eq!(translate_type("ATTACK"), "攻击");
    assert_eq!(translate_type("SKILL"), "技能");
    assert_eq!(translate_type("POWER"), "能力");
}

#[test]
fn rest_options_are_translated() {
    assert_eq!(translate_rest_option("rest"), "休息(回30%血)");
    assert_eq!(translate_rest_option("smith"), "锻造(升级)");
    assert_eq!(translate_rest_option("toke"), "回忆(移除一张牌)");
}

#[test]
fn unknown_values_pass_through_unchanged() {
    assert_eq!(translate_class("custom"), "custom");
    assert_eq!(translate_intent("custom_intent"), "custom_intent");
    assert_eq!(translate_type("custom"), "custom");
    assert_eq!(translate_rest_option("custom"), "custom");
}

#[test]
fn translate_intent_all_variants() {
    let cases = [
        ("ATTACK", "攻击"),
        ("ATTACK_BUFF", "攻击+增益"),
        ("ATTACK_DEBUFF", "攻击+减益"),
        ("ATTACK_DEFEND", "攻击+防御"),
        ("BUFF", "增益"),
        ("DEBUFF", "减益"),
        ("STRONG_DEBUFF", "强力减益"),
        ("DEBUG", "调试"),
        ("DEFEND", "防御"),
        ("DEFEND_DEBUFF", "防御+减益"),
        ("DEFEND_BUFF", "防御+增益"),
        ("ESCAPE", "逃跑"),
        ("MAGIC", "法术"),
        ("NONE", "无"),
        ("SLEEP", "睡眠"),
        ("STUN", "眩晕"),
        ("UNKNOWN", "未知"),
    ];
    for (input, expected) in cases {
        assert_eq!(translate_intent(input), expected, "intent: {input}");
    }
}

#[test]
fn translate_type_all_variants() {
    assert_eq!(translate_type("CURSE"), "诅咒");
    assert_eq!(translate_type("STATUS"), "状态");
}
