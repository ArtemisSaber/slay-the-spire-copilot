use crate::locales::EffectParserLocale;
use crate::state::CardInfo;

use super::*;

fn zh_locale() -> EffectParserLocale {
    EffectParserLocale {
        damage_keyword: "造成".into(),
        aoe_keywords: vec!["所有敌人".into(), "全部敌人".into(), "全体敌人".into()],
        random_keywords: vec!["随机".into()],
        vulnerable_keyword: "易伤".into(),
        strength_gain_keyword: "力量".into(),
        strength_lose_keyword: "失去".into(),
        enter_wrath_keywords: vec!["进入".into(), "愤怒".into()],
        enter_calm_keywords: vec!["进入".into(), "宁静".into()],
        exit_stance_keywords: vec!["退出".into(), "姿态".into()],
        enter_divinity_keywords: vec!["进入".into(), "神格".into()],
        mantra_keyword: "真言".into(),
        execute_keywords: vec!["生命值".into(), "小于等于".into()],
        exhaust_keyword: "消耗".into(),
    }
}

fn card(description: &str) -> CardInfo {
    CardInfo {
        id: "TestCard".into(),
        name: "Test".into(),
        cost: 1,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: Some("test-uuid".into()),
        description: description.into(),
        price: None,
        playable: true,
        has_target: true,
    }
}

fn skill(description: &str) -> CardInfo {
    CardInfo {
        id: "TestSkill".into(),
        name: "Test".into(),
        cost: 1,
        card_type: "SKILL".into(),
        upgraded: false,
        uuid: Some("test-uuid".into()),
        description: description.into(),
        price: None,
        playable: true,
        has_target: false,
    }
}

fn en_locale() -> EffectParserLocale {
    EffectParserLocale {
        damage_keyword: "deal".into(),
        aoe_keywords: vec!["all enemies".into(), "ALL enemies".into()],
        random_keywords: vec!["random".into()],
        vulnerable_keyword: "Vulnerable".into(),
        strength_gain_keyword: "Strength".into(),
        strength_lose_keyword: "lose".into(),
        enter_wrath_keywords: vec!["Enter".into(), "Wrath".into()],
        enter_calm_keywords: vec!["Enter".into(), "Calm".into()],
        exit_stance_keywords: vec!["Exit".into(), "stance".into()],
        enter_divinity_keywords: vec!["Enter".into(), "Divinity".into()],
        mantra_keyword: "Mantra".into(),
        execute_keywords: vec![
            "HP is".into(),
            "set its HP to 0".into(),
            "HP or less".into(),
        ],
        exhaust_keyword: "Exhaust".into(),
    }
}

fn ja_locale() -> EffectParserLocale {
    EffectParserLocale {
        damage_keyword: "与える".into(),
        aoe_keywords: vec!["すべての敵".into(), "全ての敵".into()],
        random_keywords: vec!["ランダム".into()],
        vulnerable_keyword: "脆弱".into(),
        strength_gain_keyword: "筋力".into(),
        strength_lose_keyword: "失う".into(),
        enter_wrath_keywords: vec!["憤怒".into(), "入る".into()],
        enter_calm_keywords: vec!["平静".into(), "入る".into()],
        exit_stance_keywords: vec!["構え".into(), "解除".into()],
        enter_divinity_keywords: vec!["神格".into(), "入る".into()],
        mantra_keyword: "マントラ".into(),
        execute_keywords: vec!["HPが".into()],
        exhaust_keyword: "廃棄".into(),
    }
}

fn ko_locale() -> EffectParserLocale {
    EffectParserLocale {
        damage_keyword: "줍니다".into(),
        aoe_keywords: vec!["모든 적".into(), "모든 적에게".into()],
        random_keywords: vec!["무작위".into()],
        vulnerable_keyword: "취약".into(),
        strength_gain_keyword: "힘".into(),
        strength_lose_keyword: "잃습니다".into(),
        enter_wrath_keywords: vec!["분노".into(), "들어갑니다".into()],
        enter_calm_keywords: vec!["평온".into(), "들어갑니다".into()],
        exit_stance_keywords: vec!["자세".into(), "해제".into()],
        enter_divinity_keywords: vec!["신격".into(), "들어갑니다".into()],
        mantra_keyword: "만트라".into(),
        execute_keywords: vec!["HP가".into()],
        exhaust_keyword: "소멸".into(),
    }
}

fn power(description: &str) -> CardInfo {
    CardInfo {
        id: "TestPower".into(),
        name: "Test".into(),
        cost: 1,
        card_type: "POWER".into(),
        upgraded: false,
        uuid: Some("test-uuid".into()),
        description: description.into(),
        price: None,
        playable: true,
        has_target: false,
    }
}

#[path = "combat_effects_tests/attributes.rs"]
mod attributes;
#[path = "combat_effects_tests/basics.rs"]
mod basics;
#[path = "combat_effects_tests/damage.rs"]
mod damage;
#[path = "combat_effects_tests/exhaust.rs"]
mod exhaust;
#[path = "combat_effects_tests/locales.rs"]
mod locales;
#[path = "combat_effects_tests/relevance.rs"]
mod relevance;
