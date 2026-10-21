# Curves in the translator

The core learned a curve in the last slice. The translator was the last piece
that refused one: a path with a `C` in it was left out of the document and
named in the report. Now the grammar reads `C`, `S`, `Q` and `T`, absolute
and relative, and the shape becomes a `path` directive with the paints the
element states. A real export stops losing shapes to its curves: the four
curved paths in the Penpot export are drawn, and the logo appears.

## The interface the translator writes against

The grammar produces the segments the core already had, so the document keeps
the curve and only the renderer samples it:

```rust
// src/core/path.rs (landed, unchanged by this slice)
pub enum Segment {
    Move { to: Point },
    Line { to: Point },
    Cubic { control1: Point, control2: Point, to: Point },
    Quadratic { control: Point, to: Point },
    Close,
}
```

The document grammar is the one ADR 0005 fixed, and the translator writes it
exactly:

```
path M 47 59 C 45 56 42 54 34 54 C 33 54 33 54 33 54 ... stroke #ffffff
```

- `M` takes two numbers, `L` two, `C` six, `Q` four, `Z` none.
- `fill` and `stroke` name the paints, and a path must state one of them.
- A path may hold several shapes: `M ... Z M ... Z` is one directive.

## The modules

The path grammar is split by responsibility, not by length:

| File | Lines | What it owns |
|---|---|---|
| `src/adapters/svg/path/mod.rs` | 137 | the public surface: the segments, the straight outline, the offset |
| `src/adapters/svg/path/reader.rs` | 106 | the reader of numbers and command letters |
| `src/adapters/svg/path/state.rs` | 104 | where the path is, and the pixel grid it is on |
| `src/adapters/svg/path/curve.rs` | 67 | what a curve command means, and the reflection |
| `src/adapters/svg/path/grammar.rs` | 166 | the grammar: what each command means |
| `src/adapters/svg/path/tests.rs` | 139 | the public surface, seen from outside |
| `src/adapters/svg/path/curve_tests.rs` | 150 | the curve commands, seen from outside |

`state.rs` holds the one piece of state the smooth commands need: what the
command before this one was. `S` reflects the second control point of the
previous cubic, `T` the control point of the previous quadratic, and either
falls back to the current point when the command before was not that kind of
curve. `grammar.rs` decides what each command means; `curve.rs` is the
reflection and the two curve shapes.

## What changed, and what it refuses

- `C`, `c`, `S`, `s`, `Q`, `q`, `T`, `t` are read into segments.
- `H`, `h`, `V`, `v` become `Line` segments, absolute.
- A path with more than one shape is no longer refused: the document's path
  directive takes several shapes in one directive.
- `A` and `a` are still refused, and named: "the path uses `A`, which this
  console does not draw". An arc is the next slice, with its own tests.
- A path that is an axis-aligned rectangle is still a `rect` directive. One
  with a stroke of its own is a path, because a `rect` cannot carry a stroke.
- A filled path is filled by the core with the non-zero winding rule, so the
  refusal "the fill of this path is not an axis-aligned rectangle" is gone.

## Evidence: red, then green

The expectations that this slice legitimately changes were changed first, in
their own commit, and watched fail.

**`wip(red): test(svg): a curve is drawn and an arc is still refused`**

```
running 172 tests
adapters::svg::tests::a_curve_is_drawn_as_a_path_directive --- FAILED
adapters::svg::tests::a_curve_keeps_the_paints_the_element_states --- FAILED
adapters::svg::tests::a_triangle_is_filled_rather_than_refused --- FAILED

---- adapters::svg::tests::a_curve_is_drawn_as_a_path_directive stdout ----
assertion `left == right` failed
  left: "rect 1 1 2 2 #ff9d00\n"
 right: "path M 0 0 C 1 1 2 2 3 3 fill #000000\nrect 1 1 2 2 #ff9d00\n"

---- adapters::svg::tests::a_triangle_is_filled_rather_than_refused stdout ----
assertion `left == right` failed
  left: ""
 right: "path M 1 1 L 7 1 L 1 7 Z fill #ff9d00\n"

test result: FAILED. 169 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
```

What each old test asserted, and why it was wrong:

- `a_curve_is_left_out_and_the_rest_of_the_panel_is_drawn` asserted that a
  path with a `C` is left out of the document. ADR 0005 says a curve is a
  primitive the core draws, so the shape belongs in the document.
- `a_panel_with_a_hole_still_loads_and_reports_the_hole` and
  `the_command_writes_the_panel_out_and_the_report_away` used a curve as
  their hole. The hole they mean is a shape this console cannot draw, and
  after this slice that is an arc, so the hole is an arc now and the report
  names `A`.
- `every_shape_left_out_is_named_in_the_report` counted a curve as a hole.
  The hole is an arc now.

**Green now** — `cargo test -q --lib -- --test-threads=1`:

```
running 183 tests
test result: ok. 183 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The 13 new tests cover: an absolute cubic is read into a cubic segment; a
relative cubic accumulates from the current point; a smooth cubic reflects
the previous control point; a smooth cubic after a line uses the current
point; a smooth quadratic reflects the previous control point; a smooth
quadratic after a line uses the current point; a quadratic is read into a
quadratic segment; `H` and `V` become lines; a control point off the pixel
grid is rounded; a path with a curve has no straight outline; a path is
moved whole; a curve keeps the paints the element states; and a triangle's
fill is drawn.

Four tests changed rather than appeared, because the grammar now speaks in
segments: the straight-edge tests say what the segments are, the test that
named a curve by name now names an arc by name, and the test that reported
a second shape now keeps it.

## The whole-repo gate

```
$ make check
python3 tools/check_line_budget.py
0 exempt, 12 in the warning band, 0 frozen files grew, 0 over the ceiling
line budget OK
cargo fmt --all --check
cargo check --offline --jobs 1 --all-targets
cargo clippy --offline --jobs 1 --all-targets -- -D warnings
test result: ok. 183 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ echo $?
0
```

## The real export

`/tmp/penpot-export.svg` is the Penpot export of a panel: 515 by 255 pixels,
one logo made of two paths of curves, and two labels.

**Before** — `cargo run -q --bin oko-svg -- /tmp/penpot-export.svg`:

```
/tmp/penpot-export.svg: what could not be drawn:
  /tmp/penpot-export.svg:100: the clip-path `url(#frame-clip-...-render-1)` is not applied, ...
  /tmp/penpot-export.svg:100: the fill is the paint server `url(#fill-0-render-3-0)`
  /tmp/penpot-export.svg:100: the path uses `C`, which this console does not draw
  /tmp/penpot-export.svg:100: the path uses `C`, which this console does not draw
  /tmp/penpot-export.svg:100: the path uses `C`, which this console does not draw
  /tmp/penpot-export.svg:100: the path uses `C`, which this console does not draw
  /tmp/penpot-export.svg:100: the fill is the paint server `url(#fill-0-render-13-0)`
```

**After** — the same command:

```
/tmp/penpot-export.svg: what could not be drawn:
  /tmp/penpot-export.svg:100: the clip-path `url(#frame-clip-...-render-1)` is not applied, ...
  /tmp/penpot-export.svg:100: the fill is the paint server `url(#fill-0-render-3-0)`
  /tmp/penpot-export.svg:100: the fill is the paint server `url(#fill-0-render-13-0)`
```

The four `C` lines are gone. What is left is a clip path and two gradients,
which are the next holes and not this slice's.

The document the translator wrote, `/tmp/penpot2.oko`:

```
text 172 175 "in your notes" 4 #b0b0b0
text 343 66 "for Obsidian" 3 #b7b7b7
path M 47 59 C 45 56 42 54 34 54 C 33 54 33 54 33 54 C 33 58 33 71 33 75 C 33 75 33 75 34 75 C 42 76 45 79 47 81 M 47 59 C 49 56 52 54 60 54 C 60 54 61 54 61 54 C 61 58 61 71 61 75 C 61 75 60 75 60 75 C 52 76 49 79 47 81 M 47 59 L 47 81 stroke #ffffff
path M 60 57 L 65 57 C 65 57 66 57 66 58 L 66 80 C 66 80 65 81 64 80 C 62 80 60 79 57 79 C 51 79 47 82 47 82 C 47 82 42 79 37 79 C 34 79 31 80 30 80 C 29 81 28 80 28 80 L 28 58 C 28 57 29 57 29 57 L 33 57 stroke #ffffff
rect 51 53 16 30 #000000
```

The curves are in the document as curves. Nothing was flattened into lines.

## The shape, seen

`cargo run -q --bin oko -- /tmp/penpot2.oko 515 127`, with the colours
resolved (`#` is the white stroke, `o` is the grey label, `.` is the
background):

The logo, columns 24 to 70 and rows 25 to 43, **before**:

```
 25 |..............................................|
 26 |..............................................|
 27 |..............................................|
 28 |..............................................|
 29 |..............................................|
 30 |..............................................|
 31 |..............................................|
 32 |..............................................|
 33 |..............................................|
 34 |..............................................|
 35 |..............................................|
 36 |..............................................|
 37 |..............................................|
 38 |..............................................|
 39 |..............................................|
 40 |..............................................|
 41 |..............................................|
 42 |..............................................|
```

and **after**:

```
 25 |..............................................|
 26 |..............................................|
 27 |.........###########..........................|
 28 |....######..........##...##...................|
 29 |....#....#............###.....................|
 30 |....#....#.............#......................|
 31 |....#....#.............#......................|
 32 |....#....#.............#......................|
 33 |....#....#.............#......................|
 34 |....#....#.............#......................|
 35 |....#....#.............#......................|
 36 |....#....#.............#......................|
 37 |....#....#####.........#......................|
 38 |....#.........####.....#......................|
 39 |....#......######.####.#.##...................|
 40 |....#######......##########...................|
 41 |.......................#......................|
 42 |..............................................|
```

Before, the logo was not in the panel at all: the two paths that draw it
were the four refusals, and the one black rectangle in the document is black
on a black background. After, the lamp and the book are there, curved where
the source curves.

The label, which was always there, is still there:

```
 87 |....oooooooooooooooooooo....oooo............oooo............................oooo..|
 88 |....oooooooooooooooooooo....oooo............oooo............................oooo..|
 89 |....oooooooooooooooooooo....oooooooo........oooo............................oooo..|
 90 |............oooo............oooooooo........oooo............................oooo..|
 91 |............oooo............oooooooooooo....oooo............................oooooo|
```

## What I did not do

- **Arcs.** `A` and `a` are refused by name, as before. The conversion from
  an arc to cubics is the next slice, with its own tests.
- **The core.** `src/core/**` is untouched. The core's `Segment` was already
  what the grammar needed.
- **The export.** The Penpot SVG is not committed. It is a third-party file
  and it is not a fixture of this repo.

## What I deferred, and one finding

- `src/adapters/svg/mod.rs` is outside the scope of this slice, and its
  preamble still says the translator reads a "straight-line `path`" and leaves
  "a curve" out of the document. Both sentences are now wrong. It is a
  two-line fix in a file I was not allowed to write.
- `src/adapters/svg/rectangle.rs` asks the path module for a straight
  outline, and that question is now narrow: only a path that is one shape of
  straight edges has one. The old API (`path::outline` and `path::Path`) is
  kept for it. When rectangle.rs is next touched, the outline it needs can be
  read straight from the segments, and the old pair can go.
- `src/adapters/svg/emit.rs` is at 199 lines, one under the ceiling. The
  next change to it needs a seam, and the natural one is a module per kind
  of directive, which the next slice's scope should allow.

## The commits

```
9b8183d feat(svg): the translator reads a curve and writes a path
4451af7 wip(red): test(svg): a curve is drawn and an arc is still refused
6d1ad66 refactor(core): the curve arithmetic is its own module
```

The red commit is red on purpose and is marked `wip(red):` for that reason;
the commit after it is the one that passes the whole gate.
