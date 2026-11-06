// Thin page glue only: copy pixels from WASM to the canvas, forward keys.
// Every pixel is drawn by oko-minimal's pure core in the .wasm module.
let wasm = null;
let panel = 0; // 0 = LCARS (PR #4), 1 = modern 1080p
let playing = true;
let startMs = performance.now();
let lastRenderMs = 0;
let frames = 0;
let fpsAt = performance.now();
let quit = false;

const canvas = document.getElementById("screen");
const ctx = canvas.getContext("2d");
const statusEl = document.getElementById("status");
const sourceEl = document.getElementById("source");
const enc = new TextEncoder();
const dec = new TextDecoder();

const PANELS = [
  { file: "lcars.oko", label: "LCARS — PR #4", throttleMs: 0 },
  { file: "modern-1080.oko", label: "Modern 1080p", throttleMs: 66 },
];

function mem() {
  return new Uint8Array(wasm.exports.memory.buffer);
}

function wasmError() {
  const ptr = wasm.exports.preview_error_ptr();
  const len = wasm.exports.preview_error_len();
  return dec.decode(mem().slice(ptr, ptr + len));
}

function writeScratch(offset, text) {
  const bytes = enc.encode(text);
  mem().set(bytes, offset);
  return bytes.length;
}

function updateFlags() {
  const bits = wasm.exports.preview_flags();
  document.getElementById("chip-tactical").classList.toggle("on", !!(bits & 1));
  document.getElementById("chip-engineering").classList.toggle("on", !!(bits & 2));
  document.getElementById("chip-science").classList.toggle("on", !!(bits & 4));
  document.getElementById("chip-alert").classList.toggle("on", !!(bits & 8));
}

function loadPanel(which) {
  panel = which;
  quit = false;
  const rc = wasm.exports.preview_init(which);
  if (rc !== 0) {
    statusEl.textContent = "init failed: " + wasmError();
    return;
  }
  const w = wasm.exports.preview_width();
  const h = wasm.exports.preview_height();
  canvas.width = w;
  canvas.height = h;
  canvas.classList.toggle("pixelated", which === 0);
  startMs = performance.now();
  lastRenderMs = 0;
  fetch(PANELS[which].file)
    .then((r) => r.text())
    .then((t) => { sourceEl.textContent = t; })
    .catch(() => { sourceEl.textContent = "(source not served)"; });
  updateFlags();
  statusEl.textContent = PANELS[which].label + " — " + w + "x" + h + ", looping: " +
    (wasm.exports.preview_is_looping() ? "yes" : "no");
}

function renderFrame(now) {
  const t0 = performance.now();
  const ptr = wasm.exports.preview_render(BigInt(Math.floor(now - startMs)));
  const w = wasm.exports.preview_width();
  const h = wasm.exports.preview_height();
  const bytes = mem().slice(ptr, ptr + w * h * 4);
  const img = new ImageData(new Uint8ClampedArray(bytes), w, h);
  ctx.putImageData(img, 0, 0);
  frames++;
  const ms = performance.now() - t0;
  if (performance.now() - fpsAt > 1000) {
    statusEl.textContent = PANELS[panel].label + " — " + w + "x" + h +
      " — " + frames + " fps — render " + ms.toFixed(1) + " ms";
    frames = 0;
    fpsAt = performance.now();
  }
}

function loop(now) {
  requestAnimationFrame(loop);
  if (!playing || quit || !wasm) return;
  if (now - lastRenderMs < PANELS[panel].throttleMs) return;
  lastRenderMs = now;
  renderFrame(now);
}

function pressKey(code) {
  if (!wasm || quit) return;
  const outcome = wasm.exports.preview_key(code);
  updateFlags();
  if (outcome === 3) {
    quit = true;
    statusEl.textContent = "quit — press R to restart this panel";
  }
}

window.addEventListener("keydown", (e) => {
  if (quit && (e.key === "r" || e.key === "R")) { loadPanel(panel); return; }
  if (e.key === "Enter") pressKey(13);
  else if (e.key === "Escape") pressKey(27);
  else if (e.key.length === 1) pressKey(e.key.charCodeAt(0));
});

document.querySelectorAll("[data-key]").forEach((b) => {
  b.addEventListener("click", () => pressKey(b.dataset.key.charCodeAt(0)));
});
document.getElementById("btn-lcars").addEventListener("click", () => loadPanel(0));
document.getElementById("btn-modern").addEventListener("click", () => loadPanel(1));
document.getElementById("btn-play").addEventListener("click", (e) => {
  playing = !playing;
  e.target.textContent = playing ? "Pause" : "Play";
});

// The clock terminal, when the panel has one: the browser's real time,
// written through the core's own Panel::write. No shell involved.
setInterval(() => {
  if (!wasm || quit) return;
  const scratch = wasm.exports.preview_scratch_ptr();
  const termLen = writeScratch(scratch, "clock");
  const now = new Date();
  // One line only: the modern panel's clock viewport is one row tall,
  // so a second line would push the time itself out of view.
  const text = now.toTimeString().slice(0, 8);
  const textLen = writeScratch(scratch + 256, text);
  wasm.exports.preview_write_term(scratch, termLen, scratch + 256, textLen);
}, 1000);

async function boot() {
  try {
    const response = await fetch("oko_preview.wasm");
    let result;
    try {
      result = await WebAssembly.instantiateStreaming(response, {});
    } catch (_e) {
      const bytes = await (await fetch("oko_preview.wasm")).arrayBuffer();
      result = await WebAssembly.instantiate(bytes, {});
    }
    wasm = result.instance;
    loadPanel(0);
    requestAnimationFrame(loop);
  } catch (err) {
    statusEl.textContent = "failed to load WASM: " + err;
  }
}
boot();
