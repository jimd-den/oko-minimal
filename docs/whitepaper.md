# ỌKỌ — the screen as a contract

## The claim

A display server is a broker between programs and pixels. On a machine whose
first screen is a login prompt, the broker is a liability: it needs a GPU, a
session bus, a graph of processes, and a reason to exist before anyone has
logged in.

ỌKỌ removes the broker and keeps the contract. The contract is a **display
list**: an ordered set of rectangles and lines, rendered into a frame by a pure
function. A program that wants to paint produces a list. The console owns the
device and decides who may present. Between the list and the device there is no
protocol to negotiate, no surface to allocate, no compositor to schedule. There
is a frame, a lease, and a focused surface.

This is the difference between a theme and a computer. A theme is a picture of
an interface; a computer is a contract you can build one on. The panels may look
like an instrument wall, but the thing underneath is a small, total, testable
pipeline that happens to draw rectangles very well.

## Two halves and one line

`core` is pure. Given the same scene and the same frame size it returns the same
pixels; given the same document it returns the same scene or the same errors. It
has no clock, no filesystem, no terminal, and no globals. `adapters` is the
outside world: it reads bytes, writes escape sequences, and translates an SVG
export. `main` names a file and a terminal; nothing else does.

The line between them is the display list. A `Scene` is a short sequence of
`Clear`, `Rect` and `Line`. A `Frame` is an opaque grid of pixels. `render` is
the only function that turns one into the other. A `document` is the text a
person edits; it parses into a `Scene`, and a bad document changes nothing.

## Why a display list, not a tree

Instrument panels are made of rules, bars, frames and large type. A document
layout engine is the wrong shape for that, and a retained scene graph is state
that every reload must synchronise. A flat list is cheaper to produce, compare,
serialise and discard. It is also the unit of hot editing: parse the file, get a
list, swap it, redraw. Nothing else in the system needs to know that the file
changed.

The list will grow: text, images, clipped groups, transforms. Each addition
needs a primitive, a renderer case, and a test. The order is deliberate. A
primitive that no test demanded is a guess, and guesses become compatibility
burdens.

## States and events

A screen is not a function of time. It is a function of state, and state changes
when events arrive. Two state machines guard the screen:

- a **lease** grants one owner the right to present, with a generation that
  makes a stale frame impossible after revocation;
- **focus** decides where a key goes: the reserved escape chord first, then the
  focused surface, then nowhere. Unfocused input is dropped, not queued.

`Session::dispatch` composes them and returns at most one effect. The session
decides; the runtime performs. That division is what lets the rules be tested
without a terminal, and it is the same division a supervisor and a compositor
will use when they become processes.

## Authoring

Draw in Penpot or Figma, export SVG, translate. The translator is a whitelist:
rectangles, lines, polylines, straight paths, translations, flat colours. It
refuses curves, text, rotation and opacity by name and line. It writes the same
document a person would type, and that document is the source of truth. A
visual editor can come later and write the same text; it is not a prerequisite.

The bounding is the point. An SVG file is an untrusted program in most systems;
here it is a bounded input to a total function. One mebibyte, four thousand
elements, four thousand points, and the document's own coordinate limits. The
translator cannot emit a document its parser would reject.

## The line is the design

No source file exceeds two hundred lines, no function is much longer than a
dozen. This is not tidiness. A file that fits in one read is a file that can be
held in a REPL variable, read whole by a reviewer, and delegated without a
summary. The size limit is what makes purity practical: small pure functions are
callable in one line, so they are explored instead of guessed at.

Comments carry reasoning, not narration: what a caller may rely on, why an
alternative was rejected, which invariant must hold, what will break if touched.
The lint is green or it is not; the budget is a script, not a matter of taste.

## What exists today

- `core`: geometry, frames, scenes, Cohen–Sutherland clipping, Bresenham lines,
  the document parser, the lease, focus, and the session reducer.
- `adapters`: a terminal encoder (two square pixels per cell, 24-bit colour), a
  painter, and the bounded SVG translator.
- `oko panel.oko [columns rows]` paints one frame. `oko-svg panel.svg` writes the
  document. Examples for both.
- 76 tests. The gate is `make check`: line budget, format, types, clippy, tests.

## What is deliberately absent

Text and a bundled font. Pointer input. Images, arcs, paths with curves,
transforms and opacity. The surface wire format between processes. The boot
console, getty handoff and the framebuffer presenters. A visual editor.

Each is a decision with an ADR, not a forgotten feature. The invariant they
serve is the one from SRD-002: the machine must always reach a usable first
screen and a login prompt without a browser, a GPU, a display server or a
network. Everything drawn above that line is optional, and a failed renderer
must return control to the console.
