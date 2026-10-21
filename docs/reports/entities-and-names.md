# Entities and colour names in the SVG reader

Branch `feature/entities-and-names`. Base: `main` at `fc10a84`.

## What changed

Two things, and nothing else.

1. **Entity references are decoded.** The five entities XML defines
   (`&amp; &lt; &gt; &quot; &apos;`), a decimal reference (`&#10;`) and a
   hexadecimal one (`&#x27;`) become the characters they name, in text runs
   and in attribute values alike. An entity the translator does not know is a
   hard error that names it, because a guess would put a character in the
   panel that the author never typed.
2. **Colour names are read.** The CSS basic set - the sixteen names a browser
   has always known - beside `#rgb`, `#rrggbb` and `rgb(...)`. A colour with
   an alpha channel is refused with a message that says so, because the
   document format holds an opaque colour and the console draws no
   transparency yet.

## The seams

| Module | Role |
|---|---|
| `src/adapters/svg/entity.rs` (new) | The decoder. Pure: text in, text out, or an error naming the entity. |
| `src/adapters/svg/names.rs` (new) | The CSS name table. Pure: word in, colour out. |
| `src/adapters/svg/scanner.rs` | Decodes a run of character data before delivering it. The event stream is unchanged: `Start`, `End`, `Text`. |
| `src/adapters/svg/tag.rs` | Decodes an attribute value before storing it. |
| `src/adapters/svg/values.rs` | `colour` reads the name table after the forms it already knew. |
| `src/adapters/svg/cursor.rs` | Stopped refusing `&`; the layer that understands the grammar now decodes. |

`translate(&str) -> Result<String, SvgError>` and the shape of `SvgError` are
untouched. Nothing outside `src/adapters/svg/` was touched. No dependency was
added. The reader is not a tree; the scanner still produces the same three
events.

The one judgement worth naming: `rgba(...)` is refused outright rather than
read when its alpha is one, because the brief asks for a refusal with a
message that says so, and a colour the document cannot hold should not be
approximated. `transparent` is the same case: it is a colour with no alpha,
so it is refused with the same message. `names::colour` still knows it, so
the next slice that can draw transparency has the word ready.

## The commands

`cargo test -q -- --test-threads=1`:

```
running 128 tests
test result: ok. 128 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s

running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
```

128 unit tests, up from 111: the five named entities, a decimal and a
hexadecimal reference, an unknown entity refused by name, an entity inside an
attribute value, a colour name read, a translucent colour refused, a soft
line break still one line, and a whole-SVG named colour.

`cargo fmt --all` run; `cargo clippy --all-targets -- -D warnings` clean;
`make check` exits 0 (line budget, fmt, types, clippy, tests).

## The two real files

Downloaded, unmodified:

- Figma: `https://raw.githubusercontent.com/ttskch/svg-paper/master/js/tests/resources/real-world-paper-figma.svg`
  (1861 entity references, 633 of them colour names: `black` 428, `white` 205)
- Penpot: `https://raw.githubusercontent.com/Floydv149/bibleLinkerPro/master/images/dark.svg`
  (52 entity references: `&apos;` 40, `&quot;` 12)

Before this change both files were refused with `entities are not allowed`
before a single word was drawn. Now:

```
$ cargo run -q --bin oko-svg -- /tmp/real/figma.svg
/tmp/real/figma.svg:line 1048: an attribute value is too long

$ cargo run -q --bin oko-svg -- /tmp/real/penpot.svg
/tmp/real/penpot.svg:line 1: `fill` on `<svg>` would be inherited; put it on each shape
```

Neither error is about entities and neither is about colours. The refusal has
moved.

### The next wall, Figma

`line 1048: an attribute value is too long`. That is the `<image>` element
with its base64 PNG in `xlink:href`, and the reader's 1 KiB limit on one
attribute value. The reader limit is a slice of its own: a raster image in
an export is not a shape this console draws.

Behind it, with the `<image>` element removed, the next wall is
`line 4: a filled path must be an axis-aligned rectangle; use fill="none" for
strokes` - which is the curve problem the brief said would be there.

And the text of that export does translate now. Figma's own text element,
taken from the file, with the root `fill="none"` removed:

```
$ cargo run -q --bin oko-svg -- /tmp/real/figma_text.svg
text 887 133 "ESTIMATE" 8 #000000
text 1564 2307 "Discounted" 4 #000000
text 1580 2336 "Subtotal" 4 #000000
```

`fill="black"` is read as `#000000`, and the `&#10;` soft line break in
`Discounted&#10;` decodes to a newline that the label rules collapse, so
`Discounted` stays one line.

### The next wall, Penpot

`line 1: `fill` on `<svg>` would be inherited; put it on each shape`. The
root `<svg>` carries `fill="none"`, and the group rule refuses a fill on a
group because it would be inherited by every shape inside it.

Behind it, with the root fill removed, the next wall is `line 1: `style` is
not supported` - the `<style>` element with its `@font-face` rules, which is
the element whitelist's business, not the reader's.

Note what the reader now does with Penpot's `<style>` block: the `&apos;`
and `&quot;` inside it are decoded rather than refused. The block is still
refused, but as an unsupported element, by the element whitelist.

## What was not done

- The reader was not redesigned. The scanner still emits `Start`, `End` and
  `Text`; entity decoding sits inside it.
- `translate`, `SvgError` and every public type are as they were.
- The skip-and-report work of ADR 0004 was not started.
- The colour of shapes, the fill rules, the transform rules and the element
  whitelist were not touched.
- No third-party file was committed; the two real exports were downloaded to
  `/tmp` for the proof only.

## A note on where this was built

The shared working tree had another agent's half-finished redesign in it
(`paint.rs` replaced by a `paint/` directory that does not compile). To
avoid clobbering it, this slice was built in a separate worktree at
`/home/violindocgfx/oko/oko-entities` on branch
`feature/entities-and-names`, based on `main` at `fc10a84`. It touches only
`src/adapters/svg/{scanner,cursor,tag,values,mod}.rs` and adds
`src/adapters/svg/{entity,names}.rs`, so it merges cleanly with the other
work.
