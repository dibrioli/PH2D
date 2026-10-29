//! O VÉU do Show-wet sobre o composite pronto (filho de [`super`], LOC cap de [`super::composite`]).

/// Show-wet veil (doc 22 §2.7 — the model's damp overlay, adapted from an
/// opaque sheet to a LAYER): a cool translucent slate composited over the
/// result wherever the sheet is damp, so wetness reads over paint AND over
/// nothing; the meniscus glint keeps the model's rim law. Display-only —
/// the end-session door recomposites clean, so none of this ever bakes.
const WET_VEIL_ALPHA: f64 = 0.18;
const WET_VEIL_RGB: [f64; 3] = [56.0, 80.0, 104.0];
/// The model's light direction (top-left) for the meniscus glint.
const WET_LIGHT_X: f64 = -std::f64::consts::FRAC_1_SQRT_2;
const WET_LIGHT_Y: f64 = -std::f64::consts::FRAC_1_SQRT_2;

/// O véu sobre `canvas` (largura `w`) na caixa de pixels; `g` = a grade ATIVA do motor, `rr` = a razão.
pub(super) fn pinta_o_veu(
    g: &ph2d_wet_paint::grid::Grid,
    canvas: &mut [u8],
    (w, rr): (usize, usize),
    (gw, gh): (usize, usize),
    (px0, py0, px1, py1): (usize, usize, usize, usize),
) {
    // Show-wet: the damp overlay, display-only (never baked). Reads
    // the ACTIVE grid's wetness byte + film exactly like the model;
    // the veil is composited OVER the finished pixel so it shows on
    // painted and unpainted texels alike (an inspection overlay may
    // override the alpha-lock's look — the bake never carries it).
    let sg = g.s;
    // ⚠️ O véu percorre PIXELS e resolve a célula pelo vizinho mais
    // próximo — deliberadamente SEM o bilinear do pigmento. O menisco
    // é um gradiente do filme MEDIDO na grade (`film[i±1]`), e
    // interpolá-lo desenharia uma crista que o solver não tem: o véu
    // é uma anotação de *"aqui está molhado"*, não uma medição de cor.
    // Na razão 1 cada pixel É a sua célula e isto é byte-idêntico.
    for py in py0..py1 {
        let cy = (py / rr + 1).min(gh);
        for px in px0..px1 {
            let cx = (px / rr + 1).min(gw);
            let i = cx + cy * sg;
            let film = f64::from(g.film[i]);
            let mut wet_sig = f64::from(g.wet[i]) / 255.0;
            if film > 0.02 {
                let t = (film / 2.5).min(1.0);
                let sv = t * t * (3.0 - 2.0 * t);
                if sv > wet_sig {
                    wet_sig = sv;
                }
            }
            if wet_sig <= 0.01 {
                continue;
            }
            let o = (py * w + px) * 4;
            let a_v = WET_VEIL_ALPHA * wet_sig;
            let ca = f64::from(canvas[o + 3]) / 255.0;
            let oa = a_v + ca * (1.0 - a_v);
            // Meniscus glint — the model's rim shine, additive.
            let gx = (f64::from(g.film[i + 1]) - f64::from(g.film[i - 1])) * 0.5;
            let gy = (f64::from(g.film[i + sg]) - f64::from(g.film[i - sg])) * 0.5;
            let mag = (gx * gx + gy * gy).sqrt();
            let shine = if mag > 0.01 {
                let rim = (mag * 0.7).min(1.0);
                18.3 * rim
                    * rim
                    * (0.5 + 0.5 * ((gx / mag) * WET_LIGHT_X + (gy / mag) * WET_LIGHT_Y))
            } else {
                0.0
            };
            for ch in 0..3 {
                let cc = f64::from(canvas[o + ch]);
                let mixed = (WET_VEIL_RGB[ch] * a_v + cc * ca * (1.0 - a_v)) / oa.max(1e-6) + shine;
                canvas[o + ch] = mixed.round().clamp(0.0, 255.0) as u8;
            }
            canvas[o + 3] = (oa * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
}
