# 0004 — A panel draws what it can and reports what it cannot

Status: accepted · Date: 2026-10-21

## Context

A real export does not load. The research in
`docs/reports/svg-export-shapes.md` ran the console's own binary against two
real files and both were refused before a single word was drawn: a Figma
export at the entity in a soft line break, a Penpot export at the entity in a
font family name.

That was the first wall, and it was fixable. The second wall is the shape of
the translator itself. It is all-or-nothing: one shape it cannot draw fails
the whole file. A Penpot export carries four paths and all four of them
curve; a Figma export carries four hundred and fifteen paths, of which two
curve. So even with the inputs understood, the file would still fail, on a
shape nobody can see.

The rule that produced this is the one the whitepaper states: a silent
approximation is a picture nobody can trace back to the source. That rule is
about *approximating*. Refusing everything is not a stricter version of it; it
is the same as having no translator at all, because no real export contains
only straight edges and flat colours.

## Decision

The translator separates two kinds of failure.

**A hard error** is a file the translator cannot honestly translate: malformed
XML, an entity it does not know, a document that would break the format's own
limits — the operation budget, the coordinate range, the size of a label.
These are errors, with a line number, and no document is produced.

**A skip** is a shape the translator cannot draw: a curve, a paint reference
such as `url(#gradient)`, a transform it does not implement, an element
outside the whitelist, a label with a character the built-in font does not
have. The shape is left out, and the reason is recorded with its line.

`translate` therefore returns a translation, not a string: the document, and
the list of what was left out. The command line writes the document to
standard output and the list to standard error, and exits zero — a panel with
holes is still a panel, and the holes are named.

## Consequences

- A real export loads. A panel drawn from Penpot's own export shows its
  labels and its rectangles, and reports the four curved shapes it could not
  draw.
- The console must show the report. A panel that silently lost a shape is the
  failure the whitepaper warns about; a panel that says which shape it lost
  is not.
- The whitepaper's rule stays, and gets sharper: approximations are still
  refused, and what cannot be approximated is reported rather than hidden.
- A curve is the biggest thing a real export can lose. Curves are also the
  next thing the display list would need, and skipping them is the honest
  reason to add them.
- The translator's public signature changes, so every caller changes with it:
  the `oko-svg` command, the tests, and the acceptance test.

## Alternatives

- **Refuse the file, as today.** Rejected: no real export loads, so the
  authoring bridge is a claim rather than a feature.
- **Approximate a curve with a polyline.** Rejected: it needs a curve
  rasteriser before it can be honest, and until then the approximation would
  be silent — the thing this project refuses to do.
- **Fail the panel on any skip.** Rejected: that is the status quo with a
  different error message.
- **Refuse only the shapes, silently.** Rejected: silence is the failure mode
  this whole project is built against.
