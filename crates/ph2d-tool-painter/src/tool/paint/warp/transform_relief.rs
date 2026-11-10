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
use super::transform_mesh::MeshCell;
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
        self.compose_floating_relief(dirty, &|dx, dy| {
            let inside = dy >= affected.y
                && dy < affected.y + affected.h
                && dx >= affected.x
                && dx < affected.x + affected.w;
            inside.then(|| minv.apply([dx as f32, dy as f32]))
        });
    }

    /// O irmão da malha (B06.2): o corpo vai pelas MESMAS células que a cor, com a mesma posse (a
    /// última célula que reclama o texel, na ordem da cor) — o mapa por texel é montado uma vez.
    pub(super) fn composite_mesh_relief(&mut self, cells: &[MeshCell], dirty: Region) {
        if !self.paint.warp.affect_relief
            || self
                .paint
                .deform
                .xform_patch
                .as_ref()
                .is_none_or(|p| p.relief.is_none())
        {
            return;
        }
        let dw = dirty.w as usize;
        let mut map = vec![[f32::NAN; 2]; dw * dirty.h as usize];
        for cell in cells {
            let b = cell.bbox();
            let (x0, x1) = (b.x.max(dirty.x), (b.x + b.w).min(dirty.x + dirty.w));
            let (y0, y1) = (b.y.max(dirty.y), (b.y + b.h).min(dirty.y + dirty.h));
            for y in y0..y1 {
                for x in x0..x1 {
                    if let Some(sp) = cell.locate(x, y) {
                        map[(y - dirty.y) as usize * dw + (x - dirty.x) as usize] = sp;
                    }
                }
            }
        }
        self.compose_floating_relief(dirty, &|dx, dy| {
            let sp = map[(dy - dirty.y) as usize * dw + (dx - dirty.x) as usize];
            (!sp[0].is_nan()).then_some(sp)
        });
    }

    /// `over` do corpo sobre `dirty`: `locate(x, y)` diz onde o texel lê o patch (`None` = só a base).
    /// B06.A2: escreve direto nos planos forkados, em faixas de linhas disjuntas (o mesmo bit em
    /// qualquer nº de fios; oráculo serial em `transform_relief_oracle_tests.rs`).
    fn compose_floating_relief(
        &mut self,
        dirty: Region,
        locate: &(dyn Fn(u32, u32) -> Option<[f32; 2]> + Sync),
    ) {
        #[cfg(test)]
        if oracle_tests::pelo_oraculo() {
            return self.compose_floating_relief_serial(dirty, locate);
        }
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
        let layer = fr.layer;
        let area = Some(dirty);
        let fork = super::super::plane_fork::fork_heights;
        let win = &self.undo.write_state;
        let dh = self
            .heights
            .get_mut(&layer)
            .filter(|e| e.len() == n)
            .map(|e| fork(e, win, layer, (w, h), area).as_mut_slice());
        let dc = self
            .covers
            .get_mut(&layer)
            .filter(|e| e.len() == n)
            .map(|e| {
                super::super::plane_fork::fork_covers(e, win, layer, (w, h), area).as_mut_slice()
            });
        let dm = self.mats.get_mut(&layer).filter(|e| e.len() == n).map(|e| {
            super::super::plane_fork::fork_mats(e, win, layer, (w, h), area).as_mut_slice()
        });
        let wu = w as usize;
        let fr = &fr;
        let band = |y0: u32,
                    bh: &mut Option<&mut [f32]>,
                    bc: &mut Option<&mut [u8]>,
                    bm: &mut Option<&mut [MaterialBytes]>,
                    nrows: u32| {
            for ry in 0..nrows {
                let dy = y0 + ry;
                for rx in 0..dirty.w {
                    let dx = dirty.x + rx;
                    let gi = (dy * w + dx) as usize;
                    let li = (ry * w + dx) as usize;
                    let (oh, oc, om) = texel(fr, w, h, gi, locate(dx, dy));
                    if let Some(p) = bh.as_deref_mut() {
                        p[li] = oh;
                    }
                    if let Some(p) = bc.as_deref_mut() {
                        p[li] = oc;
                    }
                    if let Some(p) = bm.as_deref_mut() {
                        p[li] = om;
                    }
                }
            }
        };
        let bands = bands_for(dirty);
        let rpb = (dirty.h as usize).div_ceil(bands.max(1)).max(1);
        let (vh, vc, vm) = (
            faixas(dh, dirty, wu, rpb),
            faixas(dc, dirty, wu, rpb),
            faixas(dm, dirty, wu, rpb),
        );
        let jobs: Vec<_> = vh.into_iter().zip(vc).zip(vm).enumerate().collect();
        let band = &band;
        let run = |(bi, ((mut bh, mut bc), mut bm)): (usize, _)| {
            let y0 = dirty.y + (bi * rpb) as u32;
            let nrows = (dirty.y + dirty.h - y0).min(rpb as u32);
            band(y0, &mut bh, &mut bc, &mut bm, nrows);
        };
        if jobs.len() <= 1 {
            jobs.into_iter().for_each(run);
        } else {
            std::thread::scope(|s| {
                let hs: Vec<_> = jobs.into_iter().map(|j| s.spawn(move || run(j))).collect();
                for h in hs {
                    h.join().expect("transform relief band panicked");
                }
            });
        }
        // A luz lê a VIZINHANÇA de um texel (a normal é diferença central), então um texel logo fora da
        // caixa é iluminado por uma inclinação que mudou dentro dela — o mesmo crescimento de 1 que o
        // `warp_render_relief` faz pelo mesmo motivo.
        if let Some(g) = super::super::region::grow_region(dirty, 1, w, h) {
            self.mark_dirty(g);
        }
    }
}

/// As linhas de `dirty` de um plano (largura inteira), partidas em faixas de `rpb` linhas.
fn faixas<T>(o: Option<&mut [T]>, dirty: Region, wu: usize, rpb: usize) -> Vec<Option<&mut [T]>> {
    let n = (dirty.h as usize).div_ceil(rpb);
    match o {
        Some(p) => p[dirty.y as usize * wu..(dirty.y + dirty.h) as usize * wu]
            .chunks_mut(rpb * wu)
            .map(Some)
            .collect(),
        None => (0..n).map(|_| None).collect(),
    }
}

/// Quantas faixas: a mesma régua da cor (`for_dirty_rows_parallel`: série abaixo de 2^17 texels).
fn bands_for(dirty: Region) -> usize {
    #[cfg(test)]
    if let Some(n) = oracle_tests::faixas_forcadas() {
        return n;
    }
    const PARALLEL_MIN_AREA: usize = 1 << 17;
    if (dirty.w as usize * dirty.h as usize) < PARALLEL_MIN_AREA || dirty.h <= 1 {
        return 1;
    }
    std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1)
        .clamp(1, dirty.h as usize)
}

/// Um texel do `over` do corpo, com a COBERTURA no lugar do alfa — a mesma lei que a cor usa.
/// Onde não sobra tinta o texel fica em zero (cobertura zero é *não há tinta aqui*).
fn texel(
    fr: &FloatingRelief,
    w: u32,
    h: u32,
    gi: usize,
    sp: Option<[f32; 2]>,
) -> (f32, u8, MaterialBytes) {
    let cb = f32::from(fr.c_base.as_ref().map_or(0, |v| v[gi])) / 255.0;
    let (hb, mb) = (fr.h[gi], fr.m[gi]);
    let (cp, hp, mp) = if let Some(sp) = sp {
        (
            f32::from(bilinear_u8(&fr.c_patch, w, h, sp[0], sp[1])) / 255.0,
            bilinear_f32(&fr.h, w, h, sp[0], sp[1]),
            bilinear_mat(&fr.m, w, h, sp[0], sp[1]),
        )
    } else {
        (0.0, 0.0, [0u8; 7])
    };
    let back = cb * (1.0 - cp);
    let total = cp + back;
    let oc = (total * 255.0).round().clamp(0.0, 255.0) as u8;
    let (mut oh, mut om) = (0.0f32, [0u8; 7]);
    if total > 0.0 {
        oh = ((hp * cp + hb * back) / total).clamp(-H_MAX, H_MAX);
        for c in 0..7 {
            om[c] = ((f32::from(mp[c]) * cp + f32::from(mb[c]) * back) / total)
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
    (oh, oc, om)
}

#[cfg(test)]
#[path = "transform_relief_oracle_tests.rs"]
mod oracle_tests;
