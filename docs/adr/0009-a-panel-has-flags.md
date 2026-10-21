# 0009 — A panel has flags, and a key sets a flag

Status: accepted · Date: 2026-10-21

## Context

ADR 0007 built the loop: the runtime reads keys, hands them to the session,
and performs the effects. But the session's answer for an ordinary key is
`Deliver { surface, key }` - and there is no surface. A key reaches a
surface and nothing happens, because the document format has no actions.

The whitepaper says a screen is a function of state, and that state changes
when events arrive. Today the only state is the lease and the focus. A
panel's own state - what it is showing right now - is not state at all, and
so a panel cannot answer a key.

The temptation is to let the document say what to do with a key in free
form: `on key q exit`, or worse, a line of shell. A panel is a bounded
input to a total function, and that is a promise the translator already
makes about SVG; the same promise has to hold for the panel's own grammar.

## Decision

**A panel is a scene, a set of flags, and a set of bindings. A key changes a
flag, and a flag decides what is drawn.**

- The document gains three things:
  - `flag <name> [true|false]` declares a named flag. A flag that is not
    declared is not a flag, and naming one in a binding that was never
    declared is refused.
  - `when <name>` on a shape's line says the shape is drawn only while the
    flag is true. It is another clause on the same line as the shape, the
    way the motion is (ADR 0008), so a person reading a panel sees the
    shape and the condition together.
  - `on key <key> <action>` binds a key to an action. The actions are
    `set <name> true`, `set <name> false`, and `quit`. Nothing else: a
    panel may not run a command, read a file or change anything outside
    itself.
- A key is one character, or one of the named keys: `enter`, `tab`,
  `backspace`, `escape`, `up`, `down`, `left`, `right`. The reserved escape
  chord is the session's and is not bindable, because it is the way back to
  the console and a panel must not be able to take it.
- The core gains a panel: the scene, the flags and the bindings, and two
  questions. `panel.key(k)` applies the binding for a key and says whether
  the panel changed. `panel.scene(now)` is the scene to draw - the shapes
  whose flag is false are not in it, and the rest are at the moment asked
  for.
- The flags live in the panel and not in the session. The session is about
  the screen's authority - the lease, the focus - and the flags are about
  the content. The runtime is what connects them: the session says which
  surface a key goes to, and the runtime gives it to the panel.
- The panel's state is not the session's, and the two are not merged. A
  second panel would have its own flags, and the lease would still say who
  may present.

## Consequences

- A panel answers a key, and the answer is visible: a shape appears, a
  shape goes away, the program ends.
- The display list's element grows a third part: the shape, the motion, and
  the condition under which it is drawn. Every place that builds one
  changes, and the compiler says so.
- The grammar stays small and total. There is no expression, no variable
  arithmetic, and no way out of the panel.
- A panel cannot do what a script does, and that is the point: a panel is a
  document a person edits, and a document that can run a command is a
  document that can be edited into one.

## Revisit if

- a panel needs a number rather than a flag - a counter, a scroll
  position - which is a value rather than a boolean;
- a binding needs a modifier - `ctrl+s` - which the named keys do not carry
  yet;
- a panel needs more than one surface, and the session's focus has to choose
  between panels rather than between a panel and nothing.
