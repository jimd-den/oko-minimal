# 0011 — A panel may be written in a real font

Status: accepted · Date: 2026-10-21

## Context

The instruction is plain: support embedding real font files, out of the box,
for TrueType, for OpenType, and for variable fonts.

That reverses ADR 0003, which rejected embedding a real font "for now"
because it adds a font licence, a rasteriser and a metrics engine to the boot
path. Two of those three costs are real and are paid on purpose now. One is
not.

- **A font licence is a real cost.** A font file in this repository is a
  third-party file with a licence that has to be honoured, shipped and named.
  The licence file sits next to the font, and the name the panel uses is the
  name the licence is for.
- **A metrics engine is a real cost, and it is not paid.** The project's
  rasteriser already samples a curve, and its path painter already fills a
  shape with the non-zero winding rule. A glyph is an outline, an outline is
  a path, and a path is something this project can already draw. The text
  that says the panel's own `path` directive draws is the same text.
- **A reader for the font formats is a real cost, and it is the part that
  has nothing to do with this project's ideas and everything to do with being
  correct.** The tables, the character map, the composite glyphs, the CFF
  charstrings, the deltas of a variable font: that is a large body of
  specification, and a large body of specification is worth having once,
  from people who maintain it.

So the question is not whether to read font files. The instruction says so.
The question is whether to write the reader.

## Decision

**The project keeps the drawing and borrows the reading.**

- `skrifa` is the dependency: a pure-Rust reader for OpenType fonts, from
  the Fontations project, under MIT or Apache-2.0. It reads TrueType
  outlines, CFF outlines, the character map, the advance widths, and the
  deltas of a variable font, and it scales an outline to a size. It does not
  shape, hint or draw. It is the project's only dependency besides `libc`,
  and neither is a C library.
- Everything that is *this project* stays here: the drawing of an outline
  into the frame, the layout of a run of glyphs, the document format, the
  terminal. An outline arrives from the reader as a path - the same path
  type the panel's own `path` directive uses - and the core's path painter
  draws it with the winding rule it already has. A glyph is therefore
  nothing new to the core, and the rasteriser grows a case, not a renderer.
- **Shaping is not done.** A run of text is laid out by advancing: each
  glyph is placed at the previous one's advance width. There is no kerning,
  no ligature, no complex script, and no hinting. A panel is a panel and not
  a word processor. A text that needs shaping is named in the report rather
  than drawn wrongly.
- **The faces are embedded in the binary** and chosen by name in the
  document. A panel says `font "dejavu-mono"` and the labels that follow are
  set in that face; a panel that says nothing is set in the built-in font.
  The number in a text directive is the built-in font's scale, or the size in
  pixels of a named face's em box. The document says which face it is using,
  so the two are never confused; a panel that mixes them is asking for
  trouble it can have.
- **A variable font is a font at a location.** The panel may name the axes
  and the coordinates: `font "inter" 16 wght=700 wdth=100`. A face with no
  axes is a face with one location, and the same code draws it.

## Consequences

- A panel may be written in any of the three formats, and the difference
  between them is inside the reader rather than inside the project.
- The built-in font stays, and stays the default: a panel that says nothing
  about fonts is a panel that runs on anything.
- The binary grows by the size of the faces it embeds. That is the cost of
  the licence and the cost of "out of the box", and it is paid once.
- A character a face does not have is still a hole, and is still named. A
  real face simply has far fewer holes than fifty-five glyphs.
- The size rule of ADR 0003 does not change for the built-in font. For a
  named face, a size in pixels is exact, and the approximation of the
  translator's em box does not apply.

## Revisit if

- shaping is needed, which means a run is not a sequence of glyphs and the
  panel's label is not a string;
- a panel needs a font it does not have, which is a matter of adding a file
  and a name;
- the embedded faces are too heavy to ship, and the panel names a file on
  disk instead.
