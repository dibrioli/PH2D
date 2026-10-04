//! **Os FIOS na aguada** (Sketchy · Wire · os degraus do Ribbon; doc 46 §2-7) — um fio é COBERTURA
//! fina sobre o papel seco e PIGMENTO dentro da aguada molhada: entra nos acumuladores da sessão (a
//! cor por `over`), e a borda escura, a granulação e o papel agem nele no composite do quadro como em
//! qualquer traço. Irmão do depósito digital ([`super::thread_deposit`]), que escreve a tela.

use super::Region;
use super::watercolor_accum::{WASH_DEPOSIT_PEAK, splat_keep};
use crate::tool::PainterTool;
use ph2d_painter_brush::stroke::threads::Thread;
use ph2d_painter_brush::thread_raster::{ThreadInk, threads_alpha};

impl PainterTool {
    /// Deposita o feixe na aguada viva. `rect` é a janela do feixe (já com o Tiling), a mesma que o
    /// depósito digital escreveria.
    ///
    /// ⚠️ A cor é a do PINCEL, por `over`: o Mixer (Charge/Pull) e o Pigment misturam por DAB, e um
    /// fio não é um dab — não há carga a esgotar nem superfície apanhada no caminho dele.
    pub(super) fn fios_na_aguada(&mut self, threads: &[Thread], ink: ThreadInk, rect: Region) {
        let (fw, fh) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if fw == 0 || fh == 0 || rect.w == 0 || rect.h == 0 {
            return;
        }
        #[allow(clippy::cast_precision_loss)]
        let origin = [rect.x as f32, rect.y as f32];
        let alpha = threads_alpha(threads, ink, rect.w as usize, rect.h as usize, origin);
        if self.paint.stroke_coverage.len() != fw * fh {
            self.paint.stroke_coverage = vec![0u8; fw * fh];
        }
        if self.paint.stroke_color.len() != fw * fh * 4 {
            self.paint.stroke_color = vec![0u8; fw * fh * 4];
        }
        for dirty in [
            &mut self.paint.wet_frame_dirty,
            &mut self.paint.wet_cum_dirty,
            &mut self.paint.wet_stroke_dirty,
        ] {
            *dirty = Some(dirty.map_or(rect, |d| super::union_region(d, rect)));
        }
        let (sel, prot, alock) = self.wet_splat_gates();
        let gated = sel.is_some() || prot.is_some() || alock.is_some();
        let mut planos = self.planos_da_aguada();
        for row in 0..rect.h as usize {
            let y = rect.y as usize + row;
            if y >= fh {
                break;
            }
            for cx in 0..rect.w as usize {
                let x = rect.x as usize + cx;
                if x >= fw {
                    break;
                }
                let fio = f32::from(alpha[row * rect.w as usize + cx]) / 255.0;
                if fio <= 0.0 {
                    continue;
                }
                let idx = y * fw + x;
                let keep = if gated {
                    splat_keep(
                        sel.as_deref().map(Vec::as_slice),
                        prot.as_deref().map(Vec::as_slice),
                        alock.as_deref().map(Vec::as_slice),
                        idx,
                    )
                } else {
                    1.0
                };
                // ⚠️ **O fio só ESTENDE a aguada sobre papel SECO** (cobertura zero). O aro é um
                // realce da fronteira da cobertura: um fio que a subisse DENTRO da aguada (no ombro
                // do traço) criava uma fronteira interna e o aro riscava-a de escuro — olhado, a
                // teia saía em rabiscos pretos dentro do traço e, densa, fundia numa poça lisa. Fio
                // molhado dentro de tinta molhada funde sem orla: ali ele deposita só o pigmento.
                planos.deposita(idx, WASH_DEPOSIT_PEAK * keep * fio, Cobertura::SoNoSeco);
            }
        }
    }
}

/// Como um depósito que NÃO é dab sobe a cobertura da aguada.
#[derive(Clone, Copy)]
pub(super) enum Cobertura {
    /// Como um carimbo: por `max` (a mancha do Solid).
    Max,
    /// Só onde o papel está seco (o fio, ver [`PainterTool::fios_na_aguada`]).
    SoNoSeco,
}

/// **Os planos da sessão molhada que um depósito que NÃO é dab escreve** — o fio e a mancha do
/// Solid ([`super::watercolor_solido`]): UMA porta para o corpo por texel, para os dois não
/// divergirem. A cor é a do PINCEL, por `over` (o Mixer e o Pigment misturam por DAB); os planos
/// opcionais só se escrevem quando um carimbo os armou, e ali o depósito é tinta cheia (densidade
/// plena, reserva cheia), como um dab de ponta lisa.
pub(super) struct PlanosDaAguada<'a> {
    cov: &'a mut [u8],
    cor: &'a mut [u8],
    dens: &'a mut [u8],
    depl: &'a mut [u8],
    prox: &'a mut [u8],
    own: &'a mut [u8],
    own_v: Option<u8>,
    col: [f32; 3],
}

impl PainterTool {
    /// Os planos para [`PlanosDaAguada::deposita`] — o dono do estilo nasce aqui se a sessão o pede.
    pub(super) fn planos_da_aguada(&mut self) -> PlanosDaAguada<'_> {
        let n = self.source_size.0 as usize * self.source_size.1 as usize;
        let own_v = (!self.paint.wet_styles.table.is_empty())
            .then(|| self.paint.wet_styles.current_owner());
        if own_v.is_some() && self.paint.wet_styles.owner.len() != n {
            self.paint.wet_styles.owner = vec![0u8; n];
        }
        PlanosDaAguada {
            cov: &mut self.paint.stroke_coverage,
            cor: &mut self.paint.stroke_color,
            dens: &mut self.paint.stroke_density,
            depl: &mut self.paint.stroke_deplete,
            prox: &mut self.paint.stroke_deplete_prox,
            own: &mut self.paint.wet_styles.owner,
            own_v,
            col: self.paint.brush.color.map(|c| c.clamp(0.0, 1.0)),
        }
    }
}

impl PlanosDaAguada<'_> {
    /// Deposita `a` (o peso já com as guardas) no texel `idx`.
    pub(super) fn deposita(&mut self, idx: usize, a: f32, cobertura: Cobertura) {
        if a <= 0.0 {
            return;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let v = (a * 255.0) as u8;
        match cobertura {
            Cobertura::Max => self.cov[idx] = self.cov[idx].max(v),
            Cobertura::SoNoSeco if self.cov[idx] == 0 => self.cov[idx] = v,
            Cobertura::SoNoSeco => {}
        }
        if !self.dens.is_empty() {
            self.dens[idx] = 255;
        }
        if !self.depl.is_empty() {
            super::watercolor_reserve::splat_level(
                &mut self.depl[idx],
                &mut self.prox[idx],
                0.0,
                1.0,
            );
        }
        if let Some(o) = self.own_v {
            self.own[idx] = o;
        }
        let i = idx * 4;
        let da = f32::from(self.cor[i + 3]) / 255.0;
        let na = a + da * (1.0 - a);
        for c in 0..3 {
            let dc = f32::from(self.cor[i + c]) / 255.0;
            let out = (self.col[c] * a + dc * da * (1.0 - a)) / na;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                self.cor[i + c] = (out * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
            }
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            self.cor[i + 3] = (na * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
        }
    }
}
