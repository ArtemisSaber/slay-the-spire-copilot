use super::*;

#[test]
fn vars_contain_heal() {
    let state = make_state(vec![card_with_desc("回复 5 点生命")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("heal").copied(), Some(5.0));
}

#[test]
fn vars_contain_draw() {
    let state = make_state(vec![card_with_desc("抽 2 张牌")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("draw").copied(), Some(2.0));
}

#[test]
fn vars_contain_str_gain() {
    let state = make_state(vec![card_with_desc("获得 3 点 力量")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("str_gain").copied(), Some(3.0));
}

#[test]
fn vars_contain_dex_gain() {
    let state = make_state(vec![card_with_desc("获得 2 点 敏捷")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("dex_gain").copied(), Some(2.0));
}

#[test]
fn vars_contain_poison() {
    let state = make_state(vec![card_with_desc("给予 5 层 中毒")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("poison").copied(), Some(5.0));
}

#[test]
fn vars_contain_vulnerable() {
    let state = make_state(vec![card_with_desc("给予 2 层 易伤")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("vulnerable").copied(), Some(2.0));
}

#[test]
fn vars_contain_weak() {
    let state = make_state(vec![card_with_desc("给予 3 层 虚弱")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("weak").copied(), Some(3.0));
}

#[test]
fn vars_contain_energy_gain() {
    let state = make_state(
        vec![card_with_desc("[E] [E] 造成 8 点伤害")],
        vec![jaw_worm()],
    );
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("energy_gain").copied(), Some(2.0));
}

#[test]
fn vars_contain_str_loss() {
    let state = make_state(vec![card_with_desc("敌人失去 4 点力量")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("str_loss").copied(), Some(4.0));
    assert_eq!(vars.get("str_loss_temp").copied(), Some(0.0));
}

#[test]
fn vars_contain_str_loss_temp() {
    let state = make_state(
        vec![card_with_desc("敌人失去 2 点力量。一回合")],
        vec![jaw_worm()],
    );
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("str_loss").copied(), Some(2.0));
    assert_eq!(vars.get("str_loss_temp").copied(), Some(1.0));
}

#[test]
fn vars_contain_mantra() {
    let state = make_state(vec![card_with_desc("获得 3 层 真言")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("mantra").copied(), Some(3.0));
}

#[test]
fn vars_contain_focus_gain() {
    let state = make_state(vec![card_with_desc("获得 2 点 集中")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("focus_gain").copied(), Some(2.0));
}

#[test]
fn vars_contain_channel_orb() {
    let state = make_state(vec![card_with_desc("充能 1 个 闪电 球")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("channel_orb_Lightning").copied(), Some(1.0));
}

#[test]
fn vars_contain_evoke_orb() {
    let state = make_state(
        vec![card_with_desc("激发 你的 所有 闪电 球")],
        vec![jaw_worm()],
    );
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("evoke_orb_Lightning").copied(), Some(1.0));
}

#[test]
fn vars_contain_orb_slot_expand() {
    let state = make_state(
        vec![card_with_desc("获得 1 个 充能球栏位")],
        vec![jaw_worm()],
    );
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("expand_count").copied(), Some(1.0));
}

#[test]
fn vars_contain_exits_stance() {
    let state = make_state(
        vec![card_with_desc("退出 当前 姿态。造成 8 点伤害")],
        vec![jaw_worm()],
    );
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("exits_stance").copied(), Some(1.0));
}

#[test]
fn vars_contain_enters_wrath() {
    let state = make_state(vec![card_with_desc("进入 愤怒 姿态")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("enters_wrath").copied(), Some(1.0));
}

#[test]
fn vars_contain_enters_calm() {
    let state = make_state(vec![card_with_desc("进入 宁静")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("enters_calm").copied(), Some(1.0));
}

#[test]
fn vars_contain_ethereal() {
    let state = make_state(
        vec![card_with_desc("造成 6 点伤害。 虚无")],
        vec![jaw_worm()],
    );
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("ethereal").copied(), Some(1.0));
}
