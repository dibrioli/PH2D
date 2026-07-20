// Renderer (spec section 13): composites the interior of the layer grids
// into an RGBA byte image (cell (x,y) -> pixel (x-1, y-1)). DOM-free - the
// shell blits the buffer with putImageData; the export path reuses it.
//
// The look, per pixel: a paper-tinted near-white base; each pigment layer
// alpha-composited settled-then-suspended, with the tooth offset (granulation)
// and a SIGNED mass-gradient emboss added INTO the pigment color channels so
// grain shows through the paint; an optional damp overlay (cool darkening +
// a meniscus glint on the film's rim); optional K-M glaze stacking instead of
// alpha-over. A render-only "paper visibility" master fades the tooth in the
// appearance without ever touching the physics array.

import { alphaOfMass } from './opacity.js';
import { srgbToLinear, linearToSrgb, kmGlazeChannelLinear } from './colorops.js';
import { hash2 } from './rng.js';

const INV_SQRT2 = 0.7071067811865476;
const LIGHT_X = -INV_SQRT2, LIGHT_Y = -INV_SQRT2; // light from top-left

function smoothstep01(t) { return t <= 0 ? 0 : t >= 1 ? 1 : t * t * (3 - 2 * t); }

/**
 * Render a sub-rectangle (cell coords, inclusive, [1..W] x [1..H]) of the
 * layer stack into out (Uint8ClampedArray, W*H*4).
 * cfg = { P (tuning values), layers: [{grid, opacity, visible}], activeGrid,
 *         showWet, kmGlaze, extBypass }.
 */
export function renderRegion(cfg, out, x0, y0, x1, y1) {
  const layers = cfg.layers.filter((l) => l.visible);
  const P = cfg.P;
  const base0 = cfg.layers[0].grid; // paper source (shared array anyway)
  const { S, W } = base0;
  const paperVis = P.paperVisibility;
  const grainVis = P.visualGrain;
  const embossK = P.emboss;
  const showWet = cfg.showWet;
  const act = cfg.activeGrid;
  const ext = !cfg.extBypass;
  const rake = ext ? P.extRakeLight : 0;
  const valley = ext ? P.extValleyGran : 0;
  const sheen = ext ? P.extWetSheen : 0;
  const dither = ext ? P.extGrainDither : 0;
  const edgeTint = ext ? (P.extEdgeTint - 0.5) * 2 : 0; // signed, 0 = neutral
  const glaze = cfg.kmGlaze;

  for (let cy = y0; cy <= y1; cy++) {
    let i = x0 + cy * S;
    let o = ((cy - 1) * W + (x0 - 1)) * 4;
    for (let cx = x0; cx <= x1; cx++, i++, o += 4) {
      const papRaw = base0.paper[i];
      const pap = 0.5 + (papRaw - 0.5) * paperVis;
      // Base: the sheet, paper-tinted near-white.
      let r = 255 + (pap * 30 - 30), g = r, b = r;
      // Rake light (extension): grazing light across the paper relief.
      if (rake > 0) {
        const gx = (base0.paper[i + 1] - base0.paper[i - 1]) * 0.5;
        const gy = (base0.paper[i + S] - base0.paper[i - S]) * 0.5;
        let lit = (gx * LIGHT_X + gy * LIGHT_Y) * 90 * rake;
        if (lit > 12) lit = 12; else if (lit < -12) lit = -12;
        r += lit; g += lit; b += lit;
      }
      // Optional K-M glaze stacking works in linear reflectance.
      let lr = 0, lg = 0, lb = 0;
      if (glaze) {
        lr = srgbToLinear(Math.min(255, Math.max(0, r)) / 255);
        lg = srgbToLinear(Math.min(255, Math.max(0, g)) / 255);
        lb = srgbToLinear(Math.min(255, Math.max(0, b)) / 255);
      }

      for (const layer of layers) {
        const lgrd = layer.grid;
        const op = layer.opacity;
        const sMass = lgrd.sett[i], fMass = lgrd.susp[i];
        const total = sMass + fMass;
        if (total <= 0 || op <= 0) continue;
        // Granulation offset: the tooth relief printed into the pigment,
        // fading out linearly as total mass goes 1000 -> 3000 (thick paint
        // hides the grain).
        let fade = 1;
        if (total > 1000) fade = total >= 3000 ? 0 : (3000 - total) / 2000;
        let v = (pap * 100 - 40) * grainVis * fade;
        // Signed emboss from the mass gradient: one side lightens, the other
        // darkens - a soft bevel, not an outline. Vertical weighted 2x.
        const mL = lgrd.sett[i - 1] + lgrd.susp[i - 1], mR = lgrd.sett[i + 1] + lgrd.susp[i + 1];
        const mU = lgrd.sett[i - S] + lgrd.susp[i - S], mD = lgrd.sett[i + S] + lgrd.susp[i + S];
        let emb = ((mR - mL) * 0.5 + (mD - mU) * 1.0) * 0.008 * embossK;
        if (emb > 40) emb = 40; else if (emb < -40) emb = -40;
        v += emb;
        // Valley granulation (extension): settled reads denser in troughs.
        let settForAlpha = sMass;
        if (valley > 0) settForAlpha = sMass * (1 + valley * 3 * Math.max(0, 0.5 - pap));
        const aS = alphaOfMass(settForAlpha) * op;
        const aF = alphaOfMass(fMass) * op;
        if (glaze) {
          // Stack by reflectance: paper -> settled -> suspended, linear light.
          if (aS > 0) {
            lr = kmGlazeChannelLinear(lr, srgbToLinear(clampByte(lgrd.settR[i] + v) / 255), aS);
            lg = kmGlazeChannelLinear(lg, srgbToLinear(clampByte(lgrd.settG[i] + v) / 255), aS);
            lb = kmGlazeChannelLinear(lb, srgbToLinear(clampByte(lgrd.settB[i] + v) / 255), aS);
          }
          if (aF > 0) {
            lr = kmGlazeChannelLinear(lr, srgbToLinear(clampByte(lgrd.suspR[i] + v) / 255), aF);
            lg = kmGlazeChannelLinear(lg, srgbToLinear(clampByte(lgrd.suspG[i] + v) / 255), aF);
            lb = kmGlazeChannelLinear(lb, srgbToLinear(clampByte(lgrd.suspB[i] + v) / 255), aF);
          }
        } else {
          // Alpha-over, settled then suspended; the tooth offset v rides
          // inside the pigment color so grain shows THROUGH the color.
          if (aS > 0) {
            r = r * (1 - aS) + (lgrd.settR[i] + v) * aS;
            g = g * (1 - aS) + (lgrd.settG[i] + v) * aS;
            b = b * (1 - aS) + (lgrd.settB[i] + v) * aS;
          }
          if (aF > 0) {
            r = r * (1 - aF) + (lgrd.suspR[i] + v) * aF;
            g = g * (1 - aF) + (lgrd.suspG[i] + v) * aF;
            b = b * (1 - aF) + (lgrd.suspB[i] + v) * aF;
          }
        }
        // Edge tint (extension): bidirectional darken/lighten on mass fronts.
        if (edgeTint !== 0) {
          const gmx = (mR - mL) * 0.5, gmy = (mD - mU) * 0.5;
          const gm = Math.sqrt(gmx * gmx + gmy * gmy);
          const w = smoothstep01((gm - 100) / 1300);
          const tint = edgeTint * 130 * w;
          if (!glaze) { r -= tint; g -= tint; b -= tint; }
        }
      }
      if (glaze) {
        r = linearToSrgb(lr) * 255; g = linearToSrgb(lg) * 255; b = linearToSrgb(lb) * 255;
      }

      // Show-wet overlay (active layer): a realistic damp look - darken cool
      // plus a meniscus glint on the film's rim.
      if (showWet) {
        const film = act.film[i];
        let wetSig = act.wet[i] / 255;
        if (film > 0.02) {
          const s = smoothstep01(Math.min(1, film / 2.5));
          if (s > wetSig) wetSig = s;
        }
        if (wetSig > 0.01) {
          r *= 1 - 2.4 * 0.04 * wetSig;
          g *= 1 - 1.6 * 0.04 * wetSig;
          b *= 1 - 0.8 * 0.04 * wetSig;
          const gx = (act.film[i + 1] - act.film[i - 1]) * 0.5;
          const gy = (act.film[i + S] - act.film[i - S]) * 0.5;
          const mag = Math.sqrt(gx * gx + gy * gy);
          if (mag > 0.01) {
            const rim = Math.min(1, mag * 0.7);
            const shine = 18.3 * rim * rim * (0.5 + 0.5 * ((gx / mag) * LIGHT_X + (gy / mag) * LIGHT_Y));
            r += shine; g += shine; b += shine;
          }
        }
      }
      // Wet sheen (extension): specular keyed on the paper gradient x light.
      if (sheen > 0) {
        const film = act.film[i];
        if (film > 0.05) {
          const win = smoothstep01((film - 0.05) / 2.45);
          const gx = (base0.paper[i + 1] - base0.paper[i - 1]) * 0.5;
          const gy = (base0.paper[i + S] - base0.paper[i - S]) * 0.5;
          const spec = Math.max(0, gx * LIGHT_X + gy * LIGHT_Y) * 160 * sheen * win;
          r += spec; g += spec; b += spec;
        }
      }
      // Static grain dither (extension): per-pixel hash, +-0.5 * knob.
      if (dither > 0) {
        const d = (hash2(cx, cy, 0xd17e) - 0.5) * dither;
        r += d; g += d; b += d;
      }
      out[o] = r; out[o + 1] = g; out[o + 2] = b; out[o + 3] = 255;
    }
  }
}

function clampByte(v) { return v < 0 ? 0 : v > 255 ? 255 : v; }

/**
 * Pigment-only export (spec section 15b): straight-alpha over transparent.
 * alpha = combined coverage aS + aF(1-aS); RGB = settled-then-suspended
 * straight-alpha compositing, over-composited across visible layers.
 */
export function renderPigmentOnly(layersIn, out) {
  const layers = layersIn.filter((l) => l.visible);
  const g0 = layersIn[0].grid;
  const { S, W, H } = g0;
  for (let cy = 1; cy <= H; cy++) {
    let i = 1 + cy * S;
    let o = ((cy - 1) * W) * 4;
    for (let cx = 1; cx <= W; cx++, i++, o += 4) {
      // Accumulate premultiplied, then unpremultiply at the end.
      let pr = 0, pg = 0, pb = 0, pa = 0;
      for (const layer of layers) {
        const lg = layer.grid, op = layer.opacity;
        const aS = alphaOfMass(lg.sett[i]) * op;
        const aF = alphaOfMass(lg.susp[i]) * op;
        const la = aS + aF * (1 - aS);
        if (la <= 0) continue;
        // Layer color: settled, then suspended over it (straight alpha).
        let cr = lg.settR[i], cg = lg.settG[i], cb = lg.settB[i];
        if (aS <= 0) { cr = lg.suspR[i]; cg = lg.suspG[i]; cb = lg.suspB[i]; }
        else if (aF > 0) {
          cr = cr * (1 - aF) + lg.suspR[i] * aF;
          cg = cg * (1 - aF) + lg.suspG[i] * aF;
          cb = cb * (1 - aF) + lg.suspB[i] * aF;
        }
        // Over-composite onto the accumulator (premultiplied).
        pr = pr * (1 - la) + cr * la;
        pg = pg * (1 - la) + cg * la;
        pb = pb * (1 - la) + cb * la;
        pa = pa + la * (1 - pa);
      }
      if (pa > 0) {
        out[o] = pr / pa > 255 ? 255 : pr / pa;
        out[o + 1] = pg / pa > 255 ? 255 : pg / pa;
        out[o + 2] = pb / pa > 255 ? 255 : pb / pa;
        out[o + 3] = pa * 255;
      } else {
        out[o] = 0; out[o + 1] = 0; out[o + 2] = 0; out[o + 3] = 0;
      }
    }
  }
}
