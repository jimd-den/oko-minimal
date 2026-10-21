//! The tree a scanner builds, seen from the outside.

use super::*;

/// The text of the first run of characters in a tree.
fn run(source: &str) -> Option<String> {
    let root = document(source).expect("an export");
    root.children.iter().find_map(|child| match child {
        Node::Text(text) => Some(text.content.clone()),
        Node::Element(_) => None,
    })
}

#[test]
fn the_declaration_and_comments_are_skipped() {
    let root = document("<?xml version=\"1.0\"?><!-- a note --><g></g>").expect("a group");

    assert_eq!(root.name, "g");
}

#[test]
fn an_entity_in_a_run_is_decoded() {
    assert_eq!(run("<text>a&amp;b</text>"), Some("a&b".to_string()));
}

#[test]
fn an_unknown_entity_in_a_run_is_refused_by_name() {
    let error = document("<text>a&nbsp;b</text>").expect_err("an entity");

    assert_eq!(error.line, 1);
    assert!(error.problem.contains("&nbsp;"));
}
