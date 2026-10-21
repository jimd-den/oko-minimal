# 0007 — The runtime is a loop, and the loop has no device in it

Status: accepted · Date: 2026-10-21

## Context

`oko panel.oko` reads a document, paints it, prints the bytes and exits.
That is a renderer, not a display server, and it does not deliver the promise
the project makes: a panel is a text file that is edited while it is on the
screen, and the screen follows.

The core has been ready for a loop for a while. `Session::dispatch` takes an
event and returns at most one effect. `Lease` says who may present. `Focus`
says where a key goes, and the escape chord is the way back to the console.
All of it is pure and tested. What is missing is the thing that drives them:
something that reads a device, notices that a file changed, paints, and hands
the screen back when it is told to.

The risk in writing that thing is that it becomes untestable. A loop that
only runs against a real terminal can only be tested by looking at it.

## Decision

**The loop is a state machine with no device in it, and the device is a
shell around it.**

- `adapters::runtime` holds the session, the current document and the last
  frame it painted. It is driven by two things it is told about: an event
  arrived, or the document changed. It answers with what should happen:
  present this frame, or hand the screen back.
- Everything that touches a device lives outside it: the terminal's raw
  mode, the bytes it reads, the file's timestamp, the sleep between looks.
  The runtime is handed the document and told when it changed.
- The terminal adapter grows the device half: it puts the terminal in raw
  mode, reads bytes with a timeout so the loop can also look at the file,
  writes the frame, and restores the terminal when it is done. The restore
  runs on every way out, including a panic, because a terminal left in raw
  mode is a broken session for the person who ran it.
- The panel is presented on the alternate screen, so it does not scroll the
  person's history away, and the cursor is hidden, because a cursor blinking
  on top of a panel is a cursor on top of a panel.
- The file is watched by polling its timestamp. There is no dependency to
  add for it, and a poll every tenth of a second is not the interesting part
  of this project.
- A document that does not parse does not blank the screen. The last good
  panel stays, and the error is named with its line, the way the parser
  already names it. An editor that blanks the panel on every keystroke of a
  half-typed line is an editor that is always empty.
- The escape chord hands the screen back: the lease is revoked, the
  terminal is restored, and the program ends.

This is the first place the project reaches for the operating system's
terminal settings, and therefore the first place it can leave a terminal
broken. That is why `libc` is the one dependency it takes, and why the
restore is a guard rather than a call at the end of `main`.

The binary keeps the one-shot paint as an option, under `--once`, because a
script wants a frame and not a program that never returns. The default is
the loop.

## What the loop does not do yet

- The document format has no actions, so a key that reaches a focused
  surface has nothing to do. The routing exists and is tested; the panel
  format is what is missing, and that is a separate decision.
- There is no second session, so the lease is granted once and the escape
  chord is the only way to lose it. The lease is a state machine ready for
  a second session; the runtime is not what will provide one.
- No pointing device, no scrolling region, no palette fallback.

## Consequences

- `oko panel.oko` becomes a program you leave running and edit beside.
- The runtime is tested without a terminal: the tests hand it a document and
  an event and ask what it would do.
- The bytes a terminal sends are turned into events by their own small
  module, because an arrow key is three bytes and one key, and a reader that
  counted bytes would count an arrow key as three keys.

## Revisit if

- the document format grows actions, and the keys that reach a surface have
  something to do;
- a second session appears, and the lease has more than one holder to choose
  between;
- the terminal adapter grows a palette fallback, which is a different
  adapter and not this one.
