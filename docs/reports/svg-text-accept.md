# SVG text acceptance test

ROLE: test author. One acceptance test plus the example file it reads.
SCOPE OWNED: `tests/acceptance/main.rs`, `examples/text-panel.svg`.

## The example

`examples/text-panel.svg` is new, and is exactly the file the brief specified:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<svg width="80" height="48" viewBox="0 0 80 48" xmlns="http://www.w3.org/2000/svg">
  <rect x="0" y="0" width="80" height="48" fill="#05070a"/>
  <text x="6" y="10" font-size="21" fill="#ff9d00">OKO</text>
  <text x="6" y="24" font-size="7" fill="#c9d1d9">SYSTEM READY</text>
</svg>
```

## The test

`tests/acceptance/main.rs` gains one test, `an_svg_label_becomes_a_text_operation`.
It names the use case in the user's language and asserts the fact in two
places that must agree: the document the translator writes, and the scene the
document parses into.

```rust
#[test]
fn an_svg_label_becomes_a_text_operation() {
    let svg = include_str!("../../examples/text-panel.svg");

    let document = oko::adapters::svg::translate(svg).expect("a supported export");
    let scene =
        oko::core::document::parse(&document).expect("the translator writes valid documents");

    let directives: Vec<&str> = document
        .lines()
        .filter(|line| line.starts_with("text "))
        .collect();
    assert_eq!(
        directives,
        [
            "text 6 10 \"OKO\" 3 #ff9d00",
            "text 6 24 \"SYSTEM READY\" 1 #c9d1d9"
        ]
    );

    let labels: Vec<oko::core::scene::Op> = scene
        .ops()
        .iter()
        .filter(|op| matches!(op, oko::core::scene::Op::Text { .. }))
        .cloned()
        .collect();
    assert_eq!(
        labels,
        [
            oko::core::scene::Op::Text {
                at: oko::core::geom::Point::new(6, 10),
                text: "OKO".to_string(),
                scale: 3,
                color: oko::core::geom::Rgba::rgb(255, 157, 0),
            },
            oko::core::scene::Op::Text {
                at: oko::core::geom::Point::Point::new(6, 24),
                ...
```

(The second half is elided here for length; the file holds it in full.)

The two assertions describe one fact - the label becomes a text operation - at
the two boundaries a user of the translator can see: the document text and the
scene operation. The rect in the example is deliberately not asserted on, so
the test speaks only about text.

## The red run

```
$ cargo test -q --test acceptance -- --test-threads=1
running 6 tests
.... 4/6
an_svg_label_becomes_a_text_operation --- FAILED
.
failures:

---- an_svg_label_becomes_a_text_operation stdout ----

thread 'an_svg_label_becomes_a_text_operation' (65299) panicked at tests/acceptance/main.rs:79:55:
a supported export: SvgError { line: 4, problem: "text content is not supported" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

failures:
    an_svg_label_becomes_a_text_operation

test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

The red is for the right reason: the XML reader refuses the character run
inside `<text>` with `text content is not supported`, on line 4 of the example,
which is the line of the first `<text>` element. Every pre-existing acceptance
test still passes (5 passed). The test is not ignored and not marked
expected-to-fail.

## The line budget

```
$ python3 tools/check_line_budget.py
warn    src/adapters/svg/path/mod.rs: 167 lines, ceiling 200
warn    src/adapters/svg/values.rs: 166 lines, ceiling 200
warn    src/core/raster.rs: 165 lines, ceiling 200
warn    src/core/syntax.rs: 184 lines, ceiling 200

0 exempt, 4 in the warning band, 0 frozen files grew, 0 over the ceiling
line budget OK
```

`tests/acceptance/main.rs` is 118 lines, well under the 200 ceiling. The
warning band is pre-existing `src/` files, not mine.

## What makes it green

The other child's implementation must make the translator read a character run
as a text event and write one directive per run, so that:

- the document contains exactly `text 6 10 "OKO" 3 #ff9d00` and
  `text 6 24 "SYSTEM READY" 1 #c9d1d9` among its `text ` lines;
- the scene holds `Op::Text` at `Point::new(6, 10)` with scale 3 and colour
  `#ff9d00`, and at `Point::new(6, 24)` with scale 1 and colour `#c9d1d9`.

That is the ADR 0003 rule: `scale = max(1, round(font_size / 7))`, so 21
becomes 3 and 7 becomes 1. When the translator emits those directives, both
assertions pass and the test goes green without a change to the test file.

## The green run

The other child's implementation landed while this test was in place. The test
then passed unchanged - no assertion was loosened to reach green:

```
$ cargo test -q --test acceptance -- --test-threads=1
running 6 tests
......
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

The whole suite is green too:

```
$ cargo test -q -- --test-threads=1
running 6 tests
......
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

(the remaining test binaries report 0 tests, and one meta test passes)

## Note for the parent

I did not touch `src/**`, other tests, `Cargo.toml`, `Makefile` or `AGENTS.md`.
`cargo clippy --all-targets -- -D warnings` passes with my test in place (0
warnings).

`cargo fmt --all --check` reports a formatting difference in
`src/adapters/svg/xml.rs` - that is the other child's file, not mine, and it is
their call to fix it. My own file is rustfmt-clean:
`rustfmt --edition 2024 --check tests/acceptance/main.rs` exits 0.
