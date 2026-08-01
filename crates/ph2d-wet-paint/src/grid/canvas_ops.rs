//! **AS AÇÕES DE FOLHA INTEIRA** — filho de [`super`] (teto de LOC da
//! workspace), cortado por RESPONSABILIDADE: lá mora *o que a grade É* (os
//! planos, a bbox, a faixa viva, o snapshot); aqui, *o que uma ação do artista
//! sobre a FOLHA TODA faz com ela* — molhar, secar, limpar.
//!
//! As três compartilham a mesma forma, e é isso que as agrupa: percorrem a
//! folha inteira e não um retângulo, então quem as chama marca o canvas sujo
//! por COMPLETO (o composite seguinte não tem cache anterior a remendar).

use super::*;

/// Wetness byte a given paper tooth stamps: valleys (low paper) read wetter.
#[inline]
pub fn wet_byte_from_paper(paper_value: f64) -> u8 {
    let mut v = 2.0 - 2.0 * paper_value;
    if v > 1.0 {
        v = 1.0;
    }
    if v < 0.0 {
        v = 0.0;
    }
    (v * 255.0) as u8
}

// ---------------------------------------------------------------------------
// Canvas-wide actions (SPEC §12)
// ---------------------------------------------------------------------------

/// Wet canvas: raise the wetness byte to the paper-derived value via max over
/// the whole interior. Injects NO water and touches no bbox — the sim stays
/// idle, but subsequent strokes bleed everywhere and show-wet reads damp.
pub fn wet_canvas(g: &mut Grid) {
    let s = g.s;
    for y in 1..=g.h {
        let mut i = 1 + y * s;
        for _x in 1..=g.w {
            let b = wet_byte_from_paper(g.paper[i] as f64);
            if b > g.wet[i] {
                g.wet[i] = b;
            }
            i += 1;
        }
    }
}

/// Dry canvas: one-shot O(area) — settle every cell's suspended mass into the
/// settled layer (opacity-composite color, same as the dry pass), zero water,
/// both velocity fields and wetness, and empty the bbox.
pub fn dry_canvas(g: &mut Grid, mix: ColorMix) {
    let s = g.s;
    let mut out = [0.0f64; 3];
    for y in 1..=g.h {
        let mut i = 1 + y * s;
        for _x in 1..=g.w {
            let dm = g.susp[i] as f64;
            if dm > 0.0 {
                settle_composite(g, i, dm, mix, &mut out);
                g.sett[i] = (g.sett[i] as f64 + dm) as f32;
                g.susp[i] = 0.0;
            }
            g.film[i] = 0.0;
            g.wet[i] = 0;
            i += 1;
        }
    }
    // A velocidade mora na grade de FLUXO — mesmo interior, outra régua.
    let fg = g.flow;
    for cy in 1..=fg.h {
        let mut i = 1 + cy * fg.s;
        for _cx in 1..=fg.w {
            g.vel_x[i] = 0.0;
            g.vel_y[i] = 0.0;
            g.flow_x[i] = 0.0;
            g.flow_y[i] = 0.0;
            i += 1;
        }
    }
    g.empty_bbox();
}

/// Opacity-composite `dm` of suspended pigment into the settled layer's color
/// at cell i (SPEC §6.2 step 3). NOT mass-weighted: coverage-weighted, so a
/// thin new glaze barely shifts an already-opaque settled color.
#[inline]
pub fn settle_composite(g: &mut Grid, i: usize, dm: f64, mix: ColorMix, out: &mut [f64; 3]) {
    let a_sett = alpha_of_mass(g.sett[i] as f64);
    let a_in = alpha_of_mass(dm);
    if a_sett > 0.0 {
        let u = a_sett * (1.0 - a_in);
        let w = a_in / (u + a_in);
        let sc = g.sett_rgb[i];
        let uc = g.susp_rgb[i];
        mix.mix(
            sc[0] as f64,
            sc[1] as f64,
            sc[2] as f64,
            uc[0] as f64,
            uc[1] as f64,
            uc[2] as f64,
            w,
            out,
        );
        g.sett_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
    } else {
        g.sett_rgb[i] = g.susp_rgb[i];
    }
}

/// Clear: zero all dynamic state; the paper is untouched.
pub fn clear_canvas(g: &mut Grid) {
    g.film.fill(0.0);
    g.susp.fill(0.0);
    g.susp_rgb.fill([0.0; 3]);
    g.sett.fill(0.0);
    g.sett_rgb.fill([0.0; 3]);
    g.vel_x.fill(0.0);
    g.vel_y.fill(0.0);
    g.flow_x.fill(0.0);
    g.flow_y.fill(0.0);
    g.wet.fill(0);
    g.active.fill(0);
    g.bloom.fill(0);
    g.empty_bbox();
    // Aqui a faixa PODE ser zerada: esta é a porta que zerou a velocidade.
    g.clear_spans();
}
