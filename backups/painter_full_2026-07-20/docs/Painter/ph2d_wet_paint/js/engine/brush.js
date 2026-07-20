// Brush stamp (spec section 7): radialFalloff(d, hardness) x bristleTexture.
//
// Porosity is load-bearing. The bristle texture is ~94% near-zero felt with
// ~950 tiny bright tips, so a painted blob is a SIEVE of wet channels, not a
// uniform film: the solver can nucleate discrete rivulets at its edge and the
// reservoir keeps its pigment. The texture is anchored to the dab center in
// canvas pixels (1 texel = 1 px at every brush size), so grain has constant
// physical size and the tips streak with the stroke.

import { makeRng } from './rng.js';

export const BRISTLE_SIZE = 128;
const BMASK = BRISTLE_SIZE - 1;
const BRISTLE_SEED = 0x7a5b0010;

// Tip-distribution calibration (spec section 7, deposit-integral bullet).
// The paper gate integrates (min(1, falloff*tex*intensity) - gateTerm)+, so
// what matters is how much tip AREA sits in the ~0.12-0.45 texel band: a
// plateau-topped cross-section holds that band instead of spending it on a
// soft sub-gate skirt. TWO tip populations, BOTH placed uniformly at random
// over the whole tile (the uniformity clause is binding: any deliberate,
// region-based placement makes the stroke profile a function of the brush
// radius - one-sided or hollow strokes):
//  - FINE tips (the sieve): sub-gate grain, the visual felt between lanes;
//  - CORE tips (a fraction of the count): the gate-clearing bristle lanes.
//    Many medium cores rather than few large ones keeps the per-disc lane
//    density statistically uniform, so every radius finds the same streaky
//    structure anywhere in its footprint (spec section 7 uniformity + the
//    section 18.12 lane-structure bounds).
// Tuned so the defaults hit: tile mean ~0.02, ~6-7% of texels > 0.04, origin
// deposit integral ~4.0 with 13-23 gate-clearing texels, 3x3-grid integral
// mean 4.0 +-25% with min >= 1.5, and the section 18.11 budgets at both
// r=12 and r=25 (see test/texture-sweep.mjs for the search harness).
const FINE_RADIUS_MIN = 0.4;    // radius = (MIN + rng*SPAN) * tipSize knob
const FINE_RADIUS_SPAN = 0.4;
const FINE_STRENGTH_MIN = 0.03; // strength = (MIN + rng*SPAN) * tipStrength knob
const FINE_STRENGTH_SPAN = 0.05; // sub-gate grain: visual tooth, no deposit
const CORE_FRACTION = 0.26;     // share of the bristle count drawn as core tips
const CORE_RADIUS_MIN = 0.9;
const CORE_RADIUS_SPAN = 0.6;
const CORE_STRENGTH_MIN = 0.16;
const CORE_STRENGTH_SPAN = 0.16;
const TIP_PLATEAU = 0.55;       // fraction of the radius at full strength

/**
 * Build the 128x128 bristle tile from the paint-group knobs.
 * Deterministic: same knobs -> same texture.
 *  - felt: near-zero noise floor between the hairs, mean = felt knob.
 *  - tips: plateau-topped round dots (fine sieve + core blobs), max-blended.
 * `calib` (optional) overrides the distribution constants - used only by the
 * calibration harness in test/; the app always builds with the defaults.
 */
export function buildBristleTexture(knobs, calib = null) {
  const C = calib || {
    fineRadiusMin: FINE_RADIUS_MIN, fineRadiusSpan: FINE_RADIUS_SPAN,
    fineStrengthMin: FINE_STRENGTH_MIN, fineStrengthSpan: FINE_STRENGTH_SPAN,
    coreFraction: CORE_FRACTION, coreRadiusMin: CORE_RADIUS_MIN,
    coreRadiusSpan: CORE_RADIUS_SPAN, coreStrengthMin: CORE_STRENGTH_MIN,
    coreStrengthSpan: CORE_STRENGTH_SPAN, plateau: TIP_PLATEAU, seed: BRISTLE_SEED,
  };
  const felt = knobs.felt ?? 0.01;
  const count = Math.max(0, Math.round(knobs.bristleCount ?? 950));
  const tipStrength = knobs.bristleStrength ?? 1;
  const tipSize = knobs.bristleSize ?? 1;
  const rng = makeRng(C.seed);
  const t = new Float32Array(BRISTLE_SIZE * BRISTLE_SIZE);
  for (let i = 0; i < t.length; i++) t[i] = rng() * 2 * felt; // mean = felt
  const invFall = 1 / (1 - C.plateau);
  const coreCount = Math.round(count * C.coreFraction);
  for (let b = 0; b < count; b++) {
    const rr = rng(), rs = rng();
    const isCore = b < coreCount;
    const radius = (isCore
      ? C.coreRadiusMin + rr * C.coreRadiusSpan
      : C.fineRadiusMin + rr * C.fineRadiusSpan) * tipSize;
    const strength = (isCore
      ? C.coreStrengthMin + rs * C.coreStrengthSpan
      : C.fineStrengthMin + rs * C.fineStrengthSpan) * tipStrength;
    // Uniform random placement for EVERY tip - no reserved regions, no
    // re-draws (spec section 7 uniformity clause).
    const cx = rng() * BRISTLE_SIZE, cy = rng() * BRISTLE_SIZE;
    const reach = Math.ceil(radius);
    const inv = 1 / (radius + 0.0001);
    const x0 = Math.floor(cx) - reach, x1 = Math.floor(cx) + reach;
    const y0 = Math.floor(cy) - reach, y1 = Math.floor(cy) + reach;
    for (let yy = y0; yy <= y1; yy++) {
      for (let xx = x0; xx <= x1; xx++) {
        const dx = xx - cx, dy = yy - cy;
        const d = Math.sqrt(dx * dx + dy * dy) * inv;
        if (d >= 1) continue;
        // Plateau top with a smoothstep shoulder: round, but not skirt-heavy.
        let w = 1;
        if (d > C.plateau) {
          const u = (d - C.plateau) * invFall;
          w = 1 - u * u * (3 - 2 * u);
        }
        const v = strength * w;
        const idx = (yy & BMASK) * BRISTLE_SIZE + (xx & BMASK);
        if (v > t[idx]) t[idx] = v; // max-blend keeps tips crisp
      }
    }
  }
  return t;
}

/** Nearest-texel wrap sample, offsets in canvas px from the dab center. */
export function sampleBristle(tex, dxPx, dyPx) {
  return tex[((Math.round(dyPx)) & BMASK) * BRISTLE_SIZE + ((Math.round(dxPx)) & BMASK)];
}

/** Radial falloff: a = (1-d)*hardness; 1 when a>=1, else smoothstep(a). */
export function radialFalloff(d, hardness) {
  const a = (1 - d) * hardness;
  if (a >= 1) return 1;
  if (a <= 0) return 0;
  return a * a * (3 - 2 * a);
}

// Ellipse axes (along-stroke / across-stroke), in units of the brush radius.
const SHAPE_AXES = { round: null, flat: [1, 0.35], fan: [1.4, 0.5] };

/**
 * Iterate every stamp pixel of one dab, restricted to the brushable area
 * [2..W-1] x [2..H-1]. Calls cb(i, x, y, falloff, texture) - the caller
 * multiplies them (plus its own intensity/clamps); they stay separate because
 * the wet-edge-softening extension keys on the falloff alone.
 *
 * shape 'flat'/'fan' is an ellipse in the stroke frame (dirX/dirY = stroke
 * direction unit vector); with no direction yet it falls back to the disc.
 * Texture coords for shaped brushes are the ROTATED px offsets, so the grain
 * rides the stroke frame.
 */
export function forEachStampPixel(grid, tex, cx, cy, r, hardness, shape, dirX, dirY, cb) {
  const { S, W, H } = grid;
  const axes = SHAPE_AXES[shape] || null;
  const hasDir = axes && (dirX !== 0 || dirY !== 0);
  const reach = hasDir ? r * axes[0] : r;
  const x0 = Math.max(2, Math.ceil(cx - reach));
  const x1 = Math.min(W - 1, Math.floor(cx + reach));
  const y0 = Math.max(2, Math.ceil(cy - reach));
  const y1 = Math.min(H - 1, Math.floor(cy + reach));
  if (x1 < x0 || y1 < y0) return;
  const invR = 1 / r;
  let invA = 0, invB = 0;
  if (hasDir) { invA = 1 / (axes[0] * r); invB = 1 / (axes[1] * r); }
  for (let y = y0; y <= y1; y++) {
    let i = x0 + y * S;
    for (let x = x0; x <= x1; x++, i++) {
      const dx = x - cx, dy = y - cy;
      let d2, tx, ty;
      if (hasDir) {
        const along = dx * dirX + dy * dirY;
        const across = -dx * dirY + dy * dirX;
        const na = along * invA, nb = across * invB;
        d2 = na * na + nb * nb;
        tx = along; ty = across;
      } else {
        const nx = dx * invR, ny = dy * invR;
        d2 = nx * nx + ny * ny;
        tx = dx; ty = dy;
      }
      if (d2 >= 1) continue; // reject on squared distance before any sqrt
      const fall = radialFalloff(Math.sqrt(d2), hardness);
      if (fall <= 0) continue;
      const texv = sampleBristle(tex, tx, ty);
      if (texv > 0) cb(i, x, y, fall, texv);
    }
  }
}
