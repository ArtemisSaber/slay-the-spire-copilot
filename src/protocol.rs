use std::io::{self, Write};

pub fn send_ready_to(writer: &mut impl Write) {
    let _ = writeln!(writer, "ready");
    let _ = writer.flush();
}

pub fn send_wait_to(writer: &mut impl Write) {
    let _ = writeln!(writer, "WAIT 30");
    let _ = writer.flush();
}

#[allow(dead_code)]
pub fn send_state_to(writer: &mut impl Write) {
    let _ = writeln!(writer, "STATE");
    let _ = writer.flush();
}

pub fn send_ready() {
    send_ready_to(&mut io::stdout().lock());
}

pub fn send_wait() {
    send_wait_to(&mut io::stdout().lock());
}

#[cfg(test)]
mod tests {
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
}
