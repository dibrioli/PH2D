//! Color math (port of `colorops.js`, SPEC §14): HSV<->RGB for UI, sRGB<->
//! linear, and Kubelka–Munk single-constant subtractive mixing (opt-in).
//!
//! Engine colors are ALWAYS 0..255 floats and are never quantized to bytes:
//! a wet cell can re-mix thousands of times, and a byte round-trip on every
//! mix drifts washes toward black.
//!
//! The sRGB transfer runs only when the K–M checkbox is ON or the glaze stack
//! renders — never in the default hot path (HR-5) — but when it does run it
//! runs 9-15 times per cell, which made it the whole cost of both
//! EXPERIMENTAL knobs. It lives in [`transfer`], node-tabulated from `libm`
//! and interpolated with IEEE basic operations only: same cross-OS
//! bit-identity, ~11x the speed. Read that module before touching it.

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

pub use ph2d_pigment::transfer;

// The standard sRGB EOTF and its inverse, plus the 0..255-domain doors the
// K–M and glaze sites use. ONE door each: every experimental site goes through
// these, and the tables live behind them so no call site can pick a different
// transfer than its neighbour.
pub use ph2d_pigment::{ks_of_srgb255, linear_to_srgb, srgb_to_linear, srgb255_of_linear};

// ── Kubelka–Munk: a lei MUDOU-SE para a folha [`ph2d_pigment`] (ordem do dono 2026-09-20, «os três
// meios passam a misturar igual»). Ela não mudou uma linha de matemática — mudou de ENDEREÇO, para
// que o Digital e a Aquarela leiam esta e não uma cópia. Este motor delega, logo continua
// byte-idêntico; ⛔ nada de reescrever `ColorMix` aqui, que é a segunda cópia que diverge. ──
pub use ph2d_pigment::{ColorMix, reflectance_of_ks};

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
            // One door per corner per channel: the `/255` rescale rides inside
            // the table's index instead of costing a division each.
            ks += weights[k] * ks_of_srgb255(colors[k * 3 + ch]);
        }
        out[ch] = srgb255_of_linear(reflectance_of_ks(ks * inv_total));
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
    fn km_mix_endpoints_are_bit_exact_at_any_colour() {
        // A weight of 0 returns dst and a weight of 1 returns src, to the
        // BIT, for every colour — including BELOW the 1/255 reflectance floor,
        // where the K/S round trip provably cannot recover the input (every
        // colour there shares one K/S). The endpoints do not go through K/S at
        // all, so the floor is irrelevant to them; asserting equality rather
        // than a tolerance is what pins that, and it is what keeps a pass that
        // deposits nothing from tinting paint it did not touch.
        let mut out = [0.0; 3];
        for (d, s) in [
            ([200.0, 90.0, 60.0], [40.0, 120.0, 180.0]),
            ([3.0, 1.0, 0.0], [255.0, 254.0, 250.0]), // both ends, floor included
        ] {
            ColorMix::Km.mix(d[0], d[1], d[2], s[0], s[1], s[2], 0.0, &mut out);
            assert_eq!(out, d, "w=0 must return dst unchanged");
            ColorMix::Km.mix(d[0], d[1], d[2], s[0], s[1], s[2], 1.0, &mut out);
            assert_eq!(out, s, "w=1 must return src exactly");
        }
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
