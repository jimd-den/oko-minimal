# 0017 — A panel holds terminals, feeds, and command bindings

Status: accepted · Date: 2026-10-22
Supersedes: the no-command restriction of ADR 0009

## Context

ỌKỌ is both a graphical display-list console before a display manager and a
Unix terminal that can be hot-edited into esoteric instrument panels (such as a
Star Trek LCARS operating system). To explore the Unix way from inside a panel,
an author needs to:

1. toggle subsystem flags with a single key (`on key t toggle tactical`);
2. place live terminal viewports (`term`) inside the graphical layout;
3. populate those viewports from periodic or one-shot Unix commands and APIs
   (`feed`), from key-bound commands (`on key 1 run bridge "uname -a"`), or by
   typing commands interactively into a focused terminal (`focus` / `input`).

The core must stay pure (`std` data only, no process spawning, no I/O, no
clock). All process execution belongs in `adapters`.

## Decision

**The panel declares named terminal viewports (`term`) and command feeds
(`feed`), and answers keys with pure effects that the adapter shell executes.**

- **Flag toggling:** `on key <key> toggle <flag>` flips a declared boolean flag.
- **Terminal viewports (`term`):**

      term <x> <y> <w> <h> <name> <scale> <colour> [when <flag>]

  A `Term` in `core::term` holds its bounding rectangle, scale, colour,
  optional `Condition`, line buffer, and live input line. It wraps and scrolls
  text to fit the character grid that `(w, h)` and `scale` define, sanitizing
  any character outside the built-in font. When `Panel::scene(now)` builds the
  display list, each visible `Term` lowers its visible rows (plus `> <input>_`
  when focused) into `Shape::Text` elements.
- **Feeds (`feed`):**

      feed <name> "<command>" [every <duration>]

  A `Feed` records the target `term` name, the command string, and an optional
  poll interval in milliseconds. `Panel::due_feeds(last, now)` is a pure query
  returning the `(term, command)` pairs that should run at `now`.
- **Command and focus bindings:**
  - `on key <key> run <name> "<command>"` returns
    `Outcome::Run { term, command }`.
  - `on key <key> focus <name>` (or top-level `input <name>`) focuses the named
    terminal. While a terminal is focused, printable characters and `backspace`
    edit its input line, `escape` releases terminal focus back to panel
    bindings, and `enter` echoes `$ <command>` and returns
    `Outcome::Run { term, command }`.
- **Validation:** Naming an undeclared `term` in `feed`, `run`, `focus`, or
  `input`, or declaring two `term`s with the same name, is refused in
  `Panel::new` with the exact line number.
- **Adapter execution:** `adapters::command::run` executes `/bin/sh -c` and
  returns bounded output, which `Runtime::output(term, text)` writes back into
  the panel.

## Consequences

- `core` remains 100% pure and testable in one line without spawning processes.
- `core::raster` and `adapters::presenter` need no new primitives because
  `Term` lowers directly into `Shape::Text` in `Panel::scene(now)`.
