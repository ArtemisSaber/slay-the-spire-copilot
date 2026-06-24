use std::io::{self, Write};

fn write_command_to(writer: &mut impl Write, command: &str) {
    let _ = writeln!(writer, "{command}");
    let _ = writer.flush();
}

pub fn send_ready_to(writer: &mut impl Write) {
    write_command_to(writer, "ready");
}

#[allow(
    dead_code,
    reason = "Auto-play executor integration is staged after protocol writers."
)]
pub fn send_choose_to(writer: &mut impl Write, index: usize) {
    write_command_to(writer, &format!("choose {index}"));
}

#[allow(
    dead_code,
    reason = "Auto-play executor integration is staged after protocol writers."
)]
pub fn send_play_to(writer: &mut impl Write, hand_index: usize, target_index: Option<usize>) {
    let one_based = hand_index + 1;
    match target_index {
        Some(target_index) => {
            write_command_to(writer, &format!("play {one_based} {target_index}"));
        }
        None => {
            write_command_to(writer, &format!("play {one_based}"));
        }
    }
}

#[allow(
    dead_code,
    reason = "Auto-play executor integration is staged after protocol writers."
)]
pub fn send_end_to(writer: &mut impl Write) {
    write_command_to(writer, "end");
}

#[allow(
    dead_code,
    reason = "Auto-play executor integration is staged after protocol writers."
)]
pub fn send_proceed_to(writer: &mut impl Write) {
    write_command_to(writer, "proceed");
}

#[allow(
    dead_code,
    reason = "Auto-play executor integration is staged after protocol writers."
)]
pub fn send_skip_to(writer: &mut impl Write) {
    write_command_to(writer, "skip");
}

#[allow(
    dead_code,
    reason = "Auto-play executor integration is staged after protocol writers."
)]
pub fn send_leave_to(writer: &mut impl Write) {
    write_command_to(writer, "leave");
}

#[allow(
    dead_code,
    reason = "Auto-play executor integration is staged after protocol writers."
)]
pub fn send_return_to(writer: &mut impl Write) {
    write_command_to(writer, "return");
}

#[allow(
    dead_code,
    reason = "Auto-play executor integration is staged after protocol writers."
)]
pub fn send_wait_to(writer: &mut impl Write, ms: u64) {
    write_command_to(writer, &format!("wait {ms}"));
}

#[allow(
    dead_code,
    reason = "Auto-play executor integration is staged after protocol writers."
)]
pub fn send_state_to(writer: &mut impl Write) {
    write_command_to(writer, "state");
}

pub fn send_ready() {
    send_ready_to(&mut io::stdout().lock());
}

#[cfg(test)]
#[path = "tests/protocol_tests.rs"]
mod tests;
