# The presenter paints only what changed

A second tester pushed this branch's panels until the presenter became
the story, and Jim directed the fix: *the presenter must be efficient
at heavy animations and must not waste memory on the parts that do not
need to be animated.* The workload is named, not hypothetical: complex
sweeping data animations on bare-bones computers. This change is the
presenter half of that requirement. ADR 0014 records the decision.

## What changed

- `presenter::screen_update(previous, frame)` - the bytes that bring a
  screen from one frame to the next: each run of changed cells, placed
  at its address, in the frame's own colours. Identical frames update
  to zero bytes. A size change falls back to a whole `screen` painting.
  The cell geometry `rows` used privately was extracted as `cells`, so
  the full painter and the update painter read a frame the same way;
  `encode` and `screen` are byte-for-byte unchanged.
- The shell remembers the frame on the screen and presents updates
  against it. The first frame is painted whole.
- The sample panels the testing produced are in `examples/`:
  `lcars-bridge`, `mu-th-ur`, `lcars-computer`, `lcars-1080p`, and
  `finance-terminal` - the last is a working ledger whose figures
  reconcile, with a scan line that sweeps the table.

## A bug this found

The shell painted the loaded frame *before* entering the alternate
screen. The alternate screen starts blank, and a panel with no motion
is never repainted (ADR 0012), so a still panel was invisible on a real
terminal until a key or an edit forced a repaint. Byte-counting tests
cannot see this; a terminal emulator can. Evidence, before the fix:
`tmux` running the shell on a static panel captured twelve blank lines.
After: the panel, present, colours and all. The fix is ordering - the
first painting now happens after the display is taken - and it is the
same move that makes update state honest: the remembered frame must be
the frame on *this* screen.

## The evidence

Written test-first (red against a fake that returned the full screen,
then green): identical frames update to nothing; an update paints only
the changed cell, at its address; unchanged rows are not addressed; a
size change is the full screen. `make check` green - 321 unit tests
(317 before), 8 acceptance, 1 meta, line budget OK.

Measured live, real program, pseudo-terminal, whole load timeline on
the wire, before → after:

| Panel | Before | After |
|---|---|---|
| finance-terminal, 1920×540 | ~100MB (~19MB/s) | **4.29MB** |
| lcars-1080p, 1920×540 | ~19MB/s sustained | **4.27MB total** |
| lcars-computer, 256×60 | ~1.7MB/s | **0.18MB total** |

And a single flag flip at 1080p - opening the budget view, which
repaints most of the stage - fell from 7.07MB to 2.25MB: an update
costs the change, and a full-stage view is most of a change.

## What this does not do

The runtime still renders every frame whole; only the painting is
diffed. On a slow processor the raster, not the wire, now caps the
frame rate - at 1080p the old code managed ~5 frames per second, and
this change does not claim to raise that ceiling, only to stop paying
for it in bytes. Scene-side damage (not rendering what did not move)
is the follow-up, and the core is where it would have to live.
