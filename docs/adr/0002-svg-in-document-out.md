# 0002 — SVG in, panel document out

Status: accepted · Date: 2026-10-21 · Text amended by
`docs/adr/0003-svg-text-mapping.md`

## Context

Esoteric panels need a visual editor. Building one is a large detour: selection,
handles, snapping, undo, and a canvas that is itself a display-server client.
Penpot and Figma already do that work, and both export SVG. SRD-002 already names
SVG as an optional authoring client rather than a runtime format.

The runtime's input is a display list. A document is its text form. The question
is whether the authoring tool writes the document directly, writes a scene, or
writes something the console must interpret at runtime.

## Decision

**A bounded SVG-to-document translator is the only new component.**

- Input: the subset of SVG that a panel export uses — `svg`, `g`, `rect`,
  `line`, `polyline`, straight-line `path` (`M`, `L`, `H`, `V`, `Z`), `translate`
  transforms, `viewBox` origin, hex and `rgb()` colours, full opacity, and
  `text` with its `tspan` lines, which ADR 0003 added later.
- Output: the existing panel document, one directive per line.
- The document remains the source of truth. It is human-readable, diffable, hot
  reloadable, and independent of the editor that produced it.
- The translator is total and bounded: 1 MiB input, 4096 elements, 64 attributes
  per element, 4096 path points, and the document format's own coordinate and
  size limits. Every refusal names a line.
- Curves, rotation, scale, opacity below one, inheritance on groups and unknown
  elements are refused, not approximated. Text was refused on the same grounds
  until the built-in font made a mapping possible; ADR 0003 records what
  replaced that refusal. A silent approximation is a picture nobody can trace
  back to the source.

`oko-svg panel.svg > panel.oko` is the whole bridge. The translator is a library
function (`adapters::svg::translate`) first and a binary second, so it is tested
without a filesystem.

## Alternatives

- **A built-in visual editor.** Deferred, not rejected. The document is the seam
  an editor would write; the translator makes an editor optional on day one.
- **Render SVG at runtime.** Rejected: a full SVG renderer is a large, untrusted
  surface in the boot path, and it would make the document redundant.
- **A Figma/Penpot API integration.** Rejected: network, tokens and a moving
  schema in the boot path.
- **HTML/CSS as the authoring language.** Rejected: it pulls a browser into the
  one place the project is trying to remove it from.
- **Emit a scene directly.** Rejected: the text is the artefact a person can fix.

## Consequences

- A v1 design cannot use curves, rotation or transparency. Text was the first
  limitation to remove, and it has been: the bitmap font exists and the
  translator maps SVG text onto the document's text directive. Curves and
  transforms still need new display-list primitives.
- Scaling from a design-time viewBox to a runtime frame is not performed. The
  panel is authored in pixels at the target size; this is a deliberate limit.
- The translator must track the document grammar's limits. It imports the same
  constants, so the two cannot drift, and it counts the operations it writes,
  because one element can become many directives.
- The editor's output is not trusted. The XML scanner refuses doctypes and
  entities, and the translator refuses anything outside its whitelist.

## Revisit if

- curves become necessary for the first real panel, which forces a path fill
  rule — the text half of this trigger has already fired, and ADR 0003 is the
  decision that answered it;
- exports begin to use `<use>`, `<defs>` or stylesheets, which forces a symbol
  table and a second pass;
- an embedded editor appears, which should write the document and delete the
  translator's reason to grow.
