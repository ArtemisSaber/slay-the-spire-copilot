use super::*;

#[test]
fn ready_output_is_correct() {
    let mut buf = Vec::new();
    send_ready_to(&mut buf);
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output, "ready\n");
}

#[test]
fn wait_output_is_correct() {
    let mut buf = Vec::new();
    send_wait_to(&mut buf, 30);
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output, "wait 30\n");
}

#[test]
fn state_output_is_correct() {
    let mut buf = Vec::new();
    send_state_to(&mut buf);
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output, "state\n");
}

#[test]
fn choose_output_is_correct() {
    let mut buf = Vec::new();
    send_choose_to(&mut buf, 2);
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output, "choose 2\n");
}

#[test]
fn play_without_target_output_is_correct() {
    let mut buf = Vec::new();
    send_play_to(&mut buf, 1, None);
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output, "play 2\n");
}

#[test]
fn play_with_target_output_is_correct() {
    let mut buf = Vec::new();
    send_play_to(&mut buf, 0, Some(1));
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output, "play 1 1\n");
}

#[test]
fn end_output_is_correct() {
    let mut buf = Vec::new();
    send_end_to(&mut buf);
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output, "end\n");
}

#[test]
fn reward_navigation_outputs_are_correct() {
    let mut buf = Vec::new();
    send_proceed_to(&mut buf);
    send_skip_to(&mut buf);
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines, vec!["proceed", "skip"]);
}

#[test]
fn room_exit_outputs_are_correct() {
    let mut buf = Vec::new();
    send_leave_to(&mut buf);
    send_return_to(&mut buf);
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines, vec!["leave", "return"]);
}

#[test]
fn only_ready_and_wait_are_emitted() {
    let mut buf = Vec::new();
    send_ready_to(&mut buf);
    send_wait_to(&mut buf, 30);
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines, vec!["ready", "wait 30"]);
}
