//! O laço de píxel do composite molhado como função PURA das entradas (B11-cont3, passo do B11.3): lê só os planos,
//! a base congelada, os gates e o vidro da base, e escreve as linhas da caixa. Nada da ferramenta nem do motor: quem
//! o chama no fio do quadro ([`super::super::composite`]) e quem o chamar no compositor dão o mesmo byte.

use super::super::adormecida::Congelada;
use super::super::composite_oleo::SemOleo;
use super::super::grao_no_filme::GraoNoFilme;
use super::super::grid_map::SampleU;
use super::super::smooth_edges::Suaves;
use crate::compositor::vidro::Vidro;
use rayon::prelude::*;

/// Tudo o que o laço de píxel lê (emprestado).
#[derive(Clone, Copy)]
pub(in crate::tool::paint::wetpaint) struct LacoDePixel<'a> {
    /// Largura da tela em píxeis.
    pub(in crate::tool::paint::wetpaint) w: usize,
    /// `(px0, py0, px1, py1)`.
    pub(in crate::tool::paint::wetpaint) caixa_px: (usize, usize, usize, usize),
    pub(in crate::tool::paint::wetpaint) grade: (usize, usize),
    pub(in crate::tool::paint::wetpaint) rr: usize,
    pub(in crate::tool::paint::wetpaint) sampler: SampleU,
    pub(in crate::tool::paint::wetpaint) pigment: &'a [u8],
    pub(in crate::tool::paint::wetpaint) base: &'a [u8],
    pub(in crate::tool::paint::wetpaint) sombra: &'a [u8],
    pub(in crate::tool::paint::wetpaint) grao_cel: &'a [u8],
    pub(in crate::tool::paint::wetpaint) congelada: Option<&'a Congelada>,
    pub(in crate::tool::paint::wetpaint) suavizada: Option<&'a Suaves>,
    pub(in crate::tool::paint::wetpaint) sem_oleo: &'a SemOleo,
    pub(in crate::tool::paint::wetpaint) orla: bool,
    pub(in crate::tool::paint::wetpaint) grao: Option<&'a GraoNoFilme<'a>>,
    pub(in crate::tool::paint::wetpaint) gsel: Option<&'a [u8]>,
    pub(in crate::tool::paint::wetpaint) gprot: Option<&'a [u8]>,
    pub(in crate::tool::paint::wetpaint) alock_on: bool,
    pub(in crate::tool::paint::wetpaint) modo: ph2d_painter_brush::BrushBlend,
    pub(in crate::tool::paint::wetpaint) vazio: bool,
    pub(in crate::tool::paint::wetpaint) exata: bool,
    pub(in crate::tool::paint::wetpaint) vbase:
        Option<&'a crate::tool::paint::watercolor_vidro::JanelaDoVidro>,
}

/// Escreve `linhas` (as linhas `py0..py1` da tela, `w * 4` bytes cada), o vidro de cada linha e a cobertura da marca.
///
/// ROW-PARALLEL pelo ADR-0109: cada linha lê os planos no seu próprio offset e escreve só o seu; nenhuma lê a saída
/// de outra. Byte-idêntico por construção (muda qual fio avalia qual linha, nunca o que a linha diz).
pub(in crate::tool::paint::wetpaint) fn laco_de_pixel(
    l: &LacoDePixel<'_>,
    linhas: &mut [u8],
    vlinhas: Vec<Option<&mut [Vidro]>>,
    cob_linhas: Vec<Option<&mut [u16]>>,
) {
    let LacoDePixel {
        w,
        caixa_px: (px0, py0, px1, _),
        grade: (gw, gh),
        rr,
        sampler,
        pigment,
        base,
        sombra,
        grao_cel,
        congelada,
        suavizada,
        sem_oleo,
        orla,
        grao,
        gsel,
        gprot,
        alock_on,
        modo,
        vazio,
        exata,
        vbase,
    } = *l;
    let stride = w * 4;
    let gate_on = gsel.is_some() || gprot.is_some();
    let one_cell_per_pixel = sampler.is_identity();
    linhas
        .par_chunks_mut(stride)
        .zip(vlinhas.into_par_iter())
        .zip(cob_linhas.into_par_iter())
        .enumerate()
        .for_each(|(k, ((row, mut vrow), mut crow))| {
            let py = py0 + k;
            let gb = py * stride;
            // A metade-`y` do amostrador, resolvida UMA vez por linha.
            let srow = sampler.row(py);
            for px in px0..px1 {
                let lo = px * 4; // dentro da linha
                let o = gb + lo; // global, para os planos e os gates
                if let Some(p) = congelada.and_then(|f| f.pixel((px, py, o), pigment, sombra, base))
                {
                    row[lo..lo + 4].copy_from_slice(p);
                    continue;
                }
                // O pigmento, em premultiplicado normalizado (`pc * pa`) — a forma em que o `over` abaixo o consome.
                let dele = if one_cell_per_pixel {
                    super::super::smooth_edges::premul_da_celula(pigment, o)
                } else {
                    super::super::smooth_edges::premul_amostrado(srow.at(pigment, px))
                };
                // Smooth Edges: o pigmento que o pixel mostra, com a cobertura da borda alisada (a fração na
                // COBERTURA premultiplicada — exacta também sobre uma camada transparente).
                let (premul, pa) = match (suavizada, sem_oleo) {
                    (Some(sv), Some((sx, sy, sw, so))) => {
                        let (cx, cy) = ((px / rr).min(gw - 1) + 1, (py / rr).min(gh - 1) + 1);
                        let sem = cx >= *sx
                            && cy >= *sy
                            && so.get((cy - sy) * sw + cx - sx).copied().unwrap_or(false);
                        sv.aplica_com(px, py, dele, orla || sem)
                    }
                    (Some(sv), None) => sv.aplica(px, py, dele),
                    (None, _) => dele,
                };
                // W3b: o dente do papel na tinta (Relevo do papel, Transmission); `d = 0` ao bit.
                let premul = match grao {
                    Some(gf) if pa > 0.0 && !grao_cel.is_empty() => {
                        let d = grao_cel[(py / rr).min(gh - 1) * gw + (px / rr).min(gw - 1)];
                        if d == 0 {
                            premul
                        } else {
                            let k = pa * 255.0;
                            gf.sombreia(px, py, f32::from(d) / 255.0, premul.map(|v| v / k))
                                .map(|v| v * k)
                        }
                    }
                    _ => premul,
                };
                let escuro = if !sombra.is_empty() {
                    // §BUG-bordas-W3b: sem o furo de uma célula (o pontinho escuro da orla).
                    ph2d_wet_paint::oleo_real::sombra_sem_furo(
                        sombra,
                        (gw, gh),
                        ((px / rr).min(gw - 1), (py / rr).min(gh - 1)),
                    )
                } else {
                    0
                };
                // A sombra do Óleo Real sobre o que está por baixo: preto por cima, com a fracção `escuro` — pelas
                // mesmas gates que a tinta (selecção/protecção, alfa preso). Sob a orla da tinta ela é o papel sobre
                // o qual o pigmento se compõe.
                let sombreada: [u8; 4];
                let mut cobre_a_sombra = 0.0;
                let base_px: &[u8] = if escuro > 0 {
                    let mut s = f32::from(escuro) / 255.0;
                    if gate_on {
                        s *= super::super::super::watercolor_accum::splat_keep(
                            gsel,
                            gprot,
                            None,
                            o / 4,
                        );
                    }
                    cobre_a_sombra = s;
                    let ba = f32::from(base[o + 3]) / 255.0;
                    let oa = if alock_on { ba } else { s + ba * (1.0 - s) };
                    let mut b = [0u8; 4];
                    for ch in 0..3 {
                        let bc = f32::from(base[o + ch]);
                        b[ch] = if oa > 0.0 {
                            (bc * ba * (1.0 - s) / oa).round().clamp(0.0, 255.0) as u8
                        } else {
                            0
                        };
                    }
                    b[3] = (oa * 255.0).round() as u8;
                    if pa <= 0.0 {
                        let igual = row[lo..lo + 4] == b;
                        row[lo..lo + 4].copy_from_slice(&b);
                        // O vidro da aguada por baixo: o preto da sombra cobre cada canal por igual.
                        if let (Some(v), Some(vb), false) = (vrow.as_deref_mut(), vbase, igual) {
                            let antes =
                                crate::compositor::vidro::alfas(&base[o..o + 4], &vb[o / 4]);
                            v[px] = crate::compositor::vidro::sela(
                                b,
                                if alock_on {
                                    antes
                                } else {
                                    crate::compositor::vidro::cobre(antes, s)
                                },
                            );
                        }
                        continue;
                    }
                    sombreada = b;
                    &sombreada
                } else {
                    &base[o..o + 4]
                };
                if pa <= 0.0 {
                    // Base verbatim — the gates keep-lerp TOWARD the base, so they are exact no-ops on this branch.
                    let igual = row[lo..lo + 4] == base[o..o + 4];
                    row[lo..lo + 4].copy_from_slice(&base[o..o + 4]);
                    if let (Some(v), Some(b), false) = (vrow.as_deref_mut(), vbase, igual) {
                        v[px] = b[o / 4];
                    }
                    continue;
                }
                let mut cobertura = pa;
                if modo == ph2d_painter_brush::BrushBlend::Mix {
                    let ba = base_px[3] as f32 / 255.0;
                    let oa = pa + ba * (1.0 - pa);
                    for ch in 0..3 {
                        let bc = base_px[ch] as f32;
                        row[lo + ch] = ((premul[ch] + bc * ba * (1.0 - pa)) / oa).round() as u8;
                    }
                    row[lo + 3] = (oa * 255.0).round() as u8;
                } else {
                    // O modo da SESSÃO pela lei do traço digital (`blend_over`, sRGB straight): a cor straight do
                    // pigmento sobre a base, pela cobertura `pa`.
                    let inv = 1.0 / (pa * 255.0);
                    let cor = [premul[0] * inv, premul[1] * inv, premul[2] * inv];
                    let dst = [
                        f32::from(base_px[0]) / 255.0,
                        f32::from(base_px[1]) / 255.0,
                        f32::from(base_px[2]) / 255.0,
                        f32::from(base_px[3]) / 255.0,
                    ];
                    let out = ph2d_painter_brush::blend_over_modo(modo, dst, cor, pa, vazio);
                    for c in 0..4 {
                        row[lo + c] = (out[c] * 255.0).round().clamp(0.0, 255.0) as u8;
                    }
                }
                if gate_on {
                    let keep =
                        super::super::super::watercolor_accum::splat_keep(gsel, gprot, None, o / 4);
                    if keep < 1.0 {
                        cobertura *= keep;
                        for c in 0..4 {
                            let painted = f32::from(row[lo + c]);
                            let orig = f32::from(base[o + c]);
                            row[lo + c] = (painted * keep + orig * (1.0 - keep))
                                .round()
                                .clamp(0.0, 255.0) as u8;
                        }
                    }
                }
                if alock_on {
                    // Alpha-lock: the α reference IS `sess.base` (the gate's buffer, served by `wet_splat_gates`'
                    // wet-session arm).
                    row[lo + 3] = base[o + 3];
                }
                if let Some(c) = crow.as_deref_mut() {
                    c[px - px0] = if exata {
                        (cobertura.clamp(0.0, 1.0) * f32::from(u16::MAX)).round() as u16
                    } else {
                        u16::MAX
                    };
                }
                if let (Some(v), Some(b)) = (vrow.as_deref_mut(), vbase)
                    && alock_on
                {
                    // Com o alfa preso à base a cobertura não muda: o texel novo, os alfas da base.
                    let px4 = [row[lo], row[lo + 1], row[lo + 2], row[lo + 3]];
                    v[px] = crate::compositor::vidro::sela(
                        px4,
                        crate::compositor::vidro::alfas(&base[o..o + 4], &b[o / 4]),
                    );
                } else if let (Some(v), Some(b)) = (vrow.as_deref_mut(), vbase) {
                    let antes = crate::compositor::vidro::alfas(&base[o..o + 4], &b[o / 4]);
                    let px4 = [row[lo], row[lo + 1], row[lo + 2], row[lo + 3]];
                    v[px] = crate::compositor::vidro::sela(
                        px4,
                        crate::compositor::vidro::cobre(
                            crate::compositor::vidro::cobre(antes, cobre_a_sombra),
                            cobertura,
                        ),
                    );
                }
            }
        });
}
