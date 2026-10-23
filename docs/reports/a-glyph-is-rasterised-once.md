# A glyph is rasterised once

ADR 0014 gave panels real faces and ADR 0013 made each glyph dear: an
outline read from the font, drawn on a grid four times finer than the
panel's, and brought down. Testing a 1920×1080 Star Trek operations
panel set in Inter found the bill. Under full animation the panel ran
about two seconds behind the wall clock, and typing into its focused
console died after three to five characters — Enter never landed. The
identical panel in the built-in bitmap face typed flawlessly. Jim
directed the fix: *make the font path fast, test-first.*

## Why it was slow

Nothing in the text path was cached, at any level:

- `Face::glyph()` ran skrifa's outline extraction for every character
  of every label on every render.
- The draw then cloned the glyph's path, allocated a fresh 4× fine
  frame, painted the path into it, and scanned it back down — per
  glyph, per frame.
- The runtime re-renders the whole scene on every tick and every event,
  and the shell loop is single-threaded, so each typed character queued
  a full scene render behind the previous one. Single keys stayed
  usable (one render each); a typed line never caught up.

## What changed

- `core::typeface::stamp` (new): a glyph rasterised once — the exact
  pixels the fine-grid drawing sets, their offset from the baseline
  point, and the advance — pressed onto later frames with `blit`. The
  body of the old `draw` module moved here; `draw` is deleted and its
  two tests are carried into `stamp_tests` with their intent unchanged.
- `core::typeface::cache` (new): the stamps, keyed by face, character,
  em size, weight, and colour. Capped at 4096; a full cache clears
  itself rather than growing without bound.
- `core::text::named::paint`: cache-first. A hit presses the stamp; a
  miss draws the glyph once, stamps it, and keeps it. The space
  advance for a character the face lacks is now computed only when
  such a character appears, not once per label.
- ADR 0018 records the decision.

Written test-first, per the house rules: the cache tests, the stamp
tests, and a label-level test (`painting_a_label_again_reuses_the_
stamps_of_the_first_painting`) were watched failing against stubs —
four failures, all on the behaviour assertions — before the
implementation made them pass.

## Measurements

Debug builds throughout, this machine (rustc 1.98.1; the repo pins
1.99.1). The gate at the base of this branch was verified before the
change: 414 tests green at `40322aae`.

**Text load.** Painting 23 lines of 45 characters in Inter at scale 3
(the modern panel's console load, ~1,035 glyphs) for 30 rounds, same
temporary harness before and after:

| | 30 rounds | per round |
|---|---|---|
| before | 17.50 s | ~583 ms |
| after | 0.21 s | ~7 ms |

About **81×** on the load that was choking the loop.

**The panel itself.** The same 1920×1080 Inter panel, the same tmux
typing script (`f`, then `echo quadrant-ok` character by character,
then Enter), before and after:

- before: the input line stalls at `> echo_`; Enter never takes effect.
- after: `> echo quadrant-ok` executes, prints `quadrant-ok`, and a
  fresh prompt stands ready.

**CPU while animating**, sampled over 5 s at 1920×1080: 97% of one
core before, 86% after. The honest remainder: the scene is still
rendered whole each tick and shapes are still repainted each frame —
that half of the efficiency requirement stays open. What this change
removes is the text cost, which was the difference between a panel
that animates and a panel that can also be typed into.

## Gate

`make check` passes on the changed tree: 423 tests, 0 failed (412 unit
+ 10 acceptance + 1 meta; the change adds 11 and carries 2 forward
from the deleted `draw` tests), clippy clean with `-D warnings`,
`cargo fmt` clean, line budget OK (largest new file: `stamp.rs` at
194 lines, inside the ceiling).
