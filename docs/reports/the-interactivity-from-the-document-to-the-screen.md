# The interactivity, from the document to the screen

A panel declares its flags, binds its keys, and says which shape is drawn
under which flag.

```oko
flag cursor
clear #101820
text 4 5 "ENTER: SHOW  TAB: HIDE" 1 #4fc3f7
rect 4 10 12 7 #ff9d00 when cursor
on key enter set cursor true
on key tab set cursor false
```

## What changed, and why it is shaped this way

**A document describes a panel, not a bare scene.** A panel is a scene, a set
of flags and a set of bindings (ADR 0009), so `document::parse` now returns a
`core::panel::Panel` and a `Scene` is what a panel draws. Everything that
read a scene - the presenter, the runtime, the translator's tests - reads the
panel's scene now, and the compiler said so at every one of them.

**The three lines of the grammar** live in `document::panel`:

- `flag <name> [true|false]` declares a flag. A flag with no value is
  false: a panel that has just been drawn has nothing turned on yet.
- `when <name>` is a clause on a shape's line, after any motion, so a line
  reads as the shape, then what it becomes, then when it is painted at all.
- `on key <key> set <name> true|false` and `on key <key> quit` bind a key.
  The key is a named key (`enter`, `tab`, `backspace`, `escape`, `up`,
  `down`, `left`, `right`) or a single character. The escape *chord* is not
  bindable, because it is the way back to the console.

**The refusals** name the line they came from:

- a flag used before it is declared - in a binding or in a condition - is
  refused with the line of the use, not the line of the declaration
- a flag declared twice is refused
- a key that is not named is refused by name
- a flag's value is `true` or `false`, and anything else is refused

**The runtime gives the key to the panel first**, and the session hears it
only if the panel does not want it. The panel is the content; the session
holds the screen's authority. The escape chord cannot be bound, so it always
reaches the session.

## A bug this found

The shell presented a frame only when the document changed. A resize
re-rendered the frame and nothing put it on the screen, so a resized panel
was never seen. The shell now presents when the runtime says the picture
changed - which is also what a flag change needs.

## The evidence

Through a pseudo-terminal, on a real terminal, with the real program:

```
the orange shape before the key: 0
the orange shape after the key:  4
```

The rectangle is not on the screen until the enter key arrives, and it is
afterwards.

## The tests

Fourteen new tests, and each one was watched failing: with the document's
readers disabled and the runtime's key handling disabled, every one of them
failed, and none of the others did.

- twelve in `core::document::panel_tests`: the flag's value, the shape's
  condition, the key that sets a flag, the key that ends the program, the
  undeclared flag with its line, the flag declared twice, the unknown key.
- two in `adapters::runtime_tests`: the key that reaches the panel and
  changes the frame, and the binding that hands the screen back.

The document's own refusals kept their tests, through the document's own
door: the unknown directive, the short colour, the coordinate out of range,
the trailing word, the unclosed label, the character the font does not have
and the zero scale.
