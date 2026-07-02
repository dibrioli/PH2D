// Per-cell simulation state (spec section 2) + canvas-wide actions that are
// pure field operations (spec section 12: wet canvas / dry canvas / clear).
//
// Layout: the paintable canvas is W x H cells, one cell per pixel, stored in
// padded arrays with stride S = W + 2 and rows = H + 2 (one border cell on
// every side). Interior cells are x in [1..W], y in [1..H]; index x + y*S.
// Brushes may only touch [2..W-1] x [2..H-1] - the outermost interior ring is
// a drain the boundary pass wipes every frame, so drips run off the sheet
// instead of pooling at the edge.
//
// Two pigment layers is the core model: brushes deposit SUSPENDED pigment
// (moves with the flow); drying transfers it to SETTLED (stuck to the paper);
// re-wetting lifts a little back. Two velocity fields is the core trick:
// gravity accumulates in the PERSISTENT field, while the TRANSIENT flow is
// rebuilt from it every frame - the absorbency brake only applies during the
// 1-in-4 rebuild, so a drip keeps its momentum on the other three frames.

import { alphaOfMass } from './opacity.js';

export const DEFAULT_WIDTH = 900;
export const DEFAULT_HEIGHT = 450;

export function createGrid(width = DEFAULT_WIDTH, height = DEFAULT_HEIGHT) {
  const W = width, H = height;
  const S = W + 2;
  const rows = H + 2;
  const n = S * rows;
  return {
    W, H, S, rows, cells: n,
    film: new Float32Array(n),        // free water depth, 0..waterCap
    susp: new Float32Array(n),        // suspended pigment mass (moves)
    suspR: new Float32Array(n),       // suspended color, 0..255 floats
    suspG: new Float32Array(n),
    suspB: new Float32Array(n),
    sett: new Float32Array(n),        // settled pigment mass (fixed)
    settR: new Float32Array(n),       // settled color, 0..255 floats
    settG: new Float32Array(n),
    settB: new Float32Array(n),
    velX: new Float32Array(n),        // persistent velocity (gravity lands here)
    velY: new Float32Array(n),
    flowX: new Float32Array(n),       // transient flow, rebuilt each frame
    flowY: new Float32Array(n),
    wet: new Uint8Array(n),           // wetness byte: "the sheet is damp here"
    paper: new Float32Array(n),       // tooth height 0..1, baked incl. pad ring
    active: new Uint8Array(n),        // 0 skip, 1/2 solver processes
    bloom: new Uint8Array(n),         // per-cell budget for the backrun extension
    // Active bounding box (solver iterates only inside it) + fluid flag.
    bx0: 0, by0: 0, bx1: -1, by1: -1,
    hasFluid: false,
    // Paper identity (re-baked by paper.js; part of history snapshots).
    paperPreset: 'cold', paperSheet: 0,
  };
}

/** Empty the active bbox and drop the fluid flag. */
export function emptyBBox(g) {
  g.bx0 = 0; g.by0 = 0; g.bx1 = -1; g.by1 = -1;
  g.hasFluid = false;
}

/** Grow the active bbox to include a rect (canvas cell coords), clamped to the interior. */
export function expandBBox(g, x0, y0, x1, y1) {
  x0 = Math.max(1, x0); y0 = Math.max(1, y0);
  x1 = Math.min(g.W, x1); y1 = Math.min(g.H, y1);
  if (x0 > x1 || y0 > y1) return;
  if (!g.hasFluid || g.bx0 > g.bx1) {
    g.bx0 = x0; g.by0 = y0; g.bx1 = x1; g.by1 = y1;
  } else {
    if (x0 < g.bx0) g.bx0 = x0;
    if (y0 < g.by0) g.by0 = y0;
    if (x1 > g.bx1) g.bx1 = x1;
    if (y1 > g.by1) g.by1 = y1;
  }
  g.hasFluid = true;
}

/** Wetness byte a given paper tooth stamps: valleys (low paper) read wetter. */
export function wetByteFromPaper(paperValue) {
  let v = 2 - 2 * paperValue;
  if (v > 1) v = 1;
  if (v < 0) v = 0;
  return (v * 255) | 0;
}

// ---------------------------------------------------------------------------
// Canvas-wide actions (spec section 12)
// ---------------------------------------------------------------------------

/**
 * Wet canvas: raise the wetness byte to the paper-derived value via max over
 * the whole interior. Injects NO water and touches no bbox - the sim stays
 * idle, but subsequent strokes bleed everywhere and show-wet reads damp.
 */
export function wetCanvas(g) {
  const { S, W, H, wet, paper } = g;
  for (let y = 1; y <= H; y++) {
    let i = 1 + y * S;
    for (let x = 1; x <= W; x++, i++) {
      const b = wetByteFromPaper(paper[i]);
      if (b > wet[i]) wet[i] = b;
    }
  }
}

/**
 * Dry canvas: one-shot O(area) - settle every cell's suspended mass into the
 * settled layer (opacity-composite color, same as the dry pass), zero water,
 * both velocity fields and wetness, and empty the bbox.
 */
export function dryCanvas(g, mixColor) {
  const { S, W, H } = g;
  const out = [0, 0, 0];
  for (let y = 1; y <= H; y++) {
    let i = 1 + y * S;
    for (let x = 1; x <= W; x++, i++) {
      const dm = g.susp[i];
      if (dm > 0) {
        settleComposite(g, i, dm, mixColor, out);
        g.sett[i] += dm;
        g.susp[i] = 0;
      }
      g.film[i] = 0;
      g.velX[i] = 0; g.velY[i] = 0;
      g.flowX[i] = 0; g.flowY[i] = 0;
      g.wet[i] = 0;
    }
  }
  emptyBBox(g);
}

/**
 * Opacity-composite `dm` of suspended pigment into the settled layer's color
 * at cell i (spec 6.2 step 3). NOT mass-weighted: coverage-weighted, so a
 * thin new glaze barely shifts an already-opaque settled color.
 */
export function settleComposite(g, i, dm, mixColor, out) {
  const aSett = alphaOfMass(g.sett[i]);
  const aIn = alphaOfMass(dm);
  if (aSett > 0) {
    const u = aSett * (1 - aIn);
    const w = aIn / (u + aIn);
    mixColor(g.settR[i], g.settG[i], g.settB[i], g.suspR[i], g.suspG[i], g.suspB[i], w, out);
    g.settR[i] = out[0]; g.settG[i] = out[1]; g.settB[i] = out[2];
  } else {
    g.settR[i] = g.suspR[i]; g.settG[i] = g.suspG[i]; g.settB[i] = g.suspB[i];
  }
}

/** Clear: zero all dynamic state; the paper is untouched. */
export function clearCanvas(g) {
  g.film.fill(0);
  g.susp.fill(0); g.suspR.fill(0); g.suspG.fill(0); g.suspB.fill(0);
  g.sett.fill(0); g.settR.fill(0); g.settG.fill(0); g.settB.fill(0);
  g.velX.fill(0); g.velY.fill(0);
  g.flowX.fill(0); g.flowY.fill(0);
  g.wet.fill(0);
  g.active.fill(0);
  g.bloom.fill(0);
  emptyBBox(g);
}

// ---------------------------------------------------------------------------
// History snapshots (spec section 15). The transient flow arrays are scratch
// rebuilt every frame from the persistent field, so they are not captured.
// ---------------------------------------------------------------------------

export function snapshotGrid(g) {
  return {
    film: g.film.slice(),
    susp: g.susp.slice(), suspR: g.suspR.slice(), suspG: g.suspG.slice(), suspB: g.suspB.slice(),
    sett: g.sett.slice(), settR: g.settR.slice(), settG: g.settG.slice(), settB: g.settB.slice(),
    velX: g.velX.slice(), velY: g.velY.slice(),
    wet: g.wet.slice(), active: g.active.slice(), bloom: g.bloom.slice(),
    bx0: g.bx0, by0: g.by0, bx1: g.bx1, by1: g.by1,
    hasFluid: g.hasFluid,
    paperPreset: g.paperPreset, paperSheet: g.paperSheet,
  };
}

/** Restore a snapshot. Returns true if the paper identity changed (caller re-bakes). */
export function restoreGrid(g, s) {
  g.film.set(s.film);
  g.susp.set(s.susp); g.suspR.set(s.suspR); g.suspG.set(s.suspG); g.suspB.set(s.suspB);
  g.sett.set(s.sett); g.settR.set(s.settR); g.settG.set(s.settG); g.settB.set(s.settB);
  g.velX.set(s.velX); g.velY.set(s.velY);
  g.flowX.fill(0); g.flowY.fill(0);
  g.wet.set(s.wet); g.active.set(s.active); g.bloom.set(s.bloom);
  g.bx0 = s.bx0; g.by0 = s.by0; g.bx1 = s.bx1; g.by1 = s.by1;
  g.hasFluid = s.hasFluid;
  const paperChanged = g.paperPreset !== s.paperPreset || g.paperSheet !== s.paperSheet;
  g.paperPreset = s.paperPreset; g.paperSheet = s.paperSheet;
  return paperChanged;
}
