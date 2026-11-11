//! O DEPÓSITO de um texel: o tecto do traço (máscara por-traço), a força u16, o blend e o encode. Fase 3 de
//! [`super::stamp_band`]; devolve se escreveu o texel.

use super::tinta::Tinta;
use super::{DabCtx, Invariantes};
use crate::dab::{encode, stamp_rgba};

/// Onde o texel mora: `i` no `dst` (RGBA) e `mi` na máscara / na força (um byte / um u16 por texel).
pub(in crate::dab) struct Onde {
    pub(in crate::dab) i: usize,
    pub(in crate::dab) mi: usize,
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
pub(in crate::dab) fn pousa(
    ctx: &DabCtx,
    inv: &Invariantes,
    dst: &mut [u8],
    mask: Option<&mut [u8]>,
    forca: Option<&mut [u16]>,
    onde: Onde,
    w: f32,
    maior_sub: f32,
    tinta: Tinta,
) -> bool {
    let Tinta {
        color,
        stamp_alpha,
        g,
    } = tinta;
    let i = onde.i;
    let prev = [
        f32::from(dst[i]) / 255.0,
        f32::from(dst[i + 1]) / 255.0,
        f32::from(dst[i + 2]) / 255.0,
        f32::from(dst[i + 3]) / 255.0,
    ];
    // Mode TextureAlpha stamps the ramp as an RGBA image so translucent areas LOWER the
    // sprite's own alpha (punch it transparent) — alpha-lock can't apply (it edits alpha).
    // The other modes gate coverage by the dest alpha when alpha-locked + blend with the brush
    // mode (colour blend modes keep the dest alpha; you only paint where there's coverage).
    let out = if inv.texture_alpha {
        let m = w * g * ctx.coverage;
        if m <= 0.0 {
            return false;
        }
        stamp_rgba(prev, color, stamp_alpha, m)
    } else {
        let a = match mask {
            // The per-stroke coverage buffer answers "how much of THIS stroke's paint is here?",
            // and its law lives in ONE place ([`crate::stroke_cover::cover_add`]): the
            // Accumulate-OFF cap, where the dab profile is a RATE toward the ceiling. Whatever
            // the numbers, `a = add/(1 − m)` makes the canvas hold exactly
            // `pre_stroke·(1 − m) + colour·m` — the deposit lands on the state the stroke
            // started from, by telescoping and with no copy of the canvas (see the module).
            // At `cap = 1` this reduces to the EXACT maskless alpha sequence, so a full-strength
            // interior is byte-identical whether the buffer is threaded or not.
            Some(m_buf) => {
                let mi = onde.mi;
                let m = f32::from(m_buf[mi]) / 255.0;
                // O Wash pára no MAIOR dab: na cápsula, o maior sub-dab.
                let add = if ctx.spec.wash {
                    let w = if ctx.capsula.is_some() { maior_sub } else { w };
                    crate::stroke_cover::cover_max(m, w, g, ctx.tecto, ctx.flow)
                } else {
                    crate::stroke_cover::cover_add(m, w, g, ctx.tecto, inv.flow_do_tecto, inv.area)
                };
                let Some(add) = add else {
                    return false; // this dab adds nothing here
                };
                m_buf[mi] = ((m + add) * 255.0 + 0.5) as u8;
                let a = add / (1.0 - m).max(1e-4);
                if ctx.preserve_alpha { a * prev[3] } else { a }
            }
            None if ctx.preserve_alpha => w * g * ctx.coverage * prev[3],
            None => w * g * ctx.coverage,
        };
        if a <= 0.0 {
            return false;
        }
        if let Some(f) = forca {
            crate::stroke_cover::compoe_forca(&mut f[onde.mi], a);
        }
        crate::blend_vazio::blend_over_pigment_modo(
            ctx.spec.blend,
            prev,
            color,
            a,
            ctx.spec.pigmento(),
            ctx.spec.blend_sobre_vazio,
        )
    };
    dst[i] = encode(out[0]);
    dst[i + 1] = encode(out[1]);
    dst[i + 2] = encode(out[2]);
    dst[i + 3] = encode(out[3]);
    true
}
