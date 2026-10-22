# 0015 — The damage is computed in the presenter

Status: accepted · Date: 2026-10-21

## Context

A panel is redrawn sixty times a second. The obvious way to do that sends
the whole screen every time, and re-rasterises a whole frame into a fresh
buffer every time. On a terminal that is a screenful of bytes per tick to
say nothing, and a full-size allocation per tick to say nothing.

The question is where the damage — the part of the picture that actually
changed — is computed. Two answers were on the table.

- **In the scene:** a shape with legs (ADR 0008) knows what it will become,
  so the scene could say which shapes moved and where their pixels went.
- **In the presenter:** the presenter already holds the frame it last showed
  and the frame it is about to show, so it can compare them.

## Decision

**The damage is computed in the presenter, by comparing the cells.**

- The core is untouched by this question. A scene is a function of a moment
  and is rasterised whole; the core has no notion of a dirty region, and
  this decision does not give it one.
- The presenter keeps the terminal's cells as they were last shown, and
  compares them with the cells of the new frame. The cells that differ are
  the damage.
- The damage is sent as runs: one cursor move per run of cells, then the
  cells. A frame that is the same as the last one sends nothing at all.
- The frame is reused. One buffer is kept, cleared and painted into; a tick
  does not allocate a second frame.

## Alternatives

- **Dirty regions from the shapes that have legs.** Rejected: a shape with
  legs has no bounds until it is sampled at a moment, and the panel that
  contains it is the sum of many shapes whose legs may overlap. The scene
  would have to answer "what changed between two moments", which is a second
  notion of time beside the one it already has, and a second source of truth
  about what a frame is. The presenter already holds both frames; the
  comparison is one pass over the cells and no new concept.
- **Compare frames in the core.** Rejected for the same reason from the
  other side: the core would need to keep the previous frame, which is state
  the core does not have, for a question that only the terminal asks.

## Consequences

- The presenter sends what changed, and nothing else. A panel at rest is
  silent.
- The core stays a pure function of a moment and a size, and the adapters
  stay where the outside world is.
- The cost that is not removed: the scene is still rasterised whole each
  tick. That cost is the rasteriser's, and the rasteriser is not where the
  bytes go.

## Revisit if

- rasterising a whole scene becomes the bottleneck, which would be an
  argument for the core answering "what changed" — and not for the shapes
  knowing where their pixels went.
