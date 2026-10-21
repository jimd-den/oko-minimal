# The interactivity in the core

The panel has a state of its own: the flags it holds, the bindings it
answers, and the scene it draws. This is the core's half of ADR 0009; the
document's grammar is another person's slice.

## What the core gained

- `core::scene::Condition` - the condition under which an element is drawn:
  `Always` or `Flag(name)`. `core::scene::Element` grew a third part, the
  condition, next to the shape and the motion.
- `core::scene::Element::when(shape, motion, condition)` - the element that
  is drawn only while its condition holds. `Element::new(shape, motion)`
  still builds an element that is drawn always, so the document layer does
  not have to say `Always` to say nothing.
- `core::scene::Scene::satisfying(holds)` - the scene of the elements whose
  condition holds. A pure function; the panel decides what holds, because
  the flags live there.
- `core::binding::{Action, Binding, RESERVED_ESCAPE, BindError}` - the keys
  a panel answers. The actions are the whole vocabulary: `set <flag> true`,
  `set <flag> false`, `quit`.
- `core::panel::{Flag, Outcome, Panel, PanelError}` - the panel itself: a
  scene, the flags it holds, the bindings it answers.

## The two decisions that were mine

**Where the condition's type lives.** In `core::scene`, with the rest of the
display list's vocabulary. The element needs it and the panel needs it, and
the panel holds the scene, so the type must not make the two modules depend
on each other. In `scene` the dependency points one way: the panel knows
the scene, and the scene knows nobody. A `Condition::Flag` carries only a
name; the flags themselves live in the panel, and the panel is what says
whether a name is true.

**What `panel.key(k)` returns.** An `Outcome`, not a `bool`:

```rust
pub enum Outcome {
    Unchanged,  // nothing changed; the panel needs no repaint
    Changed,    // a flag changed; the runtime should repaint
    Quit,       // the binding asked the program to end
}
```

A `bool` answers "did the panel change?" but `quit` is an action a binding
may have, and a bool has nowhere to put it: the panel did not change, and
the program is ending. An enum carries all three answers, and the runtime
can ask `matches!(outcome, Outcome::Changed)` when it wants the old
question answered.

A key that sets a flag to the value it already holds is `Unchanged`: the
panel did not change, so there is nothing to repaint.

## The rules the panel keeps

- The reserved escape chord (`ctrl+alt+escape`) may not be bound. The
  refusal is at `Binding::new`, so an escape binding never exists, and the
  error names the chord.
- A flag that is not declared is not a flag: `Panel::new` refuses a binding
  or a condition that names one, and the error names the flag.
- The flags live in the panel and not in the session. The session is about
  the screen's authority; the flags are about the content.

## The red and the green

Red, before the panel had its rules (7 failures, each on the behaviour and
not on a compile error):

```
---- a_binding_for_the_escape_chord_is_refused_and_named ----
panicked at src/core/panel_tests.rs:103:65:
the reserved chord: Binding { key: Key { code: Escape, ... } }
---- a_binding_sets_a_flag ----
assertion `left == right` failed  left: Some(false)  right: Some(true)
---- a_binding_with_the_action_quit_says_so ----
assertion `left == right` failed  left: Unchanged  right: Quit
---- a_key_that_sets_a_flag_says_the_panel_changed ----
assertion `left == right` failed  left: Unchanged  right: Changed
---- a_shape_with_a_condition_is_not_in_the_scene_while_its_flag_is_false ----
assertion `left == right` failed  left: 1  right: 0
---- a_binding_that_names_a_flag_that_was_not_declared_is_refused ----
panicked: an undeclared flag
---- a_condition_that_names_a_flag_that_was_not_declared_is_refused ----
panicked: an undeclared flag
test result: FAILED. 249 passed; 7 failed
```

The scene's own red, when `satisfying` kept everything:

```
---- a_scene_keeps_the_elements_whose_condition_holds ----
assertion `left == right` failed  left: 2  right: 1
---- a_scene_drops_the_elements_whose_condition_does_not_hold ----
assertion `left == right` failed  left: 2  right: 1
test result: FAILED. 239 passed; 2 failed
```

Green:

```
cargo test -q --lib -- --test-threads=1
running 257 tests
test result: ok. 257 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The branch had 236 lib tests before this slice; there are 257 now.

## The gate

```
$ make check
python3 tools/check_line_budget.py
line budget OK
cargo fmt --all --check
cargo check --jobs 1 --all-targets
cargo clippy --jobs 1 --all-targets -- -D warnings
cargo test --jobs 1 -- --test-threads=1
...
test result: ok. 257 passed (lib); 8 passed (acceptance); 1 passed (meta)

$ python3 tools/check_line_budget.py
0 exempt, 17 in the warning band, 0 frozen files grew, 0 over the ceiling
line budget OK
```

Sizes: `core/panel.rs` 182, `core/binding.rs` 74, `core/scene.rs` 142,
`core/panel_tests.rs` 165, `core/binding_tests.rs` 41,
`core/scene_tests.rs` 109.

## One thing that was already broken

`make check` was red on this branch before this slice: `tests/acceptance/
main.rs` still spoke the scene's old language (`scene::Op::Path`,
`scene.ops()`), which the merge of `main` removed, so `cargo check
--all-targets` failed to compile the acceptance test. It is fixed here, in
its own commit, with every assertion kept:

```rust
let oko::core::shape::Shape::Path { path, .. } = &scene.elements()[0].shape else {
    panic!("the arc is a path operation");
};
```

If another slice owns that file, that commit is easy to drop.

## What the document's grammar needs from this slice

```rust
use oko::core::binding::{Action, Binding};
use oko::core::panel::{Flag, Panel};
use oko::core::scene::{Condition, Element};

// flag <name> [true|false]
let flag = Flag::new("shown", false);
// when <name>
let element = Element::when(shape, motion, Condition::Flag("shown".to_string()))?;
// on key <key> <action>
let binding = Binding::new(key, Action::Set { flag: "shown".to_string(), value: true })?;
let panel = Panel::new(scene, vec![flag], vec![binding])?;
```

`Panel::new` refuses a binding or a condition that names a flag that was
not declared, so the document layer can pass the flags through and let the
panel be the judge.
