use std::io::{self, Write};

pub fn send_ready() {
    let mut stdout = io::stdout().lock();
    let _ = writeln!(stdout, "ready");
    let _ = stdout.flush();
}

pub fn send_wait() {
    let mut stdout = io::stdout().lock();
    let _ = writeln!(stdout, "WAIT 30");
    let _ = stdout.flush();
}
