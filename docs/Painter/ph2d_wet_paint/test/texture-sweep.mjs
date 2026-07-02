// Scratch calibration search for the bristle texture (spec sections 7,
// 18.11, 18.12 - v3). Placement is UNIFORM by spec; only the population
// parameters (core fraction / radii / strengths) and the deterministic seed
// are searched. Every candidate is checked against:
//   - tile mean / %>0.04,
//   - origin deposit integral + clearing count,
//   - the 3x3-grid uniformity clause (mean 4.0 +-25%, min >= 1.5),
//   - stroke budgets at r=12 AND r=25 (mass/water/coverage),
//   - lane structure at both radii (upper-half share, spread, max gap).
// Run: node test/texture-sweep.mjs [maxSeeds]

import { buildBristleTexture, radialFalloff, BRISTLE_SIZE } from '../js/engine/brush.js';
import {
  createEngine, activeGrid, pointerDown, pointerFrame, pointerUp, releaseFrame,
} from '../js/engine/painter.js';

const BMASK = BRISTLE_SIZE - 1;

function integralAt(tex, cx, cy) {
  let integral = 0, clearing = 0;
  for (let dy = -12; dy <= 12; dy++) {
    for (let dx = -12; dx <= 12; dx++) {
      const d2 = (dx * dx + dy * dy) / 144;
      if (d2 >= 1) continue;
      const f = radialFalloff(Math.sqrt(d2), 9);
      const t = tex[((cy + dy) & BMASK) * BRISTLE_SIZE + ((cx + dx) & BMASK)];
      let v = Math.min(1, f * t * 2.415) - 0.3;
      if (v > 0) { integral += v; clearing++; }
    }
  }
  return { integral, clearing };
}

export function texStats(tex) {
  let mean = 0, a4 = 0;
  for (const v of tex) { mean += v; if (v > 0.04) a4++; }
  mean /= tex.length;
  const origin = integralAt(tex, 0, 0);
  // Uniformity: a 3x3 grid of well-spread tile positions (torus).
  let gSum = 0, gMin = Infinity;
  for (const cy of [0, 43, 85]) {
    for (const cx of [0, 43, 85]) {
      const { integral } = integralAt(tex, cx, cy);
      gSum += integral;
      if (integral < gMin) gMin = integral;
    }
  }
  return {
    mean, p4: (100 * a4) / tex.length,
    integral: origin.integral, clearing: origin.clearing,
    gridMean: gSum / 9, gridMin: gMin,
  };
}

/** One section-18.11 stroke; returns budgets + section-18.12 lane metrics. */
export function strokeCheck(tex, sizeSlider, r) {
  const e = createEngine({ width: 300, height: 100 });
  if (tex) e.brushTex = tex;
  e.sliders.size = sizeSlider;
  pointerDown(e, 20, 50);
  let x = 20;
  for (let f = 0; f < 65; f++) { x += 4; pointerFrame(e, x, 50); }
  pointerUp(e);
  while (releaseFrame(e, x, 50)) { /* no tail */ }
  const g = activeGrid(e);
  let mass = 0, water = 0, band = 0, touched = 0;
  for (let y = 1; y <= g.H; y++) {
    for (let cx = 1; cx <= g.W; cx++) {
      const i = cx + y * g.S;
      mass += g.susp[i] + g.sett[i];
      water += g.film[i];
      if (Math.abs(y - 50) <= r && cx >= 20 && cx <= 280) {
        band++;
        if (g.susp[i] + g.sett[i] > 0) touched++;
      }
    }
  }
  // Lane structure: suspended mass per row over the central 220 columns.
  const rows = [];
  for (let dy = -r; dy <= r; dy++) {
    let s = 0;
    for (let cx = 40; cx <= 259; cx++) s += g.susp[cx + (50 + dy) * g.S];
    rows.push(s);
  }
  const maxRow = Math.max(...rows);
  let upper = 0, lower = 0, above5 = 0, run = 0, maxRun = 0;
  for (let k = 0; k < rows.length; k++) {
    const dy = k - r;
    if (dy < 0) upper += rows[k];
    else if (dy > 0) lower += rows[k];
    if (rows[k] > 0.05 * maxRow) above5++;
    if (rows[k] < 0.02 * maxRow) { run++; if (run > maxRun) maxRun = run; } else run = 0;
  }
  return {
    mass, water, cov: touched / band,
    upperShare: upper / (upper + lower),
    above5, maxRun,
  };
}

const knobs = { felt: 0.01, bristleCount: 950, bristleStrength: 1, bristleSize: 1 };
const fine = { fineRadiusMin: 0.4, fineRadiusSpan: 0.4, fineStrengthMin: 0.03, fineStrengthSpan: 0.05, plateau: 0.55 };

if (process.argv[1] && process.argv[1].endsWith('texture-sweep.mjs')) {
  const maxSeeds = Number(process.argv[2] ?? 120);
  const paramSets = [];
  for (const coreFraction of [0.18, 0.22, 0.26]) {
    for (const [coreRadiusMin, coreRadiusSpan] of [[0.8, 0.5], [0.9, 0.6]]) {
      for (const [coreStrengthMin, coreStrengthSpan] of [[0.16, 0.16], [0.17, 0.17]]) {
        paramSets.push({ coreFraction, coreRadiusMin, coreRadiusSpan, coreStrengthMin, coreStrengthSpan });
      }
    }
  }
  let evaluated = 0, passes = 0;
  for (const ps of paramSets) {
    for (let s = 0; s < maxSeeds; s++) {
      const seed = (0x7a3e0000 + s * 7919) >>> 0;
      const trial = { ...fine, ...ps, seed };
      const tex = buildBristleTexture(knobs, trial);
      const st = texStats(tex);
      if (st.integral < 3.5 || st.integral > 4.5 || st.clearing < 13 || st.clearing > 23) continue;
      if (st.gridMean < 3.2 || st.gridMean > 4.8 || st.gridMin < 1.7) continue;
      if (st.mean < 0.017 || st.mean > 0.026 || st.p4 > 9) continue;
      evaluated++;
      const r12 = strokeCheck(tex, 10 / 33, 12);
      const r12ok =
        Math.abs(r12.mass - 235000) <= 235000 * 0.11 &&
        Math.abs(r12.water - 245) <= 245 * 0.11 &&
        r12.cov >= 0.205 && r12.cov <= 0.265 &&
        r12.upperShare >= 0.36 && r12.upperShare <= 0.64 && r12.above5 >= 8 && r12.maxRun <= 5;
      const r25 = r12ok ? strokeCheck(tex, 0.7, 25) : null;
      const ok = r25 !== null &&
        Math.abs(r25.mass - 890000) <= 890000 * 0.13 &&
        Math.abs(r25.water - 835) <= 835 * 0.13 &&
        r25.cov >= 0.335 && r25.cov <= 0.44 &&
        r25.upperShare >= 0.36 && r25.upperShare <= 0.64 &&
        r25.above5 >= 21 && r25.maxRun <= 11;
      if (ok) passes++;
      console.log(
        (ok ? 'PASS ' : '     ') +
        `cF${ps.coreFraction} rM${ps.coreRadiusMin} cS${ps.coreStrengthMin} sd 0x${seed.toString(16)}` +
        ` | mean ${st.mean.toFixed(4)} p4 ${st.p4.toFixed(1)} int ${st.integral.toFixed(2)} clr ${st.clearing}` +
        ` gM ${st.gridMean.toFixed(2)} gMin ${st.gridMin.toFixed(2)}` +
        ` | r12 m${Math.round(r12.mass / 1000)}k w${r12.water.toFixed(0)} c${(r12.cov * 100).toFixed(1)} u${(r12.upperShare * 100).toFixed(0)} a5:${r12.above5} run:${r12.maxRun}` +
        (r25 ? ` | r25 m${Math.round(r25.mass / 1000)}k w${r25.water.toFixed(0)} c${(r25.cov * 100).toFixed(1)} u${(r25.upperShare * 100).toFixed(0)} a5:${r25.above5} run:${r25.maxRun}` : ''));
    }
  }
  console.log(`engine-evaluated ${evaluated}, full passes ${passes}`);
}
