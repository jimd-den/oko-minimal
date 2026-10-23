//! Executing a panel's shell commands and feeds in the outside world.
//!
//! The core never spawns a process: it names a terminal viewport and a
//! command string in `Effect::Run` or `Feed`. This adapter runs the command
//! through `/bin/sh -c` and returns bounded plain text for the runtime to
//! hand back to the panel.

use std::process::Command;

/// Maximum number of bytes kept from a single command's output.
pub const MAX_OUTPUT_BYTES: usize = 4096;

/// Run `command` with `/bin/sh -c` and return its bounded output.
pub fn run(command: &str) -> String {
    let Ok(output) = Command::new("sh").arg("-c").arg(command).output() else {
        return "COMMAND FAILED".to_string();
    };
    let mut bytes = output.stdout;
    if bytes.is_empty() && !output.stderr.is_empty() {
        bytes = output.stderr;
    }
    if bytes.len() > MAX_OUTPUT_BYTES {
        bytes.truncate(MAX_OUTPUT_BYTES);
    }
    String::from_utf8_lossy(&bytes)
        .trim_end_matches(['\r', '\n'])
        .to_string()
}
