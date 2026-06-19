use super::*;

fn i18n() -> I18n {
    I18n::load()
}

#[test]
fn loads_enough_to_cover_base_game() {
    let i = i18n();
    assert!(i.cards.len() > 400, "cards: {}", i.cards.len());
    assert!(i.relics.len() > 180, "relics: {}", i.relics.len());
    assert!(i.monsters.len() > 60, "monsters: {}", i.monsters.len());
    assert!(i.powers.len() > 160, "powers: {}", i.powers.len());
    assert!(i.potions.len() > 40, "potions: {}", i.potions.len());
}

#[test]
fn card_by_id_returns_chinese() {
    let i = i18n();
    assert_eq!(i.card("Strike_R"), Some("打击"));
    assert_eq!(i.card("Defend_R"), Some("防御"));
    assert_eq!(i.card("Eruption"), Some("暴怒"));
    assert_eq!(i.card("Vigilance"), Some("警惕"));
}

#[test]
fn monster_by_id_returns_chinese() {
    let i = i18n();
    assert_eq!(i.monster("JawWorm"), Some("大颚虫"));
    assert_eq!(i.monster("Cultist"), Some("邪教徒"));
    assert_eq!(i.monster("SlimeBoss"), Some("史莱姆老大"));
}

#[test]
fn relic_by_id_returns_chinese() {
    let i = i18n();
    assert_eq!(i.relic("Burning Blood"), Some("燃烧之血"));
    assert_eq!(i.relic("NeowsBlessing"), Some("涅奥的悲恸"));
    assert_eq!(i.relic("PureWater"), Some("至纯之水"));
}

#[test]
fn power_by_id_returns_chinese() {
    let i = i18n();
    assert_eq!(i.power("Strength"), Some("力量"));
    assert_eq!(i.power("Vulnerable"), Some("易伤"));
    assert_eq!(i.power("Weakened"), Some("虚弱"));
}

#[test]
fn potion_by_id_returns_chinese() {
    let i = i18n();
    assert_eq!(i.potion("FearPotion"), Some("恐惧药水"));
    assert_eq!(i.potion("Fire Potion"), Some("火焰药水"));
}

#[test]
fn unknown_ids_return_none_so_caller_falls_back() {
    let i = i18n();
    assert_eq!(i.card("SomeModCard"), None);
    assert_eq!(i.monster("CustomEnemy"), None);
    assert_eq!(i.relic("FancyRelic"), None);
}

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
fn card_desc_returns_chinese() {
    let i = i18n();
    assert!(i.card_desc("Uppercut").unwrap().contains("伤害"));
    assert!(i.card_desc("Strike_R").unwrap().contains("伤害"));
    assert!(i.card_desc("Defend_R").unwrap().contains("格挡"));
}

#[test]
fn unknown_card_desc_returns_none() {
    let i = i18n();
    assert_eq!(i.card_desc("NonexistentCard"), None);
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
