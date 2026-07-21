//! Color math (port of `colorops.js`, SPEC §14): HSV<->RGB for UI, sRGB<->
//! linear, and Kubelka–Munk single-constant subtractive mixing (opt-in).
//!
//! Engine colors are ALWAYS 0..255 floats and are never quantized to bytes:
//! a wet cell can re-mix thousands of times, and a byte round-trip on every
//! mix drifts washes toward black.
//!
//! `pow` routes through `libm` (cross-OS bit-identical); it only runs when
//! the K–M checkbox is ON or the glaze stack renders — never in the default
//! hot path (HR-5).

/// h in [0,360), s,v in [0,1] -> (r,g,b) 0..255 floats.
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> [f64; 3] {
    let c = v * s;
    let hp = (((h % 360.0) + 360.0) % 360.0) / 60.0;
    let x = c * (1.0 - ((hp % 2.0) - 1.0).abs());
    let (mut r, mut g, mut b) = (0.0, 0.0, 0.0);
    if hp < 1.0 {
        r = c;
        g = x;
    } else if hp < 2.0 {
        r = x;
        g = c;
    } else if hp < 3.0 {
        g = c;
        b = x;
    } else if hp < 4.0 {
        g = x;
        b = c;
    } else if hp < 5.0 {
        r = x;
        b = c;
    } else {
        r = c;
        b = x;
    }
    let m = v - c;
    [(r + m) * 255.0, (g + m) * 255.0, (b + m) * 255.0]
}

/// r,g,b 0..255 -> (h 0..360, s 0..1, v 0..1).
pub fn rgb_to_hsv(r: f64, g: f64, b: f64) -> [f64; 3] {
    let rn = r / 255.0;
    let gn = g / 255.0;
    let bn = b / 255.0;
    let max = rn.max(gn).max(bn);
    let min = rn.min(gn).min(bn);
    let d = max - min;
    let mut h = 0.0;
    if d > 0.0 {
        if max == rn {
            h = 60.0 * (((gn - bn) / d) % 6.0);
        } else if max == gn {
            h = 60.0 * ((bn - rn) / d + 2.0);
        } else {
            h = 60.0 * ((rn - gn) / d + 4.0);
        }
    }
    if h < 0.0 {
        h += 360.0;
    }
    [h, if max == 0.0 { 0.0 } else { d / max }, max]
}

/// Standard sRGB EOTF, c in [0,1] -> linear [0,1].
#[inline]
pub fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        libm::pow((c + 0.055) / 1.055, 2.4)
    }
}

/// Inverse EOTF, linear [0,1] -> sRGB [0,1].
#[inline]
pub fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * libm::pow(c, 1.0 / 2.4) - 0.055
    }
}

// ---------------------------------------------------------------------------
// Kubelka–Munk single-constant mixing. A reflectance R maps to K/S =
// (1-R)^2 / 2R; mixtures are LINEAR in K/S, so convert, lerp, invert:
// R = 1 + KS - sqrt(KS^2 + 2 KS). Reflectance floored at 1/255 so pure
// black cannot make KS blow up.
// ---------------------------------------------------------------------------

const R_FLOOR: f64 = 1.0 / 255.0;

#[inline]
fn ks_of_reflectance(r: f64) -> f64 {
    let rr = if r < R_FLOOR { R_FLOOR } else { r };
    ((1.0 - rr) * (1.0 - rr)) / (2.0 * rr)
}

#[inline]
fn reflectance_of_ks(ks: f64) -> f64 {
    1.0 + ks - (ks * ks + 2.0 * ks).sqrt()
}

/// Mix one linear-light channel toward another by weight w in K/S space.
/// Both endpoints exact: w=0 -> dst, w=1 -> src.
#[inline]
pub fn km_mix_channel_linear(dst_lin: f64, src_lin: f64, w: f64) -> f64 {
    let ks = (1.0 - w) * ks_of_reflectance(dst_lin) + w * ks_of_reflectance(src_lin);
    reflectance_of_ks(ks)
}

/// How the engine blends two pigment colors. `Plain` is the default (the
/// composites of SPEC §6/§10/§11); `Km` is the "pigment mixing (K–M)"
/// experimental checkbox. An enum, not a fn pointer: the hot loops match on
/// it once per call and the compiler sees through both arms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMix {
    Plain,
    Km,
}

impl ColorMix {
    /// Mix dst -> src by weight w, both 0..255 float channels, into `out`.
    #[inline]
    pub fn mix(
        self,
        dr: f64,
        dg: f64,
        db: f64,
        sr: f64,
        sg: f64,
        sb: f64,
        w: f64,
        out: &mut [f64; 3],
    ) {
        match self {
            ColorMix::Plain => {
                out[0] = dr + (sr - dr) * w;
                out[1] = dg + (sg - dg) * w;
                out[2] = db + (sb - db) * w;
            }
            ColorMix::Km => {
                out[0] = linear_to_srgb(km_mix_channel_linear(
                    srgb_to_linear(dr / 255.0),
                    srgb_to_linear(sr / 255.0),
                    w,
                )) * 255.0;
                out[1] = linear_to_srgb(km_mix_channel_linear(
                    srgb_to_linear(dg / 255.0),
                    srgb_to_linear(sg / 255.0),
                    w,
                )) * 255.0;
                out[2] = linear_to_srgb(km_mix_channel_linear(
                    srgb_to_linear(db / 255.0),
                    srgb_to_linear(sb / 255.0),
                    w,
                )) * 255.0;
            }
        }
    }
}

/// K–M weighted mean of up to 4 engine colors (the advection's incoming-color
/// mean when pigment mixing is ON): mixtures are linear in K/S, so average
/// the corners' K/S ratios with the pull weights.
/// `colors` = flat [r0,g0,b0, r1,g1,b1, ...] (0..255); `inv_total` = 1/Σw.
pub fn km_weighted_mean_color(
    colors: &[f64; 12],
    weights: &[f64; 4],
    count: usize,
    inv_total: f64,
    out: &mut [f64; 3],
) {
    for ch in 0..3 {
        let mut ks = 0.0;
        for k in 0..count {
            ks += weights[k] * ks_of_reflectance(srgb_to_linear(colors[k * 3 + ch] / 255.0));
        }
        out[ch] = linear_to_srgb(reflectance_of_ks(ks * inv_total)) * 255.0;
    }
}

/// K–M glaze stack of one pigment layer (linear reflectance r_top, coverage a)
/// over a backdrop (linear reflectance r_bot). Energy-bounded: a=1 -> r_top,
/// a=0 -> r_bot. Guard the denominator against r_top * r_bot ~ 1.
#[inline]
pub fn km_glaze_channel_linear(r_bot: f64, r_top: f64, a: f64) -> f64 {
    let rt = a * r_top;
    let tt = 1.0 - a;
    let den = 1.0 - rt * r_bot;
    let r = rt + (tt * tt * r_bot) / if den < 1e-6 { 1e-6 } else { den };
    r.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_round_trips() {
        let [r, g, b] = hsv_to_rgb(210.0, 0.76, 0.82);
        let [h, s, v] = rgb_to_hsv(r, g, b);
        assert!((h - 210.0).abs() < 0.01 && (s - 0.76).abs() < 0.01 && (v - 0.82).abs() < 0.01);
    }

    #[test]
    fn km_mix_endpoints_are_exact_above_the_floor() {
        // Endpoints are exact ONLY above the 1/255 reflectance floor — a
        // channel darker than that (linear) is floored on the way into K/S
        // space and cannot round-trip. The reference behaves identically;
        // pick endpoint colors whose linear channels all clear the floor.
        let mut out = [0.0; 3];
        ColorMix::Km.mix(200.0, 90.0, 60.0, 40.0, 120.0, 180.0, 0.0, &mut out);
        assert!((out[0] - 200.0).abs() < 1e-6 && (out[2] - 60.0).abs() < 1e-6);
        ColorMix::Km.mix(200.0, 90.0, 60.0, 40.0, 120.0, 180.0, 1.0, &mut out);
        assert!((out[0] - 40.0).abs() < 1e-6 && (out[2] - 180.0).abs() < 1e-6);
    }

    #[test]
    fn km_yellow_plus_blue_makes_green() {
        // The signature of subtractive mixing: yellow + blue reads GREEN —
        // the green channel dominates harder than in an additive lerp (the
        // absolute values are darker; subtractive mixing absorbs).
        let mut km = [0.0; 3];
        ColorMix::Km.mix(250.0, 205.0, 40.0, 10.0, 70.0, 150.0, 0.5, &mut km);
        let mut plain = [0.0; 3];
        ColorMix::Plain.mix(250.0, 205.0, 40.0, 10.0, 70.0, 150.0, 0.5, &mut plain);
        assert!(km[1] > km[0] && km[1] > km[2], "K-M mid not green: {km:?}");
        let km_dominance = km[1] / km[0].max(km[2]);
        let plain_dominance = plain[1] / plain[0].max(plain[2]);
        assert!(
            km_dominance > plain_dominance,
            "K-M green dominance {km_dominance:.3} !> plain {plain_dominance:.3}"
        );
    }

    #[test]
    fn glaze_is_energy_bounded() {
        assert!((km_glaze_channel_linear(0.3, 0.8, 1.0) - 0.8).abs() < 1e-12);
        assert!((km_glaze_channel_linear(0.3, 0.8, 0.0) - 0.3).abs() < 1e-12);
    }
}
