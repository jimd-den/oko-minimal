# oko-minimal — WASM preview

A browser preview of the Star Trek panels on this branch, built from the
branch's own pure core — no JavaScript draws anything. The page's JS only
copies pixels to a canvas and forwards keys.

- `src/lib.rs` — the shim: `document::parse`, `Panel::scene(now)`,
  `raster::render_into`, exposed to the page through `src/exports.rs`.
  The core is reused verbatim via `#[path]`; no adapter is compiled in.
- `site/` — the page: `index.html` + `preview.js`, no framework.
- `modern-1080.oko` — the heavy 1920x1080 Star Trek design this preview
  shows beside `examples/lcars.oko`. It lives here, not in `examples/`,
  because it is the preview's data, not a console example.
- `test.mjs` — the headless check the Action runs before deploying.

## How it republishes

Push to this branch and `.github/workflows/preview-pages.yml` rebuilds
the WASM, runs `test.mjs` against it, and deploys to GitHub Pages. No
hand-built artifact is ever pushed: the `.wasm` is built in the Action
on every run. The live site is `https://jimd-den.github.io/oko-minimal/`.

## Honest limits

Terminal viewports show labelled placeholder text and feeds / `run`
bindings do not execute — there is no shell in a browser. The 1080p
panel is software-rendered, so it runs at roughly 10–15 fps in the
browser; the small LCARS panel runs at full frame rate.
