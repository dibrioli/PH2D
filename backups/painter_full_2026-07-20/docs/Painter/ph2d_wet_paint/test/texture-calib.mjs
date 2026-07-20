// Calibration reporter for the DEFAULT bristle texture: prints every binding
// number of spec sections 7 (deposit integral + uniformity), 18.11 (stroke
// budgets at r=12 and r=25) and 18.12 (lane structure). Thin wrapper around
// the search harness's measurement functions.
// Run: node test/texture-calib.mjs

import { buildBristleTexture } from '../js/engine/brush.js';
import { texStats, strokeCheck } from './texture-sweep.mjs';

const tex = buildBristleTexture({ felt: 0.01, bristleCount: 950, bristleStrength: 1, bristleSize: 1 });
const s = texStats(tex);
console.log('tile     | mean', s.mean.toFixed(4), '(~0.02) | %>0.04', s.p4.toFixed(1), '(~6%)');
console.log('integral | origin', s.integral.toFixed(2), '(4.0 +-15%) | clearing', s.clearing,
  '(13-23) | 3x3 mean', s.gridMean.toFixed(2), '(4.0 +-25%) | min', s.gridMin.toFixed(2), '(>= 1.5)');
for (const [slider, r, mT, wT, mTol, wTol, c0, c1] of [
  [10 / 33, 12, 235000, 245, 0.12, 0.12, 0.20, 0.27],
  [0.7, 25, 890000, 835, 0.15, 0.15, 0.33, 0.45],
]) {
  const b = strokeCheck(tex, slider, r);
  const ok =
    Math.abs(b.mass - mT) <= mT * mTol && Math.abs(b.water - wT) <= wT * wTol &&
    b.cov >= c0 && b.cov <= c1 &&
    b.upperShare >= 0.35 && b.upperShare <= 0.65 &&
    b.above5 >= (r === 12 ? 7 : 20) && b.maxRun <= (r === 12 ? 6 : 12);
  console.log(`r=${r}${ok ? ' OK ' : ' !! '}| mass ${Math.round(b.mass)} (${mT} +-${mTol * 100}%)` +
    ` | water ${b.water.toFixed(1)} (${wT} +-${wTol * 100}%) | cov ${(b.cov * 100).toFixed(1)}% (${c0 * 100}-${c1 * 100}%)` +
    ` | upper ${(b.upperShare * 100).toFixed(1)}% | rows>5% ${b.above5} | max gap ${b.maxRun}`);
}
