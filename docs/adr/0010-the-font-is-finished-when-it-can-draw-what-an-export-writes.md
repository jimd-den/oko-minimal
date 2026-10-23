# 0010 — The font is finished when it can draw what an export writes

Status: accepted · Date: 2026-10-21
Amended: 2026-10-21, when the fitting was written and the font turned out
to be wider than the faces the exports were set in.

## Context

The built-in font is five columns by seven rows, one bit per pixel, with
fifty-five glyphs and a fixed advance of six pixels. Lowercase is drawn as
uppercase, so a panel that says `in your notes` says `IN YOUR NOTES`, and a
character the font does not have is not drawn at all: it is a hole in the
panel, named in the report.

Two things follow from the font being unfinished, and both are visible in a
real export. The Figma quote page draws its labels in a fixed pitch, so the
columns of the table run into each other, because the design was set in
proportional Roboto and the panel is set in a monospaced bitmap. And the
exports write text properties the translator does not read: `letter-spacing`,
`textLength` with `lengthAdjust`, `text-anchor`, `font-weight`. The
translator reads `font-size` and nothing else about the font.

There is a second reading of "finishing font support": embedding a real font
file. That is a different decision with a different cost. ADR 0003 rejected
it, and rejected it for reasons that have not changed: a font licence, a
rasteriser and a metrics engine in the boot path, for a panel format whose
labels are short.

## Decision

**The font is finished when it can draw what an export writes, and lay it out
the way the design intended.** Three parts, in this order.

**One: the character set.** The font gains real lowercase letters, and the
punctuation and symbols an export writes. The rule "lowercase is drawn as
uppercase" goes away, and with it the shout. A character the font still does
not have is still a hole, and is still named - but there are fewer of them,
and the set is the whole of ASCII that a label can reasonably carry.

**Two: the metrics.** The font becomes proportional. Each glyph carries its
own width, so a word is as wide as its letters and not as wide as the widest
letter times the number of letters. Some glyphs are narrower - an `I`, a
period, a bracket - and the space between words is narrower than a letter.
The glyph's width is part of the glyph, not a guess made at draw time.

**Three: the export's own text properties.** The translator reads
`letter-spacing`, `textLength` with `lengthAdjust`, and `text-anchor`.

- `letter-spacing` is the space the design put between the letters. The
  document's text directive gains one optional clause, `spacing N`, which
  adds N pixels between the glyphs. The font's own advance is the floor; the
  spacing is what the design asked for on top of it.
- `textLength` with `lengthAdjust="spacing"` is a length the design holds
  the text to. The translator computes the spacing that fills it, the way a
  browser does, and the panel's text occupies the width the design gave it.
  A length *shorter* than the text is not held to: the built-in font is
  wider than the face the design was set in, so holding to it would overlap
  the letters. The text is drawn at the font's own width and the report says
  so, with the two widths in it.
- `text-anchor` decides which end of the text the position refers to:
  `start` is the left edge, `middle` is the centre, `end` is the right edge.
  A right-aligned label in an export is right-aligned in the panel.
- `lengthAdjust="spacingAndGlyphs"` also stretches the glyphs themselves.
  That is not drawn: the label is fitted by its spacing, and the report says
  so, because a stretched bitmap glyph is not a shape this font has. The
  layout is right and the letterforms are the font's own.

## Consequences

- A panel says what it says, in the case the design used.
- The font's table grows, and the text renderer reads a width per glyph
  instead of one advance for all of them.
- The document's text directive grows one optional clause and stays one
  directive: `text X Y "LABEL" SCALE #rrggbb spacing 2`.
- The scale rule does not change. A font size still becomes a glyph scale
  by the approximation ADR 0003 decided, because a proportional bitmap font
  has no em box to measure.
- The specimen is the evidence: a panel that draws every glyph the font has,
  and a panel that draws a sentence in it. A font is not finished because a
  table has entries; it is finished when the sentence looks like a sentence.

## Revisit if

- a real font file is adopted, which changes the scale rule, the label
  grammar and the metrics at once;
- a design needs letterforms this bitmap font does not have, such as a
  script, and the answer is a second font rather than a bigger table;
- the text needs to be shaped, which means the run is not a sequence of
  glyphs, and the document's label is not a string.
