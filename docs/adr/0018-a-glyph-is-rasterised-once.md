# 0018 — A glyph is rasterised once

Status: accepted · Date: 2026-10-23

## Context

ADR 0014 set a panel in a real face, and ADR 0013 draws each glyph on a
grid four times finer than the panel's and brings it down. Until now that
price was paid in full for every glyph on every frame: the outline was
read from the face with skrifa, the path cloned onto a fresh fine frame,
painted, and scanned back down — and nothing was kept. A frame of a
working panel holds over a thousand glyphs, most of them the same letters
pressed again.

The shell loop is single-threaded — read keys, run feeds, render,
present — so render cost is input latency. Measured on a 1920×1080 panel
set in Inter under full animation, the console's text load alone cost
about 583 ms per frame in a debug build; the panel ran about two seconds
behind the wall clock, and typing into a focused terminal fell behind
after a few characters and never caught up. The built-in face was
unaffected: its glyphs are a table lookup. The requirement behind this is
the one recorded on PR #1: a panel must stay efficient under heavy
animation, and text is where the frames go.

## Decision

**A glyph is rasterised once into a stamp, and the stamp is kept.**

- A **stamp** (`core::typeface::stamp`) is what one glyph paints: the
  pixels the fine-grid drawing sets, the offset of the first of them from
  the baseline point the glyph is pressed at, and the glyph's advance.
  It holds exactly the values the drawing passed to `Frame::set`, so
  pressing a stamp and drawing the glyph are the same picture by
  construction.
- Stamps are **kept in one cache** (`core::typeface::cache`), keyed by
  the face, the character, the em size, the weight, and the colour —
  everything that makes one stamp different from another. Painting a
  label presses stamps; a miss draws the glyph, makes its stamp, and
  keeps it.
- The cache is **capped** at 4096 stamps. When it is full it clears
  itself rather than growing without bound; a panel re-stamps its
  glyphs over the following frames. The policy is deliberately the
  simplest one that bounds memory.
- The old `draw` module is gone. Its two tests — a glyph lands on the
  frame, and it stands on its baseline — are carried into the stamp
  tests with their intent unchanged.

## Consequences

- The measured text load (23 lines of 45 characters, Inter at scale 3,
  debug build) falls from 17.50 s for 30 rounds to 0.21 s — about 81× —
  and the 1080p Inter panel that could not take typing now executes a
  typed `echo quadrant-ok` in its focused terminal. The full report is
  `docs/reports/a-glyph-is-rasterised-once.md`.
- The first frame pays the old price: the cache warms as glyphs are
  first drawn. A panel that changes size, colour, or font pays again
  for the new stamps, once each.
- Memory is bounded by the cap: 4096 stamps of at most a few kilobytes
  each.
- What this does not do: the scene is still rendered whole on every
  tick, and shapes are still repainted every frame. That is the other
  half of the efficiency requirement and remains open, as ADR 0015
  records for the presenter side. This decision removes the text cost,
  which was the half that made panels unusable.
