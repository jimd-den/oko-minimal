# Docs refresh 2 — the paper and ADRs 0002 and 0003 against the code

Files edited: `docs/whitepaper.md`, `docs/adr/0002-svg-in-document-out.md`,
`docs/adr/0003-svg-text-mapping.md`. No source file, test, tool, example or
other document was touched.

Every claim below was checked against the source at the commit this branch
carries (`52f7b4d feat(svg): a real export loads, and says what it could not
draw`), and the two real exports were run through the console's own binary.

## The commands

```
$ cargo test -q -- --test-threads=1
running 144 tests
....................................................................................... 87/144
.........................................................
test result: ok. 144 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 7 tests
.......
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**144 + 7 + 1 = 152 tests.** The number in the paper is 152.

```
$ make check
...
warn    src/adapters/svg/shapes.rs: 173 lines, ceiling 200
warn    src/adapters/svg/translator.rs: 162 lines, ceiling 200
warn    src/core/raster.rs: 165 lines, ceiling 200
warn    src/core/syntax.rs: 184 lines, ceiling 200

0 exempt, 4 in the warning band, 0 frozen files grew, 0 over the ceiling
line budget OK
...
running 144 tests
test result: ok. 144 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
...
test a_broken_panel_document_names_every_bad_line ... ok
test a_panel_document_becomes_terminal_blocks ... ok
test a_panel_with_a_hole_still_loads_and_reports_the_hole ... ok
test an_svg_export_becomes_a_panel_document ... ok
test an_svg_label_becomes_a_text_operation ... ok
test the_command_writes_the_panel_out_and_the_report_away ... ok
test the_escape_chord_hands_the_screen_back_to_the_console ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
...
test every_source_file_stays_within_the_line_budget ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s

Doc-tests oko

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

MAKE_EXIT=0
```

`make check` exits 0.

The same two commands were run again after the last edit to the three
documents, and produced the same result: 144 + 7 + 1 = 152 tests, and
`make check` exiting 0.

## The evidence behind the claims

Both real exports were downloaded and run through the console's own binary
(`target/debug/oko-svg`), so the paper's claims about a real export are
executed, not inferred.

```
$ ./target/debug/oko-svg /tmp/figma.svg > /tmp/figma.oko
/tmp/figma.svg: what could not be drawn:
  /tmp/figma.svg:55: the path uses `C`, which this console does not draw
  /tmp/figma.svg:56: the path uses `C`, which this console does not draw
  /tmp/figma.svg:1041: the fill is the paint server `url(#pattern0)`
exit 0

$ ./target/debug/oko-svg /tmp/penpot.svg > /tmp/penpot.oko
/tmp/penpot.svg: what could not be drawn:
  /tmp/penpot.svg:100: the clip-path `url(#frame-clip-a348dfa7-...)` is not applied, so this element may show more than the source drew
  /tmp/penpot.svg:100: the fill is the paint server `url(#fill-0-render-3-0)`
  /tmp/penpot.svg:100: the path uses `C`, which this console does not draw
  /tmp/penpot.svg:100: the path uses `C`, which this console does not draw
  /tmp/penpot.svg:100: the path uses `C`, which this console does not draw
  /tmp/penpot.svg:100: the path uses `C`, which this console does not draw
  /tmp/penpot.svg:100: the fill is the paint server `url(#fill-0-render-13-0)`
exit 0
```

The Figma file is 1235 operations, the Penpot file 3. The Figma export loses
two shapes to curves; the Penpot export loses four. Both exit zero and both
name what they left out.

## Claims changed, and the source each was checked against

### `docs/whitepaper.md`

| # | Line(s) | Old | New | Checked against |
|---|---|---|---|---|
| 1 | 74-83 | "It refuses curves, rotation, scale and opacity below one by name and line." | A curve, a rotation, a scale, a paint server, an element outside the whitelist and a character the built-in font does not have are each **left out of the document and named in the report** with the line of the export and the reason in words; the panel around it still loads. | `src/adapters/svg/translator.rs` (`skip`, `element`), `src/adapters/svg/outcome.rs` (`Outcome::Undrawn`), `src/adapters/svg/path/mod.rs` (`Path::Undrawn`), `src/adapters/svg/transform.rs` (`Transform::Undrawn`), `src/adapters/svg/text/run.rs` (`Label::Undrawable`), `src/adapters/svg/mod.rs` (`Translation`, `Skipped`). The old sentence was false: a curve is not a refusal. |
| 2 | 68-72 | (no mention) | The translator "reads an export as its tool wrote it": the entities XML defines are decoded, the basic colour names are read beside the digits, a fill inherited from a group is honoured, a fill hidden in a style attribute is found, and the elements that hold definitions are stepped over rather than refused. | `src/adapters/svg/entity.rs` (`decode`, the five named entities plus numeric references), `src/adapters/svg/names.rs` (the sixteen CSS names), `src/adapters/svg/paint/mod.rs` (`Inherited::stated` reads the `style` attribute when the attribute is absent; `Inherited::inherit` hands a group's fill down), `src/adapters/svg/elements.rs` (`STRUCTURAL`). |
| 3 | 80-82 | (no mention) | A translucent colour is left out and named, because the document format has no alpha; an `opacity` attribute is not read at all, and a shape that states one is drawn opaque. | `src/adapters/svg/paint/mod.rs` (`Paint::Unsupported` for a translucent colour), `src/adapters/svg/colour.rs` (`Colour::Translucent`). The `opacity` half was verified by execution: `<rect ... opacity="0.5"/>` translates to `rect 1 1 2 2 #ff0000` with no report. **See the finding below.** |
| 4 | 119 | "114 tests." | "152 tests." | `cargo test -q -- --test-threads=1` prints 144 + 7 + 1 = 152. The old number was two releases out of date. |
| 5 | 114-115 | "the bounded SVG translator, which reads labels as well as shapes." | "...which reads labels as well as shapes and reports the shapes it could not draw." | `src/adapters/svg/mod.rs` (`Translation.skipped`), `src/bin/oko-svg.rs` (`report`). |
| 6 | 116-118 | "`oko-svg panel.svg` writes the document." | "`oko-svg panel.svg` writes the document to standard output and the report to standard error." | `src/bin/oko-svg.rs` (`print!` for the document, `eprintln!` for the report), and the acceptance test `the_command_writes_the_panel_out_and_the_report_away`. |
| 7 | 87-89 | "the longest is the path grammar's reader, which reads a small language in one place" | "the longest is the one that reads a path, which reads a small language in one place" | The longest function is `outline` in `src/adapters/svg/path/mod.rs` (85 lines) — the path **grammar**, not the **reader** (`src/adapters/svg/path/reader.rs`, whose longest is `number` at 35). The old wording named the wrong file. |
| 8a | 28 | "`main` names a file and a terminal; nothing else does." | "`main` and the binaries name files and terminals; nothing else does." | `src/main.rs` and `src/bin/oko-svg.rs` both name a file; `src/bin/oko-svg.rs` was in the tree since the first commit. The old sentence was already false. |
| 8 | 123-125 | "Pointer input. Images, arcs, paths with curves, transforms and opacity." | "Pointer input. In the display list: images, arcs, curves, transforms and opacity." | The old sentence mixed the display list with the translator: a path with a curve is no longer refused by the translator, it is skipped, but the display list still has no curve primitive. `src/core/scene.rs` (`Op` has only `Clear`, `Rect`, `Line`, `Text`), `src/core/geom.rs` ("blending arrives with the `opacity` primitive"), `src/core/event.rs` (`Event` has only `Key` and `Resize`). The absent list itself is unchanged in substance: pointer input, images, arcs, curves, transforms, opacity, the surface wire format, the boot console, the framebuffer presenters and a visual editor are all still absent. |

### `docs/adr/0002-svg-in-document-out.md`

| # | Line(s) | Old | New | Checked against |
|---|---|---|---|---|
| 9 | 3-4 | "Text amended by `docs/adr/0003-svg-text-mapping.md`" | adds "`docs/adr/0004-draw-what-you-can-report-what-you-cannot.md`" | `docs/adr/0004-draw-what-you-can-report-what-you-cannot.md` exists and changed this ADR's output and failure model. |
| 10 | 30-32 | "Output: the existing panel document, one directive per line." | "Output: the existing panel document, one directive per line, **and the list of the shapes that were left out of it, each with the line of the export it was on.**" | `src/adapters/svg/mod.rs` (`Translation { document, skipped }`, `Skipped { line, reason }`). |
| 11 | 42-50 | "Curves, rotation, scale, opacity below one and unknown elements **are refused**, not approximated." | "Curves, rotation, scale, a translucent colour and unknown elements are **not approximated**. ADR 0004 separated the two kinds of failure: a file the translator cannot read is an error, and a shape it cannot draw is left out of the document and named in the report." | `src/adapters/svg/translator.rs` (a curve, an undrawable transform and an unknown element each call `skip`), `src/adapters/svg/error.rs` (a hard error carries a line and stops the translation). The old sentence described the all-or-nothing translator that no longer exists. |
| 12 | 34-41 | "Every refusal names a line." | "Every refusal names a line, **and so does every shape that was left out**." Also the bound now includes "4096 operations written". | `src/adapters/svg/mod.rs` (`Skipped.line`), `src/adapters/svg/emit.rs` (`Writer` counts operations and refuses at `MAX_OPS`), `src/core/document/mod.rs` (`MAX_OPS = 4096`). |
| 13 | 47-49 | "A paint reference such as `url(#gradient)` **is refused** by name" | "...**is named by the report** for the same reason" | `src/adapters/svg/paint/mod.rs` (`Paint::Unsupported` for a `url(...)` reference), `src/adapters/svg/shapes.rs` (`Outcome::Undrawn`), `src/adapters/svg/translator.rs` (`skip`). Verified by execution against the real Figma export: `the fill is the paint server `url(#pattern0)`` is in the report, and the file still exits zero. |
| 14 | 52-58 | "`oko-svg panel.svg > panel.oko` is the whole bridge." | Adds: "It returns a translation rather than a string: the document, and the shapes that were left out of it. The command writes the document to standard output and the list to standard error, so a pipe keeps only the panel and a person still sees what was lost." | `src/adapters/svg/mod.rs` (`pub fn translate(...) -> Result<Translation, SvgError>`), `src/bin/oko-svg.rs`. The old sentence was still true but incomplete: the pipe keeps the panel *and* the person still sees the report. |
| 15 | 73-79 | "A v1 design cannot use curves, rotation or transparency. ... Curves and transforms still need new display-list primitives." | Adds: "and until they exist a real export loses shapes to them and the report says which." | Executed against both real exports: the Figma export loses two shapes to `C` curves, the Penpot export four. |
| 16 | 83-85 | "The XML scanner refuses doctypes, and decodes the entities an export writes, and the translator refuses anything outside its whitelist." | "The reader refuses declarations, and decodes the entities an export writes, and the translator draws only what its whitelist names: anything else is left out of the document and reported." | `src/adapters/svg/scanner.rs` (`skip_noise` refuses a `<!` declaration; `text` decodes entities), `src/adapters/svg/tag.rs` (attribute values are decoded), `src/adapters/svg/elements.rs` (`is_drawn`). |
| 17 | 86-91 (new) | (no mention) | "The `opacity`, `fill-opacity` and `stroke-opacity` attributes are the one thing the translator does not read. A shape that states one is drawn opaque, and the report does not say so. This is the silent approximation every other rule here exists to prevent, and it is a gap rather than a decision: the earlier translator refused these attributes by name, and the redesign that read the file as a tree dropped the check without recording it." | **A finding, not a stale claim.** `grep -rn 'opacity' src/` finds no reader for the attribute. The old translator had `paint::require_opaque`, which refused `opacity`, `fill-opacity` and `stroke-opacity` by name; it is gone since `7858105`. Executed: `<rect fill="#ff0000" opacity="0.5"/>` becomes `rect 1 1 2 2 #ff0000`, drawn opaque, with nothing in the report. |
| 18 | 26-29 | "hex, `rgb()` and `rgba()` colours ... **full opacity**" | "opaque colours — hex, `rgb()`, an `rgba()` with full alpha, and the CSS colour names" | `src/adapters/svg/colour.rs` (`channels` returns `Translucent` when the alpha is below one, and `parse` names it). `rgba(0,0,0,0.5)` is not an opaque colour the translator draws. |
| 19 | 24-26 | "translate transforms" | "translate transforms and the `matrix()` that is a translation written the long way" | `src/adapters/svg/transform.rs` — a `matrix()` whose linear part is the identity is read as a `Shift`. A Penpot export writes a `matrix` on every group, so this is load-bearing. |
| 20 | 96-102 | "exports begin to use `<use>`, `<defs>` or stylesheets, which forces a symbol table and a second pass" | Adds: "the definitions and the stylesheet an export carries are stepped over today, and a `<use>` outside a definition is reported as an element the console does not draw" | `src/adapters/svg/elements.rs` (`STRUCTURAL` holds `defs`, `style`, `pattern`, the gradients, `clipPath`, `symbol`, `mask`, `filter`; `is_drawn` does not hold `use`, so a `use` outside a definition is reported). Both real exports carry `<defs>` and one carries a `<style>`. |

### `docs/adr/0003-svg-text-mapping.md`

| # | Line(s) | Old | New | Checked against |
|---|---|---|---|---|
| 21 | 3-4 | "Status: accepted · Date: 2026-10-21" | adds "· Text amended by `docs/adr/0004-draw-what-you-can-report-what-you-cannot.md`" | `docs/adr/0004-...` changed what a character outside the font does. |
| 22 | 8-13 | "The SVG translator, however, **refuses** `<text>` outright" | "...**refused** `<text>` outright" (past tense) | The translator reads text now; the Context section records the situation the decision answered, so it is in the past tense. |
| 23 | 25 | "The XML reader gains a **text event**." | "The XML reader gains a **text node**." | `src/adapters/svg/xml.rs` (`Node::Text`), `src/adapters/svg/scanner.rs` (the reader builds a tree, not a stream of events). |
| 24 | 32-33 | "A text **event** outside a `<text>` or a `<tspan>` element is an error" | "A text **node** outside ..." | As above. Verified by execution: text outside a `<text>` is still a hard error (`text is only supported inside a `<text>` or a `<tspan>`). |
| 25 | 35-38 | "Every other element keeps its current behaviour." | "...an element that holds definitions is stepped over, and one the console does not draw is left out of the document and named in the report rather than refused." | `src/adapters/svg/elements.rs` (`is_structural` vs `is_drawn`), `src/adapters/svg/translator.rs` (`element`). |
| 26 | 81-86 | "Every character must exist in the built-in font; a character that does not **is an error naming it**." | "...a character that does not **is left out of the document and named in the report**, because it is a shape the console cannot draw rather than a file it cannot read." | `src/adapters/svg/text/run.rs` (`Label::Undrawable`), `src/adapters/svg/label.rs` (`run_text` calls `skip` for it). The report `docs/reports/finish-redesign.md` records the same change to the test `a_character_outside_the_font_is_refused`. The old sentence was false: a character outside the font is not a hard error today. |
| 27 | 107-110 | "it cannot represent a label that the built-in font cannot draw" | "a label the built-in font cannot draw is left out of the document and named in the report, with the panel around it still loading" | As above. |

## Claims checked and found still true

- `core` is pure: no `std::fs`, no clock, no `static` in `src/core/` (grepped).
- Cohen–Sutherland clipping (`src/core/clip.rs`), Bresenham lines (`src/core/raster.rs`).
- A terminal encoder: two square pixels per cell, 24-bit colour (`src/adapters/terminal.rs`).
- One mebibyte, four thousand elements, four thousand path points (`src/adapters/svg/mod.rs::MAX_INPUT`, `src/adapters/svg/xml.rs::MAX_ELEMENTS`, `src/adapters/svg/path/mod.rs::MAX_POINTS`).
- The document's own limits: coordinates within ±32767, sizes within 16384, a label of at most 128 characters, a scale of at most eight (`src/core/syntax.rs`, `src/core/font.rs`).
- Lease, focus, session reducer, escape chord (`src/core/lease.rs`, `focus.rs`, `session.rs`).
- `oko panel.oko [columns rows]` and `oko-svg panel.svg` (`src/main.rs`, `src/bin/oko-svg.rs`); examples for both (`examples/panel.svg`, `examples/login.oko`, `examples/text-panel.svg`).
- A label's size is the authored size rounded to whole glyph heights, capped at eight, and it is drawn in uppercase (`src/adapters/svg/text/run.rs::scale`, `src/core/font.rs`).
- The document remains the source of truth; a visual editor is not a prerequisite (`docs/adr/0002`).
- The absent list is unchanged in substance: pointer input, images, arcs, curves, transforms, opacity, the surface wire format, the boot console, the framebuffer presenters, and a visual editor.

## The finding: `opacity` is read by nobody

The whitepaper used to say the translator "refuses curves, rotation, scale and
opacity below one by name and line". Curves, rotation and scale are now
skips, and the paper says so. **Opacity is neither a skip nor a refusal: the
attribute is not read at all.**

The old translator had `paint::require_opaque`, which read `opacity`,
`fill-opacity` and `stroke-opacity` and refused any value other than one.
That function is gone (`7858105` removed it while the translator was being
rebuilt to read a tree). Nothing replaced it. `grep -rn 'opacity' src/`
returns only the `rgba()` alpha path in `src/adapters/svg/colour.rs` and two
comments in `src/adapters/svg/style.rs`.

The consequence, executed:

```
$ ./target/debug/oko-svg /tmp/op.svg
rect 1 1 2 2 #ff0000
/tmp/op.svg: what could not be drawn:
  /tmp/op.svg:3: the fill `rgba(0,0,0,0.5)` is translucent, and this document format has no alpha
```

A shape with `fill="#ff0000" opacity="0.5"` is drawn fully opaque and the
report says nothing. A shape with `fill="rgba(0,0,0,0.5)"` is reported. The
two express the same thing, and the console treats them differently.

This is the silent approximation the project exists to refuse, so it is
recorded in ADR 0002 as a gap and in the paper as a fact. **I have not fixed
it**: the brief owns the documents, not the code. The fix is small — read the
three attributes and either report a translucent shape the way a translucent
colour is reported, or refuse the file the way the old translator did. It
wants a test first, per AGENTS.md §2.

## The paper's voice

The whitepaper was read whole before it was edited, and the edits keep its
voice: prose about a design, no dates, no "now also supports", no feature
bullet list. The new sentences in "Authoring" describe what the translator
does and what it reports, in the same register as the rest of the section.
The one bullet that changed in "What exists today" changed a count and a
clause; the list is still a list of what the code has.

## Note for the parent

- The test count is **152**, not 114.
- The `opacity` gap is the one thing here that is a code finding rather than a
  documentation finding. It is described in ADR 0002 and in the paper, and it
  is left unfixed for the code owner.
- ADR 0001 was not in scope, but it is now stale in the same way ADR 0002 was:
  it says the display list is "`Clear`, `Rect` and `Line`" and lists "Text,
  pointer input, images and transforms" as absent. `Text` arrived with the
  font. ADR 0001 is not mine to edit; flagging it for whoever owns it.
