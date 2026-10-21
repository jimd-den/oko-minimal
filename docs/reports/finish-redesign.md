# Finishing the translator redesign

The branch `wip/real-export-crashed` carried a large redesign of the SVG
translator: a scanner that builds a tree instead of an event stream, an
`Outcome` for the fate of each shape, and a `Translation` that carries the
document and the list of what was left out. It did not compile. This report
records what was broken, what was decided, and what the finished branch does
with two real exports.

The design the branch implements is `docs/adr/0004-draw-what-you-can-report-what-you-cannot.md`:
a hard error is a file that cannot be read, and a skip is a shape that cannot
be drawn. The evidence about what real exports contain is
`docs/reports/svg-export-shapes.md`.

## What was broken, and what each fix was

### 1. `src/adapters/svg/mod.rs` — the root test module was declared and gone

The branch moved the translator-level tests into per-module files
(`paint/tests.rs`, `path/tests.rs`, `text/tests.rs`) and left the declaration
`mod tests;` in `mod.rs` behind. The file did not exist, so the crate did not
compile at all.

The translator-level tests were not merely lost — they were the only place
the whole pipeline was tested. They now live in `src/adapters/svg/tests.rs`,
and they are the tests that carry the ADR's contract: a group moves its
children, a filled path becomes one rectangle, an SVG text becomes one text
directive, and the panel the translator writes is a document the parser
accepts. The file is new; it did not exist in any form on the branch.

### 2. `src/adapters/svg/translator.rs` — `self.run(run, scope)`

`run` is the entry point that takes a root element and returns a
`Translation`. The method that handles one run of text is `run_text`, and it
lives in `label.rs` beside the rest of the text handling. The call now names
`run_text`.

### 3. `src/adapters/svg/translator.rs` — `at(&self)` calling `shift(&mut self)`

`shift` records a skip when a transform is one the console does not draw, so
it needs `&mut self`. `at` therefore needs `&mut self` too, and so do its
callers (`group`, `shape`, and the text handling in `label.rs`).

### 4. `src/adapters/svg/paint/tests.rs` — `Option<Paint>` against `Paint`

`Inherited::stated` returns `Option<Paint>`, because a paint may be absent
from an element entirely. The three assertions compared it against a
`Paint`, so they were updated to the value the API returns.

### 5. `src/adapters/svg/path/tests.rs` — a fixed array against a `Vec<Point>`

`Outline.points` is a `Vec<Point>`; the assertions built a fixed array. The
three assertions now build a `Vec`.

### 6. `src/adapters/svg/text/tests.rs` — a `&str` against a `Translation`

`translate` returns a `Translation`. The assertions now read its `document`.
A helper does the unwrapping so each test reads as one assertion.

### Beyond the six: two tests that encoded the contract ADR 0004 replaced

These are the decisions the brief asked to be named, because they are the
ones where a test was changed rather than merely updated.

- `an_svg_curve_is_refused_with_its_line` (acceptance) asserted that a
  curve is a hard error. Under ADR 0004 a curve is a shape that cannot be
  drawn: the panel around it still loads, and the report names the curve
  with the line it was on. The test now says that, and it is the acceptance
  test the brief asked for: a panel with a hole still loads and reports the
  hole.
- `a_character_outside_the_font_is_refused` (text) asserted that a character
  the built-in font does not have is a hard error. Under ADR 0004 it is a
  skip, for the same reason: the label is left out, and the report names the
  character.

Neither test was weakened. Each now asserts strictly more than before: the
document is produced, the hole is named, and the line is checked.

### Beyond the six: the command wrote the wrong thing to the wrong stream

`oko-svg` printed the `Translation` with `print!`, which no longer compiles
because `Translation` is not a `Display`. The command now writes the
document to standard output and the report to standard error, and exits zero
when the file was read. A file that cannot be read at all is still an error
and still exits non-zero.

The report header is `what could not be drawn:` and each line under it is
`path:line: reason`. The count of shapes was left out of the header on
purpose: the list holds two kinds of entry — a shape that was left out, and
an attribute the console did not apply (a `clip-path`, for instance) — and a
single count would misdescribe the second kind.

### Beyond the six: `elements.rs` was dead code, and `DRAWN` was the wrong seam

The branch's `elements.rs` listed the elements the translator draws in
`DRAWN`, and the translator matched on the element name itself. The two
lists could drift, and `DRAWN` was never read. The translator now asks
`elements::is_drawn` and `elements::is_structural` before it matches, so
the list in `elements.rs` is the one that decides.

### Beyond the six: `paint/mod.rs` declared its test module unconditionally

`mod tests;` without `#[cfg(test)]` compiles the test file into the library
build, which produced three unused-import warnings. It now carries
`#[cfg(test)]`, like `path/mod.rs` and `text/mod.rs` already did.

## Decisions the ADR and the branch did not settle

1. **A shape that is not drawn is reported once, at its own line.** A path
   with a curve is reported as a curve, not as "a curve and a fill the
   console cannot draw". The reason a shape is reported is the first thing
   that stopped it being drawn.
2. **An attribute the console does not apply is reported like a shape.** A
   `clip-path` the console ignores is a hazard — the panel may show more
   than the source drew — and it is said so in the report. This is why the
   report header names no count.
3. **A test file per module, in the branch's own pattern.** `colour.rs` and
   `scanner.rs` were split into a module file and a `tests.rs` sibling, the
   pattern `paint/`, `path/` and `text/` already use. Both files were in the
   warning band (over 160 lines) and both grew, so the split was done at
   this seam rather than left to the next change.
4. **`Scope` moved out of `translator.rs` into `scope.rs`.** The inherited
   context — the offset, the paints, the style and the colour an element
   hands down — was in the translator, but two modules read it: the
   translator for shapes and `label` for text. Moving it to its own module
   is a split by responsibility rather than by length, and it takes
   `translator.rs` from 185 lines to 162.
5. **`transparent` is a translucent colour, not an opaque black.** This was
   found by merging `main`, which names `transparent` with an alpha of zero
   and whose colour reader refuses a colour it cannot draw opaque. The
   branch's `colour.rs` had no such rule, and would have drawn a transparent
   fill as solid black — the silent wrong colour the research warns about.
   The test `a_transparent_colour_is_reported` was written first and failed
   against the branch's reader; the reader now reports it as translucent.

## The proof: two real exports

Both files were downloaded and run through `cargo run -q --bin oko-svg --`.
Both exit zero, both write a document to standard output, and both name what
they left out on standard error.

### Figma

`https://raw.githubusercontent.com/ttskch/svg-paper/master/js/tests/resources/real-world-paper-figma.svg`

The document is 1235 operations: 204 rectangles, 812 lines and 219 labels.
The report names three shapes:

```
figma.svg: what could not be drawn:
  figma.svg:55: the path uses `C`, which this console does not draw
  figma.svg:56: the path uses `C`, which this console does not draw
  figma.svg:1041: the fill is the paint server `url(#pattern0)`
```

The two curves are the two rounded rectangles in the export; the ADR
predicted exactly two curves in this file. The paint server is the logo
tile.

### Penpot

`https://raw.githubusercontent.com/Floydv149/bibleLinkerPro/master/images/dark.svg`

(The URL the brief offered first, `bible-linker-pro`, returns 404; this is
the one that resolves.)

The document is three operations: two labels and one rectangle. The report
names seven shapes, and the four curved paths are among them:

```
penpot.svg: what could not be drawn:
  penpot.svg:100: the clip-path `url(#frame-clip-...)` is not applied, so this element may show more than the source drew
  penpot.svg:100: the fill is the paint server `url(#fill-0-render-3-0)`
  penpot.svg:100: the path uses `C`, which this console does not draw
  penpot.svg:100: the path uses `C`, which this console does not draw
  penpot.svg:100: the path uses `C`, which this console does not draw
  penpot.svg:100: the path uses `C`, which this console does not draw
  penpot.svg:100: the fill is the paint server `url(#fill-0-render-13-0)`
```

Every shape is on line 100 because the export writes its whole body on one
line. The line number is honest, and it is the only line there is.

The two labels that are drawn come out in the right colour — `#b0b0b0` and
`#b7b7b7` — which the research
(`docs/reports/svg-export-shapes.md`, section (e)) recorded as wrong in the
previous translator. The two labels that are not drawn are the two whose
fill is a gradient; the console cannot draw a gradient, so it names the
reference and leaves the label out.

## The gate

```
cargo build --all-targets      no errors
cargo test -q -- --test-threads=1   144 + 7 + 1 passed, 0 failed
cargo fmt --all                 clean
cargo clippy --all-targets -- -D warnings   clean
make check                      exit 0
```

The branch had 132 `#[test]` functions before this work; it has 152 now.
