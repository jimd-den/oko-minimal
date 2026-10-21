# 0003 — SVG text becomes a text directive, at an approximated scale

Status: accepted · Date: 2026-10-21

## Context

The built-in font landed with the panel document: `text X Y "LABEL" SCALE #rrggbb`
is a directive, and the renderer draws five-by-seven bitmaps at any integer
scale. The SVG translator, however, refuses `<text>` outright, because the XML
reader treats every character run as an error. The result is that a panel with
words in it — the normal case for a Penpot or Figma export — has to be
hand-authored as document text, and the two authoring paths have drifted apart.

SVG text is not a small thing. It has a font, a size, a baseline, a fill, and
usually a `<tspan>` per line. A translator that pretended to support all of that
would need a font stack, a metrics engine and a shaping pass, which is exactly
the weight this project exists to remove.

## Decision

The translator maps SVG text onto the one text directive, with these rules and
no more.

**Reading text.** The XML reader gains a text event. A character run that is
not only whitespace is delivered as `Text { content, line }`; whitespace-only
runs stay ignored, as they are today. A run containing `&` is decoded: the five XML
entities and numeric references become the characters they name. The research
in `docs/reports/svg-export-shapes.md` showed why — a real Figma export writes
`&#10;` for a soft line break, and a real Penpot export writes `&quot;` for
any font family with a space in it, so refusing an entity refused the whole
file over a character the author never typed. A text event outside a `<text>`
or a `<tspan>` element is an error, not a warning.

**Elements.** `<text>` and `<tspan>` are supported. Every other element keeps
its current behaviour.

**Position.** `x` and `y` are the label's left edge and its baseline, in
pixels, as SVG has them: a label is placed by the line it sits on, not by the
top of its box. The document's own text directive wants a top edge, so the
translator converts: the top is the baseline minus the height of the glyph,
and the height is seven pixels times the scale. That conversion is exact
rather than approximate, because the built-in font has no descenders — its
bottom row is the baseline.

A `<tspan>` may carry its own `x` and `y`; otherwise it inherits the
`<text>`'s. A `dy` adds to the baseline it inherits. Neither Figma nor Penpot
writes `dy`: Figma gives a second `<tspan>` its own absolute `y`, and Penpot
writes a second `<text>`. The rule is kept because it is cheap and correct
when it appears, not because it is common.

**Size.** `font-size` is read from the attribute or from a `style` attribute
of the form `font-size: 14px`. A missing size is 7.

**Scale.** The built-in glyph is seven pixels tall, so the scale is
`max(1, round(font_size / 7))`, capped at eight. This is an approximation, and
it is the one approximation in the translator: SVG measures an em box, this
measures a glyph. A 21-pixel label becomes scale 3, a 7-pixel label scale 1.

**Colour.** `fill` is read exactly as it is for other shapes, with black for a
missing fill and no text for `fill="none"`. A Penpot export keeps the fill in
its `style` attribute instead of a `fill` attribute, so the style is read when
the attribute is absent, and a fill inherited from a group is honoured.

**Precedence.** Where a size is stated twice, the `font-size` attribute wins
over a `style` attribute, even though CSS says the opposite. The attribute is
what an exporter writes last, and what a person edits by hand.

**Inheritance.** A `<tspan>` inherits the size and the colour of its `<text>`,
not only its position. An exporter that sets the size once on the text expects
the spans to follow it.

**Lengths.** A length may carry `px` or nothing. A position list such as
`x="10 20"` is refused rather than guessed at, and the document's own limits -
a label of at most 128 characters, coordinates within plus or minus 32767 -
are enforced here, so a translated label can never be refused later by the
parser.

**Content.** A run is trimmed, and internal whitespace runs collapse to one
space. An empty result is ignored. Every character must exist in the built-in
font; a character that does not is an error naming it. Lowercase is accepted
and drawn as uppercase, which is the font's rule, not the translator's.

**Output.** One directive per run: `text X Y "LABEL" SCALE #rrggbb`. A label
may not contain a double quote, because the document grammar has no escapes;
such a label is an error.

## Alternatives

- **Keep refusing text.** Rejected: it leaves the two authoring paths
  incompatible, and the font already exists.
- **Embed a real font and shape the text.** Rejected for now: it adds a font
  licence, a rasteriser and a metrics engine to the boot path for a panel
  format whose labels are short and uppercase.
- **Map SVG font sizes to pixel sizes exactly.** Rejected: exactness needs the
  metrics this project does not have. An approximation that is documented is
  better than a promise that is false.
- **Translate `<tspan>` baselines exactly.** Rejected: `dy` is the only
  baseline mechanism the translator honours; anything else is refused.

## Consequences

- The translator can read a panel that has words in it, and it cannot
  represent a label that the built-in font cannot draw.
- Text in an SVG is drawn uppercase, at a size close to but not exactly the
  authored size.
- The document grammar stays as it is: one text directive, one label, one
  scale, one colour.
- The acceptance example is `examples/text-panel.svg`, which must translate to
  exactly two text directives.

## Revisit if

- a real font is adopted, which changes both the scale rule and the label
  grammar;
- rotated or right-aligned text appears in a real export, which needs a
  transform in the display list;
- an export uses `<tspan>` without `x`/`y` to continue a line, which needs
  text-run positions rather than element positions.
