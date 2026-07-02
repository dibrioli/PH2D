// Direct tools (spec section 11): erase, wet, dry, blow, smear.
// Each dispatches per dab (no trail), with the fixed tool hardness 4.8 and
// stamp = min(1, falloff * texture * min(3, intensity)). All respect the
// brushable area (the stamp iterator enforces it) and expand the active bbox.

import { alphaOfMass } from './opacity.js';
import { forEachStampPixel } from './brush.js';
import { expandBBox, wetByteFromPaper } from './grid.js';

export const TOOL_HARDNESS = 4.8;

function rectAround(g, x, y, r) {
  return {
    x0: Math.max(2, Math.floor(x - r) - 1), y0: Math.max(2, Math.floor(y - r) - 1),
    x1: Math.min(g.W - 1, Math.ceil(x + r) + 1), y1: Math.min(g.H - 1, Math.ceil(y + r) + 1),
  };
}

/** erase: paper-gated multiplicative removal of both layers + water. */
export function applyErase(g, P, tex, dab, eraseSlider) {
  const { susp, sett, film, paper } = g;
  const gate = P.paperGate, pigCap = P.gateSaturation;
  const force = P.eraser * eraseSlider;
  const clampI = Math.min(3, dab.intensity);
  let touched = false;
  forEachStampPixel(g, tex, dab.x, dab.y, dab.r, TOOL_HARDNESS, dab.shape, dab.dirX, dab.dirY,
    (i, x, y, fall, texv) => {
      let stamp = fall * texv * clampI;
      if (stamp > 1) stamp = 1;
      const tooth = susp[i] + sett[i] < pigCap ? paper[i] : 0.45;
      let gg = stamp - (1 - tooth) * gate;
      if (gg <= 0) return;
      if (gg > 1) gg = 1;
      const k = 1 - force * gg;
      if (k <= 0) return; // never wipe a cell outright
      susp[i] *= k; film[i] *= k;
      if (susp[i] < 1) susp[i] = 0; // stale color bytes may remain; mass 0 hides them
      sett[i] *= k;
      if (sett[i] < 1) sett[i] = 0;
      touched = true;
    });
  if (touched) {
    const r = rectAround(g, dab.x, dab.y, dab.r);
    expandBBox(g, r.x0, r.y0, r.x1, r.y1);
    return r;
  }
  return null;
}

/** wet: drip-feed water; gentle, accumulates over repeated passes. */
export function applyWet(g, P, tex, dab, waterUnit) {
  const { film, paper, wet } = g;
  const add = 2 * (waterUnit * 0.2) * (waterUnit * 0.2) * P.waterPerDab;
  const cap = P.waterCap;
  const clampI = Math.min(3, dab.intensity);
  let touched = false;
  forEachStampPixel(g, tex, dab.x, dab.y, dab.r, TOOL_HARDNESS, dab.shape, dab.dirX, dab.dirY,
    (i, x, y, fall, texv) => {
      let stamp = fall * texv * clampI;
      if (stamp > 1) stamp = 1;
      if (stamp <= 0) return;
      let f = film[i] + stamp * add;
      if (f > cap) f = cap;
      film[i] = f;
      wet[i] = wetByteFromPaper(paper[i]);
      touched = true;
    });
  if (touched) {
    const r = rectAround(g, dab.x, dab.y, dab.r);
    expandBBox(g, r.x0, r.y0, r.x1, r.y1);
    return r;
  }
  return null;
}

/** dry: shrink the film (never to exact zero) and SEAL the paper (wet = 0). */
export function applyDry(g, P, tex, dab) {
  const { film, wet } = g;
  const force = P.dryer;
  const clampI = Math.min(3, dab.intensity);
  let touched = false;
  forEachStampPixel(g, tex, dab.x, dab.y, dab.r, TOOL_HARDNESS, dab.shape, dab.dirX, dab.dirY,
    (i, x, y, fall, texv) => {
      let stamp = fall * texv * clampI;
      if (stamp > 1) stamp = 1;
      if (stamp <= 0) return;
      let f = film[i] * (0.999 - stamp * force);
      if (f < 0.001) f = 0.001; // floor: the wet map must know the stroke passed
      film[i] = f;
      wet[i] = 0; // sealed: future bleed stops here
      touched = true;
    });
  if (touched) {
    const r = rectAround(g, dab.x, dab.y, dab.r);
    expandBBox(g, r.x0, r.y0, r.x1, r.y1);
    return r;
  }
  return null;
}

/**
 * blow: needs the sim live. Injects the (asymmetrically clamped) pointer
 * displacement into the PERSISTENT velocity of wet cells, and drags the
 * wetness byte from the drag source through a read-before-write snapshot
 * (even across dry cells).
 */
export function applyBlow(g, P, tex, dab, prevX, prevY) {
  const { S, W, H, film, paper, wet, velX, velY } = g;
  const force = P.blow;
  const clampI = Math.min(3, dab.intensity);
  const rawDx = dab.x - prevX, rawDy = dab.y - prevY;
  // Weak, slightly anti-forward: each axis clamps to [-0.6, +0.3].
  const cdx = rawDx < -0.6 ? -0.6 : rawDx > 0.3 ? 0.3 : rawDx;
  const cdy = rawDy < -0.6 ? -0.6 : rawDy > 0.3 ? 0.3 : rawDy;
  const offX = Math.round(rawDx), offY = Math.round(rawDy); // UNCLAMPED source offset
  // Snapshot the wetness in the affected region so the drag reads pre-write
  // values (sampling live bytes would compound the drag within one dab).
  const r = rectAround(g, dab.x, dab.y, dab.r + Math.max(Math.abs(offX), Math.abs(offY)) + 1);
  const snapW = r.x1 - r.x0 + 1, snapH = r.y1 - r.y0 + 1;
  const snap = new Uint8Array(snapW * snapH);
  for (let y = r.y0; y <= r.y1; y++) {
    for (let x = r.x0; x <= r.x1; x++) snap[(x - r.x0) + (y - r.y0) * snapW] = wet[x + y * S];
  }
  const readSnap = (x, y) => {
    if (x < r.x0 || x > r.x1 || y < r.y0 || y > r.y1) return wet[x + y * S];
    return snap[(x - r.x0) + (y - r.y0) * snapW];
  };
  let touched = false;
  forEachStampPixel(g, tex, dab.x, dab.y, dab.r, TOOL_HARDNESS, dab.shape, dab.dirX, dab.dirY,
    (i, x, y, fall, texv) => {
      let stamp = fall * texv * clampI;
      if (stamp > 1) stamp = 1;
      if (stamp <= 0) return;
      // Skip the whole pixel when the UNCLAMPED, rounded source leaves the sheet.
      const sx = x - offX, sy = y - offY;
      if (sx < 1 || sx > W || sy < 1 || sy > H) return;
      const e = stamp * force;
      if (film[i] > 0) {
        velX[i] += cdx * e;
        velY[i] += cdy * e;
        wet[i] = wetByteFromPaper(paper[i]); // re-wet under the air jet
      }
      // Wetness dragged from the source wherever the stamp touches, wet or dry.
      wet[i] = readSnap(sx, sy) * force + wet[i] * (1 - force);
      touched = true;
    });
  if (touched) {
    const rr = rectAround(g, dab.x, dab.y, dab.r);
    expandBBox(g, rr.x0, rr.y0, rr.x1, rr.y1);
    return rr;
  }
  return null;
}

/**
 * smear: drag suspended pigment + water from the previous-dab offset (the
 * reach grows with speed). Only the SUSPENDED layer moves - the settled
 * layer is deliberately untouched, matching real smudge behavior. The source
 * is a mass-weighted bilinear sample of a pre-dab snapshot (weighting each
 * corner by its mass keeps edges from dragging toward black).
 */
export function applySmear(g, P, tex, dab, prevX, prevY) {
  const { S, susp, suspR, suspG, suspB, film } = g;
  const force = P.smear;
  const clampI = Math.min(3, dab.intensity);
  const dx = dab.x - prevX, dy = dab.y - prevY;
  const reach = Math.ceil(Math.max(Math.abs(dx), Math.abs(dy)));
  // Snapshot the affected region (+10 px pad): sampling live arrays would
  // compound the drag within one dab.
  const pad = 10 + reach;
  const r = rectAround(g, dab.x, dab.y, dab.r + pad);
  const snapW = r.x1 - r.x0 + 1, snapH = r.y1 - r.y0 + 1;
  const sMass = new Float32Array(snapW * snapH);
  const sR = new Float32Array(snapW * snapH);
  const sG = new Float32Array(snapW * snapH);
  const sB = new Float32Array(snapW * snapH);
  const sF = new Float32Array(snapW * snapH);
  for (let y = r.y0; y <= r.y1; y++) {
    for (let x = r.x0; x <= r.x1; x++) {
      const i = x + y * S, l = (x - r.x0) + (y - r.y0) * snapW;
      sMass[l] = susp[i]; sR[l] = suspR[i]; sG[l] = suspG[i]; sB[l] = suspB[i]; sF[l] = film[i];
    }
  }
  const out = [0, 0, 0];
  const mixColor = P.mixColor;
  let touched = false;
  forEachStampPixel(g, tex, dab.x, dab.y, dab.r, TOOL_HARDNESS, dab.shape, dab.dirX, dab.dirY,
    (i, x, y, fall, texv) => {
      let stamp = fall * texv * clampI;
      if (stamp > 1) stamp = 1;
      const u = Math.min(1, stamp * force);
      if (u <= 0) return;
      // Clamp the sample offset to the snapshot pad.
      let fx = x - dx, fy = y - dy;
      if (fx < r.x0) fx = r.x0; if (fx > r.x1 - 1) fx = r.x1 - 1;
      if (fy < r.y0) fy = r.y0; if (fy > r.y1 - 1) fy = r.y1 - 1;
      const x0 = Math.floor(fx), y0 = Math.floor(fy);
      const tx = fx - x0, ty = fy - y0;
      const l00 = (x0 - r.x0) + (y0 - r.y0) * snapW;
      const l10 = l00 + 1, l01 = l00 + snapW, l11 = l01 + 1;
      const w00 = (1 - tx) * (1 - ty), w10 = tx * (1 - ty), w01 = (1 - tx) * ty, w11 = tx * ty;
      // Mass-weighted color sample.
      const m00 = sMass[l00] * w00, m10 = sMass[l10] * w10, m01 = sMass[l01] * w01, m11 = sMass[l11] * w11;
      const srcMass = m00 + m10 + m01 + m11;
      if (srcMass > 0) {
        const inv = 1 / srcMass;
        const cr = (sR[l00] * m00 + sR[l10] * m10 + sR[l01] * m01 + sR[l11] * m11) * inv;
        const cg = (sG[l00] * m00 + sG[l10] * m10 + sG[l01] * m01 + sG[l11] * m11) * inv;
        const cbv = (sB[l00] * m00 + sB[l10] * m10 + sB[l01] * m01 + sB[l11] * m11) * inv;
        const inC = alphaOfMass(srcMass) * u;
        const res = alphaOfMass(susp[i]) * (1 - inC);
        if (inC + res > 0) {
          const w = inC / (inC + res);
          mixColor(suspR[i], suspG[i], suspB[i], cr, cg, cbv, w, out);
          suspR[i] = out[0]; suspG[i] = out[1]; suspB[i] = out[2];
        }
        susp[i] += (srcMass - susp[i]) * u;
      }
      // Water lerps unconditionally (plain bilinear).
      const srcFilm = sF[l00] * w00 + sF[l10] * w10 + sF[l01] * w01 + sF[l11] * w11;
      film[i] += (srcFilm - film[i]) * u;
      touched = true;
    });
  if (touched) {
    const rr = rectAround(g, dab.x, dab.y, dab.r);
    expandBBox(g, rr.x0, rr.y0, rr.x1, rr.y1);
    return rr;
  }
  return null;
}
