# 0008 — A shape may have legs, and a scene is a function of time

Status: accepted · Date: 2026-10-21

## Context

A panel is a text file, and a text file can say what a shape becomes and
when. A dot that slides across a panel, a bar that fills, a colour that
changes: none of these need a new drawing primitive. They need the drawing
command to change over time.

The whitepaper says a screen is not a function of time, it is a function of
state. That claim is about the screen's *authority* - the lease decides who
may present, and the focus decides where a key goes. It is not about the
content on the screen. The content of a panel can be a function of time, and
the authority over it is still the session's.

The naive design is to give the core a clock and let the renderer ask what
time it is. That breaks the purity the whole project rests on: the core
would have a hidden input that no test can control.

## Decision

**A shape may have legs, and the scene at a time is a pure function of that
time.**

- The display list's operation becomes a shape plus a motion. A shape is the
  value it always was: a clear, a rectangle, a line, a text, a path. A
  motion is a list of legs, and a leg says what the shape becomes and how
  long the change takes. The legs chain: the second leg starts where the
  first one ended, and the shape holds its last state when they are done.
- A shape with no legs does not move. A shape with one leg is a tween. A
  shape with several legs is a sequence, and the core interpolates between
  the two legs that surround the moment in question.
- The core has no clock and never reads one. `Scene::at(now)` is a pure
  function: the same scene and the same moment always give the same scene.
  The runtime owns the clock and asks.
- The document states the motion on the same line as the shape, so a person
  reading a panel sees the shape and what it does together:

      rect 4 4 4 4 #ff9d00 after 500ms to 60 4 4 4 #ff0000 over 2s

  A leg is `to <state> over <duration>`, and a duration is a number and a
  unit, `ms` or `s`. A shape may carry a delay before its first leg.
- What interpolates is what a tween can move: a position, a size, a colour.
  A text's label does not interpolate - a text that becomes a different text
  is refused, and named. A path may move to a path of the same structure, and
  a path that becomes a different path is refused, and named.
- The interpolation is linear. An easing curve is a function from progress to
  progress, and it is a later decision; the shape of that decision is already
  here, because a leg would carry one.
- The scene knows how long it is: the longest motion in it. A runtime that
  wants a panel to loop asks the scene for its length and wraps the clock.
  The document does not say `loop`, because looping is what the *runtime*
  does with a clock and not something a shape is.

## Consequences

- A panel can move, resize and recolour. It cannot fade: the document format
  has no alpha, and a translucent colour is still refused by name.
- The core's central type changes: an operation is a shape and a motion, not
  a shape. Every place that builds one changes, and the compiler says so.
- The runtime gains a cadence. It asks for the scene at the current moment
  and presents it, so the presenter's job stops being "paint once".
- The whitepaper's sentence about time needs one precise amendment, because
  the content of a panel is now a function of time while the authority over
  the screen is still a function of state.

## Revisit if

- a panel needs easing, which is a function from progress to progress;
- a panel needs to loop, which is a question for the runtime and not for the
  document;
- an export carries its own animation - SVG has elements for it - and the
  translator has to decide what to do with them.
