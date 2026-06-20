use std::io::{self, Write};

pub fn send_ready_to(writer: &mut impl Write) {
    let _ = writeln!(writer, "ready");
    let _ = writer.flush();
}

#[cfg(test)]
pub fn send_wait_to(writer: &mut impl Write) {
    let _ = writeln!(writer, "WAIT 30");
    let _ = writer.flush();
}

#[cfg(test)]
pub fn send_state_to(writer: &mut impl Write) {
    let _ = writeln!(writer, "STATE");
    let _ = writer.flush();
}

pub fn send_ready() {
    send_ready_to(&mut io::stdout().lock());
}

#[cfg(test)]
#[path = "tests/protocol_tests.rs"]
mod tests;
