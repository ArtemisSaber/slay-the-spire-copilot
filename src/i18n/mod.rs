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
#[path = "../tests/i18n_tests.rs"]
mod tests;
