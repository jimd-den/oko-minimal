# 0012 — The panel's clock lives in the shell

Status: accepted · Date: 2026-10-21

Context: the animation lives in the core: `Scene::at(now)` is a pure function
of a scene and a moment. Nothing yet asks for it, so a panel with a motion
stands still. The core has no clock and must not grow one (ADR 0007 keeps
the device outside the runtime; a clock is a device).

Decision: the shell keeps the clock and the origin, and asks the runtime for
a moment.

- The shell measures the time since the document arrived — the origin, not
  the epoch: a panel has no meaning in absolute time — and calls
  `Runtime::tick(now)`.
- The runtime answers whether the picture changed. It paints while the
  timeline is running, and after the last shape has come to rest it paints
  once more and then stays still.
- A panel with no motion is painted once, when it is loaded, and never
  again.
- The shell's loop turns at the frame rate, not at the file's polling rate:
  a motion is 60 frames a second, not ten.

Alternatives: a clock inside the core (rejected: a core that reads the
clock cannot be tested at a moment, and cannot be reasoned about); the shell
comparing frames itself (rejected: the runtime already knows what it
painted, and a second source of truth about the picture is a second thing
to be wrong); a timer thread (rejected: no async runtime, and the loop
already turns).

Consequences: the runtime gains one method and two words of state. The
shell's loop turns faster, so the file's timestamp is read more often — a
stat is cheap, and a 100 ms poll was a number chosen for the file rather
than for the picture.

Revisit if: the panel ever runs off the terminal, where the frame rate is
the display's and not ours.
