use crate::state::NormalizedState;

pub fn build_prompt(state: &NormalizedState) -> String {
    let mut lines: Vec<String> = vec![];

    lines.push("你是一个中文《杀戮尖塔》直播助手。请根据当前游戏状态给出简单建议。".to_string());

    if let Some(ref c) = state.character {
        lines.push(format!("角色：{}", translate_class(c)));
    }
    if let Some(f) = state.floor {
        lines.push(format!("层数：第{f}层"));
    }
    if let (Some(cur), Some(max)) = (state.current_hp, state.max_hp) {
        lines.push(format!("血量：{cur}/{max}"));
    }
    if let Some(g) = state.gold {
        lines.push(format!("金币：{g}"));
    }
    if let Some(e) = state.energy {
        lines.push(format!("能量：{e}"));
    }
    if let Some(b) = state.block {
        lines.push(format!("格挡：{b}"));
    }

    if !state.powers.is_empty() {
        let powers_str: Vec<String> = state
            .powers
            .iter()
            .map(|p| format!("{}({})", p.name, p.amount))
            .collect();
        lines.push(format!("能力：{}", powers_str.join(", ")));
    }

    if !state.relics.is_empty() {
        lines.push(format!("遗物：{}", state.relics.join(", ")));
    }

    if !state.potions.is_empty() {
        lines.push(format!("药水：{}", state.potions.join(", ")));
    }

    if !state.hand.is_empty() {
        let hand_str: Vec<String> = state
            .hand
            .iter()
            .map(|c| {
                let up = if c.upgraded { "+" } else { "" };
                format!("{}{}({}费)", c.name, up, c.cost)
            })
            .collect();
        lines.push(format!("手牌：{}", hand_str.join(", ")));
    }

    if !state.monsters.is_empty() {
        let monster_strs: Vec<String> = state
            .monsters
            .iter()
            .map(|m| {
                let mut s = m.name.to_string();
                if let (Some(cur), Some(max)) = (m.current_hp, m.max_hp) {
                    s.push_str(&format!("({}/{})", cur, max));
                }
                if let Some(ref intent) = m.intent {
                    s.push_str(&format!("[意图:{intent}]"));
                }
                if let Some(dmg) = m.damage {
                    s.push_str(&format!("[伤害:{dmg}]"));
                }
                s
            })
            .collect();
        lines.push(format!("怪物：{}", monster_strs.join(" | ")));
    }

    if !state.card_reward_choices.is_empty() {
        lines.push(format!("选牌：{}", state.card_reward_choices.join(", ")));
    }

    if let Some(ref st) = state.screen_type
        && st != "NONE"
    {
        lines.push(format!("当前界面：{st}"));
    }

    lines.push(
        "请按以下格式回复（总字数控制在120字以内）：\n推荐：\n理由：\n风险：\n吐槽：".to_string(),
    );

    lines.join("\n")
}

fn translate_class(class: &str) -> &str {
    match class {
        "IRONCLAD" => "铁甲战士",
        "THE_SILENT" => "猎人",
        "DEFECT" => "机器人",
        "WATCHER" => "观者",
        _ => class,
    }
}
