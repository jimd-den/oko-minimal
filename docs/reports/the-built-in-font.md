# The built-in font, finished in three parts

The font is finished when it can draw what an export writes, and lay it out
the way the design intended (ADR 0010). Two of the three parts are in.

## One: the character set

The font drew a lowercase letter as its uppercase, and its own test said so:
"the document format is a panel, not prose". That was a decision about what a
panel was for, and ADR 0010 makes a different one.

Twenty-six lowercase letters now have shapes of their own, standing on the
x-height in the five rows above the baseline. A descender has no room in
seven rows, so `g`, `j`, `p`, `q` and `y` are drawn without their tails, and
the module's preamble says so rather than leaving it to be discovered.

The punctuation came from the exports themselves. The Figma export writes
`#`, `&` and `;`, and the font had none of the three, so those labels were
holes in the panel named in the report. They are in the font now.

The character set was not taken from a font that already existed. The
letterforms are original work for this project, and they are released with
it, as the rest of the font is.

## Two: the metrics

A word is as wide as its letters and not as wide as the widest letter times
the number of letters. The Figma export draws its labels in a proportional
face, and the panel drew them in a fixed pitch, so the columns of a table ran
into each other.

Each glyph's width is measured from the glyph's own ink, so an `I`, a period
and a bracket are narrower than an `M`. The pen moves by the width of the
letter that was drawn, and a letter is separated from the next by one
column. A blank glyph has no ink and is given two columns: the space
between words is narrower than a letter, but it is not nothing.

The width is part of the glyph and not a guess made at draw time: the font
module measures it and hands it over.

## The evidence

Two panels, in `examples/`: one that draws every glyph the font has, and
one that draws a sentence in it. The sentence is the pangram, because a
font that can set a pangram can set a sentence.

## The three glyphs that were missing

The font had no `#`, `&` or `;` - the Figma export writes all three - and
the export's own label with a double quote in it is refused by the
translator, with the reason, before it reaches the document. A document's
label is quoted, and a quote cannot be inside a quote.

## What is not in yet

**Three: the export's own text properties.** The translator reads
`font-size` and nothing else about the font. It should read `letter-spacing`,
`textLength` with `lengthAdjust`, and `text-anchor`:

- the Figma export writes `letter-spacing="0em"` on every label, so nothing
  is lost by not reading it yet;
- the Penpot export writes `textLength` with `lengthAdjust="spacingAndGlyphs"`
  on four labels, and those four should occupy the width the design gave
  them, fitted by their spacing, with the report saying that the letterforms
  are the font's own and not stretched;
- no export writes `text-anchor`, but a right-aligned label in a panel is
  right-aligned.

The document's text directive grows one optional clause for this,
`spacing N`, and the core's text grows the width of a string so that an
anchor has something to anchor to.

**The real font.** ADR 0011: a panel may be written in a real font, and the
reader is borrowed while the drawing is kept. That work is under way; see
the pull request for where it stands.
