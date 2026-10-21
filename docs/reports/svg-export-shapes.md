# What Figma and Penpot actually put in an exported SVG

Research for `docs/adr/0003-svg-text-mapping.md`. Read-only: no source file,
test, document or configuration was changed. The only file written is this
report.

**Method.** Where a claim is about a real export, it comes from a real exported
file, quoted verbatim with its URL. Where a claim is about a source file, it
comes from that source file at a pinned URL. Several claims were also
**executed**: the console's own `oko-svg` binary was run against the real
exports, and its error messages are quoted. I did not have a Serper API key,
so web search was done through DuckDuckGo's HTML endpoint and GitHub's public
code-search APIs.

## Summary

The ADR's model of an exported text element is close to Figma's and wrong
about Penpot's in four ways that matter:

1. **Figma puts `font-size` in an XML attribute, not in `style`.** Its `style`
   attribute holds only `white-space: pre`. The `style="font-size: 14px"` form
   the ADR assumes is Penpot's, not Figma's.
2. **Penpot writes one `<text>` element per line**, not one `<text>` with
   `<tspan>` children. There is no `<tspan>` in a Penpot export at all.
3. **Neither exporter writes `dy` for a second line.** Figma writes a second
   `<tspan>` with its own absolute `y`; Penpot writes a second `<text>`. The
   `dy` rule in the ADR describes a mechanism neither tool emits.
4. **A second Penpot line is a sibling `<text>`, and Figma's second line is a
   sibling `<tspan>`, but Penpot's *first* line already has its position on the
   `<text>` itself.** So the two tools disagree about where the position
   lives, and the ADR's rules are written for Figma's arrangement.

A fifth fact is larger than all four: **Figma outlines text by default.** A
default Figma SVG export contains no `<text>` element at all.

And an execution result that outranks even that: **the console's translator
refuses both real exports before it reaches a single `<text>` element.** The
failures are not text-mapping problems. They are listed in section (e).

## (a) Figma: element and attribute names for a text label

Figma emits a `<text>` element whose content is one `<tspan>` per line. The
`<text>` carries the font attributes; the `<tspan>` carries the position.

Verbatim from a real Figma export
(`https://raw.githubusercontent.com/ttskch/svg-paper/master/js/tests/resources/real-world-paper-figma.svg`;
the owning repository's test names this file "real world paper created with
Figma" and commits it unmodified from that export —
`https://raw.githubusercontent.com/ttskch/svg-paper/main/js/tests/real-world.test.js`):

```xml
<text id="ESTIMATE" fill="black" xml:space="preserve" style="white-space: pre" font-family="Roboto" font-size="70" letter-spacing="0em"><tspan x="887" y="188.926">ESTIMATE</tspan></text>
```

and a two-line label from the same file:

```xml
<text id="Discounted Subtotal" fill="black" xml:space="preserve" style="white-space: pre" font-family="Roboto" font-size="25" letter-spacing="0em"><tspan x="1563.78" y="2335.04">Discounted&#10;</tspan><tspan x="1580.34" y="2364.04">Subtotal</tspan></text>
```

| Fact | Evidence |
|---|---|
| Element is `<text>`, content is `<tspan>` | the two lines above |
| The `<text>` has **no** `x`/`y`; the `<tspan>` has them | in 218 of 218 `<text>` elements in that file, `x`/`y` appear on the `<tspan>` only (counted by scanning the file) |
| `font-size` is an **attribute** on `<text>`, unitless (`font-size="70"`) | the lines above; the set of `font-size` values in that file is `23, 25, 30, 31, 43, 70` — all bare numbers |
| `font-family` is an **attribute** (`font-family="Roboto"`) | same |
| `fill` is an **attribute**, and can be a **colour name**, not hex | `fill="black"` above; the whole file uses `black`, `white`, `none`, `url(#pattern0)` |
| `letter-spacing` is an attribute, in `em` (`letter-spacing="0em"`) | same |
| `style` holds only `white-space: pre` | the only `style` value on any `<text>` in the file is `white-space: pre` |
| `xml:space="preserve"` is present | same |
| `id` carries the **layer name** when "Include id attribute" is on | `id="ESTIMATE"`, `id="Discounted Subtotal"` |

**Figma outlines text by default.** This is the most consequential fact for
the console. Figma's own help says: "Text is exported as glyphs by default.
This means that you won't be able to edit the text layer once exported. To
preserve text editing, click [Outline text] and uncheck it"
(`https://help.figma.com/hc/en-us/articles/13402894554519-Export-formats-and-settings-for-static-designs`).
The same page lists "Outline text" among the SVG export settings.

The REST API exposes this as a query parameter with the default spelled out.
From Figma's own OpenAPI specification
(`https://raw.githubusercontent.com/figma/rest-api-spec/main/openapi/openapi.yaml`,
path `/v1/images/{file_key}`):

```yaml
- name: svg_outline_text
  in: query
  description: |-
    Whether text elements are rendered as outlines (vector paths) or as `<text>` elements in SVGs.
    ...
  schema:
    type: boolean
    default: true
```

So `GET /v1/images/:key?format=svg` returns **paths, not text**, unless the
caller passes `svg_outline_text=false`. Figma's plugin API says the same
thing: `svgOutlineText?: boolean` — "Defaults to `true`"
(`https://raw.githubusercontent.com/figma/plugin-typings/master/plugin-api.d.ts`,
`interface ExportSettingsSVGBase`).

A third-party write-up of the same switch agrees: "on → text becomes `<path>`
geometry... Off → text stays `<text>` elements"
(`https://freesvgonline.com/blog/figma-svg-export-settings/`).

## (b) Penpot: element and attribute names for a text label

Penpot writes **one `<text>` element per line**, each with its own absolute
`x` and `y`, and each carrying a rich `style` attribute. There are no
`<tspan>` elements.

Verbatim from a real Penpot export
(`https://raw.githubusercontent.com/Floydv149/bibleLinkerPro/master/images/dark.svg`):

```xml
<text x="171.5" y="202.727" dominant-baseline="ideographic" textLength="171.08" lengthAdjust="spacingAndGlyphs" style="text-transform: none; font-family: &quot;Open Sans&quot;; letter-spacing: normal; font-style: normal; font-weight: 700; white-space: pre; font-size: 26px; text-decoration: none solid rgb(176, 176, 176); direction: ltr; fill: rgb(176, 176, 176); fill-opacity: 1;" class="fills" id="fills-a348dfa7-072b-8015-8005-bf7aa92d0a84">in your notes</text>
```

and from the same file, a label whose fill is a gradient, where `fill` moves
**out of `style` and into an attribute** as a `url(#...)` reference:

```xml
<text x="125.5" y="169.182" dominant-baseline="ideographic" textLength="263.565" lengthAdjust="spacingAndGlyphs" fill="url(#fill-0-render-3-0)" style="text-transform: none; font-family: &quot;Open Sans&quot;; letter-spacing: normal; font-style: normal; font-weight: 700; white-space: pre; font-size: 30px; text-decoration: none solid rgba(0, 0, 0, 0); direction: ltr;" class="fills" id="fills-a348dfa7-072b-8015-8005-bf7aa92d0a83">Link to your Bible</text>
```

A second independent Penpot export,
`https://raw.githubusercontent.com/free-explorers/veshell/main/docs/assets/github_logo.svg`,
shows the same shape with a different fill form
(`style="...; fill: rgb(0, 104, 136); fill-opacity: 1;"`).

| Fact | Evidence |
|---|---|
| Element is `<text>`, **one per line** | both Penpot files above; each label is a single `<text>` with no `<tspan>` |
| The line's position is `x`/`y` **on the `<text>`** | `x="171.5" y="202.727"` |
| `font-size` lives in the **`style` attribute**, with a `px` unit | `font-size: 26px`, `font-size: 30px` |
| `font-family`, `font-weight`, `font-style`, `letter-spacing`, `text-transform`, `text-decoration`, `direction`, `white-space` all live in `style` | the same `style` string |
| `fill` is in `style` as `fill: rgb(...)` **or** in a `fill` attribute as `url(#...)` | the two Penpot examples above |
| `dominant-baseline="ideographic"` | the same |
| `textLength` and `lengthAdjust="spacingAndGlyphs"` | the same |
| `class="fills"` and `id="fills-<uuid>"` | the same |

**Why one `<text>` per line.** Penpot's SVG text renderer walks
`position-data` and emits a separate `<text>` element for each entry:
`frontend/src/app/main/ui/shapes/text/svg_text.cljs`
(`https://raw.githubusercontent.com/penpot/penpot/develop/frontend/src/app/main/ui/shapes/text/svg_text.cljs`)
iterates `(d/enumerate position-data)` and renders `[:> :text props (:text data)]`
inside one `[:> :g group-props]`. The per-line text runs come from
`frontend/src/app/util/text_position_data.js`
(`https://raw.githubusercontent.com/penpot/penpot/develop/frontend/src/app/util/text_position_data.js`),
which splits a paragraph into one entry per rendered line using
`getClientRects`.

**Two Penpot export paths exist, and they disagree.**

- The **browser path** (the frontend's "copy as SVG" / object export)
  produces the `<text>`-per-line shape shown above. Sources:
  `frontend/src/app/util/code_gen/markup_svg.cljs`
  (`https://raw.githubusercontent.com/penpot/penpot/develop/frontend/src/app/util/code_gen/markup_svg.cljs`)
  and `frontend/src/app/main/render.cljs`
  (`https://raw.githubusercontent.com/penpot/penpot/develop/frontend/src/app/main/render.cljs`),
  whose `object-svg` component wraps the shape in `<svg id="screenshot-...">`.
  The `id="screenshot-a348dfa7-..."` attribute in the bibleLinkerPro file
  matches that pattern exactly.
- The **WASM/headless path** produces a different shape, and it is what
  `export-shapes` uses when the `render-wasm/v1` feature flag is on
  (`frontend/src/app/main/data/exports/assets.cljs`,
  `https://raw.githubusercontent.com/penpot/penpot/develop/frontend/src/app/main/data/exports/assets.cljs`).
  Its own committed snapshot, in Penpot's repository, is:

```xml
<text fill="#E17FDA" font-size="200" font-family="Source Sans Pro" x="0, 130.37109, 263.08594, 360.83984" y="181">
		HOLA
	</text>
```

  (`https://raw.githubusercontent.com/penpot/penpot/develop/render-wasm/src/render/svg/snapshots/render_wasm__render__svg__tests__exports_solid_text_with_font_face.snap`).
  Here `font-size` is an **attribute** (not `style`), the fill is a hex
  **attribute**, and — note this — **`x` is a comma-separated list of per-glyph
  x positions**, not one number. Source:
  `render-wasm/src/render/svg/text.rs`
  (`https://raw.githubusercontent.com/penpot/penpot/develop/render-wasm/src/render/svg/text.rs`).

## (c) Does either put `font-size` in a `style` attribute?

| | `font-size` in `style`? | Exact form |
|---|---|---|
| **Figma** | **No** | `font-size="70"` as an attribute, unitless. `style` holds only `white-space: pre`. |
| **Penpot (browser path)** | **Yes** | `font-size: 26px` — one declaration among about ten, separated by `; `, unit `px`. |
| **Penpot (WASM path)** | **No** | `font-size="200"` as an attribute, unitless. |

The ADR says: "`font-size` is read from the attribute or from a `style`
attribute of the form `font-size: 14px`." **That sentence is correct as
written** — it covers both forms. But the wording suggests the `style`
attribute holds that one declaration, and it does not: Penpot's holds ten.
The current implementation (`src/adapters/svg/text/mod.rs`, `declared_size`
and `style_property`) already splits `style` on `;` and matches the property
name, and `pixels` accepts an optional `px` suffix. **That part of the spec is
sound against both tools.** The ADR should say the `style` attribute is a
declaration list, so the next reader does not narrow it to a single pair.

## (d) How is a second line of a label expressed?

| | Mechanism | `dy` used? |
|---|---|---|
| **Figma** | a second `<tspan>` inside the same `<text>`, with its own absolute `x` and `y` | **No** |
| **Penpot (browser path)** | a second `<text>` element, sibling of the first, inside the same `<g class="text-container">`, with its own absolute `x` and `y` | **No** |
| **Penpot (WASM path)** | not verified | not verified |

Figma, verbatim (same file as in (a)):

```xml
<tspan x="1563.78" y="2335.04">Discounted&#10;</tspan><tspan x="1580.34" y="2364.04">Subtotal</tspan>
```

The two absolute `y` values differ by 29 for a `font-size="25"` label — a
line advance, not an offset from the first baseline.

Penpot, verbatim (`https://raw.githubusercontent.com/Floydv149/bibleLinkerPro/master/images/dark.svg`),
two `<text>` elements inside one `<g class="text-container" id="shape-...">`:

```xml
<text x="125.5" y="169.182" ...>Link to your Bible</text>
...
<text x="171.5" y="202.727" ...>in your notes</text>
```

**In neither tool's output does `dy` appear.** Counting over the whole Figma
sample: `dy="` occurs **0** times, and `<tspan ... dy=` occurs **0** times.
In both Penpot files, no `dy` attribute occurs on any element.

The ADR's rule — "A `dy` adds to the y it inherits, which is how exporters
express a second line" — describes a mechanism that **neither exporter uses**.
The `dy` handling in the code is not wrong; the ADR's claim about it is. The
ADR's own "Revisit if" clause already anticipates a related case: "an export
uses `<tspan>` without `x`/`y` to continue a line". The real case is the
opposite and simpler: **the second line carries its own absolute `y`**, which
the current `of_span` code already handles
(`Some(_) => origin.1 + position(element, "y")?`).

One consequence is worth spelling out: because Figma puts the position on the
`<tspan>` and Penpot puts it on the `<text>`, the two tools need opposite
rules for the *first* line. Figma's `<text>` has no `x`/`y` at all, so the
translator must take the position from the `<tspan>`; Penpot's `<text>` has
both. The current implementation handles Figma's shape (a `<tspan>` with its
own `x`/`y` overrides the `<text>`'s) and handles Penpot's shape (the
`<text>` has its own `x`/`y`, and there is no `<tspan>`), so **both work
today** — but only because the rules are written loosely enough to cover
both. The ADR should say so, because the two shapes are not the same.

## (e) The three most likely ways a real export breaks the spec

Ranked by how likely each is to appear in an instrument-panel design, where
panels are dense grids of small labels and gauges.

**First, the finding that reframes this section: I ran the console's own
translator against both real exports.** The command was
`target/debug/oko-svg <file>`, built from this repo at
`/home/violindocgfx/oko/oko-minimal`. Neither file translates. Neither
failure is a text-mapping failure; both happen long before a `<text>` is
read. The full cascade, each line an actual error message from the binary:

Against the real Figma export:

```
/tmp/figma.svg:line 5: entities are not allowed
```

Against the real Penpot export:

```
/tmp/penpot.svg:line 1: entities are not allowed
```

Stripping the entities and retrying continues the cascade:

```
figma:   line 1048: an attribute value is too long
penpot:  line 1: `fill` on `<svg>` would be inherited; put it on each shape
```

and so on, through `style`, `defs`, `matrix(...)` transforms, `url(#...)`
fills, curve commands, and finally `fill` on a `<g>`. After removing all of
those, the Penpot file does translate, and the text comes out like this:

```
rect 0 0 515 255 #000000
text 126 141 "Link to your Bible" 4 #000000
text 172 175 "in your notes" 4 #000000
text 343 66 "for Obsidian" 3 #000000
rect 51 53 16 30 #000000
text 47 57 "Bible linker Pro" 5 #000000
```

Note the colours: the source label was `fill: rgb(176, 176, 176)`, and the
output is `#000000`. **The colour is wrong and nothing says so.** That is the
worst of the failure modes, and it is in the list below.

### 1. Entities and inherited paint: the file is refused before any text is read

**Likelihood: certain.** This is not a probability, it is a fact about the
two files I have.

The console's XML reader refuses any `&` anywhere, in a text run
(`src/adapters/svg/scanner.rs`: "entities are not allowed") and in an
attribute value (`src/adapters/svg/cursor.rs`, `quoted()`: same message).
Both real exports contain `&`:

- **Figma** puts numeric character references in `id` attributes for
  non-ASCII layer names — `id="&#233;&#149;&#183;&#230;&#150;&#185;&#229;&#189;&#162; 2"`
  — and `&#10;` (a newline) inside a `<tspan>` for a soft line break.
- **Penpot** puts `&quot;` inside the `style` attribute of every `<text>`
  whose font family has a space in it (`font-family: &quot;Open Sans&quot;`),
  and `&apos;` inside the `<style>` element's `@font-face` rules.

The 12 `&quot;` occurrences in the bibleLinkerPro file and the `&#10;` in
the Figma file are enough to refuse the whole document.

**Fix if it fires:** this is a scanner decision, not a text decision, but the
text rules depend on it. Two options, and the choice belongs to the ADR:
either the reader decodes the five predefined XML entities (`&amp; &lt; &gt;
&quot; &apos;`) and numeric character references, which is a small,
well-defined set; or the ADR states that only exports with no entities are
in scope, and the exporter instructions tell designers to avoid non-ASCII
layer names. **Decoding is the better answer**, because a designer cannot
avoid a newline in a two-line label, and a designer with a font family like
"Open Sans" cannot avoid `&quot;`.

The same cascade also shows the other refusals a real export hits before its
text is read: `fill` on `<svg>` (Figma writes `fill="none"` on the root,
Penpot writes `fill="none"` on the root too — both are refused by
`paint::plain_group`), a `<style>` element, a `<defs>` element, and
`transform="matrix(...)"` (Penpot writes a `matrix` on every group; the
translator only understands `translate(...)`). ADR 0002 already names `<defs>`
and `<use>` as a "revisit if" trigger; this is that trigger firing, and it
fires on the very first real export, for text reasons that are only partly
about text.

### 2. The colour arrives as a name, as `rgb()` inside `style`, or as `url(#...)`

**Likelihood: very high.** Three different colour forms, three different
failures, and one of them is silent.

- **A named colour.** The Figma export uses `fill="black"` on all 218 of its
  `<text>` elements (and `fill="white"` on its paths). The console's colour
  reader (`src/adapters/svg/values.rs`, `colour`) accepts exactly three
  forms: `#rgb`, `#rrggbb`, and `rgb(r, g, b)`. It refuses `black` with the
  message "`black` is not a colour this translator understands". **Verified
  by running the binary** on the real Figma `<text>` element: the file is
  refused, and with `fill="#000000"` substituted the same element translates
  cleanly to `text 887 133 "ESTIMATE" 8 #000000`.
- **The colour is in `style`, not in an attribute.** Penpot's solid-colour
  text puts it there: `style="...; fill: rgb(176, 176, 176); fill-opacity: 1;"`.
  The console's `paint::fill` reads only the `fill` **attribute**, so a
  Penpot label with a solid fill is drawn in the default black. **Verified by
  execution**: the real Penpot label `fill: rgb(176, 176, 176)` came out as
  `#000000`. This is a **wrong colour, silently** — the worst outcome of the
  three, because a panel that looks almost right is harder to find than one
  that refuses to load.
- **A paint-server reference.** Penpot's gradient text puts
  `fill="url(#fill-0-render-3-0)"` in the attribute. The colour reader
  refuses it: "`url(#fill-0-render-3-0)` is not a colour this translator
  understands". **Verified by execution** against the real file.

**Fix if it fires:** extend `values::colour` to the CSS colour names that
actually appear (at minimum `black` and `white`, which the Figma export
uses on every element) and to `rgba(...)`, since Penpot writes
`rgba(0, 0, 0, 0)` in `text-decoration`. For `url(#...)`: a paint-server
reference is a different kind of thing from a colour and needs its own
decision — resolve the referenced `<linearGradient>`/`<pattern>`, or refuse
with a message that names the reference — rather than being fed to a colour
parser. And `paint::fill` for text must fall back to the `style`
attribute's `fill` declaration, distinguishing "no fill stated" from "fill
stated in `style`". A `fill-opacity` below 1 in that same `style` string
needs a decision too; the ADR currently assumes opaque fills only.

### 3. Figma exports no text at all, because outlining is the default

**Likelihood: very high.** Any Figma SVG export made with the default
settings, and any `GET /v1/images/:key?format=svg` without
`svg_outline_text=false`, contains the label as a `<path>` outline. The
console's translator would produce a panel of shapes and no words, silently
and without an error — the label is present as geometry, so nothing fails.

This breaks the *spec's premise* rather than one of its rules: the ADR's text
mapping is only reachable if the designer unchecks "Outline text" or the
caller passes `svg_outline_text=false`. The spec should say so.

**Fix if it fires:** nothing in the translator. This is a **settings and
documentation** fix — the ADR must state that a Figma export must be taken
with "Outline text" off (or `svg_outline_text=false`), and the exporter
instructions for designers must say the same. Optionally the translator
could *notice* a suspiciously path-like label, but that is a guess, not a
rule.

### Also worth knowing, below the top three

- **Penpot's `x` may be a comma-separated list.** In the WASM export path,
  `x="0, 130.37109, 263.08594, 360.83984"` is one x per glyph. The console's
  `values::integer` would refuse that string. This only affects the WASM
  path, which is behind a feature flag today, but it is a hard failure if a
  designer exports through it. The fix is to take the first number and
  document that per-glyph positioning is not honoured.
- **Figma emits numeric character references in text.** `&#10;` (a newline)
  appears inside a `<tspan>` in the Figma sample, and `&#233;` and friends
  appear in `id` attributes. The console's scanner **refuses any run
  containing `&`**, so a Figma export with a soft line break inside one
  `<tspan>` fails the whole file. The ADR explicitly says "A run containing
  `&` is refused: entities are not decoded" — a deliberate decision, but it
  means a real Figma export can be rejected for a newline. `&#10;` at least is
  safe to decode, since it is whitespace and the label rules collapse
  whitespace anyway.
- **Figma's `<text>` has no `x`/`y`; the `<tspan>` does.** The ADR says "A
  `<tspan>` may carry its own `x`/`y`; otherwise it inherits the `<text>`'s"
  — the code already does this, and the Figma shape is handled correctly.
  Worth stating in the spec because it is the opposite of Penpot's, where the
  `<text>` itself carries the position and there is no `<tspan>` at all.
- **Baseline semantics.** Both tools place `y` at the **baseline**, and
  Penpot adds `dominant-baseline="ideographic"`. The console's `run::emit`
  already subtracts one glyph height to get a top edge. That is the right
  approximation for a font with no descenders, and it is worth keeping the
  reasoning in the ADR, because "y is the top edge" is not what either
  exporter means.
- **The `<text>` element in the WASM path wraps its content in whitespace.**
  The snapshot shows the label on its own line, indented with tabs. The label
  rules trim and collapse whitespace, so this is harmless — but it is worth
  knowing that a run can arrive with leading and trailing whitespace.

## What I could not verify

- **The exact shape of a Figma export made with "Outline text" off via the
  REST API.** The `svg_outline_text=false` path is documented in the OpenAPI
  spec, but I had no token and no live file key, so I could not fetch one.
  The `<text>`/`<tspan>` shape above comes from a Figma export committed to a
  public repository, not from the REST endpoint.
- **The Penpot WASM path's handling of a multi-line label.** The snapshot
  quoted above is a single line. Whether a second line becomes a second
  `<text>`, a `<tspan>`, or a newline in the content is **not verified**.
- **Whether Figma's `fill` is always a colour name.** The one public export I
  found uses only `black`, `white`, `none` and `url(#pattern0)`. I have no
  second independent Figma export with text to confirm the pattern is
  general. (Two Figma REST-API-derived exports exist at
  `https://raw.githubusercontent.com/marcomontalbano/figma-export/main/packages/website/output/svgstore/icons.svg`
  and its `svgstore-monochrome` sibling, but neither contains a text element.
  The monochrome one does use `fill="white"` for a named colour, and the
  colour one uses `fill="#fff"`, which suggests Figma's colour syntax varies
  even within one export.)
- **Which Penpot path a designer is most likely to hit.** Both exist today;
  the browser path is the default unless the `render-wasm/v1` feature flag is
  on, but I did not check whether that flag is on by default in the current
  release.
- **A Figma export that has been through SVGO or another optimiser.** The
  sample I used sits in a repository that post-processes SVG, so the file I
  read may already be optimised. I checked its "expected" twin
  (`expected-real-world-paper-figma.svg`) and the `<text>` shape survives
  optimisation unchanged, so the shape is not an artefact of the optimiser,
  but I cannot rule out that a *pre*-optimisation Figma export differs.
- **Whether Figma's `fill` name is `black` because of the document or the
  tool.** The Figma sample's own paths use `fill="white"` and `fill="black"`,
  and the Adobe XD sibling export in the same directory uses `fill="#fff"`
  (`https://raw.githubusercontent.com/ttskch/svg-paper/master/js/tests/resources/real-world-paper-xd.svg`),
  so the two tools' exporters differ in exactly this way. Which one Figma
  writes for a general design is not established by one file.
