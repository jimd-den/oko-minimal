# Docs refresh — whitepaper and ADR 0002

Files owned and edited: `docs/whitepaper.md`, `docs/adr/0002-svg-in-document-out.md`.
No source file, test, tool or other document was touched.

## Claims changed, and the source each was checked against

### `docs/whitepaper.md`

| # | Line | Old | New | Checked against |
|---|---|---|---|---|
| 1 | 11-12 | "an ordered set of rectangles and lines" | "an ordered set of rectangles, lines and text" | `src/core/scene.rs` — `Op` has `Clear`, `Rect`, `Line`, `Text` |
| 2 | 34-35 | "`Clear`, `Rect` and `Line`" | "`Clear`, `Rect`, `Line` and `Text`" | `src/core/scene.rs` — `enum Op` |
| 3 | 45 | "The list will grow: text, images, clipped groups, transforms." | "The list will grow: images, clipped groups, transforms." | `src/core/scene.rs` — text is present, so it is no longer a future item |
| 4 | 66-72 | translator "refuses curves, text, rotation and opacity" | refuses "curves, rotation, scale and opacity below one"; text is in the whitelist, and a label's size is the authored size rounded to whole glyph heights, capped at eight, drawn uppercase | `src/adapters/svg/translator.rs` (`"text" \| "tspan"` accepted), `src/adapters/svg/values.rs::translate` (only `translate(...)`), `src/adapters/svg/paint.rs::require_opaque` (opacity below one refused), `src/adapters/svg/text/run.rs::scale` (`round(size / 7)`, clamped to `font::MAX_SCALE` = 8), `src/core/font.rs` (lowercase drawn as uppercase) |
| 5 | 76-81 | "The translator cannot emit a document its parser would reject." | The translator writes the format's own limits into its output; the operation budget is the parser's to enforce | `src/adapters/svg/emit.rs`, `src/adapters/svg/text/run.rs` (coordinate, size, label and scale limits enforced at translation time) versus `src/core/document/mod.rs::MAX_OPS` (4096 operations). **The old sentence was false.** A `<polyline>` of 300 points expands to 299 `line` directives, and 200 such polylines produce 7800 directives, which the parser then refuses with "a document may hold at most 4096 operations". Verified by running `oko-svg` and `oko` on such a file |
| 6 | 84-86 | "no function is much longer than a dozen" | "most functions are a dozen lines or fewer; the longest is the path grammar's reader, which reads a small language in one place" | Measured across the 252 functions in `src/`: 208 are 12 physical lines or fewer, median 6; the longest is `outline` in `src/adapters/svg/path/mod.rs` at 89 lines. **The old sentence was false.** This was already false before the font work — `outline` is the same 89 lines in commit `63ae929` |
| 7 | 96-98 | `core` list had no font | adds "the five-by-seven bitmap font and the text it draws" | `src/core/font.rs` (`GLYPH_WIDTH` 5, `GLYPH_HEIGHT` 7), `src/core/text.rs` |
| 8 | 99-100 | adapters list had no mention of labels | "the bounded SVG translator, which reads labels as well as shapes" | `src/adapters/svg/text/` |
| 9 | 102 | "76 tests" | "114 tests" | `cargo test -q -- --test-threads=1` prints 107 (lib) + 6 (acceptance) + 1 (meta) = 114 |
| 10 | 108-110 | "Text and a bundled font. Pointer input. …" | "Pointer input. Images, arcs, paths with curves, transforms and opacity. …" | `src/core/font.rs` and `src/core/text.rs` exist; `src/core/document/directive.rs` parses `text`. The rest of the absent list is unchanged and still true: no pointer event in `src/core/event.rs` (only `Key`, `Resize`); no arc, image, curve or transform primitive in `src/core/scene.rs`; no opacity primitive (alpha is carried but never blended, `src/core/geom.rs`); no surface wire format, boot console or framebuffer presenter in the tree |

### `docs/adr/0002-svg-in-document-out.md`

| # | Line | Old | New | Checked against |
|---|---|---|---|---|
| 11 | 3-4 | "Status: accepted · Date: 2026-10-21" | adds "· Text amended by `docs/adr/0003-svg-text-mapping.md`" | `docs/adr/0003-svg-text-mapping.md` exists and is the decision that added SVG text |
| 12 | 24-27 | Input subset did not include text | adds "`text` with its `tspan` lines, which ADR 0003 added later" | `src/adapters/svg/translator.rs` accepts `text` and `tspan` |
| 13 | 31-36 | "Curves, text, rotation, scale, opacity below one … are refused" | text removed from the refusal list; records that text "was refused on the same grounds until the built-in font made a mapping possible; ADR 0003 records what replaced that refusal" | `src/adapters/svg/translator.rs`, `docs/adr/0003-svg-text-mapping.md` |
| 14 | 51-54 | "A v1 design cannot use curves, text, rotation or transparency. Text is the first limitation to remove, when the bitmap font exists" | text removed from the cannot-use list; records that the font now exists and the translator maps SVG text onto the document's text directive | `src/core/font.rs`, `src/adapters/svg/text/run.rs` |
| 15 | 60-62 | "The translator must track the document grammar's limits. It imports the same constants, so the two cannot drift." | adds "— except the operation budget, which the translator cannot know, because one element can become many directives" | Same finding as row 5. The translator imports `MAX_COORDINATE`, `MAX_SIDE` and `MAX_TEXT_CHARS` (`src/adapters/svg/emit.rs`, `src/adapters/svg/text/run.rs`), but `MAX_OPS` is a count the translator cannot know |
| 16 | 68-70 | "text and curves become necessary for the first real panel, which forces the font and a path fill rule" | "curves become necessary for the first real panel, which forces a path fill rule — the text half of this trigger has already fired, and ADR 0003 is the decision that answered it" | The trigger the ADR names has fired: the font exists and ADR 0003 is the answer to it |

## Claims checked and found still true

- `core` is pure: no `std::fs`, no clock, no `static` in `src/core/` (grepped).
- Cohen–Sutherland clipping (`src/core/clip.rs`), Bresenham lines (`src/core/raster.rs`).
- Terminal encoder: two square pixels per cell, 24-bit colour (`src/adapters/terminal.rs`).
- One mebibyte, four thousand elements, four thousand points (`src/adapters/svg/mod.rs::MAX_INPUT`, `src/adapters/svg/xml.rs::MAX_ELEMENTS`, `src/adapters/svg/path/mod.rs::MAX_POINTS`).
- Lease, focus, session reducer, escape chord (`src/core/lease.rs`, `focus.rs`, `session.rs`).
- `oko panel.oko [columns rows]` and `oko-svg panel.svg` (`src/main.rs`, `src/bin/oko-svg.rs`); examples for both (`examples/panel.svg`, `examples/login.oko`, `examples/text-panel.svg`).
- The absent list is unchanged except for text and the font: pointer input, images, arcs, curves, transforms, opacity, the surface wire format, the boot console and the framebuffer presenters are all still absent.

## Note for the parent

Two false claims were found and corrected that the brief did not mention, both
pre-existing since the first commit `63ae929` and unrelated to the font or the
SVG text work:

1. The whitepaper said the translator "cannot emit a document its parser would
   reject". It can: one `<polyline>` becomes many `line` directives, and the
   document's 4096-operation budget is the parser's to enforce.
2. The whitepaper said "no function is much longer than a dozen". The path
   grammar's reader is 89 lines.

Both are now stated accurately. The code was not changed — these are
documentation claims about existing behaviour.
