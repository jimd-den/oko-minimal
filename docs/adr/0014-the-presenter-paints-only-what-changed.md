# 0014 — The presenter paints only what changed

Status: accepted · Date: 2026-10-21

Context: the shell repainted the whole frame on every tick of a panel's
timeline, and the whole frame again for a single keypress. Measured on
the panels in `examples/` (see the report): at 1920×1080 a full frame is
~3.7MB of terminal bytes, a finance panel's six-second timeline cost
~100MB on the wire, and under 1% of the cells change between frames
while a sweep crosses. On SRD-002's serial target a full frame at any
size is seconds of line time, so sweeping data animation - the workload
these panels exist for - cannot cross the wire whole.

Decision: the presenter gains a third shape, `screen_update(previous,
frame)`, which emits only the runs of cells whose colours changed, each
placed at its address. The shell remembers the frame the screen is
showing and asks for an update against it; the first frame, and any
frame whose size differs from the previous one, is painted whole with
`screen`, because a diff against a different geometry is a guess.

Alternatives: dirty regions computed in the scene (rejected for now:
the wire saving does not need them, and the core stays free of
presentation state; the raster cost of a frame is unchanged by this
decision and remains as follow-up work). Repainting whole rows that
contain a change (rejected: a 5-pixel sweep would still cost a full
row at every step, and rows are the wide axis of these panels).

Consequences: the wire cost of motion is now proportional to the
motion. During a timeline the shell still renders every frame whole -
only the painting is diffed - so a slow raster still caps the frame
rate; that is the next problem, and it is now the only one on the wire
path. Stateful painting makes the shell's ordering load-bearing: the
frame is painted after the display is taken, because the alternate
screen starts blank (see the report for the bug this fixes).

Revisit if: frames grow large enough that remembering the previous
frame is itself a cost, or the raster becomes the binding constraint
and scene-side damage is measured to beat presenter-side diffing.
