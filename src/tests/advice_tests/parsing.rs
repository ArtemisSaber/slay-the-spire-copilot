use super::*;

#[test]
fn parse_advice_response_all_fields() {
    let raw = "推荐：武装\n理由：攻击牌占比过高(10/16)，需要技能牌来平衡攻防节奏。\n风险：武装前期抽到且手牌无高价值目标时会卡手。\n吐槽：这卡组攻击力爆表但像个莽夫！";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "武装");
    assert_eq!(
        fields.reason,
        "攻击牌占比过高(10/16)，需要技能牌来平衡攻防节奏。"
    );
    assert_eq!(fields.risk, "武装前期抽到且手牌无高价值目标时会卡手。");
    assert_eq!(fields.commentary, "这卡组攻击力爆表但像个莽夫！");
}

#[test]
fn parse_advice_response_partial_fields() {
    let raw = "推荐：跳过\n理由：都不好";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "跳过");
    assert_eq!(fields.reason, "都不好");
    assert_eq!(fields.risk, "");
    assert_eq!(fields.commentary, "");
}

#[test]
fn parse_advice_response_multiline_reason() {
    let raw = "推荐：武装\n理由：攻击牌占比过高\n需要更多技能牌\n平衡攻防节奏\n风险：前期卡手";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "武装");
    assert_eq!(
        fields.reason,
        "攻击牌占比过高\n需要更多技能牌\n平衡攻防节奏"
    );
    assert_eq!(fields.risk, "前期卡手");
    assert_eq!(fields.commentary, "");
}

#[test]
fn parse_advice_response_multiline_commentary() {
    let raw = "推荐：跳过\n理由：没有好牌\n风险：错过发育机会\n吐槽：哈哈\n选牌也能这么背";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.commentary, "哈哈\n选牌也能这么背");
}

#[test]
fn parse_advice_response_empty() {
    let fields = parse_advice_response("", test_locale());
    assert_eq!(fields.recommendation, "");
    assert_eq!(fields.reason, "");
    assert_eq!(fields.risk, "");
    assert_eq!(fields.commentary, "");
}

#[test]
fn parse_advice_response_content_before_first_field_is_ignored() {
    let raw = "形势不错。  角色：铁甲战士  层数：5\n推荐：武装\n理由：好";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "武装");
    assert_eq!(fields.reason, "好");
}

#[test]
fn parse_advice_response_blank_lines_between_fields() {
    let raw = "推荐：武装\n\n理由：好\n\n\n风险：弱\n\n吐槽：烂";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "武装");
    assert_eq!(fields.reason, "好");
    assert_eq!(fields.risk, "弱");
    assert_eq!(fields.commentary, "烂");
}

#[test]
fn parse_advice_response_only_commentary() {
    let raw = "吐槽：这个卡组太极端了";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "");
    assert_eq!(fields.reason, "");
    assert_eq!(fields.risk, "");
    assert_eq!(fields.commentary, "这个卡组太极端了");
}

#[test]
fn parse_advice_response_real_example_card_reward() {
    let raw = "推荐：武装\n理由：攻击牌占比过高(10/16)，需要技能牌来平衡攻防节奏。武装的低费和升级能力能提升整副卡组的质量。\n风险：武装前期抽到且手牌无高价值目标时会卡手。\n吐槽：这卡组攻击力爆表但像个莽夫！学点生存技巧吧，别光想着打打打！";

    let fields = parse_advice_response(raw, test_locale());

    assert!(!fields.recommendation.is_empty());
    assert!(!fields.reason.is_empty());
    assert!(!fields.risk.is_empty());
    assert!(!fields.commentary.is_empty());
}

#[test]
fn parse_advice_response_real_example_rest() {
    let raw = "推荐：休息\n理由：血量极低(12/75)，下一场战斗可能遇到精英，必须保证生存。\n风险：错过锻造机会，卡组强度提升推迟。\n吐槽：活着才有输出！别贪了，先回血保命吧。";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "休息");
    assert!(fields.reason.contains("血量极低"));
    assert!(fields.risk.contains("锻造"));
    assert!(fields.commentary.contains("活着"));
}

#[test]
fn parse_advice_response_field_names_with_punctuation() {
    let raw = "推荐：跳过。\n理由：都不好。\n风险：无。\n吐槽：。";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "跳过。");
    assert_eq!(fields.reason, "都不好。");
    assert_eq!(fields.risk, "无。");
    assert_eq!(fields.commentary, "。");
}
