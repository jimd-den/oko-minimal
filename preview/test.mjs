// Headless check of the WASM preview: no browser, no canvas.
// Instantiates the module, renders both panels, and asserts the real
// core is doing the drawing (clear colour, animation over time, keys).
// Usage: node preview/test.mjs [path-to-wasm]
// The GitHub Action passes the freshly built wasm; locally the default
// is the release build of this crate.
import { readFile } from "node:fs/promises";

const wasmPath =
  process.argv[2] ??
  "preview/target/wasm32-unknown-unknown/release/oko_wasm_preview.wasm";
const bytes = await readFile(wasmPath);
const { instance } = await WebAssembly.instantiate(bytes, {});
const e = instance.exports;
const mem = () => new Uint8Array(e.memory.buffer);

function render(now) {
  const ptr = e.preview_render(BigInt(now));
  if (!ptr) throw new Error("render returned null pointer");
  const w = e.preview_width();
  const h = e.preview_height();
  return { w, h, px: mem().slice(ptr, ptr + w * h * 4) };
}
function pixel(frame, x, y) {
  const i = (y * frame.w + x) * 4;
  return [frame.px[i], frame.px[i + 1], frame.px[i + 2], frame.px[i + 3]];
}
function countNonBlack(frame) {
  let n = 0;
  for (let i = 0; i < frame.px.length; i += 4) {
    if (frame.px[i] || frame.px[i + 1] || frame.px[i + 2]) n++;
  }
  return n;
}
function framesDiffer(a, b) {
  let n = 0;
  for (let i = 0; i < a.px.length; i++) if (a.px[i] !== b.px[i]) n++;
  return n;
}
function err() {
  const ptr = e.preview_error_ptr();
  const len = e.preview_error_len();
  return new TextDecoder().decode(mem().slice(ptr, ptr + len));
}

// --- LCARS (examples/lcars.oko, the branch's Star Trek panel) ---
let rc = e.preview_init(0);
if (rc !== 0) throw new Error(`lcars init failed: ${err()}`);
console.log("lcars size:", e.preview_width(), "x", e.preview_height());
const t0 = render(0);
const t1000 = render(1000);
console.log("lcars clear pixel (0,0):", pixel(t0, 0, 0), "expected [5,7,10,255]");
console.log("lcars non-black pixels @0ms:", countNonBlack(t0));
console.log("lcars bytes differing 0ms vs 1000ms:", framesDiffer(t0, t1000));
console.log("lcars looping:", e.preview_is_looping());
if (JSON.stringify(pixel(t0, 0, 0)) !== JSON.stringify([5, 7, 10, 255])) {
  throw new Error("lcars clear colour is wrong — not the real core output");
}
if (framesDiffer(t0, t1000) === 0) throw new Error("lcars does not animate");
// Key 'e' toggles engineering: the telemetry term appears, pixels change.
const before = render(0);
const outcome = e.preview_key("e".charCodeAt(0));
const after = render(0);
console.log("key 'e' outcome:", outcome, "pixels differing after toggle:", framesDiffer(before, after));
if (outcome !== 1) throw new Error("key 'e' did not report Changed");
if (framesDiffer(before, after) === 0) throw new Error("toggle changed nothing visible");

// --- Modern 1080p (preview/modern-1080.oko, the heavy design) ---
rc = e.preview_init(1);
if (rc !== 0) throw new Error(`modern init failed: ${err()}`);
console.log("modern size:", e.preview_width(), "x", e.preview_height());
const start = performance.now();
const m0 = render(0);
console.log("modern first render ms:", (performance.now() - start).toFixed(1));
console.log("modern clear pixel (0,0):", pixel(m0, 0, 0), "expected [4,6,10,255]");
console.log("modern non-black pixels @0ms:", countNonBlack(m0));
const m1000 = render(1000);
console.log("modern bytes differing 0ms vs 1000ms:", framesDiffer(m0, m1000));
console.log("modern looping:", e.preview_is_looping());
if (JSON.stringify(pixel(m0, 0, 0)) !== JSON.stringify([4, 6, 10, 255])) {
  throw new Error("modern clear colour is wrong");
}
if (framesDiffer(m0, m1000) === 0) throw new Error("modern does not animate");
console.log("ALL HEADLESS CHECKS PASSED");
