//! Tests for terminal viewports, feeds, and command bindings in documents.

use super::parse;
use crate::core::event::{Key, KeyCode};
use crate::core::panel::Outcome;

#[test]
fn a_document_declares_a_term_and_a_feed() {
    let text = "\
flag tactical true
term 10 12 60 24 bridge 1 #ff9d00 when tactical
feed bridge \"uname -sr\" every 2s";

    let panel = parse(text).expect("a valid panel with term and feed");

    assert_eq!(panel.terms().len(), 1);
    assert_eq!(panel.terms()[0].name(), "bridge");
    assert_eq!(panel.feeds().len(), 1);
    assert_eq!(panel.feeds()[0].command(), "uname -sr");
    assert_eq!(panel.feeds()[0].every(), Some(2_000));
}

#[test]
fn a_key_binding_runs_a_command_into_a_declared_term() {
    let text = "\
term 10 12 60 24 bridge 1 #ff9d00
on key 1 run bridge \"uname -a\"";

    let mut panel = parse(text).expect("a valid panel");

    assert_eq!(
        panel.key(Key::plain(KeyCode::Char('1'))),
        Outcome::Run {
            term: "bridge".to_string(),
            command: "uname -a".to_string(),
        }
    );
}

#[test]
fn a_key_binding_focuses_a_term_for_live_typing() {
    let text = "\
term 10 12 60 24 bridge 1 #ff9d00
on key i focus bridge";

    let mut panel = parse(text).expect("a valid panel");

    assert_eq!(panel.key(Key::plain(KeyCode::Char('i'))), Outcome::Changed);
    assert_eq!(panel.focused_term(), Some("bridge"));
    assert_eq!(panel.key(Key::plain(KeyCode::Char('p'))), Outcome::Changed);
    assert_eq!(panel.key(Key::plain(KeyCode::Char('s'))), Outcome::Changed);
    assert_eq!(
        panel.key(Key::plain(KeyCode::Enter)),
        Outcome::Run {
            term: "bridge".to_string(),
            command: "ps".to_string(),
        }
    );
}

#[test]
fn an_input_directive_focuses_a_term_at_start() {
    let text = "\
term 10 12 60 24 bridge 1 #ff9d00
input bridge";

    let panel = parse(text).expect("a valid panel");

    assert_eq!(panel.focused_term(), Some("bridge"));
}

#[test]
fn a_feed_naming_an_undeclared_term_is_refused_with_its_line() {
    let errors = parse("clear #000000\nfeed ghost \"uptime\"").expect_err("undeclared term");

    assert_eq!(errors[0].line, 2);
    assert!(errors[0].problem.contains("ghost"), "{:?}", errors[0]);
}

#[test]
fn declaring_a_term_twice_is_refused_with_its_line() {
    let text = "\
term 0 0 30 16 bridge 1 #ff9d00
term 0 20 30 16 bridge 1 #ff9d00";

    let errors = parse(text).expect_err("duplicate term");

    assert_eq!(errors[0].line, 2);
    assert!(errors[0].problem.contains("twice"), "{:?}", errors[0]);
}
