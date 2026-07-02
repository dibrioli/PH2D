// The paint trail (spec section 10): a two-step deposit.
//
// Dabs never stamp straight into the canvas. They accumulate into a
// stroke-local trail buffer, and the trail lands once per WINDOW - a
// continuous footprint instead of a chain of beads. The window is also where
// the wet-brush feel lives:
//   - a brush-tip color buffer picks up canvas color as it passes over wet
//     paint (the dirty brush) and slowly self-cleans back to the base color;
//   - landing uses opacity-composite color and SOFT mass/water caps (shed the
//     window mean instead of clamping, so pools hover at the cap);
//   - "paint drag" pulls a little state from the previous window position,
//     which smears the film forward under the stroke.
//
// Blend mode reuses the window plumbing with a saturating mask and re-mixes
// both pigment layers toward the window averages (dry paint included).

import { alphaOfMass } from './opacity.js';
import { forEachStampPixel } from './brush.js';
import { expandBBox, wetByteFromPaper } from './grid.js';

// Buffer half-size: max radius 35 plus the max window drift (~4 x spacing).
export const TRAIL_HALF = Math.ceil(35 + 4 * 6) + 2; // 61
export const TRAIL_SIZE = TRAIL_HALF * 2 + 1;        // 123

export function createTrail() {
  const n = TRAIL_SIZE * TRAIL_SIZE;
  return {
    pig: new Float32Array(n),
    water: new Float32Array(n),
    mask: new Float32Array(n), // blend mode's saturating coverage
    tipR: new Float32Array(n),
    tipG: new Float32Array(n),
    tipB: new Float32Array(n),
    baseR: 0, baseG: 0, baseB: 0,
    anchorX: 0, anchorY: 0,
    prevAnchorX: 0, prevAnchorY: 0,
    dabCount: 0,
    windowSize: 0, // C - dabs accumulated before a transfer
    // touched extent, local coords (inclusive); lx0 > lx1 means empty
    lx0: 0, ly0: 0, lx1: -1, ly1: -1,
    mode: 'paint',
  };
}

export function trailStartStroke(tr, x, y, color, mode) {
  tr.pig.fill(0); tr.water.fill(0); tr.mask.fill(0);
  tr.tipR.fill(color[0]); tr.tipG.fill(color[1]); tr.tipB.fill(color[2]);
  tr.baseR = color[0]; tr.baseG = color[1]; tr.baseB = color[2];
  tr.anchorX = Math.round(x); tr.anchorY = Math.round(y);
  tr.prevAnchorX = tr.anchorX; tr.prevAnchorY = tr.anchorY;
  tr.dabCount = 0;
  tr.windowSize = 0;
  tr.lx0 = 0; tr.ly0 = 0; tr.lx1 = -1; tr.ly1 = -1;
  tr.mode = mode;
}

/** Frame-segment callback: size the window from the chord length. */
export function trailOnSegment(tr, chordLen, spacing) {
  const cap = tr.mode === 'blend' ? 4 : 2;
  tr.windowSize = Math.min(cap, Math.floor(chordLen / Math.max(0.0001, spacing)));
}

function touch(tr, lx, ly) {
  if (tr.lx1 < tr.lx0) { tr.lx0 = tr.lx1 = lx; tr.ly0 = tr.ly1 = ly; return; }
  if (lx < tr.lx0) tr.lx0 = lx; if (lx > tr.lx1) tr.lx1 = lx;
  if (ly < tr.ly0) tr.ly0 = ly; if (ly > tr.ly1) tr.ly1 = ly;
}

/**
 * Accumulate one paint dab into the trail. dab = {x, y, r, hardness,
 * intensity, waterAmount, dryGate, shape, dirX, dirY}. Returns true when the
 * window is full and the caller must transfer.
 */
export function trailAccumulatePaint(tr, g, P, tex, dab, extBypass) {
  if (tr.dabCount === 0) { // the window's first dab anchors it
    tr.anchorX = Math.round(dab.x); tr.anchorY = Math.round(dab.y);
  }
  const { susp, sett, paper, film, wet } = g;
  const gain = P.pigmentPerDab;
  const gate = P.paperGate;
  const pigCap = P.gateSaturation;
  forEachStampPixel(g, tex, dab.x, dab.y, dab.r, dab.hardness, dab.shape, dab.dirX, dab.dirY,
    (i, x, y, fall, texv) => {
      let stamp = fall * texv * dab.intensity;
      if (stamp > 1) stamp = 1;
      if (stamp <= 0) return;
      // Paper gate: tooth peaks always take pigment, valleys reject it -
      // that per-pixel pass/reject IS the granulation. Heavily loaded cells
      // read a flat 0.45 tooth (the grain is already buried).
      const tooth = susp[i] + sett[i] < pigCap ? paper[i] : 0.45;
      let deposit = stamp - (1 - tooth) * gate;
      if (!extBypass) {
        // Dry-brush extension: raise the gate subtraction (broken color).
        deposit -= dab.dryGate * P.extDryBrush * 0.6;
      }
      if (deposit <= 0) return;
      if (!extBypass) {
        // Wet-edge softening extension: thin the dab's rim on wet paper.
        const softness = Math.min(1, film[i] / 3) * P.extWetSoften;
        deposit *= 1 - softness * (1 - fall);
      }
      // Wetness seed: OVERWRITE (not max) - repainting can dry the byte down.
      wet[i] = wetByteFromPaper(tooth);
      const lx = x - tr.anchorX + TRAIL_HALF;
      const ly = y - tr.anchorY + TRAIL_HALF;
      if (lx < 0 || lx >= TRAIL_SIZE || ly < 0 || ly >= TRAIL_SIZE) return;
      const l = lx + ly * TRAIL_SIZE;
      tr.pig[l] += deposit * gain;
      tr.water[l] += deposit * dab.waterAmount;
      touch(tr, lx, ly);
    });
  tr.dabCount++;
  return tr.dabCount > tr.windowSize;
}

/** Accumulate one blend dab: a saturating mask, no water/tip/wetness writes. */
export function trailAccumulateBlend(tr, g, P, tex, dab) {
  if (tr.dabCount === 0) {
    tr.anchorX = Math.round(dab.x); tr.anchorY = Math.round(dab.y);
  }
  const { susp, sett, paper } = g;
  const gate = P.paperGate;
  const pigCap = P.gateSaturation;
  const force = P.blendForce;
  forEachStampPixel(g, tex, dab.x, dab.y, dab.r, dab.hardness, dab.shape, dab.dirX, dab.dirY,
    (i, x, y, fall, texv) => {
      let stamp = fall * texv * dab.intensity;
      if (stamp > 1) stamp = 1;
      const tooth = susp[i] + sett[i] < pigCap ? paper[i] : 0.45;
      const e = (stamp - (1 - tooth) * gate) * force;
      if (e <= 0) return;
      const lx = x - tr.anchorX + TRAIL_HALF;
      const ly = y - tr.anchorY + TRAIL_HALF;
      if (lx < 0 || lx >= TRAIL_SIZE || ly < 0 || ly >= TRAIL_SIZE) return;
      const l = lx + ly * TRAIL_SIZE;
      tr.mask[l] = tr.mask[l] * (1 - e) + e; // scrubbing builds toward 1
      touch(tr, lx, ly);
    });
  tr.dabCount++;
  return tr.dabCount > tr.windowSize;
}

/**
 * Land the paint window on the canvas (spec 10 "transfer"), then roll it.
 * Returns the touched canvas rect {x0,y0,x1,y1} or null.
 */
export function trailTransferPaint(tr, g, P) {
  const out = [0, 0, 0];
  const mixColor = P.mixColor;
  // 1. Tip self-cleaning: every texel eases back toward the stroke color.
  const clean = P.tipClean;
  if (clean > 0) {
    const { tipR, tipG, tipB, baseR, baseG, baseB } = tr;
    for (let l = 0; l < tipR.length; l++) {
      tipR[l] += (baseR - tipR[l]) * clean;
      tipG[l] += (baseG - tipG[l]) * clean;
      tipB[l] += (baseB - tipB[l]) * clean;
    }
  }
  if (tr.lx1 < tr.lx0) { rollWindow(tr); return null; }
  const { S, W, H, susp, sett, film, wet } = g;
  const tipRetain = 1 - P.pickup;

  // 2. Tip pickup (the dirty brush) - reads the PRE-deposit canvas.
  for (let ly = tr.ly0; ly <= tr.ly1; ly++) {
    const cy = tr.anchorY + (ly - TRAIL_HALF);
    if (cy < 2 || cy > H - 1) continue;
    for (let lx = tr.lx0; lx <= tr.lx1; lx++) {
      const cx = tr.anchorX + (lx - TRAIL_HALF);
      if (cx < 2 || cx > W - 1) continue;
      const i = cx + cy * S;
      const wS = alphaOfMass(sett[i]) * 0.5;
      const wF = alphaOfMass(susp[i]);
      const fe = wS + wF;
      if (fe <= 0) continue; // blank paper keeps the tip color
      const inv = 1 / fe;
      const cr = (g.settR[i] * wS + g.suspR[i] * wF) * inv;
      const cg = (g.settG[i] * wS + g.suspG[i] * wF) * inv;
      const cb = (g.settB[i] * wS + g.suspB[i] * wF) * inv;
      const k = tipRetain + (1 - tipRetain) * (1 - Math.min(1, fe));
      const l = lx + ly * TRAIL_SIZE;
      tr.tipR[l] = tr.tipR[l] * k + cr * (1 - k);
      tr.tipG[l] = tr.tipG[l] * k + cg * (1 - k);
      tr.tipB[l] = tr.tipB[l] * k + cb * (1 - k);
    }
  }

  // 3. Soft-cap shedding means over cells holding trail pigment.
  let sumPig = 0, sumWater = 0, count = 0;
  for (let ly = tr.ly0; ly <= tr.ly1; ly++) {
    for (let lx = tr.lx0; lx <= tr.lx1; lx++) {
      const l = lx + ly * TRAIL_SIZE;
      if (tr.pig[l] > 0) { sumPig += tr.pig[l]; sumWater += tr.water[l]; count++; }
    }
  }
  const shedPig = count > 0 ? (sumPig / count) * 1.05 : 0;
  const shedWater = count > 0 ? (sumWater / count) * 1.05 : 0;

  // 4. Land the window + 5. paint drag.
  const dragW1 = Math.trunc(P.drag * 255);       // integer byte weights: they
  const dragW2 = Math.trunc((1 - P.drag) * 255); // sum < 256 on purpose, so
  const waterCap = P.waterCap;                   // each drag leaks ~0.8% wetness
  let rx0 = W, ry0 = H, rx1 = 0, ry1 = 0;
  for (let ly = tr.ly0; ly <= tr.ly1; ly++) {
    const cy = tr.anchorY + (ly - TRAIL_HALF);
    if (cy < 2 || cy > H - 1) continue;
    for (let lx = tr.lx0; lx <= tr.lx1; lx++) {
      const l = lx + ly * TRAIL_SIZE;
      const v = tr.pig[l];
      if (v <= 0) continue;
      const cx = tr.anchorX + (lx - TRAIL_HALF);
      if (cx < 2 || cx > W - 1) continue;
      const i = cx + cy * S;
      const inA = alphaOfMass(v);
      const old = susp[i];
      if (old > 0) {
        // Opacity composite of the (possibly dirty) tip over the resident.
        // (Sub-unit masses read alpha 0 from the table; guard the 0/0 case.)
        const e9 = alphaOfMass(old) * (1 - inA);
        if (e9 + inA > 0) {
          const w = inA / (e9 + inA);
          mixColor(g.suspR[i], g.suspG[i], g.suspB[i], tr.tipR[l], tr.tipG[l], tr.tipB[l], w, out);
          g.suspR[i] = out[0]; g.suspG[i] = out[1]; g.suspB[i] = out[2];
        }
        // Soft cap: past 3000 the cell sheds the window mean instead of
        // clamping, so heavy paint hovers at the cap.
        let nm = old < 3000 ? old + v : old + v - shedPig;
        susp[i] = nm < 0 ? 0 : nm;
      } else {
        // Virgin cell: takes the tip color outright, and drinks its first
        // wetting twice (this add + the general one below).
        g.suspR[i] = tr.tipR[l]; g.suspG[i] = tr.tipG[l]; g.suspB[i] = tr.tipB[l];
        susp[i] = v;
        film[i] += tr.water[l];
      }
      if (sett[i] > 3000) {
        const ns = sett[i] - shedPig;
        sett[i] = ns < 0 ? 0 : ns;
      }
      let nf = film[i] < waterCap ? film[i] + tr.water[l] : film[i] + tr.water[l] - shedWater;
      if (nf < 0.00001) nf = 0.00001; // trace floor: the wet map must know
      film[i] = nf;
      // 5. Paint drag: pull from the same window offset at the PREVIOUS
      // anchor - the film gets tugged along under the stroke.
      const sx = tr.prevAnchorX + (lx - TRAIL_HALF);
      const sy = tr.prevAnchorY + (ly - TRAIL_HALF);
      if (sx >= 2 && sx <= W - 1 && sy >= 2 && sy <= H - 1) {
        const si = sx + sy * S;
        const hadSusp = susp[i] > 0, hadSett = sett[i] > 0;
        susp[i] += (susp[si] - susp[i]) * P.drag;
        sett[i] += (sett[si] - sett[i]) * P.drag;
        film[i] += (film[si] - film[i]) * P.drag;
        if (!hadSusp && susp[i] > 0) {
          g.suspR[i] = g.suspR[si]; g.suspG[i] = g.suspG[si]; g.suspB[i] = g.suspB[si];
        }
        if (!hadSett && sett[i] > 0) {
          g.settR[i] = g.settR[si]; g.settG[i] = g.settG[si]; g.settB[i] = g.settB[si];
        }
        wet[i] = ((wet[si] * dragW1 + wet[i] * dragW2) / 256) | 0;
      }
      if (cx < rx0) rx0 = cx; if (cx > rx1) rx1 = cx;
      if (cy < ry0) ry0 = cy; if (cy > ry1) ry1 = cy;
    }
  }
  rollWindow(tr);
  if (rx1 < rx0) return null;
  expandBBox(g, rx0 - 1, ry0 - 1, rx1 + 1, ry1 + 1);
  return { x0: rx0, y0: ry0, x1: rx1, y1: ry1 };
}

/**
 * Blend transfer (spec section 11 "blend"): window averages over mask-active
 * cells, then every masked cell relaxes toward them - suspended AND settled
 * (dry paint re-mixes), water, and the wetness byte.
 */
export function trailTransferBlend(tr, g, P) {
  if (tr.lx1 < tr.lx0) { rollWindow(tr); return null; }
  const { S, W, H, susp, sett, film, wet } = g;
  const out = [0, 0, 0];
  const mixColor = P.mixColor;
  // Pass 1: averages.
  let n = 0, sSusp = 0, sSett = 0, sFilm = 0, sWet = 0;
  let sR = 0, sG = 0, sB = 0, tR = 0, tG = 0, tB = 0, wSusp = 0, wSett = 0;
  for (let ly = tr.ly0; ly <= tr.ly1; ly++) {
    const cy = tr.anchorY + (ly - TRAIL_HALF);
    if (cy < 2 || cy > H - 1) continue;
    for (let lx = tr.lx0; lx <= tr.lx1; lx++) {
      if (tr.mask[lx + ly * TRAIL_SIZE] <= 0) continue;
      const cx = tr.anchorX + (lx - TRAIL_HALF);
      if (cx < 2 || cx > W - 1) continue;
      const i = cx + cy * S;
      n++;
      sSusp += susp[i]; sSett += sett[i]; sFilm += film[i]; sWet += wet[i];
      sR += g.suspR[i] * susp[i]; sG += g.suspG[i] * susp[i]; sB += g.suspB[i] * susp[i];
      wSusp += susp[i];
      tR += g.settR[i] * sett[i]; tG += g.settG[i] * sett[i]; tB += g.settB[i] * sett[i];
      wSett += sett[i];
    }
  }
  if (n === 0) { rollWindow(tr); return null; }
  const avgSusp = sSusp / n, avgSett = sSett / n, avgFilm = sFilm / n, avgWet = sWet / n;
  const aR = wSusp > 0 ? sR / wSusp : 0, aG = wSusp > 0 ? sG / wSusp : 0, aB = wSusp > 0 ? sB / wSusp : 0;
  const bR = wSett > 0 ? tR / wSett : 0, bG = wSett > 0 ? tG / wSett : 0, bB = wSett > 0 ? tB / wSett : 0;
  // Pass 2: relax every masked cell toward the window mix.
  let rx0 = W, ry0 = H, rx1 = 0, ry1 = 0;
  for (let ly = tr.ly0; ly <= tr.ly1; ly++) {
    const cy = tr.anchorY + (ly - TRAIL_HALF);
    if (cy < 2 || cy > H - 1) continue;
    for (let lx = tr.lx0; lx <= tr.lx1; lx++) {
      let a = tr.mask[lx + ly * TRAIL_SIZE];
      if (a <= 0) continue;
      if (a > 1) a = 1;
      const cx = tr.anchorX + (lx - TRAIL_HALF);
      if (cx < 2 || cx > W - 1) continue;
      const i = cx + cy * S;
      if (wSusp > 0) {
        const inC = alphaOfMass(avgSusp) * a;
        const res = alphaOfMass(susp[i]) * (1 - inC);
        if (inC + res > 0) {
          const w = inC / (inC + res);
          mixColor(g.suspR[i], g.suspG[i], g.suspB[i], aR, aG, aB, w, out);
          g.suspR[i] = out[0]; g.suspG[i] = out[1]; g.suspB[i] = out[2];
        }
        susp[i] += (avgSusp - susp[i]) * a;
        film[i] += (avgFilm - film[i]) * a;
      }
      if (wSett > 0) {
        const inC = alphaOfMass(avgSett) * a;
        const res = alphaOfMass(sett[i]) * (1 - inC);
        if (inC + res > 0) {
          const w = inC / (inC + res);
          mixColor(g.settR[i], g.settG[i], g.settB[i], bR, bG, bB, w, out);
          g.settR[i] = out[0]; g.settG[i] = out[1]; g.settB[i] = out[2];
        }
        sett[i] += (avgSett - sett[i]) * a;
        wet[i] += (avgWet - wet[i]) * a; // byte relaxes in the settled pass
      }
      if (cx < rx0) rx0 = cx; if (cx > rx1) rx1 = cx;
      if (cy < ry0) ry0 = cy; if (cy > ry1) ry1 = cy;
    }
  }
  rollWindow(tr);
  if (rx1 < rx0) return null;
  expandBBox(g, rx0 - 1, ry0 - 1, rx1 + 1, ry1 + 1);
  return { x0: rx0, y0: ry0, x1: rx1, y1: ry1 };
}

/** Roll: previous anchor <- this anchor; clear the trail + its extent. */
function rollWindow(tr) {
  tr.prevAnchorX = tr.anchorX; tr.prevAnchorY = tr.anchorY;
  if (tr.lx1 >= tr.lx0) {
    for (let ly = tr.ly0; ly <= tr.ly1; ly++) {
      const b = ly * TRAIL_SIZE;
      tr.pig.fill(0, b + tr.lx0, b + tr.lx1 + 1);
      tr.water.fill(0, b + tr.lx0, b + tr.lx1 + 1);
      tr.mask.fill(0, b + tr.lx0, b + tr.lx1 + 1);
    }
  }
  tr.lx0 = 0; tr.ly0 = 0; tr.lx1 = -1; tr.ly1 = -1;
  tr.dabCount = 0;
}

/** Stroke end: the remainder window is DROPPED (the release tail already
 *  emitted the fade-out; landing it would double the ending). */
export function trailDropRemainder(tr) {
  rollWindow(tr);
}
