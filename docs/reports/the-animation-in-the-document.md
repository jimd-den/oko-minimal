# The animation in the document

A panel can say what a shape becomes and when.

```oko
rect 4 4 4 4 #ff9d00 after 500ms to 60 4 4 4 #ff0000 over 2s
```

## The grammar, and why it is a grammar at all

The clauses are recognised by their **keyword**, never by the length of the
fields before them. `after`, `to` and `over` are the keywords, and the reader
asks one question after the shape's own fields: *is the next word one of
them?*

That is what lets a shape of any length carry a motion. A rectangle has five
fields and a path has a list of segments, and neither needs to know how long
its own fields are: the path ends its fields at the word `to` exactly as the
rectangle does.

- `document::words` cuts a line into words and reads the values among them. A
  quoted run is one word, so a label that holds the word `to` is not the
  keyword `to`. A duration is a number and a unit, `ms` or `s`, and the unit
  is always written.
- `document::fields` reads a shape's own fields, and the fields of the state
  it becomes, by the *same* rules, so a shape and its target cannot drift
  apart. A target of another kind is refused there, by name.
- `document::motion` reads the clauses: a delay, and then any number of
  legs, each saying what the shape becomes and how long the change takes.
  The legs chain.
- The shape and the target are handed to the core, which refuses the pairs
  that may not be one - a text that becomes a different text, a clear that
  moves - and names them.

## The refusals, and what they say

- a `to` with no `over`: a leg needs `over` and a duration
- an `over` with no `to`: `over` needs a `to` to say what the shape becomes
- a duration with no number, with no unit, or with an unknown unit
- a target of another kind of shape: *a rectangle cannot become a text*
- a text that becomes a different text, and a clear that moves

## The motion, seen

The same panel at three moments. The orange shape slides from x=8 to x=88
over two seconds, and the blue marker stands still:

```
--- 0ms ---                                  --- 1000ms ---                               --- 2000ms ---
        ##########                                   ...                                  ...########
```

The frames are the core's own: `Scene::at(now)` is a pure function, and the
renderer is given the scene it sampled. The runtime does not have a clock
yet - that is the next slice - so the frames here were produced by a small
program that asks the core for the scene at a moment and paints it.

## The tests

Fourteen tests for the grammar, and the refusals keep the tests they had:
the unknown directive, the short colour, the coordinate out of range, the
trailing word, the unclosed label, the character the font does not have and
the zero scale.

Two of the syntax layer's tests moved with the behaviour they test - the
size and the end of the line are read among the words now - and the readers
the document no longer uses were taken away, because dead code is a promise
nobody keeps.

## What this work cost

The child that wrote it thrashed for two hours over the same three modules
and never committed; the harness called it unreachable while its process
carried on writing over the work. The integration is the parent's: the
clause reader and the wiring were written, the tree compiled again, three
test lines that were malformed were corrected, and the ceiling was held by
naming two seams - the duration is the motion's own unit, and the blend's
tests have their own file.
