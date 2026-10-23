//! Tests for the shell command adapter.

use super::command;

#[test]
fn a_shell_command_returns_its_standard_output() {
    let out = command::run("printf 'LCARS ONLINE\\nSTARDATE 4711'");

    assert_eq!(out, "LCARS ONLINE\nSTARDATE 4711");
}

#[test]
fn a_command_output_is_bounded_to_the_byte_budget() {
    let out = command::run("dd if=/dev/zero bs=1024 count=8 2>/dev/null | tr '\\0' 'A'");

    assert_eq!(out.len(), command::MAX_OUTPUT_BYTES);
}
