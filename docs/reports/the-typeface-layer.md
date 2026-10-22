# The typeface layer

Worktree: `/home/violindocgfx/oko/oko-typeface`, branch `feature/typeface-finish`,
from `8172a27`. Commits: `8784e06` (the named tests), `1dfdf53` (the split).

## What the layer is now

`src/core/typeface/` reads a real font and hands the core its own path type.
The public API is exactly the one the parent fixed, and every public path is
unchanged: `core::typeface::{Face, Glyph, Setting, GRID}`, plus
`core::typeface::draw::draw`.

    mod.rs        39 lines   the preamble, the fine grid, the public names
    face.rs      158 lines   reading a font file: the faces, a face, a glyph
    pen.rs        68 lines   writing an outline into the core's path
    setting.rs    36 lines   the font a panel is set in
    draw.rs      147 lines   putting a glyph on a frame (was 144)
    pen_tests.rs  55 lines   the pen, from the inside
    typeface_tests.rs 107    the layer, from the outside

`mod.rs` was 249 lines, over the 200 ceiling, and the repo's own line budget
test said so. Before the split:

    OVER    src/core/typeface/mod.rs: 249 lines, ceiling 200 - name the seam, do not shuffle code

## The seam, and why

The seam is by behaviour, not by length.

- `face` reads a font file and says what a face knows: the faces the program
  carries, the character map, the advances, a capital height, a variable
  face's weight axis. It is the only file that talks to `skrifa`'s metadata.
- `pen` writes a font's outline into the core's `Path`. It is the only file
  that implements `skrifa::outline::OutlinePen`, and the only file that
  knows the font's y axis points up while the panel's points down.
- `setting` names a face for a panel and refuses a face the program does not
  carry. It is the only file that answers "which face, at which weight".
- `draw` puts a glyph on a frame, and is left where it was.

`mod.rs` keeps the preamble, the fine grid and the public names, so the
module is an index and nothing else. The split is a refactor: no behaviour
changed, the same test selection was green before and after.

The grid is now stated once. `draw.rs` took its own `const GRID: i32 = 4`
while the module said `pub const GRID: f32 = 4.0`; the two could have
drifted apart. `draw.rs` now takes its count from the module's `GRID`.

## The named tests, and the honest red

The six behaviours the parent named were already implemented at `8172a27`.
The new tests were therefore **green on arrival** - the behaviour was there,
the cover was not. No red was manufactured for them, and no assertion was
weakened to look busy.

    src/core/typeface_tests.rs
      a_capital_height_is_a_fraction_of_the_em        new
      a_setting_names_a_face_the_program_carries      new
      a_setting_refuses_a_face_the_program_does_not_carry   new
      two_settings_of_one_face_at_one_weight_are_equal     new
      two_settings_of_one_face_at_two_weights_are_not_equal new
      a_variable_face_is_different_at_two_weights     kept (already there)
      a_face_without_weights_takes_the_weight_it_was_drawn_with
                                                   kept, strengthened: the
                                                   whole glyph is compared now,
                                                   path and advance
    src/core/typeface/pen_tests.rs
      the_pen_writes_the_outline_into_the_core_path   new
      a_face_glyph_is_written_into_the_core_path      new

The one genuine red in this task was the repo's own rule, and it named the
file exactly: the line budget test was red because `mod.rs` was 249 lines.
The split is what turned it green for this module.

The capital heights the three faces report, for the record:

    dejavu-mono   0.7      (the face does not state one; the layer's fallback)
    source-sans-3 0.66
    inter         0.72753906

## The gate

    $ cargo test -q --lib -- --test-threads=1
    test result: ok. 343 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

343 lib tests, up from 336: five new tests outside, two new inside, one
strengthened.

    $ cargo test -q -- --test-threads=1
    running 343 tests
    test result: ok. 343 passed; 0 failed; ...
    running 8 tests
    test result: ok. 8 passed; 0 failed; ...
    running 1 test
    every_source_file_stays_within_the_line_budget --- FAILED

The whole-repo run is **not** green, and not because of this module:

    $ python3 tools/check_line_budget.py
    OVER    src/core/panel.rs: 218 lines, ceiling 200 - name the seam, do not shuffle code
    0 exempt, 19 in the warning band, 0 frozen files grew, 1 over the ceiling

`src/core/panel.rs` is the panel worker's file, not mine, and it is the only
file over the ceiling now. The typeface module contributes none. Proof, with
the panel worker's file copied out of the way in a scratch tree:

    $ python3 tools/check_line_budget.py --root /tmp/oko-typeface-budget-demo
    0 exempt, 18 in the warning band, 0 frozen files grew, 0 over the ceiling
    line budget OK

    $ cargo clippy --all-targets -- -D warnings
    (silent)

    $ cargo fmt --all --check
    Diff in .../src/core/document/mod.rs:33:

`cargo fmt --all --check` is also not silent, and again not because of this
module: `src/core/document/mod.rs` wants two `use` lines the other way round.
That is pre-existing at the base commit - `git stash` and `cargo fmt --all
--check` fails the same way on `8172a27` - and it is in the document
worker's file, which the brief told me not to edit. Every file this task
touched is clean:

    $ rustfmt --edition 2024 --check src/core/typeface/*.rs src/core/typeface_tests.rs
    (silent)

## Deferred, and what I found

- **ADR 0014 does not exist.** `cap_ratio`'s contract refers to it ("a face
  drawn to the same capital height is drawn at the same size as the built-in
  font at the same scale (ADR 0014)"), and `docs/adr/` stops at 0013. The
  number is reserved and the decision is not written down. `docs/adr/` was
  outside this task's scope, so it is reported, not written.
- **The capital height fallback is a guess, and it is deliberate.**
  `dejavu-mono` states no cap height, so the layer draws it as if its
  capitals were seven tenths of the em - the built-in bitmap font's own
  proportion. The real ratio is 1493/2048, about 0.729. Measuring it from
  the outline of a capital would be better, but it would change the size the
  panel draws that face at, and the fallback looks like a decision the
  built-in font already made. Left as it is, on purpose.
- **No shaping.** No kerning, no ligatures, no hinting; `skrifa` is asked for
  unhinted outlines. A character the face does not have is `None`, and the
  panel says so.
- **The variable face's cap height does not move with the weight.** `inter`
  states one cap height for the whole axis range, so `cap_ratio(Some(400))`
  and `cap_ratio(Some(700))` agree. The glyphs do differ; the ratio does not.
