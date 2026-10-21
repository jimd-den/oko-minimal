# ỌKỌ — the screen as a contract

## The claim

A display server is a broker between programs and pixels. On a machine whose
first screen is a login prompt, the broker is a liability: it needs a GPU, a
session bus, a graph of processes, and a reason to exist before anyone has
logged in.

ỌKỌ removes the broker and keeps the contract. The contract is a **display
list**: an ordered set of rectangles, lines and text, rendered into a frame by a
pure function. A program that wants to paint produces a list. The console owns the
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
export. `main` and the binaries name files and terminals; nothing else does.

The line between them is the display list. A `Scene` is a short sequence of
elements, and an element is a shape, a motion and a condition. A `Frame` is an
opaque grid of pixels. `render` is the only function that turns one into the
other. A `document` is the text a person edits; it parses into a `Panel` - a
scene, a set of flags and a set of bindings - and a bad document changes
nothing.

## Why a display list, not a tree

Instrument panels are made of rules, bars, frames and large type. A document
layout engine is the wrong shape for that, and a retained scene graph is state
that every reload must synchronise. A flat list is cheaper to produce, compare,
serialise and discard. It is also the unit of hot editing: parse the file, get a
list, swap it, redraw. Nothing else in the system needs to know that the file
changed.

The list will grow: images, clipped groups, transforms. Each addition
needs a primitive, a renderer case, and a test. The order is deliberate. A
primitive that no test demanded is a guess, and guesses become compatibility
burdens.

## States and events

A screen's *authority* is not a function of time. It is a function of state,
and state changes when events arrive. A screen's *content* may be a function
of time - a shape may have legs, and the panel asks the scene what it looks
like at this moment - but the right to present is never a function of the
clock. Two state machines guard the screen:

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
rectangles, lines, paths with curves and arcs, translations, flat colours, and
text with its spans. It reads an export as its tool wrote it: the entities XML
defines are decoded, the basic colour names a browser has always known are
read beside the digits, a fill inherited from a group is honoured, a fill
hidden in a style attribute is found, and the elements that only hold
definitions are stepped over rather than refused.

It draws what it can and reports what it cannot. A rotation, a scale, a paint
server, an element outside the whitelist, a character the built-in font does
not have: each of these is left out of the document, and each is named in the
report with the line of the export it was on and the reason in words. A curve
is not among them: the document keeps the curve, and only the renderer asks it
for its points. The panel around it still loads. A label is drawn by the
built-in font, so its size is the authored size rounded to a whole number of
glyph heights, capped at eight, and it is drawn in uppercase. A translucent
colour is left out and named, because the document format has no alpha; an
`opacity` attribute is not read at all, and a shape that states one is drawn
opaque. It writes the same document a person would type, and that document is
the source of truth. A visual editor can come later and write the same text;
it is not a prerequisite.

The bounding is the point. An SVG file is an untrusted program in most systems;
here it is a bounded input to a total function. One mebibyte, four thousand
elements, four thousand points, and the document's own coordinate limits. The
translator writes the format's own limits into its output — coordinates, sizes,
labels, scales, and the count of operations — so what it writes is what the
parser accepts. One element can become many directives, so the writer counts
them and refuses the panel that would need more than the format allows.

## The line is the design

No source file exceeds two hundred lines, and most functions are a dozen lines
or fewer; the longest reads a path's `d` attribute, which is a small language
in one place. This is not tidiness. A file that fits in one read is a file that can be
held in a REPL variable, read whole by a reviewer, and delegated without a
summary. The size limit is what makes purity practical: small pure functions are
callable in one line, so they are explored instead of guessed at.

Comments carry reasoning, not narration: what a caller may rely on, why an
alternative was rejected, which invariant must hold, what will break if touched.
The lint is green or it is not; the budget is a script, not a matter of taste.

## What exists today

- `core`: geometry, frames, scenes, Cohen–Sutherland clipping, Bresenham lines,
  curves and arcs, the five-by-seven bitmap font and the text it draws, the
  document parser, the panel with its flags and bindings, the lease, focus, and
  the session reducer.
- `adapters`: a terminal encoder (two square pixels per cell, 24-bit colour), a
  painter, a runtime that is a loop with no device in it, and the bounded SVG
  translator, which reads labels as well as shapes and reports the shapes it
  could not draw.
- `oko panel.oko` presents a panel and reloads it when the file changes;
  `oko panel.oko --once` paints one frame and stops. `oko-svg panel.svg` writes
  the document to standard output and the report to standard error. Examples for
  all of them.
- 323 tests. The gate is `make check`: line budget, format, types, clippy, tests.

## What is deliberately absent

Pointer input. In the display list: images, transforms and opacity. A panel
that loops, fades, or reads an export's own animation. Shaping, kerning and
hinting of real fonts. The surface wire format between processes. The boot
console, getty handoff and the framebuffer presenters. A visual editor.

Each is a decision with an ADR, not a forgotten feature. The invariant they
serve is the one from SRD-002: the machine must always reach a usable first
screen and a login prompt without a browser, a GPU, a display server or a
network. Everything drawn above that line is optional, and a failed renderer
must return control to the console.
