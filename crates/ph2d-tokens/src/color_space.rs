//! ⭐⭐ **O ESPAÇO DE COR — OKLCH ⇄ sRGB**, irmão do [`crate::color`] pelo tecto de LOC, com o corte
//! por RESPONSABILIDADE: ali mora *que tokens existem e como um tema os resolve*, aqui **a
//! aritmética de cor**, que não sabe o que é um token nem um tema.
//!
//! ⚠️ **O endereço não mudou:** a [`crate`] re-exporta as quatro funções públicas daqui, e os
//! consumidores escrevem `ph2d_tokens::oklch_to_srgb` exactamente como antes.
//!
//! Porte do algoritmo do Björn Ottosson (<https://bottosson.github.io/posts/oklab/>).

/// Convert OKLCH → sRGB 8-bit per Björn Ottosson's algorithm
/// (https://bottosson.github.io/posts/oklab/). Out-of-gamut colors
/// are clamped per channel.
///
/// `l` in 0..1, `c` ~0..0.4, `h_deg` in degrees.
pub fn oklch_to_srgb(l: f64, c: f64, h_deg: f64) -> [u8; 3] {
    let [lr, lg, lb] = oklch_to_linear_srgb(l, c, h_deg);
    [
        linear_to_srgb_byte(lr),
        linear_to_srgb_byte(lg),
        linear_to_srgb_byte(lb),
    ]
}

/// OKLCH → **linear** sRGB, NOT clamped. Returns the raw linear RGB so
/// callers can test gamut membership ([`oklch_in_gamut`]) before the
/// per-channel clamp that [`oklch_to_srgb`] applies. `l` in 0..1, `c`
/// ~0..0.4, `h_deg` in degrees.
pub fn oklch_to_linear_srgb(l: f64, c: f64, h_deg: f64) -> [f64; 3] {
    // OKLCH → OKLAB
    let h_rad = h_deg.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();

    // OKLAB → linear LMS (cube)
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    // LMS → linear sRGB
    [
        4.076_741_662_1 * l3 - 3.307_711_591_3 * m3 + 0.230_969_929_2 * s3,
        -1.268_438_004_6 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_5 * s3,
        -0.004_196_086_3 * l3 - 0.703_418_614_7 * m3 + 1.707_614_701_0 * s3,
    ]
}

/// True if OKLCH `(l, c, h_deg)` lands inside the sRGB gamut — i.e. all
/// three linear channels fall within `[0, 1]` (small epsilon) BEFORE
/// the clamp in [`oklch_to_srgb`]. Used by the color picker to scale
/// the Chroma slider against the maximum representable chroma for the
/// current lightness + hue (so the slider top is always reachable).
pub fn oklch_in_gamut(l: f64, c: f64, h_deg: f64) -> bool {
    const EPS: f64 = 1e-4;
    oklch_to_linear_srgb(l, c, h_deg)
        .iter()
        .all(|&v| (-EPS..=1.0 + EPS).contains(&v))
}

fn linear_to_srgb_byte(linear: f64) -> u8 {
    let x = linear.clamp(0.0, 1.0);
    let srgb = if x <= 0.003_130_8 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    (srgb.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Inverse of [`oklch_to_srgb`]: sRGB 8-bit → OKLCH (L, C, H_deg).
/// Used by [`ColorValue::from_rgba8`] to keep both representations in
/// sync. Per Björn Ottosson's reference. Output: L 0..1, C 0..~0.4,
/// H 0..360.
pub fn srgb_to_oklch(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let lr = srgb_byte_to_linear(r);
    let lg = srgb_byte_to_linear(g);
    let lb = srgb_byte_to_linear(b);
    // linear sRGB → LMS
    let l = 0.412_221_470_8 * lr + 0.536_332_536_3 * lg + 0.051_445_992_9 * lb;
    let m = 0.211_903_498_2 * lr + 0.680_699_545_1 * lg + 0.107_396_956_6 * lb;
    let s = 0.088_302_461_9 * lr + 0.281_718_837_6 * lg + 0.629_978_700_5 * lb;
    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();
    // LMS → OKLAB
    let lab_l = 0.210_454_255_3 * l_ + 0.793_617_785_0 * m_ - 0.004_072_046_8 * s_;
    let lab_a = 1.977_998_495_1 * l_ - 2.428_592_205_0 * m_ + 0.450_593_709_9 * s_;
    let lab_b = 0.025_904_037_1 * l_ + 0.782_771_766_2 * m_ - 0.808_675_766_0 * s_;
    let c = (lab_a * lab_a + lab_b * lab_b).sqrt();
    let mut h = lab_b.atan2(lab_a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    (lab_l, c, h)
}

fn srgb_byte_to_linear(channel: u8) -> f64 {
    let c = channel as f64 / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
