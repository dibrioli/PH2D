// Engine facade: everything the shell (and the smoke test) drives.
// Owns the layers (up to 4 grids sharing one paper), the per-layer history
// rings, the stroke engine + trail, dab parameterization (spec section 9),
// tool dispatch, canvas-wide actions, and the render dirty-rect bookkeeping.
// DOM-free: importable in Node.

import { createGrid, expandBBox, wetCanvas, dryCanvas, clearCanvas, snapshotGrid, restoreGrid } from './grid.js';
import { generatePaperTile, bakePaper } from './paper.js';
import { createTuning } from './tuning.js';
import { buildBristleTexture } from './brush.js';
import { createSim, simStep, simFastDry, gatherParams } from './sim.js';
import { createStrokeEngine, strokeBegin, strokeTick, strokeRelease, strokeReleaseTick, finishStroke } from './stroke.js';
import {
  createTrail, trailStartStroke, trailOnSegment, trailAccumulatePaint,
  trailAccumulateBlend, trailTransferPaint, trailTransferBlend, trailDropRemainder,
} from './trail.js';
import { applyErase, applyWet, applyDry, applyBlow, applySmear, TOOL_HARDNESS } from './tools.js';

export const MAX_LAYERS = 4;
export const HISTORY_DEPTH = 6;

export const NAMED_PIGMENTS = [
  { name: 'Hansa Yellow', rgb: [250, 205, 40] },
  { name: 'Pyrrole Red', rgb: [220, 45, 40] },
  { name: 'Quinacridone Rose', rgb: [220, 55, 120] },
  { name: 'Phthalo Blue', rgb: [10, 70, 150] },
  { name: 'Ultramarine', rgb: [45, 70, 165] },
  { name: 'Viridian', rgb: [20, 120, 100] },
  { name: 'Burnt Sienna', rgb: [150, 75, 40] },
  { name: "Payne's Grey", rgb: [55, 65, 85] },
];

export const TOOLS = ['paint', 'erase', 'smear', 'blend', 'wet', 'dry', 'blow'];

function clamp01(v) { return v < 0 ? 0 : v > 1 ? 1 : v; }

export function createEngine(opts = {}) {
  const width = opts.width ?? 900;
  const height = opts.height ?? 450;
  const engine = {
    width, height,
    tuning: null,
    sim: null,
    layers: [],           // [{grid, opacity, visible, undo, redo}]
    activeLayer: 0,
    paperPreset: opts.paperPreset ?? 'cold',
    paperSheet: opts.paperSheet ?? 0,
    paperTile: null,
    brushTex: null,
    trail: createTrail(),
    stroke: null,
    // Tool state (boot defaults, spec section 16).
    tool: 'paint',
    shape: 'round',
    sliders: { size: 0.7, pressure: 0.7, water: 1.0, erase: 0.4 },
    color: [50, 140, 210],
    kmGlaze: false, // render-side experimental checkbox
    showWet: false,
    exportWithPaper: true,
    // Stroke bookkeeping.
    strokeDown: false,
    hasPrevDab: false, prevDabX: 0, prevDabY: 0,
    onDab: null, // test hook: (pressure, dab)
    // Render dirty rect (cell coords), null = clean, 'full' = repaint all.
    dirty: null,
  };

  engine.tuning = createTuning((def) => {
    if (def.rebuild === 'brush') engine.brushTex = null;   // lazy rebuild
    if (def.rebuild === 'paper') engine.paperDirty = true; // shell re-bakes on release
    if (def.rebuild === 'render') markDirtyFull(engine);
  });

  const baseGrid = createGrid(width, height);
  engine.layers.push({ grid: baseGrid, opacity: 1, visible: true, undo: [], redo: [] });
  engine.sim = createSim(baseGrid, engine.tuning);
  rebakePaper(engine);
  engine.brushTex = buildBristleTexture(engine.tuning.values);

  engine.stroke = createStrokeEngine(
    (chord) => {
      if (engine.tool === 'paint' || engine.tool === 'blend') {
        trailOnSegment(engine.trail, chord, engine.tuning.get('spacing'));
      }
    },
    (x, y, b, dirX, dirY) => dispatchDab(engine, x, y, b, dirX, dirY),
  );
  return engine;
}

export function activeGrid(engine) { return engine.layers[engine.activeLayer].grid; }

function texture(engine) {
  if (!engine.brushTex) engine.brushTex = buildBristleTexture(engine.tuning.values);
  return engine.brushTex;
}

export function rebakePaper(engine) {
  const v = engine.tuning.values;
  engine.paperTile = generatePaperTile(engine.paperPreset, engine.paperSheet, {
    contrast: v.paperContrast, fibres: v.paperFibres, grooves: v.paperGrooves,
  });
  for (const l of engine.layers) {
    bakePaper(l.grid, engine.paperTile);
    l.grid.paperPreset = engine.paperPreset;
    l.grid.paperSheet = engine.paperSheet;
  }
  engine.paperDirty = false;
  markDirtyFull(engine);
}

export function setPaper(engine, preset, sheet) {
  engine.paperPreset = preset ?? engine.paperPreset;
  if (sheet !== undefined) engine.paperSheet = sheet & 3; // 4 sheet indices
  rebakePaper(engine);
}

// ---------------------------------------------------------------------------
// Dab parameters (spec section 9) + dispatch
// ---------------------------------------------------------------------------

function dispatchDab(engine, x, y, b, dirX, dirY) {
  const P = gatherParams(engine.sim);
  const s = engine.sliders;
  const pressureBase = 0.2 + 0.7 * s.pressure; // even minimum pressure deposits
  let intensity = b * 0.5 * pressureBase;
  if (intensity < 0) intensity = 0; else if (intensity > 3) intensity = 3;
  intensity *= P.intensity; // knob applied post-clamp
  const waterUnit = s.water * 0.35;
  // Pressure-INVERSE water: a light touch is wetter, a heavy stroke denser.
  const waterAmount = (2 * waterUnit * waterUnit) / (pressureBase + 0.01) * P.waterPerDab;
  const r = 2 + s.size * 33;
  let h = 0.5 * b;
  if (h < 1) h = 1; else if (h > 3) h = 3;
  if (h > r / 2) h = r / 2;
  const hardness = h * h; // slow => 9 crisp flat-top, fast => ~2 soft puff
  const dryGate = clamp01(1 - waterAmount / 1.5) * clamp01((11 - b - 4) / 4);
  const dab = { x, y, r, hardness, intensity, waterAmount, dryGate, shape: engine.shape, dirX, dirY };
  if (engine.onDab) engine.onDab(b, dab);

  const g = activeGrid(engine);
  const tex = texture(engine);
  const bypass = engine.sim.extBypass;
  let rect = null;
  switch (engine.tool) {
    case 'paint': {
      if (trailAccumulatePaint(engine.trail, g, P, tex, dab, bypass)) {
        rect = trailTransferPaint(engine.trail, g, P);
      }
      break;
    }
    case 'blend': {
      // Blend is a tool (fixed hardness 4.8, intensity clamped to 3) that
      // routes through the trail's saturating mask window.
      const bd = Object.assign({}, dab, { hardness: TOOL_HARDNESS, intensity: Math.min(3, intensity) });
      if (trailAccumulateBlend(engine.trail, g, P, tex, bd)) {
        rect = trailTransferBlend(engine.trail, g, P);
      }
      break;
    }
    case 'erase': rect = applyErase(g, P, tex, dab, s.erase); break;
    case 'wet': rect = applyWet(g, P, tex, dab, waterUnit); break;
    case 'dry': rect = applyDry(g, P, tex, dab); break;
    case 'blow': {
      const px = engine.hasPrevDab ? engine.prevDabX : x;
      const py = engine.hasPrevDab ? engine.prevDabY : y;
      rect = applyBlow(g, P, tex, dab, px, py);
      break;
    }
    case 'smear': {
      if (engine.hasPrevDab) rect = applySmear(g, P, tex, dab, engine.prevDabX, engine.prevDabY);
      break;
    }
  }
  engine.prevDabX = x; engine.prevDabY = y; engine.hasPrevDab = true;
  if (rect) mergeDirty(engine, rect);
}

// ---------------------------------------------------------------------------
// Stroke lifecycle - feed exactly one sample per 40 Hz sim step.
// The sim is PAUSED while the pointer is down, except for the blow tool.
// ---------------------------------------------------------------------------

export function pointerDown(engine, x, y, toolOverride = null) {
  captureHistory(engine); // BEFORE each mutation
  if (toolOverride) {
    engine.strokeToolBackup = engine.tool;
    engine.tool = toolOverride;
  }
  engine.strokeDown = true;
  engine.hasPrevDab = false;
  trailStartStroke(engine.trail, x, y, engine.color, engine.tool === 'blend' ? 'blend' : 'paint');
  strokeBegin(engine.stroke, x, y, engine.tuning.get('spacing'));
}

/** One per 40 Hz step while down. */
export function pointerFrame(engine, x, y) {
  if (!engine.strokeDown) return;
  strokeTick(engine.stroke, x, y);
}

export function pointerUp(engine) {
  if (!engine.strokeDown) return;
  engine.strokeDown = false;
  strokeRelease(engine.stroke);
}

/**
 * Tick the release tail once per 40 Hz step with the hover position.
 * Returns false when the stroke has fully finished (remainder dropped).
 */
export function releaseFrame(engine, x, y) {
  if (!engine.stroke.active) return false;
  const more = strokeReleaseTick(engine.stroke, x, y);
  if (!more) endStroke(engine);
  return more;
}

function endStroke(engine) {
  finishStroke(engine.stroke);
  trailDropRemainder(engine.trail); // remainder is dropped by design
  if (engine.strokeToolBackup) { engine.tool = engine.strokeToolBackup; engine.strokeToolBackup = null; }
}

/** Immediate full stop (used by tests and tool switches mid-air). */
export function abortStroke(engine) {
  engine.strokeDown = false;
  endStroke(engine);
}

/** Should the fixed-step loop advance the sim right now? */
export function simShouldRun(engine) {
  return !engine.strokeDown || engine.tool === 'blow';
}

export function stepSimulation(engine) {
  const g = engine.sim.grid;
  const hadFluid = g.hasFluid;
  const pre = hadFluid ? { x0: g.bx0, y0: g.by0, x1: g.bx1, y1: g.by1 } : null;
  const ran = simStep(engine.sim);
  if (ran || hadFluid) {
    if (pre) mergeDirty(engine, pre);
    if (g.hasFluid) mergeDirty(engine, { x0: g.bx0, y0: g.by0, x1: g.bx1, y1: g.by1 });
  }
  return ran;
}

// ---------------------------------------------------------------------------
// Canvas-wide actions (spec section 12) - each captures history first.
// ---------------------------------------------------------------------------

export function actionWetCanvas(engine) {
  captureHistory(engine);
  wetCanvas(activeGrid(engine));
  markDirtyFull(engine); // show-wet overlay may change everywhere
}

export function actionDryCanvas(engine) {
  captureHistory(engine);
  const P = gatherParams(engine.sim);
  const g = activeGrid(engine);
  dryCanvas(g, P.mixColor);
  g.bloom.fill(0); // backrun budgets recharge on dry
  markDirtyFull(engine);
}

export function actionFastDry(engine) {
  captureHistory(engine);
  simFastDry(engine.sim);
  markDirtyFull(engine);
}

export function actionClear(engine) {
  captureHistory(engine);
  clearCanvas(activeGrid(engine));
  markDirtyFull(engine);
}

// ---------------------------------------------------------------------------
// Layers (spec section 15): up to 4, sharing one paper array; only the
// active layer simulates.
// ---------------------------------------------------------------------------

export function addLayer(engine) {
  if (engine.layers.length >= MAX_LAYERS) return false;
  const g = createGrid(engine.width, engine.height);
  g.paper = engine.layers[0].grid.paper; // shared tooth
  g.paperPreset = engine.paperPreset; g.paperSheet = engine.paperSheet;
  engine.layers.push({ grid: g, opacity: 1, visible: true, undo: [], redo: [] });
  selectLayer(engine, engine.layers.length - 1);
  return true;
}

export function removeLayer(engine, index) {
  if (engine.layers.length <= 1) return false;
  engine.layers.splice(index, 1);
  if (engine.activeLayer >= engine.layers.length) engine.activeLayer = engine.layers.length - 1;
  selectLayer(engine, engine.activeLayer);
  markDirtyFull(engine);
  return true;
}

export function selectLayer(engine, index) {
  engine.activeLayer = Math.max(0, Math.min(engine.layers.length - 1, index));
  engine.sim.grid = activeGrid(engine);
  markDirtyFull(engine);
}

// ---------------------------------------------------------------------------
// History (spec section 15): per-layer undo/redo ring, depth 6.
// ---------------------------------------------------------------------------

export function captureHistory(engine) {
  const layer = engine.layers[engine.activeLayer];
  layer.undo.push(snapshotGrid(layer.grid));
  if (layer.undo.length > HISTORY_DEPTH) layer.undo.shift();
  layer.redo.length = 0;
}

export function canUndo(engine) { return engine.layers[engine.activeLayer].undo.length > 0; }
export function canRedo(engine) { return engine.layers[engine.activeLayer].redo.length > 0; }

export function undo(engine) {
  const layer = engine.layers[engine.activeLayer];
  const snap = layer.undo.pop();
  if (!snap) return false;
  layer.redo.push(snapshotGrid(layer.grid));
  applySnapshot(engine, layer, snap);
  return true;
}

export function redo(engine) {
  const layer = engine.layers[engine.activeLayer];
  const snap = layer.redo.pop();
  if (!snap) return false;
  layer.undo.push(snapshotGrid(layer.grid));
  applySnapshot(engine, layer, snap);
  return true;
}

function applySnapshot(engine, layer, snap) {
  const paperChanged = restoreGrid(layer.grid, snap);
  if (paperChanged) {
    engine.paperPreset = layer.grid.paperPreset;
    engine.paperSheet = layer.grid.paperSheet;
    rebakePaper(engine);
  }
  markDirtyFull(engine);
}

// ---------------------------------------------------------------------------
// Dirty-rect bookkeeping (cell coords, [1..W] x [1..H])
// ---------------------------------------------------------------------------

export function mergeDirty(engine, r) {
  if (engine.dirty === 'full') return;
  const g = activeGrid(engine);
  const x0 = Math.max(1, r.x0 - 1), y0 = Math.max(1, r.y0 - 1);
  const x1 = Math.min(g.W, r.x1 + 1), y1 = Math.min(g.H, r.y1 + 1);
  if (!engine.dirty) { engine.dirty = { x0, y0, x1, y1 }; return; }
  const d = engine.dirty;
  if (x0 < d.x0) d.x0 = x0; if (y0 < d.y0) d.y0 = y0;
  if (x1 > d.x1) d.x1 = x1; if (y1 > d.y1) d.y1 = y1;
}

export function markDirtyFull(engine) { engine.dirty = 'full'; }

/** Take and clear the pending dirty region. */
export function takeDirty(engine) {
  const d = engine.dirty;
  engine.dirty = null;
  return d;
}
