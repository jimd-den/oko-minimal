# Curves in the core

A curve is now a primitive: the document can say `C`, the display list can
carry it, and the renderer samples it and draws it. The document keeps the
curve; only the renderer flattens it, and only at paint time.

## The interface the SVG side writes against

```rust
// src/core/path.rs
pub enum Segment {
    Move { to: Point },
    Line { to: Point },
    Cubic { control1: Point, control2: Point, to: Point },
    Quadratic { control: Point, to: Point },
    Close,
}
pub struct Path { pub segments: Vec<Segment> }

// src/core/scene.rs
Op::Path { path: Path, fill: Option<Rgba>, stroke: Option<Rgba> }
```

The document grammar (absolute coordinates only, one directive per line):

```
path M 10 10 C 20 0 30 0 30 10 Z fill #ff9d00
```

- `M` takes two numbers, `L` two, `C` six, `Q` four, `Z` none.
- `fill #rrggbb` and `stroke #rrggbb` name the paints; at most one of each.
- A path must state **at least one paint**, must have **at least one
  segment**, and must **start with `M`**. A `Z` must follow a shape.
- Anything else in the line is refused: `unknown path command `A``.

Note: the example in the brief, `path M 10 10 C 20 0 30 10 Z fill #ff9d00`,
is two numbers short — a cubic needs six coordinates — so it is refused with
`` `Z` is not a number ``. With six numbers it parses.

## Evidence: red, then green

Every behaviour below was seen fail before it was made to pass.

**1. Sampling a path** (`src/core/path.rs`, stub returning an empty polyline):

```
running 152 tests
core::path::tests::a_cubic_bulges_past_its_chord --- FAILED
core::path::tests::a_large_curve_is_smoother_than_a_small_one --- FAILED
core::path::tests::a_line_is_sampled_as_its_two_ends --- FAILED

---- core::path::tests::a_cubic_bulges_past_its_chord stdout ----
thread 'core::path::tests::a_cubic_bulges_past_its_chord' panicked at
src/core/path.rs:97:9:
a cubic is sampled into points

test result: FAILED. 149 passed; 3 failed; 0 ignored
```

**2. Filling a shape** (`src/core/paint.rs`, `spans` returning no spans):

> The red run below names `src/core/fill.rs` and `core::fill::tests`.
> The module was called `fill` at that moment and was renamed `paint`
> once it also held the stroke. The tests are the same tests.


```
core::fill::tests::a_triangle_covers_its_interior_and_not_its_outside --- FAILED
thread 'core::fill::tests::a_triangle_covers_its_interior_and_not_its_outside'
panicked at src/core/fill.rs:68:9:
the interior is covered

test result: FAILED. 154 passed; 1 failed; 0 ignored
```

**3. The renderer drawing a path** (`src/core/raster.rs`, the arm a no-op):

```
core::raster::tests::a_path_paints_its_interior --- FAILED
assertion `left == right` failed
  left: Some(Rgba { r: 0, g: 0, b: 0, a: 255 })
 right: Some(Rgba { r: 255, g: 255, b: 255, a: 255 })

test result: FAILED. 157 passed; 1 failed; 0 ignored
```

**4. The document grammar** (`src/core/document/path.rs`, a stub):

```
core::document::path::tests::a_path_directive_becomes_a_path_operation --- FAILED
thread '...' panicked at src/core/document/path.rs:27:68:
a path: "not written yet"

test result: FAILED. 160 passed; 1 failed; 0 ignored
```

**Green now** — `cargo test -q --lib -- --test-threads=1`:

```
running 170 tests
test result: ok. 170 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The 21 new tests cover: a triangle covers its interior and not its outside; a
cubic bulges past its chord; a large curve is sampled more finely than a
small one; a line samples as its two ends; a self-overlapping shape fills
under the non-zero rule (the overlap is filled, which the even-odd rule
would leave empty); a filled shape paints its interior; a stroked path
paints its edge and not its inside; a path directive parses; a path may
state a stroke and no fill; a quadratic command makes a quadratic segment;
an unknown command, a path with no paint, a path with no segments, a path
that does not start with `M`, a `Z` with no shape, and a path that states a
paint twice are all refused; and a path in a document reaches the frame.

## The whole-repo gate

```
$ make check
python3 tools/check_line_budget.py
0 exempt, 10 in the warning band, 0 frozen files grew, 0 over the ceiling
line budget OK
cargo fmt --all --check
cargo check --offline --jobs 1 --all-targets
cargo clippy --offline --jobs 1 --all-targets -- -D warnings
test result: ok. 170 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ echo $?
0
```

## The shape, seen

A panel written by hand — `/tmp/curve.oko`:

```
# a panel with a curve in it
clear #05070a
path M 2 8 C 8 1 18 1 24 8 C 18 15 8 15 2 8 Z fill #ff9d00
path M 26 8 C 30 3 36 3 38 8 stroke #1f6feb
```

```
$ cargo run -q --bin oko -- /tmp/curve.oko 40 12
```

The first lines of the output, raw:

```
'\x1b[38;2;5;7;10m\x1b[48;2;5;7;10m▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀\x1b[0m'
'\x1b[38;2;5;7;10m\x1b[48;2;5;7;10m▀▀▀▀▀▀▀▀▀\x1b[38;2;5;7;10m\x1b[48;2;255;157;0m▀▀▀▀▀▀▀▀\x1b[38;2;5;7;10m\x1b[48;2;5;7;10m▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀\x1b[0m'
'\x1b[38;2;5;7;10m\x1b[48;2;5;7;10m▀▀▀▀▀\x1b[38;2;5;7;10m\x1b[48;2;255;157;0m▀\x1b[38;2;255;157;0m\x1b[48;2;255;157;0m▀▀▀▀▀▀▀▀▀▀▀▀▀▀\x1b[38;2;5;7;10m\x1b[48;2;255;157;0m▀\x1b[38;2;5;7;10m\x1b[48;2;5;7;10m▀▀▀▀▀▀▀▀\x1b[38;2;5;7;10m\x1b[48;2;31;111;235m▀▀\x1b[38;2;31;111;235m\x1b[48;2;5;7;10m▀▀▀\x1b[38;2;5;7;10m\x1b[48;2;31;111;235m▀▀▀\x1b[38;2;5;7;10m\x1b[48;2;5;7;10m▀▀▀\x1b[0m'
```

The same frame, with the colours resolved (`#` is the orange fill, `+` is the
blue stroke, `.` is the background):

```
........................................
........................................
........................................
.........########.......................
......##############...........+++......
.....################........++...+++...
...####################.....+........+..
..######################...+.........+..
..######################..+...........+.
...####################.................
.....################...................
......##############....................
.........########.......................
........................................
........................................
........................................
........................................
........................................
........................................
........................................
........................................
........................................
........................................
........................................
```

## Decisions this brief did not settle

1. **The sampling rule.** One step per two pixels of the curve's *control
   polygon*, clamped to between 4 and 64 steps. The control polygon is used
   because its length bounds the curve's length from above, and measuring
   the real length would cost more than the extra steps are worth. A small
   curve is therefore cheap and a large one is smooth. Written down in
   `core::path::sample`.
2. **How a stroke is drawn.** One pixel wide, along the *sampled* polyline,
   so a curve is stroked as the curve and not as the chords between its
   samples. The width is one because the ADR defers wider strokes. The
   sampled points are rounded to the pixel they land on before the line is
   walked.
3. **The fill rule and the closing edge.** Non-zero winding, as SVG fills
   by. A shape that states a fill is filled as if it were closed, whether or
   not it says `Z` — SVG's rule, and the ADR's.
4. **What "closed" means in the sampled polyline.** A closed shape repeats
   its first point at the end. The closing edge is then drawn by the
   ordinary rule, and an open shape is not closed by accident. It also keeps
   the sampled path one flat list of lists.
5. **A path that continues after a `Z`** starts a new shape at the point
   the closed shape began, rather than drawing on into the closed one.
6. **The paints may come anywhere in the line**, though the grammar and the
   examples put them last. At most one `fill` and one `stroke`; a second one
   is refused rather than silently overriding the first.
7. **Coordinates are whole pixels**, as everywhere else in the document, and
   are range-checked to ±32767 like every other coordinate. The *sampling*
   keeps fractions, so a curve is rounded to the grid only when it is
   painted and a small shape is not snapped out of existence.
8. **The stroke is painted after the fill**, so a stroke on a filled shape
   is visible.

## Where I went outside the file list, and why

The brief listed the files I might write. Three files are outside it. The
line ceiling is mechanical, and the work did not fit in the listed files
without either exceeding 200 lines or splitting a module at a line range.
The seam I chose instead is by responsibility:

- **`src/core/paint.rs` (new).** The path painter: the non-zero winding
  fill and the stroke. It is the part of drawing a path that is genuinely
  new; the ADR says so, and a rectangle never needed it.
- **`src/core/frame.rs`.** Gained `Frame::line`: the Bresenham walker that
  used to live in `raster.rs`. It moved because both the `line` operation
  and the path stroke need it, and `raster.rs` had no room for both. The
  behaviour is unchanged: the line is still cut to the frame before it is
  walked.
- **`src/core/geom.rs`.** Gained `Sample` (a point in fractions of a pixel)
  and `evaluate` (a point on a Bézier of degree one, two or three). The
  curve arithmetic is geometry, and `core::path` had no room for it beside
  the segment types, the sampling and the tests.

Nothing else moved. The terminal encoder, the font, the lease, focus and
the session reducer are untouched, and no existing test was weakened.
