# 0014 — The font is the panel's

Status: accepted · Date: 2026-10-21

## Context

The instruction is plain: make the fonts globally changeable at runtime,
without every component having to refresh or remount when the font changes.

ADR 0011 already answered the harder half of that: the faces are embedded in
the binary and a document names one. But it answered it in the wrong place.
It put the face in the *document* and left the size of a text to mean two
things: "the built-in font's scale" for one panel and "the size in pixels of
a named face's em box" for another.

That second meaning is the problem. If the number in a text directive means
different things under different faces, then swapping the face changes what
every number means, and every label changes size with it. A panel whose font
changed would then have to be laid out again — every label measured again,
every position checked again — which is exactly the refresh the instruction
forbids. A font that can be changed at runtime must be changed *without
anything else changing*.

## Decision

**The font is a setting of the panel, and the panel has one font.**

- A `Panel` holds one `Option<typeface::Setting>`. A text shape carries no
  face, and never will: a shape says what it draws, and the panel says in
  what face. Changing the font is one assignment, and the shapes it draws
  afterwards are the shapes it drew before — their text, their positions,
  their motions and their flags are untouched.
- **The font travels with the display list.** A `Scene` carries the font, so
  the raster paints the text in the scene's font and the renderer needs no
  new parameter. A panel's `font()` and `set_font()` read and write the
  scene's font.
- **The number in a text directive is always the built-in font's scale.** One
  number, one meaning, whatever the font is. It never changes meaning when
  the font changes, because the font does not get a say in it.
- **A named face is drawn to the built-in font's capital height.** The
  built-in font's capitals fill its cell, and its cell is seven rows tall, so
  the capitals of a label at scale *n* are *7n* pixels tall. A named face is
  drawn so that its capitals are exactly that tall: the face's em size in
  pixels is `7n / cap_ratio`, where the capital ratio is the face's own
  capital height as a fraction of its em. Two faces set the same way are set
  to the same size, which is what the eye compares between typefaces.
- **The baseline does not move.** A named face's text stands on the same
  baseline the built-in font puts it on: `at.y + 7n`, the bottom row of the
  built-in font's cell.
- A document names a face with `font "inter"`, or at a weight with
  `font "inter" wght=700`. A face the program does not carry is refused by
  name.

## Alternatives

- **The face on each shape.** Rejected: changing the font would mean
  rewriting every shape in the panel, which is the refresh the instruction
  forbids, in the most literal way available.
- **The number in a text directive as the named face's em box in pixels**
  (ADR 0011's rule). Rejected: the meaning of every number in the document
  would depend on the face, so a swap would resize every label.
- **A size on the font line, `font "inter" 16`.** Rejected: the em box is
  not what the eye compares between two faces — their capital heights differ
  by up to a fifth at the same em — so a swap would visibly change the size
  of the panel; and it would be a second way to say the same thing as the
  text's own scale.
- **A face per run of text.** Not now: it is a document feature, and it
  would put the face back on the shape.

## Consequences

- A panel that names a face is a panel written in that face, throughout. The
  mixture ADR 0011 warned about — a panel that mixes the built-in font and a
  named face — is now impossible rather than merely unwise.
- Swapping the font at runtime changes the letterforms and nothing else.
  The label's size, its baseline and its position are the same before and
  after.
- The built-in font stays the default: a panel that says nothing about fonts
  is a panel that runs on anything.
- A face's advances are the face's own, so a swap changes how wide a label
  is. That is the typeface's own metrics, and not a relayout of the panel.

## Revisit if

- a panel needs two faces in one line, which is a feature of the document
  and not of the core;
- shaping is needed, which means a run is not a sequence of glyphs.
