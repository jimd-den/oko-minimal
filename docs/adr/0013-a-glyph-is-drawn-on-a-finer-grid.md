# 0013 — A glyph is drawn on a finer grid and brought down

Status: accepted · Date: 2026-10-21

Context: ADR 0011 said a glyph is a path: the outline arrives as
`core::path::Path` and the core's path painter fills it, so a real font is
not a second renderer. That promise meets a fact about the core: a path's
points are whole pixels, `geom::Point` being `i32`.

A glyph's outline is not whole pixels. Its control points sit at fractions of
a pixel - a sixteenth of a pixel at a 16 pixel em - and rounding them to
whole pixels does not move the glyph a little: it destroys it. The outline
*is* the shape, and the shape is made of those points.

The tempting fix is to give the core's path fractional points. The ripple is
wide: the path's segments, the painter, the document's path reader, the
translator, and every test that names a point. That is a refactor of the
core's geometry, and it is not this pull request's work.

Decision: **a glyph is drawn on a grid four times finer than the frame, and
brought down.**

- The font's module scales the outline up by four and builds a path of whole
  points on that finer grid. The rounding error is an eighth of a pixel,
  which no one can see.
- The core's path painter fills that path into a small frame of its own - the
  painter is the core's, the frame is the font's - and the font's module
  brings the result down into the frame the panel is drawn on.
- The reduction takes the nearest pixel. The raster has no anti-aliasing
  anywhere else either, and a font that is grey where the rest of the panel
  is flat would be a font from another program.

Alternatives: fractional points in the core's path (rejected for now: the
refactor is wide, and the fine grid buys the same accuracy inside the font
alone); the font rasterises its own glyphs (rejected: that is the second
renderer ADR 0011 refused); the outline is rounded to whole pixels (rejected:
it destroys the glyph).

Consequences: a glyph is drawn on a frame of its own, four times the size of
its place, and that frame is temporary. A text of many characters draws many
temporary frames; a glyph that is drawn once per frame is a cost the panel
pays, and a font's cache is a later decision, not this one's.

Revisit if: the core's path takes fractional points, at which point the fine
grid has nothing left to do.
