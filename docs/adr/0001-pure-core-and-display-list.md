# 0001 — A pure core, an adapter ring, and a display list between them

Status: accepted · Date: 2026-10-21

## Context

`oko-minimal` is a fresh start. The sibling `oko-mvp` explored a browser engine
in the boot path, a surface wire format, and a getty handoff; those decisions are
recorded there and are read-only prior art here. SRD-002 fixes the first screen,
the display lease and the input rules. The brief adds a second half: panels that
replace a display server, stay hot-editable, and run on anything with a terminal
or a framebuffer.

The two halves share one question: who may paint, and what is a paint. They must
not share a lot else, or the boot path inherits the UI's complexity.

## Decision

One crate, split at one line:

- `core` is pure. Geometry, frames, scenes, rasterisation, documents, the lease,
  focus and the session reducer. No I/O, no clock, no global state.
- `adapters` is the outside world. A terminal encoder, a panel painter, an SVG
  translator. Anything that can fail for a reason the program cannot control
  lives here.
- `main` and `bin/` are composition roots. They name files and terminals; no
  other module does.

The contract between the halves is a **display list**: an ordered `Scene` of
`Clear`, `Rect` and `Line` operations, rendered into an RGBA `Frame` by a pure
function. A **panel document** is the human-edited text that parses into a
`Scene`. A frame is disposable; a scene is small and comparable; a document is
diffable.

The screen's ownership is two small state machines, both pure:

- `Lease` — one owner, a generation per grant, revocation that no stale frame
  can survive.
- `Focus` — the reserved escape chord first, then the focused surface, then
  nothing; input is dropped, never queued unbounded.

`Session::dispatch(event) -> Option<Effect>` composes them. The runtime performs
effects; the session decides. Events are values (`Key`, `Resize`), so a test
needs no terminal.

## Alternatives

- **A retained scene graph.** Rejected for v1: every node is state to
  synchronise, and hot reload becomes a mutation protocol instead of a parse.
- **Immediate-mode callbacks inside the process.** Rejected: it puts application
  code in the console's trust zone before there is a reason to.
- **JSON documents.** Rejected: no JSON parser in `std`, and a line of text is
  easier to write by hand than a tree of braces.
- **Embed a browser for the UI layer.** Rejected: it is the dependency the
  project exists to remove from the boot path. An optional client can come later.
- **Keep the fixed 26-byte surface wire format from `oko-mvp`.** Deferred, not
  rejected: the in-process scene is enough for one session; the wire format is
  the right next step when apps become processes.

## Consequences

- A full frame is redrawn per present. At terminal sizes this is microseconds;
  at 1280x800 it is not. Damage rectangles are the next performance decision.
- The core can be tested in one line per rule. `cargo test` is under a second.
- Text, pointer input, images and transforms are absent from the display list
  until a test demands them. The document grammar rejects them rather than
  ignoring them.
- The console's own lifecycle (boot, login, session) is not yet a reducer here;
  the lease and focus rules it will use are.

## Revisit if

- frame rate matters on a framebuffer, which forces damage tracking;
- panels need a node identity that hot reload must preserve (focus, scroll), which
  a flat list cannot express;
- a second process needs to paint, which forces the wire format back in.
