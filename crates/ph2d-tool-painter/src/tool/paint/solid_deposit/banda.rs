//! O kernel da mancha sólida: o `over` de uma linha e os dois walkers que o percorrem.

use ph2d_painter_brush::{Tecto as _, TexelLargo, recompoe_bytes};

/// O que uma linha do `over` da mancha precisa saber — agrupado porque o kernel tem **um corpo e
/// dois walkers** e uma lista de sete argumentos repetida nos dois é como eles divergem.
pub(in crate::tool::paint) struct SolidBand<'a> {
    pub(in crate::tool::paint) cov: &'a [u8],
    pub(in crate::tool::paint) cov_stride: usize,
    pub(in crate::tool::paint) row_bytes: usize,
    pub(in crate::tool::paint) x0: usize,
    pub(in crate::tool::paint) cols: usize,
    pub(in crate::tool::paint) rgb: [u32; 3],
    pub(in crate::tool::paint) strength: u32,
    /// `true` = a mancha obedece ao tecto do traço: `strength` é o tecto e a máscara da linha diz
    /// o `m` de cada texel (ver o `write_solid`).
    pub(in crate::tool::paint) no_tecto: bool,
}

/// **UM CORPO, DOIS WALKERS** — `par` escolhe quem percorre as linhas, nunca o que a linha responde.
///
/// ⚠️ **Não existe versão paralela do kernel**, e é isso que torna a identidade uma propriedade da
/// forma em vez de uma promessa: as duas rotas chamam [`blend_solid_row`]. O que um gate de
/// identidade PODE provar é o mapeamento `linha → y`; o corpo partilhado ele não vê
/// ([[feedback_an_identity_gate_cannot_see_a_defect_in_the_shared_body]]), e quem o cobre são os
/// gates de aparência da mancha.
pub(in crate::tool::paint) fn blend_solid_rows(
    band: &mut [u8],
    mask: Option<&mut [u8]>,
    b: SolidBand<'_>,
    par: bool,
) {
    let largura = b.row_bytes / 4;
    match (mask, par) {
        (Some(m), true) => {
            use rayon::prelude::*;
            band.par_chunks_mut(b.row_bytes)
                .zip(m.par_chunks_mut(largura))
                .enumerate()
                .for_each(|(row, (d, m))| blend_solid_row(row, d, Some(m), &b));
        }
        (Some(m), false) => band
            .chunks_mut(b.row_bytes)
            .zip(m.chunks_mut(largura))
            .enumerate()
            .for_each(|(row, (d, m))| blend_solid_row(row, d, Some(m), &b)),
        (None, true) => {
            use rayon::prelude::*;
            band.par_chunks_mut(b.row_bytes)
                .enumerate()
                .for_each(|(row, d)| blend_solid_row(row, d, None, &b));
        }
        (None, false) => band
            .chunks_mut(b.row_bytes)
            .enumerate()
            .for_each(|(row, d)| blend_solid_row(row, d, None, &b)),
    }
}

/// O `over` de UMA linha: cobertura exata × opacidade do pincel, sobre o que já está lá — ou, com
/// o tecto do traço (`mask` = a linha do `stroke_mask`), só o que falta de `m` até o alvo.
fn blend_solid_row(row: usize, dst: &mut [u8], mut mask: Option<&mut [u8]>, b: &SolidBand<'_>) {
    for cx in 0..b.cols {
        let alvo = u32::from(b.cov[row * b.cov_stride + cx]) * b.strength / 255;
        let a = match mask.as_deref_mut() {
            Some(m) if b.no_tecto => {
                let tem = u32::from(m[b.x0 + cx]);
                if alvo <= tem {
                    continue;
                }
                #[allow(clippy::cast_possible_truncation)]
                {
                    m[b.x0 + cx] = alvo as u8;
                }
                ((alvo - tem) * 255 + (255 - tem) / 2) / (255 - tem)
            }
            _ => alvo,
        };
        if a == 0 {
            continue;
        }
        let p = (b.x0 + cx) * 4;
        let da = u32::from(dst[p + 3]);
        if da < 255 {
            // O `over` de ALFA DIRECTO sobre um destino translúcido — a camada transparente do papel
            // (smoke do dono 2026-10-06): a cor pesa pelo alfa de cada lado, ou a tinta escurecia
            // misturada com o preto transparente por baixo (`[132, 18, 18]` contra `[220, 30, 30]`).
            let (wa, wd) = (a * 255, da * (255 - a));
            for ch in 0..3 {
                let d = u32::from(dst[p + ch]);
                #[allow(clippy::cast_possible_truncation)]
                {
                    dst[p + ch] = ((b.rgb[ch] * wa + d * wd + (wa + wd) / 2) / (wa + wd)) as u8;
                }
            }
            #[allow(clippy::cast_possible_truncation)]
            {
                dst[p + 3] = ((wa + wd + 127) / 255) as u8;
            }
            continue;
        }
        for ch in 0..3 {
            let d = u32::from(dst[p + ch]);
            #[allow(clippy::cast_possible_truncation)]
            {
                dst[p + ch] = ((b.rgb[ch] * a + d * (255 - a)) / 255) as u8;
            }
        }
        #[allow(clippy::cast_possible_truncation)]
        {
            dst[p + 3] = (a + da * (255 - a) / 255) as u8;
        }
    }
}

/// B29.1: a mancha sob o tecto LARGO — o mesmo alvo `cobertura × tecto`, mas `m` em `u16` e a tela RECOMPOSTA
/// do pré-traço (`pré·(1 − m) + cor·m`, [`ph2d_painter_brush::recompoe_bytes`]): a lei do dab no tecto largo.
/// O mesmo par de walkers (`par` muda quem percorre, nunca o que a linha responde).
pub(in crate::tool::paint) fn blend_solid_rows_largo(
    band: &mut [u8],
    mask: &mut [TexelLargo],
    b: &SolidBand<'_>,
    cor: [f32; 3],
    par: bool,
) {
    let largura = b.row_bytes / 4;
    if par {
        use rayon::prelude::*;
        band.par_chunks_mut(b.row_bytes)
            .zip(mask.par_chunks_mut(largura))
            .enumerate()
            .for_each(|(row, (d, m))| blend_solid_row_largo(row, d, m, b, cor));
    } else {
        band.chunks_mut(b.row_bytes)
            .zip(mask.chunks_mut(largura))
            .enumerate()
            .for_each(|(row, (d, m))| blend_solid_row_largo(row, d, m, b, cor));
    }
}

fn blend_solid_row_largo(
    row: usize,
    dst: &mut [u8],
    mask: &mut [TexelLargo],
    b: &SolidBand<'_>,
    cor: [f32; 3],
) {
    for cx in 0..b.cols {
        #[allow(clippy::cast_precision_loss)]
        let alvo =
            (u32::from(b.cov[row * b.cov_stride + cx]) * b.strength) as f32 / (255.0 * 255.0);
        let t = &mut mask[b.x0 + cx];
        if alvo <= t.m() {
            continue;
        }
        let p = (b.x0 + cx) * 4;
        let px = [dst[p], dst[p + 1], dst[p + 2], dst[p + 3]];
        let Some(pre) = t.grava(alvo, px) else {
            continue;
        };
        dst[p..p + 4].copy_from_slice(&recompoe_bytes(pre, cor, t.m()));
    }
}
