# Arcs in the translator

An arc is a curve, and the document already knows how to say a curve. So the
translator reads `A` and writes the cubics that stand for the arc: the
document grammar does not grow an arc command, the core does not grow an arc
primitive, and a panel draws an arc as a curve. An arc is not a hole in a
panel, and it does not appear in the report.

## The decision

ADR 0006 — [an arc is a curve the translator spells out](../adr/0006-an-arc-is-a-curve-the-translator-spells-out.md)
— says it in one line: the document has no arc command, and the translator
does the arithmetic. This slice is that arithmetic.

The document a panel carries is unchanged:

```
path M 11 10 C 11 15 15 19 20 19 C 25 19 29 15 29 10 C 29 5 25 1 20 1 C 15 1 11 5 11 10 Z fill #ff9d00
```

Four cubics, where the export said two arcs. A circle is four cubics in a
panel, and a person editing the panel sees curves and not arcs.

## The modules

The conversion is split by responsibility, not by length:

| File | Lines | What it owns |
|---|---|---|
| `src/adapters/svg/path/arc.rs` | 126 | the command, and the turns of ninety degrees or less |
| `src/adapters/svg/path/centre.rs` | 155 | the endpoint form, and the centre form it becomes |
| `src/adapters/svg/path/reader.rs` | 131 | the reader of numbers, and of the two flags |
| `src/adapters/svg/path/state.rs` | 111 | where the path is, and the grid it is on |
| `src/adapters/svg/path/grammar.rs` | 173 | the grammar: what each command means |
| `src/adapters/svg/path/arc_tests.rs` | 180 | the shape an arc makes |
| `src/adapters/svg/path/tests.rs` | 168 | the grammar, seen from the outside |

`centre.rs` is the specification's arithmetic: the ends are taken to the
ellipse's own frame, the radii are made large enough to join them, the
centre is found, and the two angles are. `arc.rs` is the approximation: the
arc is split into turns of at most ninety degrees, and each turn is one
cubic.

## The flags are flags

The subtlety of the slice is in the reader. The two flags of an arc command
are one character each, so they may be written with no separator from the
number that follows:

```
A 10 10 0 0110 0      the flags are 0 and 1, and the point is (10, 0)
A 10 10 0 0 1 10 0   the same arc, written with spaces
```

A reader that took numbers would read the second flag as the number eleven
and the arc would be a different arc. `Reader::flag` takes one character and
leaves the rest of the digits to the numbers.

## The degenerate cases

The specification says what an arc does when it cannot be an arc, and the
translator does that and not something else:

- a radius of zero makes the arc a straight line to the end;
- the same point twice draws nothing;
- radii that are too small to join the ends are scaled up until they do.

## The accuracy

Each turn is approximated by a cubic whose control points sit on the tangent,
at the factor `4/3 · tan(δ/4)` along it. That is the standard approximation,
and its worst radial error over a quarter turn is 0.000273 of the radius:

```
max radial error: 0.0272530 = 0.00027253 of the radius
```

which is under a thousandth, as ADR 0006 claims, and less than a pixel of a
panel.

> The brief for this slice described the control points as sitting "at the
> two thirds points of the tangent". That is the cruder approximation, the
> one that puts them at two thirds of the radius, and its error is about six
> parts in a thousand — which would break both the ADR's error bound and the
> test that says a point on the arc is within a thousandth of the radius.
> The standard approximation is used, and the test below measures it.

## Evidence: red, then green

The expectations that this slice legitimately changes were changed first, in
their own commit, and watched fail.

**`wip(red): test(svg): an arc is drawn, and the hole is something else`**

```
running 186 tests
adapters::svg::path::tests::an_arc_is_read_as_cubics --- FAILED
adapters::svg::tests::an_arc_is_drawn_as_a_path_directive --- FAILED

---- adapters::svg::path::tests::an_arc_is_read_as_cubics stdout ----
thread '...' panicked at src/adapters/svg/path/tests.rs:117:5:
an arc is drawn, not refused

---- adapters::svg::tests::an_arc_is_drawn_as_a_path_directive stdout ----
thread '...' panicked at src/adapters/svg/tests.rs:169:5:
an arc is drawn, not refused

test result: FAILED. 184 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

What each old test asserted, and why it was wrong:

- `an_arc_is_reported_by_name` asserted that the grammar refuses `A`. ADR
  0006 makes the arc a curve the translator spells out, so the grammar reads
  it.
- `every_shape_left_out_is_named_in_the_report` counted an arc as a hole.
  The hole is a `<circle>` element and a stroked rectangle now, which are
  shapes the console still does not draw.
- The two acceptance tests used an arc as their hole. The hole they mean is
  a shape this console cannot draw, so it is a `<circle>` element now, and
  the report names it.

**Green now** — `cargo test -q --lib -- --test-threads=1`:

```
running 196 tests
test result: ok. 196 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The 11 new tests cover: a quarter arc becomes one cubic; a half circle
becomes two cubics; the large-arc flag picks the longer way round; the
sweep flag picks the direction; a zero radius becomes a straight line; an
arc to the same point draws nothing; the flags written without a separator
are read as flags and not as a number; a relative arc is relative to the
current point; an arc in a path becomes cubic segments; a point on the arc
is within a thousandth of the radius of the true circle; and a rotation
turns the ellipse but leaves a circle a circle.

The acceptance file has 8 tests now, the meta file 1.

## The whole-repo gate

```
$ make check
python3 tools/check_line_budget.py
0 exempt, 14 in the warning band, 0 frozen files grew, 0 over the ceiling
line budget OK
cargo fmt --all --check
cargo check --offline --jobs 1 --all-targets
cargo clippy --offline --jobs 1 --all-targets -- -D warnings
test result: ok. 196 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ echo $?
0
```

## The shape, seen

A small export, written by hand: a circle and an ellipse, both of them
arcs.

```xml
<svg viewBox="0 0 40 20" width="40" height="20">
  <path d="M11 10 A 9 9 0 1 0 29 10 A 9 9 0 1 0 11 10 Z" fill="#ff9d00"/>
  <path d="M2 10 A 4 3 0 1 0 10 10 A 4 3 0 1 0 2 10 Z" fill="none" stroke="#1f6feb"/>
</svg>
```

The report is empty — nothing was left out:

```
$ cargo run -q --bin oko-svg -- /tmp/arc.svg > /tmp/arc.oko
$ cat /tmp/arc.oko
path M 11 10 C 11 15 15 19 20 19 C 25 19 29 15 29 10 C 29 5 25 1 20 1 C 15 1 11 5 11 10 Z fill #ff9d00
path M 2 10 C 2 12 4 13 6 13 C 8 13 10 12 10 10 C 10 8 8 7 6 7 C 4 7 2 8 2 10 Z stroke #1f6feb
```

The circle is four cubics and the ellipse is four cubics, in the document,
where a person can read them.

`cargo run -q --bin oko -- /tmp/arc.oko 40 20`, with the colours resolved
(`#` is the orange fill, `+` is the blue stroke, `.` is the background):

```
  0 |.................######.................|
  1 |..............############..............|
  2 |............################............|
  3 |.....++++..##################...........|
  4 |..+++....++##################...........|
  5 |..+.......+##################...........|
  6 |...+++++++.##################...........|
  7 |............################............|
  8 |..............############..............|
  9 |.................######.................|
 10 |........................................|
```

The circle is filled, the ellipse is outlined, and both are round.

## The real exports, unchanged

Neither real export uses an arc — the Penpot one says `C`, `L` and `M`, the
Figma one says `C`, `H`, `M`, `V` and `Z` — so their documents and their
reports must be byte for byte what they were. They are: the same two
commands, before and after, with the files compared.

```
$ cmp /tmp/penpot-before.oko /tmp/penpot-after.oko
$ cmp /tmp/penpot-before.report /tmp/penpot-after.report
$ cmp /tmp/figma-before.oko /tmp/figma-after.oko
$ cmp /tmp/figma-before.report /tmp/figma-after.report
$ wc -c /tmp/penpot-before.oko /tmp/penpot-after.oko
  573 /tmp/penpot-before.oko
  573 /tmp/penpot-after.oko
$ wc -c /tmp/figma-before.oko /tmp/figma-after.oko
31250 /tmp/figma-before.oko
31250 /tmp/figma-after.oko
```

`cmp` says nothing, which means the files are the same.

## What I did not do

- **The core.** `src/core/**` is untouched. The document grammar has no arc
  command and the core has no arc primitive, which is the decision.
- **The exports.** Neither export is committed as a fixture.

## What I deferred, and one finding

- `src/adapters/svg/tests.rs` is at 187 lines and was at the ceiling when
  the arc's end-to-end test was added to it. The test moved to
  `tests/acceptance/main.rs`, where the other end-to-end expectations live
  and where there is room. The assertions are the same; only the file moved.
- `src/adapters/svg/path/grammar.rs` is at 173 lines and
  `src/adapters/svg/path/arc_tests.rs` at 180. Both are in the warning band:
  the next change to either wants a seam, and the arc's own test file is
  the one that will want it first.

## The commits

```
88007ad feat(svg): an arc is a curve the translator spells out
40df85f wip(red): test(svg): an arc is drawn, and the hole is something else
```

The red commit is red on purpose and is marked `wip(red):` for that reason;
the commit after it is the one that passes the whole gate.
