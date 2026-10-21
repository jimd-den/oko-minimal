# The runtime is a loop

The console is a loop now, and the loop has no device in it.

`oko panel.oko` presents a panel, reloads it when the file changes, and hands
the screen back when the escape chord is pressed. `oko --once panel.oko`
paints one frame and stops, because a script wants a frame and not a program
that never returns.

## What is where

- `adapters::runtime` - the state of a console, with no device in it. It
  holds the session, the last good document and the frame on the screen. It
  is told two things: that an event arrived, and that the document changed.
- `adapters::sequence` - what a run of terminal bytes means. An arrow key is
  three bytes and one key, and a reader that counted bytes would count an
  arrow key as three keys.
- `adapters::keys` - the bytes a read brought, given back one event at a
  time.
- `adapters::presenter` - the bytes a frame is made of: a stream of lines
  for a pipe, or rows placed on a screen for a terminal.
- `adapters::terminal` - the device: raw mode, a read with a timeout, the
  size of the window, and the way back to the settings that were there
  before. The `Drop` impl puts the terminal back however the program ends.
- `adapters::shell` - the loop that joins the file, the terminal and the
  runtime.
- `src/main.rs` - the composition root: the arguments, and the exit code.

## The rules, and the evidence for each

- **A document that does not parse does not blank the screen.** The last good
  panel stays and the error is named with its line. An editor that blanks the
  panel on every keystroke of a half-typed line is an editor that is always
  empty. (`adapters::runtime_tests`)
- **The escape chord hands the screen back.** The effect is
  `Effect::ReclaimDisplay` and the lease is revoked. (`runtime_tests`)
- **A resize repaints at the new size.** (`runtime_tests`)
- **The terminal is restored however the program ends, including a panic.**
  The test uses a pseudo-terminal, because a terminal is a device and the
  only honest way to test one is against a device that behaves like one. The
  test was checked with teeth: with the `Drop` impl's body emptied, it fails
  with "a terminal left in raw mode is a broken session". (`terminal_tests`)
- **The bytes a terminal sends become events.** An arrow key is one event and
  not three; the chord is the chord and not three keys; a plain character is
  a character; a character that takes more than one byte is one event; an
  escape that nothing followed is a key of its own. (`sequence_tests`,
  `keys_tests`)
- **The panel is on the alternate screen**, so it does not scroll the
  person's history away, and the cursor is hidden, because a cursor blinking
  on top of a panel is a cursor on top of a panel. (`shell.rs`)

## The whole-repo gate

```
$ make check
line budget OK
cargo fmt --all --check
cargo check --jobs 1 --all-targets
cargo clippy --jobs 1 --all-targets -- -D warnings
test result: ok. 218 passed; 0 failed; 0 ignored
test result: ok. 8 passed; 0 failed; 0 ignored
test result: ok. 1 passed; 0 failed; 0 ignored
```

## The dependency

`libc` is the crate's only dependency, and it is the operating system's
terminal settings and nothing else: raw mode, a timed read, and the size of
the window. The Makefile's default flags stopped saying `--offline`, because
the crate has a dependency now and a fresh clone cannot build with that flag:
the gate has to be able to fetch what it needs.

## What this slice does not do

- The document format has no actions, so a key that reaches a focused surface
  has nothing to do. The routing exists and is tested; the panel format is
  what is missing, and that is ADR 0009's own slice.
- There is no second session, so the lease is granted once and the escape
  chord is the only way to lose it.
- No pointing device, no scrolling region, no palette fallback.
- The frame is presented whole: there is no damage tracking, so a panel that
  changes one pixel repaints the screen.
