# 0005 — A curve is a primitive, not a polyline

Status: accepted · Date: 2026-10-21

## Context

A real export loses shapes to curves. A Penpot card loses four of them; a
Figma page loses two. A curve is also what a Figma export's *outlined* text
is made of, so this one primitive decides whether a Figma panel is viable at
all without the designer switching *Outline text* off.

The temptation is to flatten a curve into a polyline and let the existing
line renderer draw it. That is the silent approximation this project refuses:
the document would say `line` where the source said `C`, and nobody reading
the document could tell. The whitepaper's rule is that an approximation must
be traceable, and a flattened curve is not traceable at all.

The other temptation is to implement exact curve rasterisation, with
intersections found by root-finding and spans filled by winding number. That
is a large amount of delicate arithmetic in the boot path, for a shape that a
sampling renderer would draw almost identically.

## Decision

**A curve is a primitive in the display list, and the renderer samples it.**

The display list gains a path: a sequence of segments, each of which is a
move, a line, a cubic, a quadratic, or a close. A path carries a fill colour,
a stroke colour, or both, and either may be absent.

- `core::path` holds the segment types. It is pure data: no renderer, no
  document grammar, no device.
- The scene gains one operation for a path, with the path and its two paints.
- The document grammar gains one directive for a path, in the same small
  language the SVG path reader already speaks:
  `path M 10 10 C 20 0 30 10 Z fill #ff9d00`, with `M`, `L`, `C`, `Q` and `Z`
  as the commands, and `fill` and `stroke` naming the paints at the end.
  A path that states neither paint is a stroke in the current colour, as SVG
  has it; the document says so.
- The renderer samples a curve into a polyline before it draws, and fills the
  polyline with the non-zero winding rule, which is SVG's default. The
  sampling is a property of the *renderer*, not of the document: the
  document keeps the curve, so nothing is lost and nothing is misreported.
  A cubic is sampled in proportion to its size, so a small curve is cheap and
  a large one is smooth.
- A path that states a fill but is not closed fills as SVG's non-zero rule
  fills it, which is to say: the closing edge is implied.

Arcs are not part of this decision. An elliptical arc is convertible to
cubics exactly enough for a console panel, and that conversion is a separate
slice with its own tests.

## Consequences

- The document can carry a curve, so a translated path is no longer a
  different thing from a hand-written one.
- The renderer gains a scanline fill, which the rectangle path never needed.
  A fill is no longer "a rectangle", and the first curve in a panel will
  exercise it.
- The font, the terminal encoder and the lease are untouched.
- The display list grows one operation, and the rasteriser grows the part
  that is genuinely new: filling.
- The SVG translator's next slice can read `C`, `S`, `Q` and `T` and write
  `path` directives, and can stop refusing a real export for a shape it now
  knows how to draw.

## Revisit if

- an arc appears in a real export, which forces the arc-to-cubic conversion;
- a panel needs the even-odd fill rule, which the document must then be able
  to state;
- a curve must be stroked with a width wider than a pixel, which is a second
  kind of shape the renderer does not have.
