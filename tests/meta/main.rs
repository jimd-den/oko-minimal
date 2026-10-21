//! The repository's own rules, tested like any other behaviour.
//!
//! A ceiling that is only written down is a ceiling people negotiate with.
//! This test asks the checker the same question a reviewer would, and fails
//! when the answer is no.

use std::path::Path;
use std::process::Command;

#[test]
fn every_source_file_stays_within_the_line_budget() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let report = Command::new("python3")
        .arg("tools/check_line_budget.py")
        .current_dir(root)
        .output()
        .expect("python3 is available");

    assert!(
        report.status.success(),
        "the line budget check failed:\n{}\n{}",
        String::from_utf8_lossy(&report.stdout),
        String::from_utf8_lossy(&report.stderr),
    );
}
