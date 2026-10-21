# SVG text mapping — implementation report

Implements `docs/adr/0003-svg-text-mapping.md` in the SVG translator.
No change to the document grammar, the font, or the signature of `translate()`.

## What changed

| File | Change |
|---|---|
| `src/adapters/svg/xml.rs` | `Event::Text { content, line }` — the ADR's text event. |
| `src/adapters/svg/scanner.rs` | A run of character data is delivered; whitespace-only runs stay ignored; a run with `&` is refused. |
| `src/adapters/svg/translator.rs` | `<text>` and `<tspan>` are elements now; a text run becomes one directive. |
| `src/adapters/svg/text/mod.rs` | New. Where a run sits and how big it is (`Style`), the placement rules. |
| `src/adapters/svg/text/run.rs` | New. The label rules: trimming, the font check, the scale, the directive. |
| `src/adapters/svg/text/tests.rs` | New. The text rules from the outside: whole SVG in, whole document out. |
| `src/adapters/svg/mod.rs` | `mod text;` and two stale sentences that said text was refused. |

`src/adapters/svg/mod.rs` was not in the given scope. The new module cannot
exist without its declaration, and its preamble claimed the translator refuses
text, which the ADR retired. Both edits are the minimum needed.

## The seams

**The text mapping is a package, not a file.** `text/mod.rs` holds the
placement (`Style`: position, size, colour, and what a `<tspan>` inherits),
`text/run.rs` holds the label rules (trim, collapse, font check, scale, the
directive). A single `text.rs` was 201 lines before any test, so the split was
forced by the ceiling; it also splits the two responsibilities that change for
different reasons. `text/tests.rs` mirrors the module, as §2.5 asks.

**The scanner stays lexical, the translator stays semantic.** The scanner
delivers a run of characters and refuses entities; it does not know what a
`<text>` is. The translator refuses a run that is not inside a `<text>` or a
`<tspan>`, because it is the layer that knows the element stack. The ADR puts
both rules in the paragraph called "Reading text"; the outcome is the same
error with the same line, only the layer differs.

**Style is a value, not a stack of flags.** The translator keeps a stack of
`Style`, one per open `<text>`/`<tspan>`. A span is built from its parent's
style, so inheritance is one call (`of_span`) instead of a rule per attribute.

## The rules, as implemented

- Position: `x`/`y` are the label's left and top edge. A `<tspan>` may state
  its own `x`/`y`, otherwise it inherits the `<text>`'s. `dy` adds to the y it
  inherits.
- Size: `font-size`, or `font-size` in a `style` attribute (`font-size: 14px`).
  A missing size is 7.
- Scale: `max(1, round(font_size / 7))`, capped at 8.
- Colour: `fill`, read as for other shapes. Black when missing, nothing drawn
  for `fill="none"`.
- Content: trimmed, internal whitespace collapsed to one space, an empty run
  ignored, every character must be in the built-in font, a double quote refused.
- Output: one directive per run, `text X Y "LABEL" SCALE #rrggbb`.

## Where reality differed from the ADR, and what I chose

1. **The ADR does not say which `font-size` wins** when an element carries both
   `font-size="10"` and `style="font-size: 14px"`. The attribute wins here.
   CSS says the opposite. Flagging it, because a reader may want the other.
2. **The ADR states inheritance for position only.** A `<tspan>` also inherits
   the size and the colour of its `<text>` here, which is what SVG does.
3. **`px` is accepted** on `font-size` and `dy`, because the ADR's own style
   example writes `14px`. Other units (`em`, `%`, `pt`) are refused as
   unmeasurable without a font.
4. **The document grammar's own limits are enforced**: a label over 128
   characters and a coordinate outside ±32767 are refused, so the translator
   still cannot write a document the parser will reject.
5. **A group translation moves a label**, as it moves a shape. The ADR does
   not mention `transform`; refusing it would have made `<g transform=...>`
   around a `<text>` silently wrong.
6. **An `x` or `y` holding several numbers** (an SVG coordinate list) is
   refused as "not a number". Only the first of a list is meaningful to one
   label, and choosing it silently would be a guess.

## Command output

RED, before any production line — the end-to-end test through the public
`translate()`:

```
running 1 test
adapters::svg::tests::an_svg_text_becomes_a_text_directive --- FAILED
thread '...' panicked at src/adapters/svg/mod.rs:72:28:
a translated label: SvgError { line: 1, problem: "text content is not supported" }
test result: FAILED. 0 passed; 1 failed; ...
```

GREEN, the same test:

```
$ cargo test -q --lib -- --test-threads=1 an_svg_text_becomes_a_text_directive
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 105 filtered out; finished in 0.00s
```

The whole library:

```
$ cargo test -q --lib -- --test-threads=1
running 106 tests
....................................................................................... 87/106
...................
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
```

Format and lint:

```
$ cargo fmt --all --check        # exit 0
$ cargo clippy --offline --jobs 1 --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.34s   # exit 0
```

Line budget:

```
$ python3 tools/check_line_budget.py
warn    src/adapters/svg/path/mod.rs: 167 lines, ceiling 200
warn    src/adapters/svg/text/mod.rs: 173 lines, ceiling 200
warn    src/adapters/svg/values.rs: 166 lines, ceiling 200
warn    src/core/raster.rs: 165 lines, ceiling 200
warn    src/core/syntax.rs: 184 lines, ceiling 200

0 exempt, 5 in the warning band, 0 frozen files grew, 0 over the ceiling
line budget OK
```

The whole gate:

```
$ make check
...
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.92s
   Doc-tests oko
exit 0
```

The acceptance example of the ADR, `examples/text-panel.svg`, translates to
exactly two text directives:

```
$ cargo run -q --offline --bin oko-svg -- examples/text-panel.svg
rect 0 0 80 48 #05070a
text 6 10 "OKO" 3 #ff9d00
text 6 24 "SYSTEM READY" 1 #c9d1d9
```

## Honest notes on the test-first claim

The end-to-end red above was witnessed: the test failed on the old code with
"text content is not supported", and the assertion for the new behaviour was
the one that failed. The finer-grained rules (span inheritance, `dy`, the style
attribute, the refusals) were implemented in the same slice as that red and
their first run was already green, so they have no red witness of their own.
They are observable through `translate()`, so each one is a real regression
test; they are not separate red-green cycles.

The scanner's `text_content_is_refused` test was replaced by
`a_run_of_text_is_delivered` and `an_entity_in_a_run_is_refused`. The ADR
deliberately changes that behaviour, so the old test asserted the opposite of
the decision. Its intent — that character data is handled deliberately, not
ignored — is kept by the two tests that replace it.

Nothing was committed: the parent integrates and commits.
