# The SVG translator refuses a panel the document format cannot hold

## The change

The translator wrote a document the console could not read. A panel of 2000
polylines of five points became 8000 `line` directives, and the document
parser refused the result at line 4097 with "a document may hold at most 4096
operations". A document the translator wrote must be a document the console
can read, so the translator now counts the operations it writes and refuses
before it writes one too many.

## The seam

The count belongs where the directives are written, not where the elements are
read. An element is not an operation: one `<polyline>` of five points writes
four `line` directives, and one `<text>` writes one `text` directive however
long its label is. Counting elements would refuse panels the format can hold,
and counting bytes would refuse short panels the format can hold.

So `emit.rs` gained a `Writer`: the document text plus the number of
operations in it. Every directive the translator writes goes through
`Writer::write`, which counts it and refuses when the count would pass
`MAX_OPS`. `Writer::write` takes the line of the element that caused the
write, so the refusal names the line of the element that pushed the panel over
the budget.

The rest of the change is plumbing: `shapes.rs`, `text/run.rs` and
`translator.rs` now pass a `&mut Writer` where they passed a `&mut String`.
The public signature of `translate` is unchanged.

The refusal message is the parser's own wording, "a document may hold at most
4096 operations", so the two ends of the bridge say the same thing about the
same limit.

## What the tests prove

Three tests in `src/adapters/svg/mod.rs`:

- `a_panel_too_large_for_the_format_is_refused` — the 2000-polyline panel is
  refused, the error names 4096, and the error's line is the line of the
  polyline that pushed it over (the 1025th, since each polyline writes four
  operations and 1024 of them exactly fill the budget).
- `a_panel_of_the_whole_budget_is_written` — a panel of exactly 4096
  operations is written, and the parser accepts it. The budget is a ceiling,
  not a quota: the translator refuses the 4097th operation, not the 4096th.
- `a_translated_panel_is_a_document_the_parser_accepts` — a panel of a
  rectangle, a polyline and a label is translated and then parsed by
  `crate::core::document::parse`, which accepts it.

One test in `src/adapters/svg/emit.rs`:

- `the_write_that_passes_the_budget_is_refused` — the writer itself refuses
  the write that would pass the budget, with the line it was given.

## The red run

Before the fix, the new test failed for the right reason — the translation
succeeded and the test panicked on `expect_err`:

```
---- adapters::svg::tests::a_panel_too_large_for_the_format_is_refused stdout ----
thread '...' panicked at src/adapters/svg/mod.rs:102:37:
a panel over the budget: "line 0 0 1 1 #ffffff\nline 1 1 2 2 #ffffff\n..."
```

## The green run

```
$ cargo test -q -- --test-threads=1
running 111 tests
test result: ok. 111 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The suite was 107 + 6 + 1 = 114 tests before this change; it is 111 + 6 + 1 =
118 now.

```
$ cargo fmt --all
$ cargo clippy --all-targets -- -D warnings
    Checking oko-minimal v0.1.0 (/home/violindocgfx/oko/oko-minimal)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.35s

$ make check
...
test result: ok. 111 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
...
test a_broken_panel_document_names_every_bad_line ... ok
test a_panel_document_becomes_terminal_blocks ... ok
test an_svg_curve_is_refused_with_its_line ... ok
test an_svg_export_becomes_a_panel_document ... ok
test an_svg_label_becomes_a_text_operation ... ok
test the_escape_chord_hands_the_screen_back_to_the_console ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test every_source_file_stays_within_the_line_budget ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
Doc-tests oko
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

EXIT 0
```

## Decisions the brief did not settle

- **The refusal happens at the write, not before the element.** A polyline
  that would write four operations but has room for two is refused at its own
  line, and the two operations already written are discarded with the rest of
  the document, because the whole translation fails. Refusing before the
  element would mean knowing how many operations an element will write before
  writing it, which would put the count in the shape layer instead of the
  writing layer.
- **No new ADR.** The public interface of the translator did not change, no
  boundary moved, and the limit being enforced is the document format's own
  `MAX_OPS`, not a new decision. This is a bug fix inside an existing
  decision, not a new decision.
- **The message repeats the parser's wording** rather than inventing one, so
  the two ends of the bridge agree about the limit.

## Sizes

`emit.rs` 98 -> 159, `mod.rs` 93 -> 139, `shapes.rs` 121 -> 122,
`text/run.rs` 131 -> 137, `translator.rs` 138 -> 138. All well under the
200-line ceiling; no baseline entry was needed.
