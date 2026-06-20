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
    send_wait_to(&mut buf);
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output, "WAIT 30\n");
}

#[test]
fn state_output_is_correct() {
    let mut buf = Vec::new();
    send_state_to(&mut buf);
    let output = String::from_utf8(buf).unwrap();
    assert_eq!(output, "STATE\n");
}

#[test]
fn only_ready_and_wait_are_emitted() {
    let mut buf = Vec::new();
    send_ready_to(&mut buf);
    send_wait_to(&mut buf);
    let output = String::from_utf8(buf).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines, vec!["ready", "WAIT 30"]);
}
