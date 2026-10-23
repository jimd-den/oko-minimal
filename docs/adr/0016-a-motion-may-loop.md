# 0016 — A motion may loop

Status: accepted · Date: 2026-10-22

## Context

ADR 0008 gave a shape legs: a delay and a chain of `to <state> over <duration>`
steps, after which the shape comes to rest at its last target. For a one-shot
reveal or transition that is enough. For an instrument wall — a Star Trek LCARS
bridge console with a sweeping sensor bar, a pulsing warp-core gauge, and a
blinking alert beacon — different shapes on the same screen repeat at different
periods forever.

ADR 0008 left looping to the runtime wrapping a single panel-wide clock. A
single wrap period forces every moving shape on a panel to share one cycle
length, or resets a slow sweep whenever a fast indicator wraps.

## Decision

**A motion may declare `loop` after its legs, and `timeline::at` wraps the
elapsed time after the delay by the sum of its legs.**

- In a document, `loop` is written at the end of a shape's motion clause,
  before any `when <flag>` condition:

      rect 10 20 8 4 #ff9d00 to 60 20 8 4 #4fc3f7 over 1s to 10 20 8 4 #ff9d00 over 1s loop

- A motion with no legs cannot loop; `loop` without a `to` leg is refused at
  parse time with a message naming the mistake.
- Before `delay`, a looping shape holds its initial state. At or after `delay`,
  `elapsed = (now - delay) % legs_duration`, and the leg covering `elapsed` is
  interpolated linearly as before.
- `Scene::is_looping()` and `Panel::is_looping()` report whether any active
  element loops. While `panel.is_looping()` is true, `Runtime::tick(now)` never
  marks the panel settled.

## Consequences

- Each shape on an animation-heavy panel can repeat on its own period while
  one-shot transitions on the same panel still come to rest.
- `core` remains a pure function of `now`: no clock and no mutable tick counter
  enter the scene or the timeline.
