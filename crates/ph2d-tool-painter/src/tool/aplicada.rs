//! **A imagem aplicada incremental** (B10.5 da [missão 4K](../../../../docs/Painter/missao_4k/BACKLOG.md)):
//! [`PainterTool::imagem_aplicada`] guarda a última inteira e, com a mesma [`ChaveAplicada`], só recompõe e
//! re-acende a caixa marcada (`mark_dirty`) ⊕ 1 px — a luz difere as alturas dos 4 vizinhos. Um buffer só:
//! recompor a moldura é exato (a composição é texel a texel), e poupa o «não aceso» (32 MB a 4K).
//! Gate: `aplicada_tests`.

use super::*;
use std::sync::{Mutex, PoisonError};

/// O que a imagem aplicada lê além dos píxeis marcados: mudou ⇒ a leitura seguinte é inteira.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct ChaveAplicada {
    rev: u64,
    /// O `pixel_clock` depois da última escrita MARCADA: outro escritor move-o sem caixa ⇒ inteira.
    relogio: u64,
    tam: (u32, u32),
    doc: Option<u64>,
    ativa: Option<RtLayerId>,
    trivial: bool,
    papel: Option<[u8; 3]>,
    luz: (paint::SubstrateKey, paint::impasto_rig::LightRig, bool),
}

#[derive(Default)]
struct Estado {
    feita: Option<(ChaveAplicada, Vec<u8>)>,
    sujo: Option<Region>,
}

/// A última imagem aplicada e a caixa marcada desde ela. Clonar dá-a vazia (a cópia lê inteira 1×).
#[derive(Default)]
pub(crate) struct CacheAplicada(
    Mutex<Estado>,
    /// Leituras pela caixa (só para o gate: a pista tem de ser tomada).
    #[cfg(test)]
    std::sync::atomic::AtomicU32,
);

impl Clone for CacheAplicada {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl CacheAplicada {
    fn estado(&mut self) -> &mut Estado {
        self.0.get_mut().unwrap_or_else(PoisonError::into_inner)
    }

    /// A figura mudou em `r` (a porta única: `mark_dirty`), e o `pixel_clock` andou `antes → depois`.
    pub(crate) fn marca(&mut self, r: Region, antes: u64, depois: u64) {
        let e = self.estado();
        e.sujo = Some(e.sujo.map_or(r, |a| une(a, r)));
        if let Some((k, _)) = e.feita.as_mut()
            && k.relogio == antes
        {
            k.relogio = depois;
        }
    }

    /// Quantas leituras foram pela caixa (só para o gate).
    #[cfg(test)]
    pub(crate) fn pistas(&self) -> u32 {
        self.1.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// A caixa marcada desde a última leitura (só para o gate).
    #[cfg(test)]
    pub(crate) fn sujo_agora(&self) -> Option<Region> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).sujo
    }

    /// Um passo confinado move `(revisão, relógio)` sem mudar nada fora da caixa que marcou: a guardada segue-o.
    pub(crate) fn segue_o_passo_confinado(&mut self, antes: (u64, u64), depois: (u64, u64)) {
        if let Some((k, _)) = self.estado().feita.as_mut()
            && (k.rev, k.relogio) == antes
        {
            (k.rev, k.relogio) = depois;
        }
    }
}

impl PainterTool {
    fn chave_da_aplicada(&self) -> ChaveAplicada {
        ChaveAplicada {
            rev: self.layers_revision,
            relogio: self.pixel_clock,
            tam: self.source_size,
            doc: self.bound_document(),
            ativa: self.layers.active(),
            trivial: self.is_trivial_stack(),
            papel: self.papel,
            luz: self.chave_da_luz(),
        }
    }

    /// O corpo de [`Self::imagem_aplicada`]. O cadeado só troca o estado (nunca espera por trabalho).
    pub(crate) fn imagem_aplicada_incremental(&self) -> Option<(Vec<u8>, u32, u32)> {
        if self.canvas_rgba.is_empty() {
            return None;
        }
        let (w, h) = self.source_size;
        let chave = self.chave_da_aplicada();
        let Estado { feita, sujo } = std::mem::take(&mut *self.trava());
        let rgba = match feita {
            Some((k, mut px)) if k == chave && px.len() == (w as usize) * (h as usize) * 4 => {
                #[cfg(test)]
                self.aplicada
                    .1
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if let Some(r) = sujo.map(|r| moldura(r, w, h)) {
                    let caixa = self.imagem_aplicada_na_caixa((r.x, r.y, r.w, r.h))?;
                    blit_region(&mut px, w, &caixa, r);
                }
                px
            }
            _ => self.imagem_aplicada_inteira()?.0,
        };
        let out = rgba.clone();
        *self.trava() = Estado {
            feita: Some((chave, rgba)),
            sujo: None,
        };
        Some((out, w, h))
    }

    fn trava(&self) -> std::sync::MutexGuard<'_, Estado> {
        self.aplicada
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// `r` ⊕ 1 px, dentro da tela.
fn moldura(r: Region, w: u32, h: u32) -> Region {
    let (x0, y0) = (r.x.saturating_sub(1), r.y.saturating_sub(1));
    let (x1, y1) = ((r.x + r.w + 1).min(w), (r.y + r.h + 1).min(h));
    Region {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}

fn une(a: Region, b: Region) -> Region {
    let (x0, y0) = (a.x.min(b.x), a.y.min(b.y));
    let (x1, y1) = ((a.x + a.w).max(b.x + b.w), (a.y + a.h).max(b.y + b.h));
    Region {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}
