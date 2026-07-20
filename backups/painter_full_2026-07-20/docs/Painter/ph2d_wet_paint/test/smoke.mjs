// PH2D Wet Paint - acceptance smoke test (spec section 18).
// Run from the app folder:  node test/smoke.mjs   (exit 0 = all green)
// Engine-only, all knobs at defaults, everything deterministic.

import {
  createEngine, activeGrid, pointerDown, pointerFrame, pointerUp, releaseFrame,
  stepSimulation, simShouldRun,
} from '../js/engine/painter.js';
import { gatherParams } from '../js/engine/sim.js';
import { advect } from '../js/engine/solver.js';
import { expandBBox } from '../js/engine/grid.js';
import { generatePaperTile, measureTileStats, PAPER_PRESETS } from '../js/engine/paper.js';
import { createTuning, KNOB_DEFS } from '../js/engine/tuning.js';
import { buildBristleTexture, radialFalloff, BRISTLE_SIZE } from '../js/engine/brush.js';

let failures = 0;
let current = '';

function check(cond, msg) {
  if (cond) return;
  failures++;
  console.error(`  FAIL [${current}] ${msg}`);
}

function section(name, fn) {
  current = name;
  const t0 = performance.now();
  try {
    fn();
    console.log(`${failures === 0 ? 'ok' : '..'}  ${name}  (${(performance.now() - t0).toFixed(0)} ms)`);
  } catch (err) {
    failures++;
    console.error(`  FAIL [${name}] threw: ${err.stack || err}`);
  }
}

function hasNaN(arr) {
  for (let i = 0; i < arr.length; i++) if (Number.isNaN(arr[i])) return true;
  return false;
}

function sweepNaN(g, label) {
  for (const key of ['film', 'susp', 'suspR', 'suspG', 'suspB', 'sett', 'settR', 'settG', 'settB', 'velX', 'velY']) {
    check(!hasNaN(g[key]), `${label}: NaN found in ${key}`);
  }
}

/** Drive a straight scripted stroke at a fixed px/frame speed (40 Hz clock). */
function driveStroke(engine, x0, y0, x1, y1, pxPerFrame, { simDuringRelease = 0 } = {}) {
  const dx = x1 - x0, dy = y1 - y0;
  const len = Math.sqrt(dx * dx + dy * dy);
  const ux = len > 0 ? dx / len : 0, uy = len > 0 ? dy / len : 0;
  pointerDown(engine, x0, y0);
  let travelled = 0, x = x0, y = y0;
  while (travelled < len) {
    travelled = Math.min(len, travelled + pxPerFrame);
    x = x0 + ux * travelled; y = y0 + uy * travelled;
    pointerFrame(engine, x, y);
    if (simShouldRun(engine)) stepSimulation(engine);
  }
  pointerUp(engine);
  while (releaseFrame(engine, x, y)) { /* release tail ticks at 40 Hz */ }
  for (let i = 0; i < simDuringRelease; i++) stepSimulation(engine);
}

// ---------------------------------------------------------------------------
// 1 + 2: pressure fade-in and porous deposit (shared 260-px stroke)
// ---------------------------------------------------------------------------

const engineA = createEngine();
const pressures = [];
engineA.onDab = (b) => pressures.push(b);
section('1. pressure fade-in', () => {
  driveStroke(engineA, 100, 200, 360, 200, 4);
  check(pressures.length > 30, `expected many dabs, got ${pressures.length}`);
  check(pressures[0] === 0, `first dab pressure = ${pressures[0]}, expected 0`);
  const n = pressures.length;
  const steadySlice = pressures.slice(n - 20, n - 5); // last 15 minus tail 5
  const steady = steadySlice.reduce((a, v) => a + v, 0) / steadySlice.length;
  const early = pressures.slice(1, 6);
  const earlyMean = early.reduce((a, v) => a + v, 0) / early.length;
  check(earlyMean < 0.7 * steady, `early mean ${earlyMean.toFixed(2)} !< 70% of steady ${steady.toFixed(2)}`);
  const maxP = Math.max(...pressures);
  check(maxP <= 7.34, `max pressure ${maxP.toFixed(3)} > 7.34`);
});

section('2. porous deposit', () => {
  const g = activeGrid(engineA);
  let band = 0, holding = 0, total = 0;
  for (let y = 200 - 12; y <= 200 + 12; y++) {
    for (let x = 100; x <= 360; x++) {
      const i = x + y * g.S;
      band++;
      const m = g.susp[i] + g.sett[i];
      if (m > 0) holding++;
      total += m;
    }
  }
  check(total > 0, 'no pigment deposited at all');
  const frac = holding / band;
  check(frac < 0.85, `${(frac * 100).toFixed(1)}% of band cells hold pigment (must be < 85%)`);
  sweepNaN(g, 'post-stroke');
});

// ---------------------------------------------------------------------------
// 3: wet & dry tools
// ---------------------------------------------------------------------------

section('3. wet & dry tools', () => {
  const e = createEngine();
  const g = activeGrid(e);
  const center = 150 + 150 * g.S;
  e.tool = 'wet';
  driveStroke(e, 100, 150, 200, 150, 4);
  const first = g.film[center];
  check(first > 0 && first < 0.2, `one wet pass film = ${first} (need 0 < f < 0.2)`);
  check(g.wet[center] > 0, 'wetness byte at line center is 0 after wet pass');
  for (let k = 0; k < 30; k++) driveStroke(e, 100, 150, 200, 150, 4);
  const after = g.film[center];
  check(after > 5 * first, `31 wet passes film ${after} !> 5x first ${first}`);
  e.tool = 'dry';
  driveStroke(e, 100, 150, 200, 150, 4);
  check(g.wet[center] === 0, `dry pass left wetness ${g.wet[center]} (must seal to 0)`);
  check(g.film[center] >= 0.001, `dry pass film ${g.film[center]} < 0.001 floor`);
});

// ---------------------------------------------------------------------------
// 4: blow
// ---------------------------------------------------------------------------

section('4. blow', () => {
  const e = createEngine();
  const g = activeGrid(e);
  for (let y = 140; y <= 180; y++) {
    for (let x = 140; x <= 180; x++) {
      const i = x + y * g.S;
      g.film[i] = 3;
      g.wet[i] = Math.round(0.8 * 255);
    }
  }
  expandBBox(g, 140, 140, 180, 180);
  e.tool = 'blow';
  driveStroke(e, 130, 160, 190, 160, 3); // sim runs live during blow
  let maxVX = 0;
  for (let y = 2; y <= g.H - 1; y++) {
    for (let x = 2; x <= g.W - 1; x++) {
      const v = Math.abs(g.velX[x + y * g.S]);
      if (v > maxVX) maxVX = v;
    }
  }
  check(maxVX > 0 && maxVX < 0.31, `blow max |vx| = ${maxVX.toFixed(4)} (need 0 < v < 0.31)`);
  sweepNaN(g, 'post-blow');
});

// ---------------------------------------------------------------------------
// 5: advection semantics
// ---------------------------------------------------------------------------

section('5. advection semantics', () => {
  // (a) color replacement: heavy cell receiving from a light colored neighbour.
  const e = createEngine({ width: 60, height: 60 });
  const g = activeGrid(e);
  const P = gatherParams(e.sim);
  const dst = 20 + 20 * g.S, src = 19 + 20 * g.S;
  g.susp[dst] = 2500; g.suspR[dst] = 255; g.suspG[dst] = 0; g.suspB[dst] = 0;
  g.susp[src] = 300; g.suspR[src] = 0; g.suspG[src] = 0; g.suspB[src] = 255;
  g.active[dst] = 1; g.active[src] = 1;
  expandBBox(g, 18, 18, 22, 22);
  g.flowX[dst] = 1; // back-trace lands exactly on the neighbour
  advect(g, P, 0, 0);
  check(g.suspB[dst] === 255 && g.suspR[dst] === 0,
    `destination color not replaced: rgb(${g.suspR[dst]},${g.suspG[dst]},${g.suspB[dst]})`);
  check(g.susp[dst] > 2500, 'destination mass did not grow');
  // (b) no water cap during advection.
  const e2 = createEngine({ width: 60, height: 60 });
  const g2 = activeGrid(e2);
  const P2 = gatherParams(e2.sim);
  const d2 = 30 + 30 * g2.S, s2 = 29 + 30 * g2.S;
  g2.film[d2] = 8; g2.film[s2] = 8;
  g2.active[d2] = 1; g2.active[s2] = 1;
  expandBBox(g2, 28, 28, 32, 32);
  g2.flowX[d2] = 1;
  advect(g2, P2, 0, 0);
  check(g2.film[d2] > 8, `destination film ${g2.film[d2]} did not exceed the cap 8`);
});

// ---------------------------------------------------------------------------
// 6: paper statistics
// ---------------------------------------------------------------------------

section('6. paper statistics', () => {
  for (const name of ['cold', 'rough', 'hot']) {
    const p = PAPER_PRESETS[name];
    const s = measureTileStats(generatePaperTile(name, 0));
    check(Math.abs(s.mean - p.mean) < 0.02, `${name} mean ${s.mean.toFixed(3)} vs ${p.mean}`);
    check(Math.abs(s.std - p.std) < 0.04, `${name} std ${s.std.toFixed(3)} vs ${p.std}`);
    check(Math.abs(s.grad - p.grad) / p.grad < 0.18, `${name} grad ${s.grad.toFixed(4)} vs ${p.grad}`);
  }
});

// ---------------------------------------------------------------------------
// 7: blend re-mixes dry paint
// ---------------------------------------------------------------------------

section('7. blend re-mixes dry paint', () => {
  const e = createEngine();
  const g = activeGrid(e);
  for (let y = 100; y <= 140; y++) {
    for (let x = 100; x <= 140; x++) {
      const i = x + y * g.S;
      g.sett[i] = 900; g.settR[i] = 200; g.settG[i] = 40; g.settB[i] = 30;
    }
  }
  const sumOutside = () => {
    let s = 0;
    for (let y = 90; y <= 150; y++) {
      for (let x = 141; x <= 200; x++) s += g.sett[x + y * g.S];
    }
    return s;
  };
  const before = sumOutside();
  e.tool = 'blend';
  driveStroke(e, 120, 120, 170, 120, 4); // across the patch's right edge
  const grown = sumOutside() - before;
  check(grown > 100, `settled mass beyond the patch grew by ${grown.toFixed(1)} (need > 100)`);
  sweepNaN(g, 'post-blend');
});

// ---------------------------------------------------------------------------
// 8: tuning registry
// ---------------------------------------------------------------------------

section('8. tuning registry', () => {
  check(KNOB_DEFS.length >= 39, `only ${KNOB_DEFS.length} knobs`);
  const t = createTuning();
  t.set('leveling', 1.7);
  t.set('gravity', 0.03);
  t.set('brake', 0.1);
  t.resetGroup('physics');
  check(t.get('leveling') === 0.5 && t.get('gravity') === 0.005 && t.get('brake') === 1.5,
    'group reset did not restore defaults');
  t.set('waterCap', 999); // free numeric input beyond the slider range
  check(t.get('waterCap') === 999, 'out-of-range value rejected');
  t.set('waterCap', NaN);
  check(t.get('waterCap') === 8, `NaN input fell back to ${t.get('waterCap')}, expected default 8`);
});

// ---------------------------------------------------------------------------
// 9: drip + stability
// ---------------------------------------------------------------------------

function dripSetup() {
  const e = createEngine({ width: 80, height: 300 });
  const g = activeGrid(e);
  for (let y = 20; y <= 39; y++) {
    for (let x = 30; x <= 49; x++) {
      const i = x + y * g.S;
      g.film[i] = 8; g.susp[i] = 800; g.wet[i] = 255;
      g.suspR[i] = 30; g.suspG[i] = 60; g.suspB[i] = 150;
    }
  }
  expandBBox(g, 30, 20, 49, 39);
  return e;
}

function wetFrontMaxY(g) {
  let maxY = 0;
  for (let y = 1; y <= g.H; y++) {
    for (let x = 1; x <= g.W; x++) {
      if (g.film[x + y * g.S] > 0.1) { maxY = y; break; }
    }
  }
  return maxY;
}

section('9. drip + stability', () => {
  const e = dripSetup();
  e.sim.gravityOverride = { x: 0, y: 1.5 };
  for (let i = 0; i < 240; i++) stepSimulation(e);
  const g = activeGrid(e);
  const descent = wetFrontMaxY(g) - 39;
  check(descent >= 40, `gravity drip descended ${descent} rows (need >= 40)`);
  const e0 = dripSetup();
  e0.sim.gravityOverride = { x: 0, y: 0 };
  for (let i = 0; i < 240; i++) stepSimulation(e0);
  const spread = wetFrontMaxY(activeGrid(e0)) - 39;
  check(spread <= 8, `zero-gravity front moved ${spread} rows (need <= 8)`);
  for (let i = 0; i < 120; i++) stepSimulation(e);
  sweepNaN(g, 'post-drip');
});

// ---------------------------------------------------------------------------
// 10: extension neutrality
// ---------------------------------------------------------------------------

function neutralityRun(bypass) {
  const e = createEngine({ width: 300, height: 200 });
  e.sim.extBypass = bypass;
  driveStroke(e, 60, 100, 240, 100, 4);
  for (let i = 0; i < 60; i++) stepSimulation(e);
  return activeGrid(e);
}

section('10. extension neutrality', () => {
  const gOn = neutralityRun(false); // extension code paths run (neutral knobs)
  const gOff = neutralityRun(true); // extension code paths force-skipped
  for (const key of ['film', 'susp', 'sett', 'velX', 'velY']) {
    const a = gOn[key], b = gOff[key];
    let diff = -1;
    for (let i = 0; i < a.length; i++) {
      if (a[i] !== b[i]) { diff = i; break; }
    }
    check(diff === -1, `${key} differs at index ${diff}: ${a[diff]} vs ${b[diff]}`);
  }
});

// ---------------------------------------------------------------------------
// 11 + 12: stroke budget at TWO radii + lane structure (binding calibration)
// ---------------------------------------------------------------------------

/** The section-18.11 scripted stroke at a given brush size; measures budgets
 *  and the section-18.12 lane metrics. */
function budgetStroke(sizeSlider, r) {
  const e = createEngine({ width: 300, height: 100 });
  e.sliders.size = sizeSlider; // other sliders default (pressure 0.7, water 1.0)
  driveStroke(e, 20, 50, 280, 50, 4); // 260 px at 4 px/frame, sim paused, no tail
  const g = activeGrid(e);
  let mass = 0, water = 0, band = 0, touched = 0;
  for (let y = 1; y <= g.H; y++) {
    for (let x = 1; x <= g.W; x++) {
      const i = x + y * g.S;
      mass += g.susp[i] + g.sett[i];
      water += g.film[i];
      if (Math.abs(y - 50) <= r && x >= 20 && x <= 280) {
        band++;
        if (g.susp[i] + g.sett[i] > 0) touched++;
      }
    }
  }
  // Lane structure: suspended mass per row over the central 220 columns.
  const rows = [];
  for (let dy = -r; dy <= r; dy++) {
    let s = 0;
    for (let x = 40; x <= 259; x++) s += g.susp[x + (50 + dy) * g.S];
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
    grid: g, mass, water, cov: touched / band,
    upperShare: upper / (upper + lower), above5, maxRun,
  };
}

const budget12 = budgetStroke(10 / 33, 12); // radius = 2 + slider*33 = 12
const budget25 = budgetStroke(0.7, 25);     // the boot radius

section('11. stroke budget at two radii', () => {
  const b12 = budget12, b25 = budget25;
  check(Math.abs(b12.mass - 235000) <= 235000 * 0.12, `r12 mass ${Math.round(b12.mass)} outside 235000 +-12%`);
  check(Math.abs(b12.water - 245) <= 245 * 0.12, `r12 water ${b12.water.toFixed(1)} outside 245 +-12%`);
  check(b12.cov >= 0.20 && b12.cov <= 0.27, `r12 coverage ${(b12.cov * 100).toFixed(1)}% outside 20-27%`);
  check(Math.abs(b25.mass - 890000) <= 890000 * 0.15, `r25 mass ${Math.round(b25.mass)} outside 890000 +-15%`);
  check(Math.abs(b25.water - 835) <= 835 * 0.15, `r25 water ${b25.water.toFixed(1)} outside 835 +-15%`);
  check(b25.cov >= 0.33 && b25.cov <= 0.45, `r25 coverage ${(b25.cov * 100).toFixed(1)}% outside 33-45%`);
  console.log(`    r12: mass ${Math.round(b12.mass)} | water ${b12.water.toFixed(1)} | coverage ${(b12.cov * 100).toFixed(1)}%`);
  console.log(`    r25: mass ${Math.round(b25.mass)} | water ${b25.water.toFixed(1)} | coverage ${(b25.cov * 100).toFixed(1)}%`);
  sweepNaN(budget12.grid, 'post-budget-r12');
  sweepNaN(budget25.grid, 'post-budget-r25');
  // Section 7 deposit integral + its uniformity clause.
  const tex = buildBristleTexture({ felt: 0.01, bristleCount: 950, bristleStrength: 1, bristleSize: 1 });
  const integralAt = (cx, cy) => {
    let integral = 0, clearing = 0;
    for (let dy = -12; dy <= 12; dy++) {
      for (let dx = -12; dx <= 12; dx++) {
        const d2 = (dx * dx + dy * dy) / 144;
        if (d2 >= 1) continue;
        const f = radialFalloff(Math.sqrt(d2), 9);
        const t = tex[((cy + dy) & (BRISTLE_SIZE - 1)) * BRISTLE_SIZE + ((cx + dx) & (BRISTLE_SIZE - 1))];
        const v = Math.min(1, f * t * 2.415) - 0.3;
        if (v > 0) { integral += v; clearing++; }
      }
    }
    return { integral, clearing };
  };
  const origin = integralAt(0, 0);
  check(Math.abs(origin.integral - 4.0) <= 4.0 * 0.15, `origin deposit integral ${origin.integral.toFixed(2)} outside 4.0 +-15%`);
  check(origin.clearing >= 13 && origin.clearing <= 23, `origin clearing texels ${origin.clearing} outside 13-23`);
  let gSum = 0, gMin = Infinity;
  for (const cy of [0, 43, 85]) for (const cx of [0, 43, 85]) {
    const v = integralAt(cx, cy).integral;
    gSum += v;
    if (v < gMin) gMin = v;
  }
  const gMean = gSum / 9;
  check(Math.abs(gMean - 4.0) <= 4.0 * 0.25, `grid integral mean ${gMean.toFixed(2)} outside 4.0 +-25%`);
  check(gMin >= 1.5, `grid integral min ${gMin.toFixed(2)} < 1.5 (uniformity)`);
  console.log(`    integral: origin ${origin.integral.toFixed(2)} (${origin.clearing} texels) | 3x3 grid mean ${gMean.toFixed(2)} min ${gMin.toFixed(2)}`);
});

section('12. lane structure at two radii', () => {
  const b12 = budget12, b25 = budget25;
  check(b12.upperShare >= 0.35 && b12.upperShare <= 0.65,
    `r12 upper half-band carries ${(b12.upperShare * 100).toFixed(1)}% of off-path mass (need 35-65%)`);
  check(b25.upperShare >= 0.35 && b25.upperShare <= 0.65,
    `r25 upper half-band carries ${(b25.upperShare * 100).toFixed(1)}% of off-path mass (need 35-65%)`);
  check(b12.above5 >= 7, `r12 rows >5% of max: ${b12.above5} of 25 (need >= 7)`);
  check(b25.above5 >= 20, `r25 rows >5% of max: ${b25.above5} of 51 (need >= 20)`);
  check(b12.maxRun <= 6, `r12 longest <2% run: ${b12.maxRun} (need <= 6)`);
  check(b25.maxRun <= 12, `r25 longest <2% run: ${b25.maxRun} (need <= 12)`);
  console.log(`    r12: upper ${(b12.upperShare * 100).toFixed(1)}% | rows>5% ${b12.above5}/25 | max gap ${b12.maxRun}`);
  console.log(`    r25: upper ${(b25.upperShare * 100).toFixed(1)}% | rows>5% ${b25.above5}/51 | max gap ${b25.maxRun}`);
});

// ---------------------------------------------------------------------------
// Perf sanity on the full 900x450 grid.
// (a) A representative scripted session: a long juicy stroke under tilt,
//     then 240 sim steps - must run WELL under 1 s per 100 steps.
// (b) A worst-case flood (a 600x180 standing wash, ~110k wet cells) as an
//     upper-bound stability guard.
// ---------------------------------------------------------------------------

section('perf sanity (900x450, scripted stroke + 240 steps)', () => {
  const e = createEngine();
  e.sliders.water = 1.0;
  driveStroke(e, 150, 120, 700, 160, 5);
  const t0 = performance.now();
  for (let i = 0; i < 240; i++) stepSimulation(e);
  const dt = performance.now() - t0;
  const per100 = (dt / 240) * 100;
  console.log(`    240 steps in ${dt.toFixed(0)} ms  (~${per100.toFixed(0)} ms / 100 steps)`);
  check(per100 < 500, `${per100.toFixed(0)} ms per 100 steps (must be well under 1000)`);
});

section('perf guard (900x450 full flood, 240 steps)', () => {
  const e = createEngine();
  const g = activeGrid(e);
  for (let y = 40; y <= 220; y++) {
    for (let x = 100; x <= 700; x++) {
      const i = x + y * g.S;
      g.film[i] = 6; g.susp[i] = 500; g.wet[i] = 200;
      g.suspR[i] = 60; g.suspG[i] = 90; g.suspB[i] = 160;
    }
  }
  expandBBox(g, 100, 40, 700, 220);
  const t0 = performance.now();
  for (let i = 0; i < 240; i++) stepSimulation(e);
  const dt = performance.now() - t0;
  const per100 = (dt / 240) * 100;
  console.log(`    240 steps in ${dt.toFixed(0)} ms  (~${per100.toFixed(0)} ms / 100 steps)`);
  check(per100 < 1500, `flood: ${per100.toFixed(0)} ms per 100 steps (upper-bound guard 1500)`);
  sweepNaN(g, 'post-flood');
});

// ---------------------------------------------------------------------------

if (failures > 0) {
  console.error(`\n${failures} check(s) FAILED`);
  process.exit(1);
}
console.log('\nAll acceptance checks passed.');
