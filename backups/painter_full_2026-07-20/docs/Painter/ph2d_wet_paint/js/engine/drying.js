// Drying / settling / re-wetting (spec section 6.2) + fast dry (section 12).
//
// This pass is where the classic watercolor rim comes from: cells at the
// edge of a wash (few suspended neighbours => low edge factor e) evaporate
// up to ~26x faster than the interior, so pigment carried there by the flow
// settles first and darkens the boundary. Once a cell has settled >= 1000
// the edge boost stops - saturated washes stop over-darkening.

import { alphaOfMass } from './opacity.js';
import { settleComposite, emptyBBox } from './grid.js';
import { rebuildActiveRegion } from './solver.js';

function clamp01(v) { return v < 0 ? 0 : v > 1 ? 1 : v; }

/** Staining extension: bidirectional lift multiplier, exactly 1 at 0.5. */
function stainingMultiplier(s) {
  return s < 0.5 ? 1 + (0.5 - s) * 14 : (1 - s) * 2; // 0 -> 8x, 1 -> 0x
}

/**
 * One drying pass over every bbox cell holding water or suspended pigment
 * (deliberately NOT filtered by the active mask - paint dries everywhere).
 * evapBase is the cadence-adaptive scale (or 1000 for a force-dry pass);
 * rewetBase the cadence-adaptive lift floor.
 */
export function dryingPass(g, P, evapBase, rewetBase, extBypass) {
  const { S, film, susp, sett, paper } = g;
  const out = [0, 0, 0];
  const mixColor = P.mixColor;
  for (let y = g.by0; y <= g.by1; y++) {
    let i = g.bx0 + y * S;
    for (let x = g.bx0; x <= g.bx1; x++, i++) {
      const f = film[i];
      const sMass = susp[i];
      if (f <= 0 && sMass <= 0) continue;

      // Edge factor: fraction of the 3x3 neighbourhood carrying pigment.
      // A full block (9/9) reads e=1 (no boost); a rim cell reads e<1.
      let e = 1;
      if (f > 0 && sett[i] < 1000) {
        let count = 0;
        for (let dy = -S; dy <= S; dy += S) {
          const b = i + dy;
          if (susp[b - 1] > 10) count++;
          if (susp[b] > 10) count++;
          if (susp[b + 1] > 10) count++;
        }
        e = count === 9 ? 1 : count / 9;
      }

      // Evaporate: retention leak + edge-boosted linear loss.
      let newFilm = f * P.retention - evapBase * ((1 - e) * P.edgeDarkening + P.baseEvaporation);
      if (newFilm < 0.0001) newFilm = 0;
      const lost = f > 0 ? 1 - clamp01(newFilm / f) : 1;
      film[i] = newFilm;

      // Settle: the fraction of water lost carries the same fraction of
      // suspended pigment onto the paper (opacity-composited color).
      if (lost > 0 && sMass > 0) {
        let dm = sMass * lost;
        if (!extBypass) {
          // Physical granulation (extension): settle biased toward valleys.
          let bias = 1 + P.extGranulation * 0.6 * (0.5 - paper[i]) * 2;
          if (bias < 0.3) bias = 0.3; else if (bias > 1.7) bias = 1.7;
          dm *= bias;
          if (dm > sMass) dm = sMass;
        }
        settleComposite(g, i, dm, mixColor, out);
        sett[i] += dm;
        const rem = sMass - dm;
        susp[i] = rem < 0 ? 0 : rem;
      }

      // Re-wet: standing water lifts a little settled pigment back into
      // suspension; the lift grows with the EXCESS water over what the
      // suspended pigment already occupies.
      const st = sett[i];
      if (film[i] > 0 && st > 0) {
        const excess = Math.max(0, film[i] - alphaOfMass(susp[i]));
        let b = rewetBase * (1 + excess * 50);
        if (!extBypass) b *= stainingMultiplier(P.extStaining);
        b = clamp01(b);
        if (b > 0) {
          const lift = st * b;
          const aIn = alphaOfMass(st) * b;
          const u = alphaOfMass(susp[i]) * (1 - aIn);
          if (u + aIn > 0) {
            const w = aIn / (u + aIn);
            mixColor(g.suspR[i], g.suspG[i], g.suspB[i], g.settR[i], g.settG[i], g.settB[i], w, out);
            g.suspR[i] = out[0]; g.suspG[i] = out[1]; g.suspB[i] = out[2];
          } else {
            g.suspR[i] = g.settR[i]; g.suspG[i] = g.settG[i]; g.suspB[i] = g.settB[i];
          }
          susp[i] += lift;
          sett[i] = st - lift;
        }
      }
    }
  }
}

/**
 * Fast dry (spec section 12): repeatedly halve the water and run force-dry
 * passes until no fluid remains (guard <= 40 loops). No advection - it cannot
 * stall; edge darkening still forms because the settle step runs each loop.
 */
export function fastDry(g, P, rewetBase, extBypass) {
  const { S } = g;
  for (let guard = 0; guard < 40 && g.hasFluid; guard++) {
    for (let y = g.by0; y <= g.by1; y++) {
      let i = g.bx0 + y * S;
      for (let x = g.bx0; x <= g.bx1; x++, i++) g.film[i] *= 0.5;
    }
    dryingPass(g, P, 1000, rewetBase, extBypass);
    rebuildActiveRegion(g);
  }
  if (!g.hasFluid) emptyBBox(g);
}
