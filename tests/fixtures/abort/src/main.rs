//! Minimal native controlled application for the real panic=abort reporting contract.
use std::io::{BufRead, Write};
// The same production DTOs, codec and panic hook; no simulated reporting or Bevy dependency.
#[allow(dead_code)]
#[path = "../../../../src/command/mod.rs"]
pub mod command;
#[path = "../../../../src/handle/value.rs"]
pub mod handle;
#[allow(dead_code)]
#[path = "../../../../src/report/panic_adapter.rs"]
pub mod report;
pub mod session {
    pub use super::protocol;
}
#[allow(dead_code)]
#[path = "../../../../src/session/protocol.rs"]
pub mod protocol;
use command::{Command, input::keyboard::Key};
use protocol::{Capabilities, Diagnostic, Message};

fn emit(message: &Message) {
    let mut output = std::io::stdout().lock();
    serde_json::to_writer(&mut output, message).expect("fixture protocol output");
    output.write_all(b"\n").expect("fixture protocol newline");
    output.flush().expect("fixture protocol flush");
}

fn main() {
    if cfg!(panic = "unwind") {
        panic!("prepare this fixture with panic=abort");
    }
    report::capture::install_hook();
    emit(&Message::Ready {
        version: protocol::VERSION,
        capabilities: Capabilities::default(),
    });
    let mut armed = false;
    for line in std::io::stdin().lock().lines() {
        let line = line.expect("fixture protocol input");
        let (id, command) = match protocol::decode(&line) {
            Ok(value) => value,
            Err(error) => {
                emit(&error);
                continue;
            }
        };
        match &command {
            Command::KeyboardPress(press) if press.key == Key::F11 && !armed => armed = true,
            Command::KeyboardRelease(release) if release.key == Key::F11 && armed => armed = false,
            Command::Start(start) if start.ticks == 1 && armed => {
                panic!("Woodpecker diagnostic panic");
            }
            Command::Shutdown(_) => {
                emit(&protocol::completed(id, command.name(), ()));
                return;
            }
            _ => {
                emit(&Message::Rejected {
                    request_id: id,
                    command: command.name().into(),
                    error: Diagnostic::new("fixture_command", "expected F11 and one aborting tick"),
                });
                continue;
            }
        }
        emit(&protocol::completed(id, command.name(), ()));
    }
}
