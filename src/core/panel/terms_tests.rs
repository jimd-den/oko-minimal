//! Tests for the panel's terminals, feeds, and interactive focus.

use crate::core::binding::{Action, Binding};
use crate::core::event::{Key, KeyCode};
use crate::core::geom::{Rect, Rgba};
use crate::core::panel::{Feed, Flag, Outcome, Panel, PanelError};
use crate::core::scene::{Condition, Scene};
use crate::core::term::Term;

fn bridge_term() -> Term {
    Term::new(
        "bridge",
        Rect::new(4, 4, 120, 24),
        1,
        Rgba::WHITE,
        Condition::Always,
    )
}

#[test]
fn writing_to_a_term_places_its_lines_into_the_panel_scene() {
    let mut panel = Panel::new(Scene::new(), vec![], vec![])
        .expect("a panel")
        .with_terminals(vec![bridge_term()], vec![], None)
        .expect("with bridge");

    assert!(panel.write("bridge", "USS ENTERPRISE\nNCC-1701-D"));

    let scene = panel.scene(0);
    assert_eq!(scene.len(), 2);
}

#[test]
fn an_undeclared_term_in_a_feed_or_binding_is_refused() {
    let feed = Feed::new("missing", "uname -a", None);
    let err = Panel::new(Scene::new(), vec![], vec![])
        .expect("a panel")
        .with_terminals(vec![], vec![feed], None)
        .expect_err("undeclared term");

    assert_eq!(
        err,
        PanelError::UndeclaredTerm {
            term: "missing".to_string()
        }
    );
}

#[test]
fn a_focused_term_accepts_typed_keys_and_runs_on_enter() {
    let mut panel = Panel::new(Scene::new(), vec![], vec![])
        .expect("a panel")
        .with_terminals(vec![bridge_term()], vec![], Some("bridge".to_string()))
        .expect("focused bridge");

    assert_eq!(panel.key(Key::plain(KeyCode::Char('l'))), Outcome::Changed);
    assert_eq!(panel.key(Key::plain(KeyCode::Char('s'))), Outcome::Changed);
    assert_eq!(
        panel.key(Key::plain(KeyCode::Enter)),
        Outcome::Run {
            term: "bridge".to_string(),
            command: "ls".to_string(),
        }
    );
}

#[test]
fn escape_leaves_terminal_focus_so_panel_bindings_answer_again() {
    let quit = Binding::new(Key::plain(KeyCode::Char('q')), Action::Quit).expect("a binding");
    let mut panel = Panel::new(Scene::new(), vec![Flag::new("ok", true)], vec![quit])
        .expect("a panel")
        .with_terminals(vec![bridge_term()], vec![], Some("bridge".to_string()))
        .expect("focused bridge");

    assert_eq!(panel.key(Key::plain(KeyCode::Escape)), Outcome::Changed);
    assert_eq!(panel.focused_term(), None);
    assert_eq!(panel.key(Key::plain(KeyCode::Char('q'))), Outcome::Quit);
}

#[test]
fn due_feeds_are_reported_at_load_and_on_schedule() {
    let feeds = vec![
        Feed::new("bridge", "uname -sr", None),
        Feed::new("bridge", "uptime", Some(2_000)),
    ];
    let panel = Panel::new(Scene::new(), vec![], vec![])
        .expect("a panel")
        .with_terminals(vec![bridge_term()], feeds, None)
        .expect("with feeds");

    assert_eq!(panel.due_feeds(None, 0).len(), 2);
    assert_eq!(panel.due_feeds(Some(0), 1_000).len(), 0);
    assert_eq!(
        panel.due_feeds(Some(1_000), 2_000),
        vec![("bridge".to_string(), "uptime".to_string())]
    );
}
