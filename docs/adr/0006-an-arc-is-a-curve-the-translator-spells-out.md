# 0006 — An arc is a curve the translator spells out

Status: accepted · Date: 2026-10-21

## Context

An SVG path can say `A`, and an arc is a curve. The display list has curves
now (ADR 0005), and the document can say `C`.

The tempting choice is to give the document an arc command and the core an
arc primitive, so that nothing is lost in the translation. The cost is a new
command in the document grammar, a new segment in the display list, a new
case in the sampler, and a new case in the parser - for a shape that is a
particular kind of curve.

The other choice is to spell an arc out as the cubics that stand in for it,
in the translator, where the arc's meaning is known. The shape is the same
shape: an arc approximated by a cubic is within about three parts in ten
thousand of the radius.

Which is right depends on what the panel format is for. It is a text file a
person edits by hand. A cubic is a thing a person can see and change. An arc
command with seven numbers and two flags is a thing a person has to look up.

## Decision

**An arc becomes cubics, in the translator.**

- The document grammar does not grow an arc command. The core does not grow
  an arc primitive. The translator reads `A` and `a` and writes the cubics
  that stand for the arc.
- The conversion follows the SVG specification: the endpoint form becomes
  the centre form, the radii are corrected when they are too small to join
  the endpoints, a radius of zero makes the arc a straight line, which is
  what the specification says, and the arc is split into segments of at most
  ninety degrees.
- Each segment is approximated by a cubic, with the standard approximation.
  Its error is under a thousandth of the radius: the arc is the shape, the
  cubics are how the panel says it.
- The two flags of an arc command are flags, not numbers. `A 10 10 0 0 1 20 0`
  and `A 10 10 0 0110 0` mean different things, and the reader must read a
  flag as a single character, so that the second one is not read as the
  number eleven.
- The coordinates are quantised to whole pixels like every other coordinate
  in a panel, so in a panel the arc's error is the pixel grid rather than the
  approximation.
- Nothing is lost, so nothing is reported: an arc is not a hole in a panel,
  and it does not appear in the report.

## Consequences

- The SVG path grammar is complete for the commands a panel can draw: `M`,
  `L`, `H`, `V`, `C`, `S`, `Q`, `T`, `A` and `Z`.
- The document stays small: six commands, and a person editing a panel sees
  curves and not arcs.
- The core stays small: a curve is a curve.
- A panel that wants a circle draws it with cubics, and can edit them.
- The translator grows an arc conversion, which is arithmetic and a set of
  rules, and it has to be right: a wrong arc is a wrong shape, and nothing
  else in the pipeline would notice.

## Revisit if

- an export uses arcs so often that the panels are unreadable with cubics in
  them;
- a panel needs a shape that only an arc can say, such as a circle of any
  radius without four cubics.
