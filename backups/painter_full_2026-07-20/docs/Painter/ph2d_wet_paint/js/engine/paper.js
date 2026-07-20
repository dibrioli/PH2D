// Procedural paper (spec section 4).
//
// A 512x512 height tile in [0,1], tiled across the canvas with integer wrap
// and baked into the padded grid (pad ring included - border cells must read
// a real tooth or the edge capillary term diverges).
//
// The relief the solver needs is a fine quasi-periodic FELT: narrow grooves
// 1-2 px wide broken into short segments, densely packed, with one dominant
// orientation; nearly all height variance below 8 px wavelength. Long smooth
// valleys are a defect - they make drip fronts descend as fused wedges
// instead of fingering into rivulets that follow the grain.
//
// Construction: (lattice undulation + fine tooth + hash sparkle) + thousands
// of short fiber strokes, then a high-pass (subtract 0.55 x 8-px wrap box
// blur), then a contrast stretch to the preset's target mean/std with a
// final clamp to [0,1] (the clamp mints the sparse 0/1 tails real scans have).

import { makeRng, hash2 } from './rng.js';

export const TILE_SIZE = 512;
const TS = TILE_SIZE;
const TMASK = TS - 1;

// Target statistics match scans of real sheets (acceptance-tested).
// Amplitudes below are calibration values: only their RELATIVE size matters
// (the contrast stretch renormalizes), and they were tuned so the measured
// mean-gradient lands on the target for each preset.
export const PAPER_PRESETS = {
  cold: { // cold press (default): isotropic-ish felt
    mean: 0.543, std: 0.150, grad: 0.121,
    fibres: 9000, angle: 0.35, jitter: 1.0,
    toothAmp: 0.28, sparkleAmp: 0.062, fibreDepth: 0.16,
  },
  rough: { // rough: deep tooth, strongly vertical grain
    mean: 0.517, std: 0.192, grad: 0.144,
    fibres: 11000, angle: Math.PI / 2, jitter: 0.25,
    toothAmp: 0.34, sparkleAmp: 0.052, fibreDepth: 0.20,
  },
  hot: { // hot press: smooth, shallow, fine
    mean: 0.497, std: 0.106, grad: 0.098,
    fibres: 5000, angle: 0.35, jitter: 1.1,
    toothAmp: 0.20, sparkleAmp: 0.125, fibreDepth: 0.10,
  },
};

const PRESET_SEEDS = { cold: 11, rough: 23, hot: 37 };

/** Seamless bilinear value-noise from an n x n lattice, sampled at tile (x,y). */
function latticeNoise(rngValues, n, x, y) {
  const fx = (x / TS) * n, fy = (y / TS) * n;
  const x0 = fx | 0, y0 = fy | 0;
  const tx = fx - x0, ty = fy - y0;
  const x1 = (x0 + 1) % n, y1 = (y0 + 1) % n;
  const v00 = rngValues[y0 * n + x0], v10 = rngValues[y0 * n + x1];
  const v01 = rngValues[y1 * n + x0], v11 = rngValues[y1 * n + x1];
  const a = v00 + (v10 - v00) * tx;
  const b = v01 + (v11 - v01) * tx;
  return a + (b - a) * ty;
}

/**
 * Generate one paper tile. knobs = { contrast, fibres, grooves } are the live
 * paper-group multipliers (all default 1).
 */
export function generatePaperTile(presetName, sheetIndex, knobs = {}) {
  const p = PAPER_PRESETS[presetName] || PAPER_PRESETS.cold;
  const contrastMul = knobs.contrast ?? 1;
  const fibreMul = knobs.fibres ?? 1;
  const grooveMul = knobs.grooves ?? 1;
  const seed = (PRESET_SEEDS[presetName] ?? 11) * 1000003 + (sheetIndex | 0) * 7919 + 5;
  const rng = makeRng(seed);
  const t = new Float32Array(TS * TS);

  // -- component 1+2: a faint large-scale undulation (6x6 lattice) and a fine
  //    2-px tooth (256x256 lattice), both seamless.
  const big = new Float32Array(36);
  for (let i = 0; i < 36; i++) big[i] = rng();
  const FINE = 256;
  const fine = new Float32Array(FINE * FINE);
  for (let i = 0; i < fine.length; i++) fine[i] = rng();
  const sparkleSeed = (seed ^ 0x5f3759df) | 0;
  for (let y = 0; y < TS; y++) {
    for (let x = 0; x < TS; x++) {
      let v = 0.5;
      v += (latticeNoise(big, 6, x, y) - 0.5) * 0.02;               // undulation
      v += (latticeNoise(fine, FINE, x, y) - 0.5) * p.toothAmp;     // fine tooth
      v += (hash2(x, y, sparkleSeed) - 0.5) * p.sparkleAmp;         // sparkle
      t[y * TS + x] = v;
    }
  }

  // -- component 3: thousands of short fiber strokes. Each walks 5-13 steps
  //    of ~0.7 px along the grain angle (+- jitter), stamping a soft round
  //    cross-section, mostly grooves (negative) with ~38% bright ridges.
  const count = Math.max(0, Math.round(p.fibres * fibreMul));
  for (let f = 0; f < count; f++) {
    let fx = rng() * TS, fy = rng() * TS;
    const ang = p.angle + (rng() * 2 - 1) * p.jitter;
    const dx = Math.cos(ang) * 0.7, dy = Math.sin(ang) * 0.7;
    const steps = 5 + (rng() * 9) | 0;
    const halfW = 0.4 + rng() * 0.7;
    const sign = rng() < 0.38 ? 1 : -1;
    const depth = sign * (0.5 + rng() * 0.5) * p.fibreDepth * grooveMul;
    const reach = Math.ceil(halfW + 1);
    const invHw = 1 / (halfW + 0.5);
    for (let s = 0; s < steps; s++) {
      const cx = fx, cy = fy;
      const x0 = Math.floor(cx) - reach, x1 = Math.floor(cx) + reach;
      const y0 = Math.floor(cy) - reach, y1 = Math.floor(cy) + reach;
      for (let yy = y0; yy <= y1; yy++) {
        for (let xx = x0; xx <= x1; xx++) {
          const ddx = xx - cx, ddy = yy - cy;
          let d = Math.sqrt(ddx * ddx + ddy * ddy) * invHw;
          if (d >= 1) continue;
          const w = 1 - d * d * (3 - 2 * d); // soft round cross-section
          t[(yy & TMASK) * TS + (xx & TMASK)] += depth * w;
        }
      }
      fx += dx; fy += dy;
    }
  }

  // -- high-pass: subtract 0.55 x an 8-px wrap-around box blur (separable),
  //    pushing ~95% of the variance below 8 px wavelength.
  const blurred = boxBlurWrap(t, 4);
  for (let i = 0; i < t.length; i++) t[i] -= 0.55 * blurred[i];

  // -- contrast stretch to the target statistics, then clamp.
  let mean = 0;
  for (let i = 0; i < t.length; i++) mean += t[i];
  mean /= t.length;
  let variance = 0;
  for (let i = 0; i < t.length; i++) { const d = t[i] - mean; variance += d * d; }
  const std = Math.sqrt(variance / t.length) || 1;
  const gain = (p.std * contrastMul) / std;
  for (let i = 0; i < t.length; i++) {
    let v = p.mean + (t[i] - mean) * gain;
    t[i] = v < 0 ? 0 : v > 1 ? 1 : v;
  }
  return t;
}

/** Separable wrap-around box blur, radius r (window 2r+1). */
function boxBlurWrap(src, r) {
  const tmp = new Float32Array(TS * TS);
  const out = new Float32Array(TS * TS);
  const inv = 1 / (2 * r + 1);
  for (let y = 0; y < TS; y++) { // horizontal
    const row = y * TS;
    let acc = 0;
    for (let k = -r; k <= r; k++) acc += src[row + (k & TMASK)];
    for (let x = 0; x < TS; x++) {
      tmp[row + x] = acc * inv;
      acc += src[row + ((x + r + 1) & TMASK)] - src[row + ((x - r) & TMASK)];
    }
  }
  for (let x = 0; x < TS; x++) { // vertical
    let acc = 0;
    for (let k = -r; k <= r; k++) acc += tmp[(k & TMASK) * TS + x];
    for (let y = 0; y < TS; y++) {
      out[y * TS + x] = acc * inv;
      acc += tmp[((y + r + 1) & TMASK) * TS + x] - tmp[((y - r) & TMASK) * TS + x];
    }
  }
  return out;
}

/**
 * Bake a tile into the padded grid paper array, pad ring included, with
 * integer wrap (nearest texel). Cell (x,y) maps to tile texel (x-1, y-1).
 */
export function bakePaper(grid, tile) {
  const { S, rows, paper } = grid;
  for (let y = 0; y < rows; y++) {
    const ty = ((y - 1) & TMASK) * TS;
    let i = y * S;
    for (let x = 0; x < S; x++, i++) {
      paper[i] = tile[ty + ((x - 1) & TMASK)];
    }
  }
}

/** Measured statistics of a tile (used by the acceptance test and calibration). */
export function measureTileStats(t) {
  let mean = 0;
  for (let i = 0; i < t.length; i++) mean += t[i];
  mean /= t.length;
  let variance = 0, gradSum = 0;
  for (let y = 0; y < TS; y++) {
    for (let x = 0; x < TS; x++) {
      const v = t[y * TS + x];
      const d = v - mean;
      variance += d * d;
      const gx = (t[y * TS + ((x + 1) & TMASK)] - t[y * TS + ((x - 1) & TMASK)]) * 0.5;
      const gy = (t[((y + 1) & TMASK) * TS + x] - t[((y - 1) & TMASK) * TS + x]) * 0.5;
      gradSum += Math.sqrt(gx * gx + gy * gy);
    }
  }
  return {
    mean,
    std: Math.sqrt(variance / t.length),
    grad: gradSum / t.length,
  };
}
