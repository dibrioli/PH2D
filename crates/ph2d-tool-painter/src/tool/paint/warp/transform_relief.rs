//! **O corpo da tinta viaja com o patch flutuante** — a metade do relevo que a W4 não cobriu.
//!
//! A W4 (2026-07-15) deu ao Deform a advecção de `heights`/`covers`/`mats`, e a deu pelo `disp`
//! cumulativo da sessão — que só existe na metade das PINCELADAS (Push · Twist · Pinch · Wrinkle ·
//! Fold · Reconstruct). A metade do GIZMO não tem `disp`: ela **levanta** um patch e o compõe sobre a
//! base. Resultado medido em 2026-08-08 (report do Enio): um Transform movia a tinta 10 px e o corpo
//! **0,00 px** — a luz sombreando uma crista de tinta que não estava mais lá.
//!
//! ## O desenho é o ESPELHO do irmão de cor, e por obrigação
//!
//! O `relief.rs` diz a lei: *corpo e cor não podem divergir sobre "para onde foi"*. A cor do Transform
//! é `out = over(patch(m⁻¹·dst), base)`; o corpo é a MESMA frase com **cobertura no lugar do alfa** —
//! a mesma `m⁻¹`, a mesma bbox, os mesmos samplers (importados de [`super::relief`], que é o que
//! garante uma política de clamp só). Um segundo resample com política própria é como o corpo e a cor
//! começam a discordar num sítio que ninguém compara.
//!
//! ## O que é levantado, e por que quase nada é copiado
//!
//! O levante da COR parte o alfa entre patch e base (`a·m` e `a·(1−m)`). Para o corpo, só a
//! **cobertura** se parte assim: `heights` e `mats` não são quantidades, são propriedades DA tinta —
//! metade da cobertura de um texel não é meia altura, é a mesma altura com metade da tinta. Então os
//! dois lados compartilham `h` e `m` por `Arc::clone`, e só a cobertura tem duas versões.
//!
//! ⚠️ **No caso comum (sem seleção) isso custa ZERO alocação:** `c_patch` é um `Arc::clone` e
//! `c_base` é `None` (o buraco é total). Só um Transform restrito a uma seleção materializa os dois.

use super::super::Region;
use super::super::impasto_ceiling::H_MAX;
use super::relief::{bilinear_f32, bilinear_mat, bilinear_u8};
use super::transform_geom::Mat3;
use crate::tool::PainterTool;
use crate::tool::RtLayerId;
use ph2d_painter_brush::material::MaterialBytes;
use std::sync::Arc;

/// Os planos de impasto congelados no levante do patch — o gêmeo de [`super::transform_float::FloatingPatch`]
/// para o corpo da tinta.
#[derive(Clone)]
pub(crate) struct FloatingRelief {
    /// A camada que estes planos descrevem. Bits de camada são reciclados, então o render confere que
    /// ela ainda é a ativa antes de vestir este corpo em outra (o mesmo guard do `warp_render_relief`).
    pub layer: RtLayerId,
    /// Alturas congeladas no levante. **Compartilhada** pelos dois lados: a altura é propriedade da
    /// tinta, não uma quantidade a repartir.
    pub h: Arc<Vec<f32>>,
    /// Materiais congelados no levante. Compartilhado pelo mesmo motivo.
    pub m: Arc<Vec<MaterialBytes>>,
    /// A cobertura que o PATCH levou (`c·mask`). `Arc::clone` da original quando não há seleção.
    pub c_patch: Arc<Vec<u8>>,
    /// A cobertura que ficou na BASE (`c·(1−mask)`). `None` quando o levante é da camada inteira — aí
    /// o buraco é total, e um plano de zeros seria uma alocação para dizer nada.
    pub c_base: Option<Arc<Vec<u8>>>,
}

impl PainterTool {
    /// Congelar os planos de impasto para o patch que o [`PainterTool::begin_transform`] acabou de
    /// levantar. `None` quando a camada ativa não tem relevo — não há corpo a carregar, e um
    /// `FloatingRelief` vazio faria o render trabalhar para escrever o que já está lá.
    ///
    /// `mask` é a cobertura da seleção quando o Transform está restrito a ela, `None` para a camada
    /// inteira.
    pub(super) fn lift_transform_relief(
        &self,
        mask: Option<&dyn Fn(u32, u32) -> u8>,
    ) -> Option<FloatingRelief> {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let layer = self.layers.active()?;
        let heights = self.heights.get(&layer).filter(|v| v.len() == n)?;
        let covers = self.covers.get(&layer).filter(|v| v.len() == n)?;
        let mats = self.mats.get(&layer).filter(|v| v.len() == n)?;
        let (c_patch, c_base) = match mask {
            // Camada inteira: o patch leva TODA a cobertura e a base fica sem nenhuma.
            None => (Arc::clone(covers), None),
            Some(mask) => {
                let mut cp = vec![0u8; n];
                let mut cb = vec![0u8; n];
                for y in 0..h {
                    for x in 0..w {
                        let gi = (y * w + x) as usize;
                        let mk = f32::from(mask(x, y)) / 255.0;
                        let c = f32::from(covers[gi]);
                        // A MESMA partição que o alfa da cor faz — patch `c·m`, base `c·(1−m)` —, que é
                        // o que faz a identidade reconstruir a cobertura de origem.
                        cp[gi] = (c * mk).round().clamp(0.0, 255.0) as u8;
                        cb[gi] = (c * (1.0 - mk)).round().clamp(0.0, 255.0) as u8;
                    }
                }
                (Arc::new(cp), Some(Arc::new(cb)))
            }
        };
        Some(FloatingRelief {
            layer,
            h: Arc::clone(heights),
            m: Arc::clone(mats),
            c_patch,
            c_base,
        })
    }

    /// Re-compor os planos de impasto sob o mesmo mapa `minv` que a cor acabou de usar, sobre a mesma
    /// `dirty`, com a mesma `affected` — os três argumentos vêm do
    /// [`PainterTool::composite_transform`] justamente para que o corpo não os re-derive.
    ///
    /// ⚠️ **Fora da `affected` o resultado é a BASE**, e é isso que abre o BURACO: o relevo que o patch
    /// levou tem de sumir de onde ele estava, senão a luz desenha uma crista sobre pixels
    /// transparentes — o fantasma que a metade da cor já não deixa.
    pub(super) fn composite_transform_relief(
        &mut self,
        minv: Mat3,
        dirty: Region,
        affected: Region,
    ) {
        if !self.paint.warp.affect_relief {
            return;
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let Some(fr) = self
            .paint
            .deform
            .xform_patch
            .as_ref()
            .and_then(|p| p.relief.clone())
        else {
            return;
        };
        // Bits de camada são reciclados: o corpo congelado no levante não pode ser vestido por outra.
        if self.layers.active() != Some(fr.layer) || fr.h.len() != n {
            return;
        }

        // Uma passada, três planos: a cobertura composta é o PESO das outras duas, então computá-la
        // aqui e reusá-la é o que impede as três de discordarem sobre quanta tinta há num texel.
        let mut out_h = vec![0.0f32; (dirty.w * dirty.h) as usize];
        let mut out_c = vec![0u8; (dirty.w * dirty.h) as usize];
        let mut out_m = vec![[0u8; 7]; (dirty.w * dirty.h) as usize];
        for ry in 0..dirty.h {
            let dy = dirty.y + ry;
            let inside_rows = dy >= affected.y && dy < affected.y + affected.h;
            for rx in 0..dirty.w {
                let dx = dirty.x + rx;
                let gi = (dy * w + dx) as usize;
                let oi = (ry * dirty.w + rx) as usize;
                // O lado da BASE: o que ficou para trás quando o patch subiu.
                let cb = f32::from(fr.c_base.as_ref().map_or(0, |v| v[gi])) / 255.0;
                let (hb, mb) = (fr.h[gi], fr.m[gi]);
                let inside = inside_rows && dx >= affected.x && dx < affected.x + affected.w;
                let (cp, hp, mp) = if inside {
                    let sp = minv.apply([dx as f32, dy as f32]);
                    (
                        f32::from(bilinear_u8(&fr.c_patch, w, h, sp[0], sp[1])) / 255.0,
                        bilinear_f32(&fr.h, w, h, sp[0], sp[1]),
                        bilinear_mat(&fr.m, w, h, sp[0], sp[1]),
                    )
                } else {
                    (0.0, 0.0, [0u8; 7])
                };
                // `over`, com a COBERTURA no lugar do alfa — a mesma lei que a cor usa uma linha ao lado.
                let back = cb * (1.0 - cp);
                let total = cp + back;
                out_c[oi] = (total * 255.0).round().clamp(0.0, 255.0) as u8;
                if total > 0.0 {
                    out_h[oi] = ((hp * cp + hb * back) / total).clamp(-H_MAX, H_MAX);
                    for c in 0..7 {
                        out_m[oi][c] = ((f32::from(mp[c]) * cp + f32::from(mb[c]) * back) / total)
                            .round()
                            .clamp(0.0, 255.0) as u8;
                    }
                }
                // …e onde não sobrou tinta nenhuma o texel fica em zero: cobertura zero é *não há tinta
                // aqui*, e uma altura órfã ali é um número que a luz não pesa e que o próximo verbo lê.
            }
        }

        let layer = fr.layer;
        let area = Some(dirty);
        if let Some(entry) = self.heights.get_mut(&layer)
            && entry.len() == n
        {
            let dst = super::super::plane_fork::fork_heights(
                entry,
                &self.undo.write_state,
                layer,
                (w, h),
                area,
            );
            blit(dst, &out_h, dirty, w);
        }
        if let Some(entry) = self.covers.get_mut(&layer)
            && entry.len() == n
        {
            let dst = super::super::plane_fork::fork_covers(
                entry,
                &self.undo.write_state,
                layer,
                (w, h),
                area,
            );
            blit(dst, &out_c, dirty, w);
        }
        if let Some(entry) = self.mats.get_mut(&layer)
            && entry.len() == n
        {
            let dst = super::super::plane_fork::fork_mats(
                entry,
                &self.undo.write_state,
                layer,
                (w, h),
                area,
            );
            blit(dst, &out_m, dirty, w);
        }
        // A luz lê a VIZINHANÇA de um texel (a normal é diferença central), então um texel logo fora da
        // caixa é iluminado por uma inclinação que mudou dentro dela — o mesmo crescimento de 1 que o
        // `warp_render_relief` faz pelo mesmo motivo.
        if let Some(g) = super::super::region::grow_region(dirty, 1, w, h) {
            self.mark_dirty(g);
        }
    }
}

/// Copiar um bloco `dirty`-shaped de volta para o plano canvas-shaped.
fn blit<T: Copy>(dst: &mut [T], src: &[T], dirty: Region, w: u32) {
    for ry in 0..dirty.h {
        let dy = dirty.y + ry;
        let row = ((dy * w + dirty.x) as usize)..((dy * w + dirty.x + dirty.w) as usize);
        let sr = ((ry * dirty.w) as usize)..(((ry + 1) * dirty.w) as usize);
        dst[row].copy_from_slice(&src[sr]);
    }
}
