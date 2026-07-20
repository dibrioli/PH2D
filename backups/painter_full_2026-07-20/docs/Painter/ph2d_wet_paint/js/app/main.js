// App entry: boots the engine, wires the UI, and runs the fixed 40 Hz loop.
//
// Timing model (spec section 5): an accumulator over requestAnimationFrame,
// clamped to 5 steps so a background tab drops frames instead of
// fast-forwarding. Pointer input is DECIMATED to the sim clock: we record the
// latest pointer position and feed the stroke engine exactly one sample per
// 40 Hz step while the pointer is down. The sim is paused during a stroke -
// paint lands, nothing flows - except for the blow tool, which needs it live;
// drips and bleeding run on release.

import {
  createEngine, pointerDown, pointerUp, pointerFrame, releaseFrame,
  stepSimulation, simShouldRun, setPaper, markDirtyFull,
  undo, redo, canUndo, canRedo,
  actionWetCanvas, actionDryCanvas, actionFastDry, actionClear,
} from '../engine/painter.js';
import { renderRegion, renderPigmentOnly } from '../engine/render.js';
import { gatherParams } from '../engine/sim.js';
import { createView } from './view.js';
import { buildLeftPanel, setTool, setSlider, refreshLayers } from './ui-left.js';
import { buildTuningPanel } from './ui-tuning.js';
import { applyStatic } from './i18n.js';
import { CHROME_DOCS } from './chromedocs.js';
import { attachRichTooltip } from './tooltip.js';

const app = {
  engine: createEngine(),
  lang: 'en',
  view: null,
};
window.ph2dWetPaint = app; // console access for tinkering

app.view = createView(app.engine);
buildLeftPanel(app);
buildTuningPanel(app);
applyStatic(app.lang);

// ---------------------------------------------------------------------------
// Pointer wiring (left = tool, right = blow override, middle = pan in view.js)
// ---------------------------------------------------------------------------

const pointer = { down: false, x: 0, y: 0 };
const viewport = document.getElementById('canvas-viewport');

viewport.addEventListener('contextmenu', (ev) => ev.preventDefault());
viewport.addEventListener('pointerdown', (ev) => {
  if (ev.button !== 0 && ev.button !== 2) return;
  ev.preventDefault();
  viewport.setPointerCapture(ev.pointerId);
  const c = app.view.toCell(ev);
  pointer.down = true;
  pointer.x = c.x; pointer.y = c.y;
  // Right-button drag = blow regardless of the selected tool.
  pointerDown(app.engine, c.x, c.y, ev.button === 2 ? 'blow' : null);
});
viewport.addEventListener('pointermove', (ev) => {
  const c = app.view.toCell(ev);
  pointer.x = c.x; pointer.y = c.y;
});
viewport.addEventListener('pointerup', (ev) => {
  if (ev.button !== 0 && ev.button !== 2) return;
  pointer.down = false;
  pointerUp(app.engine);
});
viewport.addEventListener('dblclick', () => {
  // Double-click cycles among the 4 sheet indices of the current preset.
  setPaper(app.engine, undefined, app.engine.paperSheet + 1);
});

// ---------------------------------------------------------------------------
// Fixed-step loop
// ---------------------------------------------------------------------------

const STEP_MS = 1000 / 40;
let acc = 0;
let last = performance.now();

function tick40() {
  const e = app.engine;
  if (pointer.down) {
    pointerFrame(e, pointer.x, pointer.y); // exactly one sample per step
    if (simShouldRun(e)) stepSimulation(e); // only the blow tool keeps it live
  } else if (e.stroke.active) {
    releaseFrame(e, pointer.x, pointer.y); // fading release tail
    stepSimulation(e);
  } else {
    stepSimulation(e);
  }
}

function frame(now) {
  acc += now - last;
  last = now;
  if (acc > STEP_MS * 5) acc = STEP_MS * 5; // background tab drops frames
  while (acc >= STEP_MS) {
    acc -= STEP_MS;
    tick40();
  }
  app.view.flush();
  syncHistoryButtons();
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);

// ---------------------------------------------------------------------------
// Bottom bar
// ---------------------------------------------------------------------------

const btnUndo = document.getElementById('btn-undo');
const btnRedo = document.getElementById('btn-redo');

function syncHistoryButtons() {
  btnUndo.disabled = !canUndo(app.engine);
  btnRedo.disabled = !canRedo(app.engine);
}

btnUndo.addEventListener('click', () => { undo(app.engine); refreshLayers(app); });
btnRedo.addEventListener('click', () => { redo(app.engine); refreshLayers(app); });

document.getElementById('btn-wet-canvas').addEventListener('click', () => actionWetCanvas(app.engine));
document.getElementById('btn-dry-canvas').addEventListener('click', () => actionDryCanvas(app.engine));
document.getElementById('btn-fast-dry').addEventListener('click', () => actionFastDry(app.engine));

const btnShowWet = document.getElementById('btn-show-wet');
btnShowWet.addEventListener('click', () => {
  app.engine.showWet = !app.engine.showWet;
  btnShowWet.classList.toggle('on', app.engine.showWet);
  markDirtyFull(app.engine);
});

document.getElementById('btn-clear').addEventListener('click', () => actionClear(app.engine));

document.getElementById('chk-paper').addEventListener('change', (ev) => {
  app.engine.exportWithPaper = ev.target.checked;
});
document.getElementById('btn-save').addEventListener('click', exportPng);

document.getElementById('btn-toggle-tuning').addEventListener('click', () => {
  document.getElementById('right-panel').classList.toggle('collapsed');
});

document.getElementById('btn-lang').addEventListener('click', () => {
  app.lang = app.lang === 'en' ? 'pt' : 'en';
  applyStatic(app.lang);
  buildTuningPanel(app); // group titles + knob labels re-render per language
  refreshLayers(app);
  if (app.redrawTilt) app.redrawTilt();
  document.getElementById('btn-lang').textContent = app.lang === 'en' ? 'EN/PT' : 'PT/EN';
});

// Rich PT-BR tooltips for every bottom-bar action (spec section 16,
// "chrome documentation"), through the same floating system as the knobs.
const BAR_TIPS = [
  ['btn-undo', 'undo'], ['btn-redo', 'redo'],
  ['btn-wet-canvas', 'wetCanvas'], ['btn-dry-canvas', 'dryCanvas'],
  ['btn-fast-dry', 'fastDry'], ['btn-show-wet', 'showWet'],
  ['btn-clear', 'clear'], ['btn-save', 'savePng'],
  ['btn-toggle-tuning', 'toggleTuning'], ['btn-lang', 'lang'],
];
for (const [id, docKey] of BAR_TIPS) {
  attachRichTooltip(document.getElementById(id), CHROME_DOCS[docKey]);
}
// The "paper" export checkbox: attach to its wrapping label.
attachRichTooltip(document.getElementById('chk-paper').parentElement, CHROME_DOCS.withPaper);

// ---------------------------------------------------------------------------
// Export PNG (spec section 15): on-screen composite (with paper) or
// pigment-only over transparent; the checkbox picks.
// ---------------------------------------------------------------------------

function exportPng() {
  const e = app.engine;
  const W = e.width, H = e.height;
  const out = document.createElement('canvas');
  out.width = W; out.height = H;
  const ctx = out.getContext('2d');
  const img = ctx.createImageData(W, H);
  let name;
  if (e.exportWithPaper) {
    renderRegion({
      P: gatherParams(e.sim),
      layers: e.layers,
      activeGrid: e.layers[e.activeLayer].grid,
      showWet: false,
      kmGlaze: e.kmGlaze,
      extBypass: e.sim.extBypass,
    }, img.data, 1, 1, W, H);
    name = 'ph2d-wet-paint.png';
  } else {
    renderPigmentOnly(e.layers, img.data);
    name = 'ph2d-wet-paint-pigment.png';
  }
  ctx.putImageData(img, 0, 0);
  out.toBlob((blob) => {
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = name;
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 5000);
  }, 'image/png');
}

// ---------------------------------------------------------------------------
// Shortcuts (spec section 15): p e s n w d b tools, [ ] brush size,
// Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y - guarded when focus is in an input.
// ---------------------------------------------------------------------------

const TOOL_KEYS = { p: 'paint', e: 'erase', s: 'smear', n: 'blend', w: 'wet', d: 'dry', b: 'blow' };

window.addEventListener('keydown', (ev) => {
  const target = ev.target;
  if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA')) return;
  if ((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === 'z') {
    ev.preventDefault();
    if (ev.shiftKey) redo(app.engine); else undo(app.engine);
    refreshLayers(app);
    return;
  }
  if ((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === 'y') {
    ev.preventDefault();
    redo(app.engine);
    refreshLayers(app);
    return;
  }
  if (ev.ctrlKey || ev.metaKey || ev.altKey) return;
  const tool = TOOL_KEYS[ev.key.toLowerCase()];
  if (tool) { setTool(app, tool); return; }
  if (ev.key === '[') setSlider(app, 'size', Math.max(0, app.engine.sliders.size - 0.03));
  if (ev.key === ']') setSlider(app, 'size', Math.min(1, app.engine.sliders.size + 0.03));
});
