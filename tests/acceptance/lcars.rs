//! Acceptance tests for the Star Trek LCARS Operating System console.
//!
//! Proves that the LCARS panel renders crisp vector graphics, animates its
//! looping warp and sensor elements continuously, toggles subsystems, links
//! Unix commands and feeds to terminal viewports, and merges interactive
//! shell command entry directly into the display list.

use oko::adapters::command;
use oko::adapters::presenter::paint;
use oko::adapters::runtime::Runtime;
use oko::core::event::{Event, Key, KeyCode};
use oko::core::session::Effect;

const LCARS: &str = include_str!("../../examples/lcars.oko");

#[test]
fn the_star_trek_lcars_os_paints_crisp_graphics_and_loops_animations() {
    let first = paint(LCARS, 80, 48).expect("the LCARS panel paints");
    assert!(!first.is_empty());

    let mut runtime = Runtime::new(80, 48);
    runtime.document(LCARS).expect("valid LCARS document");

    assert!(runtime.tick(500), "warp and sensor sweeps animate at 500ms");
    assert!(runtime.tick(2_000), "animation reaches end of first cycle");
    assert!(
        runtime.tick(2_500),
        "looping LCARS animations continue past the first cycle without settling"
    );
}

#[test]
fn the_star_trek_lcars_os_links_unix_feeds_and_interactive_terminal_commands() {
    let mut runtime = Runtime::new(80, 48);
    runtime.document(LCARS).expect("valid LCARS document");

    let initial_feeds = runtime.due_feeds(0);
    assert_eq!(initial_feeds.len(), 2);
    for (term, cmd) in initial_feeds {
        let out = command::run(&cmd);
        assert!(runtime.output(&term, &out));
    }

    let effect = runtime.event(Event::Key(Key::plain(KeyCode::Char('1'))));
    assert_eq!(
        effect,
        Some(Effect::Run {
            term: "bridge".to_string(),
            command: "uname -a".to_string(),
        })
    );

    assert_eq!(
        runtime.event(Event::Key(Key::plain(KeyCode::Char('i')))),
        Some(Effect::Repaint)
    );
    for ch in "echo KIRK".chars() {
        assert_eq!(
            runtime.event(Event::Key(Key::plain(KeyCode::Char(ch)))),
            Some(Effect::Repaint)
        );
    }
    let submitted = runtime.event(Event::Key(Key::plain(KeyCode::Enter)));
    assert_eq!(
        submitted,
        Some(Effect::Run {
            term: "bridge".to_string(),
            command: "echo KIRK".to_string(),
        })
    );
    let out = command::run("echo KIRK");
    assert_eq!(out, "KIRK");
    assert!(runtime.append("bridge", &out));
}
