// Fluid solver passes (spec sections 6.1, 6.3-6.7, plus the flow-side gated
// extensions of section 17). All passes iterate only the active bounding box.
//
// The velocity design in one paragraph: gravity accumulates in the PERSISTENT
// field (velX/velY); every frame the TRANSIENT flow (flowX/flowY) is rebuilt
// from it - with leveling + capillary + the look-ahead absorbency brake on
// one frame in four, and as a cheap unbraked smoothing on the other three.
// Mass then advects along the transient flow. Because the brake only bites
// 1-in-4, a drip that would stall under constant braking keeps advancing on
// the free frames: that asymmetry is why drips run.

import { wetByteFromPaper, emptyBBox } from './grid.js';
import { hash2Signed } from './rng.js';
import { kmWeightedMeanColor } from './colorops.js';

// Scratch buffers for the K-M advection color mean (no per-cell allocation).
const kmColors = new Float64Array(12);
const kmWeights = new Float64Array(4);
const kmOut = [0, 0, 0];

const BLOOM_SEED = 0x600d;

function clampSym(v, m) { return v > m ? m : v < -m ? -m : v; }

// ---------------------------------------------------------------------------
// 6.1 Active-region rebuild (every 2nd frame)
// ---------------------------------------------------------------------------

/**
 * Rebuild the active mask + fresh bbox from the water map. Pass 1 marks
 * horizontal wet triples inside the previous (padded) bbox; pass 2 grows a
 * vertical "skirt" wherever a vertical triple sums to EXACTLY 1 - and the 2s
 * it writes count in later sums, which is load-bearing: an isolated front
 * gets a full skirt and can run 1 cell/frame, while a train of close stripes
 * starves its own skirt and waits to merge (keeps a wide front from
 * decomposing into permanent horizontal bands).
 */
export function rebuildActiveRegion(g) {
  const { S, W, H, film, active } = g;
  if (!g.hasFluid || g.bx1 < g.bx0) { emptyBBox(g); return; }
  // Clear the mask over the previous bbox padded +-2. We extend the clear to
  // the pad ring when the box touches the sheet edge, so skirt 2s written on
  // the pad cannot go stale (the spec's [1..W] clamp plus pad writes would
  // otherwise leak permanent actives on row 0 / row H+1).
  const px0 = Math.max(1, g.bx0 - 2), px1 = Math.min(W, g.bx1 + 2);
  const py0 = Math.max(1, g.by0 - 2), py1 = Math.min(H, g.by1 + 2);
  const cx0 = px0 === 1 ? 0 : px0, cx1 = px1 === W ? W + 1 : px1;
  const cy0 = py0 === 1 ? 0 : py0, cy1 = py1 === H ? H + 1 : py1;
  for (let y = cy0; y <= cy1; y++) active.fill(0, cx0 + y * S, cx1 + 1 + y * S);

  // Pass 1 - wet cells (one row/col inside the brushable area; the drain
  // ring only ever activates via the skirt).
  let fx0 = W + 1, fx1 = 0, fy0 = H + 1, fy1 = 0;
  const sx0 = Math.max(2, px0), sx1 = Math.min(W - 1, px1);
  const sy0 = Math.max(3, py0), sy1 = Math.min(H - 2, py1);
  let fired = false;
  for (let y = sy0; y <= sy1; y++) {
    let i = sx0 + y * S;
    for (let x = sx0; x <= sx1; x++, i++) {
      if (film[i - 1] + film[i] + film[i + 1] > 0) {
        active[i - 1] = 1; active[i] = 1; active[i + 1] = 1;
        fired = true;
        if (x < fx0) fx0 = x; if (x > fx1) fx1 = x;
        if (y < fy0) fy0 = y; if (y > fy1) fy1 = y;
      }
    }
  }
  if (!fired) { emptyBBox(g); return; }

  // Pass 2 - the skirt, scanned top-to-down so earlier 2s shape later sums.
  const kx0 = Math.max(1, fx0 - 2), kx1 = Math.min(W, fx1 + 2);
  const ky0 = Math.max(1, fy0 - 2), ky1 = Math.min(H, fy1 + 2);
  let nx0 = W + 1, nx1 = 0, ny0 = H + 1, ny1 = 0;
  let anyFire = false;
  for (let y = ky0; y <= ky1; y++) {
    let i = kx0 + y * S;
    for (let x = kx0; x <= kx1; x++, i++) {
      if (active[i - S] + active[i] + active[i + S] === 1) {
        active[i - S] = 2; active[i] = 2;
        if (active[i + S] === 0) active[i + S] = 2;
        anyFire = true;
        if (x < nx0) nx0 = x; if (x > nx1) nx1 = x;
        if (y < ny0) ny0 = y; if (y > ny1) ny1 = y; // the fire row itself
      }
    }
  }
  if (!anyFire) { nx0 = fx0; nx1 = fx1; ny0 = fy0; ny1 = fy1; } // defensive
  g.bx0 = Math.max(1, nx0 - 5); g.bx1 = Math.min(W, nx1 + 5);
  g.by0 = Math.max(1, ny0 - 5); g.by1 = Math.min(H, ny1 + 5);
  g.hasFluid = true;
}

// ---------------------------------------------------------------------------
// 6.3 Flow-field build (every 4th frame - the braked frame)
// ---------------------------------------------------------------------------

export function buildFlowField(g, P, gx, gy, extBypass) {
  const { S, W, film, susp, sett, paper, velX, velY, flowX, flowY, wet, active } = g;
  const cells = g.cells;
  const maxV = P.maxVelocity;
  const gMag = Math.sqrt(gx * gx + gy * gy);
  const fingering = !extBypass && P.extFingering > 0 && gMag > 0;
  let gnx = 0, gny = 0, waveLen = 1, tAxisX = 0, tAxisY = 0;
  if (fingering) {
    gnx = gx / gMag; gny = gy / gMag;
    waveLen = Math.max(4, W / P.extRivulets);
    tAxisX = -gny; tAxisY = gnx; // transverse axis
  }
  const backrun = !extBypass && P.extBackrun > 0;
  const out = backrun ? [0, 0, 0] : null;
  // Hoist every knob out of the hot loop.
  const levelK = P.leveling, levelClamp = P.levelClamp;
  const capK = P.capillary, capGate = P.capillaryGate;
  const viscThreshold = P.viscosity;
  const brakeBias = P.brake, brakeReach = P.brakeReach;
  const bx0 = g.bx0, bx1 = g.bx1, by0 = g.by0, by1 = g.by1;

  for (let y = by0; y <= by1; y++) {
    let i = bx0 + y * S;
    for (let x = bx0; x <= bx1; x++, i++) {
      if (!active[i]) continue;
      const f = film[i];
      let ex = velX[i], ey = velY[i];

      // Leveling: water flows thick -> thin, clamped per axis.
      let lx = (film[i - 1] - film[i + 1]) * levelK;
      if (lx > levelClamp) lx = levelClamp; else if (lx < -levelClamp) lx = -levelClamp;
      let ly = (film[i - S] - film[i + S]) * levelK;
      if (ly > levelClamp) ly = levelClamp; else if (ly < -levelClamp) ly = -levelClamp;
      ex += lx; ey += ly;

      // Capillary: only thin paint follows the tooth. Steepest-descent pull -
      // the asymmetric form picks the drop ACROSS the cell in the direction
      // of the steeper fall, which channels water into grain rivulets.
      if (susp[i] + sett[i] < capGate) {
        const pc = paper[i];
        const pl = paper[i - 1], pr = paper[i + 1];
        ex += (pl > pc ? (pc < pr ? pl - pr : pc - pr)
                       : (pc < pr ? pl - pc : pl - pr)) * capK;
        const pu = paper[i - S], pd = paper[i + S];
        ey += (pu > pc ? (pc < pd ? pu - pd : pc - pd)
                       : (pc < pd ? pu - pc : pu - pd)) * capK;
      }

      // Viscosity: deep water drags its persistent-field neighbours along.
      // (The knob is the film THRESHOLD; the 0.2 blend weights are fixed.)
      if (f > viscThreshold) {
        ex = 0.2 * ex + 0.2 * (velX[i - 1] + velX[i + 1] + velX[i - S] + velX[i + S]);
        ey = 0.2 * ey + 0.2 * (velY[i - 1] + velY[i + 1] + velY[i - S] + velY[i + S]);
      }

      // Wetness stamp: deep film marks the sheet damp, valleys damper -
      // future flow keeps running along established wet channels.
      if (f > 3) {
        let wv = 2 - 2 * paper[i];
        if (wv > 1) wv = 1; else if (wv < 0) wv = 0;
        wet[i] = (wv * 255) | 0;
      }

      // Fingering (extension, PRE-brake): at a drip's leading edge, add a
      // transverse sinusoidal ripple - push down at the peaks plus a slight
      // sideways component; the brake then self-selects rivulet columns.
      if (fingering && f > 0.1) {
        const j = i + Math.round(gnx) + Math.round(gny) * S;
        if (j >= 0 && j < cells && film[j] <= 0.01) {
          const phase = (2 * Math.PI * (x * tAxisX + y * tAxisY)) / waveLen;
          const push = P.extFingering * Math.min(f, 2);
          const sn = Math.sin(phase), cs = Math.cos(phase);
          ex += gnx * push * sn + tAxisX * push * 0.3 * cs;
          ey += gny * push * sn + tAxisY * push * 0.3 * cs;
        }
      }

      // Look-ahead absorbency brake: probe a few px downstream; wet or
      // flooded ground ahead lets flow keep running, dry ground stalls it.
      // Linear index arithmetic, no 2-D clamp: a probe past the array end
      // skips the brake entirely (flow at the sheet edge runs off it).
      const sLen = brakeReach / (Math.sqrt(ex * ex + ey * ey) + 0.01);
      const probe = i + Math.trunc(ex * sLen) + Math.trunc(ey * sLen) * S;
      if (probe >= 0 && probe < cells) {
        let brake = film[probe] + (3 / 255) * wet[probe] - brakeBias;
        if (brake < 0.05) brake = 0.05; else if (brake > 1) brake = 1;
        ex *= brake; ey *= brake;
      }

      // Backrun / bloom (extension, POST-brake): where this cell is much
      // wetter than a neighbour holding settled pigment, shove flow toward
      // it and lift some of its settled mass back into suspension. A per-cell
      // budget (max 6 blooming build-frames per fresh front) stops sloshing
      // from pumping thin lines; integer-hash jitter crenellates the rim.
      if (backrun) {
        const thr = 0.8 + 0.2 * hash2Signed(x, y, BLOOM_SEED);
        let met = false;
        const canBloom = g.bloom[i] < 6;
        for (let nIdx = 0; nIdx < 4; nIdx++) {
          const dxn = nIdx === 0 ? -1 : nIdx === 1 ? 1 : 0;
          const dyn = nIdx === 2 ? -1 : nIdx === 3 ? 1 : 0;
          const nb = i + dxn + dyn * S;
          const gap = f - film[nb];
          if (gap > thr && sett[nb] > 0) {
            met = true;
            if (canBloom) {
              const push = P.extBackrun * Math.min(gap, 1.5) * 0.5;
              if (dxn !== 0) ex += dxn * push; else ey += dyn * push;
              const lift = sett[nb] * 0.1;
              const w = lift / (susp[nb] + lift);
              P.mixColor(g.suspR[nb], g.suspG[nb], g.suspB[nb],
                         g.settR[nb], g.settG[nb], g.settB[nb], w, out);
              g.suspR[nb] = out[0]; g.suspG[nb] = out[1]; g.suspB[nb] = out[2];
              susp[nb] += lift; sett[nb] -= lift;
            }
          }
        }
        if (met) { if (canBloom) g.bloom[i]++; } else g.bloom[i] = 0;
      }

      flowX[i] = clampSym(ex, maxV);
      flowY[i] = clampSym(ey, maxV);
    }
  }
}

// ---------------------------------------------------------------------------
// 6.4 Velocity smoothing (the other 3 frames - never braked)
// ---------------------------------------------------------------------------

export function smoothVelocity(g, P) {
  const { S, film, velX, velY, flowX, flowY, active } = g;
  const maxV = P.maxVelocity;
  const bx0 = g.bx0, bx1 = g.bx1, by0 = g.by0, by1 = g.by1;
  for (let y = by0; y <= by1; y++) {
    let i = bx0 + y * S;
    for (let x = bx0; x <= bx1; x++, i++) {
      if (!active[i]) continue;
      let fx, fy;
      if (film[i] > 0.05) {
        fx = 0.2 * velX[i] + 0.2 * (velX[i - 1] + velX[i + 1] + velX[i - S] + velX[i + S]);
        fy = 0.2 * velY[i] + 0.2 * (velY[i - 1] + velY[i + 1] + velY[i - S] + velY[i + S]);
      } else {
        // Whatever gravity the persistent field carries passes straight through.
        fx = velX[i]; fy = velY[i];
      }
      if (fx > maxV) fx = maxV; else if (fx < -maxV) fx = -maxV;
      if (fy > maxV) fy = maxV; else if (fy < -maxV) fy = -maxV;
      flowX[i] = fx; flowY[i] = fy;
    }
  }
}

/**
 * Diffusion (extension, smoothing frames): Fickian spread of suspended
 * pigment through a still wet film. Symmetric flux to the +x/+y neighbours
 * only (each edge visited once => mass-conserving); color rides mass-weighted.
 */
export function diffusionPass(g, P) {
  const { S, film, susp, active } = g;
  const out = [0, 0, 0];
  for (let y = g.by0; y <= g.by1; y++) {
    let i = g.bx0 + y * S;
    for (let x = g.bx0; x <= g.bx1; x++, i++) {
      if (!active[i] || film[i] <= 0.1) continue;
      const rate = P.extDiffusion * Math.min(1, film[i] / 1.5);
      for (let e = 0; e < 2; e++) {
        const nb = e === 0 ? i + 1 : i + S;
        const flux = rate * (susp[i] - susp[nb]);
        const from = flux > 0 ? i : nb;
        const to = flux > 0 ? nb : i;
        let dm = Math.abs(flux);
        if (dm <= 0) continue;
        if (dm > susp[from]) dm = susp[from];
        const w = dm / (susp[to] + dm);
        P.mixColor(g.suspR[to], g.suspG[to], g.suspB[to],
                   g.suspR[from], g.suspG[from], g.suspB[from], w, out);
        g.suspR[to] = out[0]; g.suspG[to] = out[1]; g.suspB[to] = out[2];
        susp[from] -= dm; susp[to] += dm;
      }
    }
  }
}

// ---------------------------------------------------------------------------
// 6.5 Advection + gravity (every frame)
// ---------------------------------------------------------------------------

/**
 * Semi-Lagrangian CONSERVATIVE GATHER: each cell back-traces along its
 * transient flow, pulls water + suspended pigment from the 4 bilinear source
 * corners (subtracting there, clamped so no corner goes negative), and adds
 * the total here. The destination's suspended color is REPLACED by the
 * incoming mean whenever any mass arrives - a fast rivulet that delivers even
 * a little mass takes the cell's color, which is what makes color fronts
 * move. No caps: fronts pile up (rim / backrun raw material).
 * Gravity lands UNBRAKED in the persistent field, scaled by the local film.
 * Returns the max |velocity component| seen (drives the drying cadence).
 */
export function advect(g, P, gx, gy) {
  const { S, W, H, film, susp, suspR, suspG, suspB, velX, velY, flowX, flowY, active } = g;
  const maxV = P.maxVelocity;
  const bx0 = g.bx0, bx1 = g.bx1, by0 = g.by0, by1 = g.by1;
  const kmMean = P.kmMixing === true; // route the incoming mean through K-M
  let vmax = 0;
  for (let y = by0; y <= by1; y++) {
    let i = bx0 + y * S;
    for (let x = bx0; x <= bx1; x++, i++) {
      if (!active[i]) { velX[i] = 0; velY[i] = 0; continue; }
      const ux = flowX[i], uy = flowY[i];
      const axv = ux < 0 ? -ux : ux, ayv = uy < 0 ? -uy : uy;
      if (axv > vmax) vmax = axv;
      if (ayv > vmax) vmax = ayv;
      const sx = x - ux, sy = y - uy;
      // A back-trace that leaves the sheet keeps its momentum and moves
      // nothing: a drip reaching the edge must not lose its velocity.
      if (sx < 1 || sx > W || sy < 1 || sy > H) continue;
      const x0 = sx | 0, y0 = sy | 0; // positive, so |0 == floor
      const fx = sx - x0, fy = sy - y0;
      const i00 = x0 + y0 * S, i10 = i00 + 1, i01 = i00 + S, i11 = i01 + 1;
      const w00 = (1 - fx) * (1 - fy), w10 = fx * (1 - fy);
      const w01 = (1 - fx) * fy, w11 = fx * fy;

      // Persistent velocity = transient flow sampled at the source, then
      // gravity injected (unbraked) scaled by the water here.
      const f = film[i];
      let nvx = flowX[i00] * w00 + flowX[i10] * w10 + flowX[i01] * w01 + flowX[i11] * w11 + gx * f;
      let nvy = flowY[i00] * w00 + flowY[i10] * w10 + flowY[i01] * w01 + flowY[i11] * w11 + gy * f;
      if (nvx > maxV) nvx = maxV; else if (nvx < -maxV) nvx = -maxV;
      if (nvy > maxV) nvy = maxV; else if (nvy < -maxV) nvy = -maxV;
      velX[i] = nvx; velY[i] = nvy;

      // Pigment gather (pre-clamp weights drive the incoming color mean).
      // The "reduce p_k by the shortfall" clamp cannot bite here: each pull
      // p_k = susp[corner] * w_k with w_k in [0,1] is computed and subtracted
      // atomically within this cell, so a corner can never go negative.
      const m00 = susp[i00], m10 = susp[i10], m01 = susp[i01], m11 = susp[i11];
      const p00 = m00 * w00, p10 = m10 * w10, p01 = m01 * w01, p11 = m11 * w11;
      const want = p00 + p10 + p01 + p11;
      if (want >= 0.00001) {
        const inv = 1 / want;
        let rIn, gIn, bIn;
        if (kmMean) {
          // Pigment-mixing checkbox ON: the corner mean is taken in K/S space.
          kmColors[0] = suspR[i00]; kmColors[1] = suspG[i00]; kmColors[2] = suspB[i00];
          kmColors[3] = suspR[i10]; kmColors[4] = suspG[i10]; kmColors[5] = suspB[i10];
          kmColors[6] = suspR[i01]; kmColors[7] = suspG[i01]; kmColors[8] = suspB[i01];
          kmColors[9] = suspR[i11]; kmColors[10] = suspG[i11]; kmColors[11] = suspB[i11];
          kmWeights[0] = p00; kmWeights[1] = p10; kmWeights[2] = p01; kmWeights[3] = p11;
          kmWeightedMeanColor(kmColors, kmWeights, 4, inv, kmOut);
          rIn = kmOut[0]; gIn = kmOut[1]; bIn = kmOut[2];
        } else {
          rIn = (suspR[i00] * p00 + suspR[i10] * p10 + suspR[i01] * p01 + suspR[i11] * p11) * inv;
          gIn = (suspG[i00] * p00 + suspG[i10] * p10 + suspG[i01] * p01 + suspG[i11] * p11) * inv;
          bIn = (suspB[i00] * p00 + suspB[i10] * p10 + suspB[i01] * p01 + suspB[i11] * p11) * inv;
        }
        susp[i00] = m00 - p00; susp[i10] = m10 - p10;
        susp[i01] = m01 - p01; susp[i11] = m11 - p11;
        susp[i] += want;
        suspR[i] = rIn; suspG[i] = gIn; suspB[i] = bIn; // REPLACE
      }
      // Water moves the same way (no color, no threshold).
      const f00 = film[i00], f10 = film[i10], f01 = film[i01], f11 = film[i11];
      const q00 = f00 * w00, q10 = f10 * w10, q01 = f01 * w01, q11 = f11 * w11;
      film[i00] = f00 - q00; film[i10] = f10 - q10;
      film[i01] = f01 - q01; film[i11] = f11 - q11;
      film[i] += q00 + q10 + q01 + q11;
    }
  }
  return vmax;
}

// ---------------------------------------------------------------------------
// 6.6 Pressure projection (every 3rd frame)
// ---------------------------------------------------------------------------

/**
 * One cheap Jacobi relaxation toward incompressibility: kills piling-up but
 * leaves the uniform downward drift intact so drips survive. The divergence
 * and pressure scratch fields reuse the transient-flow arrays (rebuilt next
 * frame anyway).
 */
export function project(g, P) {
  const { S, W, H, velX, velY, flowX, flowY, active } = g;
  const div = flowX, prs = flowY;
  const zx0 = Math.max(0, g.bx0 - 1), zx1 = Math.min(W + 1, g.bx1 + 1);
  const zy0 = Math.max(0, g.by0 - 1), zy1 = Math.min(H + 1, g.by1 + 1);
  for (let y = zy0; y <= zy1; y++) { // neighbours of active cells must read 0
    div.fill(0, zx0 + y * S, zx1 + 1 + y * S);
    prs.fill(0, zx0 + y * S, zx1 + 1 + y * S);
  }
  for (let y = g.by0; y <= g.by1; y++) {
    let i = g.bx0 + y * S;
    for (let x = g.bx0; x <= g.bx1; x++, i++) {
      if (active[i]) div[i] = velX[i - 1] - velX[i + 1] + velY[i - S] - velY[i + S];
    }
  }
  for (let y = g.by0; y <= g.by1; y++) {
    let i = g.bx0 + y * S;
    for (let x = g.bx0; x <= g.bx1; x++, i++) {
      if (active[i]) prs[i] = (div[i] + 0.25 * (div[i - 1] + div[i + 1] + div[i - S] + div[i + S])) * 0.25;
    }
  }
  for (let y = g.by0; y <= g.by1; y++) {
    let i = g.bx0 + y * S;
    for (let x = g.bx0; x <= g.bx1; x++, i++) {
      if (!active[i]) continue;
      let nvx = velX[i] - 0.5 * (prs[i + 1] - prs[i - 1]) * P.projection;
      let nvy = velY[i] - 0.5 * (prs[i + S] - prs[i - S]) * P.projection;
      // Guard: a corrected velocity whose back-trace leaves the sheet is zeroed.
      if (x - nvx < 1 || x - nvx > W) nvx = 0;
      if (y - nvy < 1 || y - nvy > H) nvy = 0;
      velX[i] = nvx; velY[i] = nvy;
    }
  }
}

// ---------------------------------------------------------------------------
// 6.7 Boundaries - the drain
// ---------------------------------------------------------------------------

/**
 * The drain band is two cells wide on every side: the pad ring plus the first
 * interior ring. Mass that advected onto it is deleted (the drip ran off the
 * paper); wetness reads the faintest non-zero byte; velocities get their
 * tangential component zeroed first, then the normal written OUTWARD at 0.1
 * (corners end up carrying the normal value). The soft outward bias is what
 * keeps the solver stable at the edge.
 */
export function applyBoundaries(g, velocityOnly) {
  const { S, W, H, rows, film, susp, wet, velX, velY, flowX, flowY } = g;
  const leftCols = [0, 1], rightCols = [W, W + 1];
  const topRows = [0, 1], botRows = [H, H + 1];
  if (!velocityOnly) {
    for (const y of [...topRows, ...botRows]) {
      const b = y * S;
      film.fill(0, b, b + S); susp.fill(0, b, b + S); wet.fill(1, b, b + S);
    }
    for (let y = 0; y < rows; y++) {
      const b = y * S;
      for (const x of [...leftCols, ...rightCols]) { film[b + x] = 0; susp[b + x] = 0; wet[b + x] = 1; }
    }
  }
  // Tangential first, on every band...
  for (const y of [...topRows, ...botRows]) {
    const b = y * S;
    for (let x = 0; x < S; x++) { velX[b + x] = 0; flowX[b + x] = 0; }
  }
  for (let y = 0; y < rows; y++) {
    const b = y * S;
    for (const x of [...leftCols, ...rightCols]) { velY[b + x] = 0; flowY[b + x] = 0; }
  }
  // ...then the normal component, written outward.
  for (const y of topRows) { const b = y * S; for (let x = 0; x < S; x++) { velY[b + x] = -0.1; flowY[b + x] = -0.1; } }
  for (const y of botRows) { const b = y * S; for (let x = 0; x < S; x++) { velY[b + x] = 0.1; flowY[b + x] = 0.1; } }
  for (let y = 0; y < rows; y++) {
    const b = y * S;
    for (const x of leftCols) { velX[b + x] = -0.1; flowX[b + x] = -0.1; }
    for (const x of rightCols) { velX[b + x] = 0.1; flowX[b + x] = 0.1; }
  }
}
