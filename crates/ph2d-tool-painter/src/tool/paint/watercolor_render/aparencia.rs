//! **A APARÊNCIA de um texel da aguada sobre um CHÃO** — a óptica (Beer–Lambert, o corpo, a mistura
//! sobre tinta seca, o AA da silhueta) como função do chão, cortada do [`super`] (o tecto de LOC, por
//! assunto) para o núcleo a avaliar duas vezes: sobre o branco de referência (o píxel da camada, ao
//! byte o de sempre) e sobre o preto (o que o pigmento devolve) — o par de Kubelka–Munk que dá o
//! VIDRO, a transparência de cada canal (doc 48, BUGS #46).

use super::super::watercolor_lut::{Luts, gamut_alpha};
use super::super::watercolor_rewet_px::apply_wet_lift;

/// O que a óptica de um texel sabe ANTES de ver o chão: o pigmento, a densidade óptica, o corpo, o
/// levantar da re-molhagem, os pesos da mistura (a lei do [`super::super::watercolor_mistura`]) e a
/// cobertura fraccionária da silhueta.
#[derive(Clone, Copy)]
pub(super) struct Optica {
    pub pig: [u8; 3],
    pub od: f32,
    pub body_cov: f32,
    pub lift: f32,
    pub pelo_botao: f32,
    pub pela_agua: f32,
    pub aa_alpha: f32,
}

/// A aparência sobre o chão, e o que o `un-premultiply` precisa dela.
pub(super) struct Aparencia {
    pub rgb: [u8; 3],
    pub t_min: f32,
    pub a_body: f32,
}

const LUM: [f32; 3] = [0.2126, 0.7152, 0.0722];

/// A aparência de `o` sobre o chão `chao` (bytes sRGB), com a base da camada já posta sobre esse
/// chão (`base_sobre`, em tons de ecrã `0..1`).
#[inline]
pub(super) fn aparencia(o: &Optica, base_sobre: [f32; 3], chao: [u8; 3], lut: &Luts) -> Aparencia {
    let ground_lin = [
        lut.s2l[chao[0] as usize],
        lut.s2l[chao[1] as usize],
        lut.s2l[chao[2] as usize],
    ];
    let ground_enc = chao.map(|g| f32::from(g) / 255.0);
    let mut sb = base_sobre.map(ph2d_color::srgb::srgb_to_linear_unit);
    // Wet-on-wet LIFT ([`apply_wet_lift`]): rewetting walks the base's pigment toward the
    // LOCAL ground in log space (`lift` already moisture-scaled — a dried spot won't lift).
    apply_wet_lift(&mut sb, &ground_lin, o.lift, lut);
    // BODY / OPACITY (doc 13 #17): pure Beer–Lambert only subtracts, so a light-valued pigment barely
    // deposits. Body lays the pigment's OWN colour over the transmittance result (`0` ⇒ no-op).
    let pig = o.pig;
    let mut rgb = [0u8; 3];
    let mut t_lum = 0.0f32;
    let mut t_min = 1.0f32;
    // Extra alpha the BODY-shifted appearance needs to stay in gamut (0 when body off): body can push a
    // channel PAST the `1 − t_min` floor the un-premultiply assumes ([`gamut_alpha`]).
    let mut a_body = 0.0f32;
    for c in 0..3 {
        let t = lut.transmittance(pig[c], o.od);
        let optical = sb[c] * t + lut.s2l[pig[c] as usize] * (1.0 - t);
        let lin = optical + (lut.s2l[pig[c] as usize] - optical) * o.body_cov;
        rgb[c] = lut.l2s_byte(lin);
        if o.body_cov > 0.0 {
            a_body = a_body.max(gamut_alpha(f32::from(rgb[c]) / 255.0, ground_enc[c]));
        }
        t_lum += LUM[c] * t;
        t_min = t_min.min(t);
    }
    // Perceptual film opacity — drives the subtractive paint-mix amount over the base paint.
    let film_a = (1.0 - t_lum).clamp(0.0, 1.0);
    let mix_amt = o.pelo_botao.max(o.pela_agua);
    if mix_amt > 0.0 {
        // The (possibly lifted) base APPEARANCE over the ground — the mix reads the LIFTED base.
        let mix_base = [
            f32::from(lut.l2s_byte(sb[0])) / 255.0,
            f32::from(lut.l2s_byte(sb[1])) / 255.0,
            f32::from(lut.l2s_byte(sb[2])) / 255.0,
        ];
        // ⭐ DUAS leis, uma por termo ([`super::super::watercolor_mistura::alvo_sobre_seco`]).
        let mixed = super::super::watercolor_mistura::alvo_sobre_seco(
            mix_base,
            [
                f32::from(pig[0]) / 255.0,
                f32::from(pig[1]) / 255.0,
                f32::from(pig[2]) / 255.0,
            ],
            film_a,
            o.pelo_botao,
            o.pela_agua,
        );
        for c in 0..3 {
            let sub = (mixed[c].clamp(0.0, 1.0) * 255.0 + 0.5).clamp(0.0, 255.0);
            rgb[c] = (f32::from(rgb[c]) + (sub - f32::from(rgb[c])) * mix_amt) as u8;
        }
    }
    // Screen-space AA: the wash's target APPEARANCE over the base-over-ground appearance by the
    // texel's fractional silhouette coverage, in tones of the screen, BEFORE the un-premultiply.
    if o.aa_alpha < 1.0 {
        for c in 0..3 {
            let app = f32::from(rgb[c]) / 255.0;
            let v = app * o.aa_alpha + base_sobre[c] * (1.0 - o.aa_alpha);
            rgb[c] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
    Aparencia { rgb, t_min, a_body }
}
