// Color math (spec section 14): HSV<->RGB for the UI wheel, sRGB<->linear,
// and Kubelka-Munk single-constant subtractive mixing (opt-in).
//
// Engine colors are ALWAYS 0..255 floats and are never quantized to bytes:
// a wet cell can re-mix thousands of times, and a byte round-trip on every
// mix drifts washes toward black.

/** h in [0,360), s,v in [0,1] -> [r,g,b] 0..255 floats. */
export function hsvToRgb(h, s, v) {
  const c = v * s;
  const hp = (((h % 360) + 360) % 360) / 60;
  const x = c * (1 - Math.abs((hp % 2) - 1));
  let r = 0, g = 0, b = 0;
  if (hp < 1) { r = c; g = x; } else if (hp < 2) { r = x; g = c; }
  else if (hp < 3) { g = c; b = x; } else if (hp < 4) { g = x; b = c; }
  else if (hp < 5) { r = x; b = c; } else { r = c; b = x; }
  const m = v - c;
  return [(r + m) * 255, (g + m) * 255, (b + m) * 255];
}

/** r,g,b 0..255 -> [h 0..360, s 0..1, v 0..1]. */
export function rgbToHsv(r, g, b) {
  const rn = r / 255, gn = g / 255, bn = b / 255;
  const max = Math.max(rn, gn, bn), min = Math.min(rn, gn, bn);
  const d = max - min;
  let h = 0;
  if (d > 0) {
    if (max === rn) h = 60 * (((gn - bn) / d) % 6);
    else if (max === gn) h = 60 * ((bn - rn) / d + 2);
    else h = 60 * ((rn - gn) / d + 4);
  }
  if (h < 0) h += 360;
  return [h, max === 0 ? 0 : d / max, max];
}

/** Standard sRGB EOTF, c in [0,1] -> linear [0,1]. */
export function srgbToLinear(c) {
  return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
}

/** Inverse EOTF, linear [0,1] -> sRGB [0,1]. */
export function linearToSrgb(c) {
  return c <= 0.0031308 ? c * 12.92 : 1.055 * Math.pow(c, 1 / 2.4) - 0.055;
}

// ---------------------------------------------------------------------------
// Kubelka-Munk single-constant mixing.
// A reflectance R maps to an absorption/scattering ratio K/S = (1-R)^2 / 2R.
// Pigment mixtures are LINEAR in K/S, so we convert both colors, lerp the
// ratios, and invert: R = 1 + KS - sqrt(KS^2 + 2 KS). Reflectance is floored
// at 1/255 so pure black cannot make KS blow up.
// ---------------------------------------------------------------------------

const R_FLOOR = 1 / 255;

function ksOfReflectance(r) {
  const rr = r < R_FLOOR ? R_FLOOR : r;
  return ((1 - rr) * (1 - rr)) / (2 * rr);
}

function reflectanceOfKs(ks) {
  return 1 + ks - Math.sqrt(ks * ks + 2 * ks);
}

/**
 * Mix one linear-light channel value (0..1) toward another by weight w in K/S
 * space. Both endpoints exact: w=0 -> dst, w=1 -> src.
 */
export function kmMixChannelLinear(dstLin, srcLin, w) {
  const ks = (1 - w) * ksOfReflectance(dstLin) + w * ksOfReflectance(srcLin);
  return reflectanceOfKs(ks);
}

/**
 * Mix two engine colors (0..255 float channels) by weight w using K-M.
 * Full float pipeline: bytes-float -> sRGB -> linear -> K/S lerp -> back.
 * Writes into out[0..2].
 */
export function kmMixColor(dr, dg, db, sr, sg, sb, w, out) {
  out[0] = linearToSrgb(kmMixChannelLinear(srgbToLinear(dr / 255), srgbToLinear(sr / 255), w)) * 255;
  out[1] = linearToSrgb(kmMixChannelLinear(srgbToLinear(dg / 255), srgbToLinear(sg / 255), w)) * 255;
  out[2] = linearToSrgb(kmMixChannelLinear(srgbToLinear(db / 255), srgbToLinear(sb / 255), w)) * 255;
}

/** Plain linear-RGB lerp of two engine colors by weight w, into out[0..2]. */
export function plainMixColor(dr, dg, db, sr, sg, sb, w, out) {
  out[0] = dr + (sr - dr) * w;
  out[1] = dg + (sg - dg) * w;
  out[2] = db + (sb - db) * w;
}

/**
 * The engine routes every pigment-color blend through this indirection.
 * `useKm` is the "pigment mixing (K-M)" experimental checkbox; default OFF
 * keeps the plain composites of spec sections 6/10/11.
 */
export function makeColorMixer(useKm) {
  return useKm ? kmMixColor : plainMixColor;
}

/**
 * K-M weighted mean of up to 4 engine colors (used by the advection's
 * incoming-color mean when pigment mixing is ON): mixtures are linear in
 * K/S, so we average the corners' K/S ratios with the pull weights.
 * colors = flat array [r0,g0,b0, r1,g1,b1, ...] (0..255), weights sum > 0.
 */
export function kmWeightedMeanColor(colors, weights, count, invTotal, out) {
  for (let ch = 0; ch < 3; ch++) {
    let ks = 0;
    for (let k = 0; k < count; k++) {
      ks += weights[k] * ksOfReflectance(srgbToLinear(colors[k * 3 + ch] / 255));
    }
    out[ch] = linearToSrgb(reflectanceOfKs(ks * invTotal)) * 255;
  }
}

/**
 * K-M glaze stack of one pigment layer (linear reflectance rTop, coverage a)
 * over a backdrop (linear reflectance rBot). Energy-bounded: a=1 -> rTop,
 * a=0 -> rBot. Guard the denominator against rTop*rBot ~ 1.
 */
export function kmGlazeChannelLinear(rBot, rTop, a) {
  const rt = a * rTop;
  const tt = 1 - a;
  const den = 1 - rt * rBot;
  const r = rt + (tt * tt * rBot) / (den < 1e-6 ? 1e-6 : den);
  return r < 0 ? 0 : r > 1 ? 1 : r;
}
