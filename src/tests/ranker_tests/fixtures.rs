use super::*;

#[test]
fn defend_does_not_end_fight() {
    let state = comm_mod_state("comm-f16t15-defend-strike-hexaghost.json");
    let scored = ranker::rank(&state);
    let defend = find_scored(&scored, "防御");
    let rule = find_breakdown(&defend.breakdown, "combat_ends_fight");
    assert!(!rule.matched, "Defend should NOT end the fight");
    assert_eq!(rule.score, 0);
}

#[test]
fn strike_can_end_fight() {
    let state = comm_mod_state("comm-f1t1-strikes-vs-slimes.json");
    let scored = ranker::rank(&state);
    let strike = find_scored(&scored, "打击");
    let rule = find_breakdown(&strike.breakdown, "combat_ends_fight");
    assert!(!rule.matched, "Strike 6dmg vs 10HP should NOT end fight");
}

#[test]
fn defend_no_str_gain_false_positive() {
    let state = comm_mod_state("comm-f16t16-hexaghost-turn.json");
    let scored = ranker::rank(&state);
    let defend = find_scored(&scored, "防御");
    let rule = find_breakdown(&defend.breakdown, "setup_self_str_gain");
    assert!(!rule.matched, "Defend should NOT give Strength gain");
    assert_eq!(rule.score, 0);
}

#[test]
fn defend_no_dex_gain_false_positive() {
    let state = comm_mod_state("comm-f16t16-hexaghost-turn.json");
    let scored = ranker::rank(&state);
    let defend = find_scored(&scored, "防御");
    let rule = find_breakdown(&defend.breakdown, "setup_self_dex_gain");
    assert!(!rule.matched, "Defend should NOT give Dexterity gain");
    assert_eq!(rule.score, 0);
}

#[test]
fn bash_scores_vulnerable_apply() {
    let state = comm_mod_state("comm-f16t16-hexaghost-turn.json");
    let scored = ranker::rank(&state);
    let bash = find_scored(&scored, "痛击");
    let rule = find_breakdown(&bash.breakdown, "setup_vulnerable_apply");
    assert!(rule.matched, "Bash should apply Vulnerable");
    assert_eq!(rule.score, 20, "2 stacks × 10 weight = 20");
}

#[test]
fn body_slam_with_zero_block_does_not_end_fight() {
    let state = comm_mod_state("comm-f16t16-bodyslam-vs-hexaghost.json");
    let scored = ranker::rank(&state);
    let slam = find_scored(&scored, "全身撞击");
    let rule = find_breakdown(&slam.breakdown, "combat_ends_fight");
    assert!(
        !rule.matched,
        "Body Slam with 0 block vs 41 HP should NOT end fight"
    );
    assert_eq!(rule.score, 0);
}

#[test]
fn strike_scores_damage() {
    let state = comm_mod_state("comm-f16t15-defend-strike-hexaghost.json");
    let scored = ranker::rank(&state);
    let strike = find_scored(&scored, "打击");
    let rule = find_breakdown(&strike.breakdown, "core_damage");
    assert!(rule.matched, "Strike should deal damage");
    assert_eq!(
        rule.score, 109,
        "6 dmg × 1 hit × 1000 weight / 55 monster hp = 109"
    );
}

#[test]
fn defend_scores_block() {
    let state = comm_mod_state("comm-f16t16-hexaghost-turn.json");
    let scored = ranker::rank(&state);
    let defend = find_scored(&scored, "防御");
    let rule = find_breakdown(&defend.breakdown, "core_block_non_excessive");
    assert!(rule.matched, "Defend should give block");
    let b = defend
        .breakdown
        .iter()
        .find(|b| b.rule_id == "core_block_non_excessive")
        .unwrap();
    assert!(b.matched);
    assert!(b.score > 0, "Block should have positive score");
}
