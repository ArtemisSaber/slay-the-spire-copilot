use std::collections::HashMap;

pub struct I18n {
    cards: HashMap<String, String>,
    relics: HashMap<String, String>,
    monsters: HashMap<String, String>,
    powers: HashMap<String, String>,
    potions: HashMap<String, String>,
}

impl I18n {
    pub fn load() -> Self {
        I18n {
            cards: serde_json::from_str(include_str!("cards.json")).unwrap(),
            relics: serde_json::from_str(include_str!("relics.json")).unwrap(),
            monsters: serde_json::from_str(include_str!("monsters.json")).unwrap(),
            powers: serde_json::from_str(include_str!("powers.json")).unwrap(),
            potions: serde_json::from_str(include_str!("potions.json")).unwrap(),
        }
    }

    pub fn card(&self, id: &str) -> Option<&str> {
        self.cards.get(id).map(|s| s.as_str())
    }

    pub fn relic(&self, id: &str) -> Option<&str> {
        self.relics.get(id).map(|s| s.as_str())
    }

    pub fn monster(&self, id: &str) -> Option<&str> {
        self.monsters.get(id).map(|s| s.as_str())
    }

    pub fn power(&self, id: &str) -> Option<&str> {
        self.powers.get(id).map(|s| s.as_str())
    }

    pub fn potion(&self, id: &str) -> Option<&str> {
        self.potions.get(id).map(|s| s.as_str())
    }
}

pub fn translate_class(class: &str) -> &str {
    match class {
        "IRONCLAD" => "铁甲战士",
        "THE_SILENT" => "猎人",
        "DEFECT" => "机器人",
        "WATCHER" => "观者",
        _ => class,
    }
}

pub fn translate_type(t: &str) -> &str {
    match t {
        "ATTACK" => "攻击",
        "SKILL" => "技能",
        "POWER" => "能力",
        "CURSE" => "诅咒",
        "STATUS" => "状态",
        _ => t,
    }
}

pub fn translate_intent(intent: &str) -> &str {
    match intent {
        "ATTACK" => "攻击",
        "ATTACK_BUFF" => "攻击+增益",
        "ATTACK_DEBUFF" => "攻击+减益",
        "ATTACK_DEFEND" => "攻击+防御",
        "BUFF" => "增益",
        "DEBUFF" => "减益",
        "STRONG_DEBUFF" => "强力减益",
        "DEBUG" => "调试",
        "DEFEND" => "防御",
        "DEFEND_DEBUFF" => "防御+减益",
        "DEFEND_BUFF" => "防御+增益",
        "ESCAPE" => "逃跑",
        "MAGIC" => "法术",
        "NONE" => "无",
        "SLEEP" => "睡眠",
        "STUN" => "眩晕",
        "UNKNOWN" => "未知",
        _ => intent,
    }
}

pub fn translate_rest_option(opt: &str) -> &str {
    match opt {
        "rest" => "休息(回30%血)",
        "smith" => "锻造(升级)",
        "toke" => "回忆(移除一张牌)",
        "dig" => "挖遗物",
        "lift" => "举重(永久+1力量)",
        "recall" => "回忆钥匙",
        "girya" => "深蹲(力量)",
        _ => opt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_translation_files() {
        let i18n = I18n::load();
        assert!(i18n.cards.len() > 300);
        assert!(i18n.relics.len() > 100);
        assert!(i18n.monsters.len() > 50);
        assert!(i18n.powers.len() > 100);
        assert!(i18n.potions.len() > 30);
    }

    #[test]
    fn known_card_translates() {
        let i18n = I18n::load();
        assert_eq!(i18n.card("Strike_R"), Some("打击"));
        assert_eq!(i18n.card("Defend_R"), Some("防御"));
        assert_eq!(i18n.card("Bash"), Some("痛击"));
    }

    #[test]
    fn known_monster_translates() {
        let i18n = I18n::load();
        assert_eq!(i18n.monster("JawWorm"), Some("大颚虫"));
    }

    #[test]
    fn unknown_id_returns_none() {
        let i18n = I18n::load();
        assert_eq!(i18n.card("SomeModCard"), None);
    }
}
