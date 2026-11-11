//! **O tique da lâmpada** (B15.3): ângulo, elevação, intensidade, cor, ligar uma lâmpada e Show Impasto mudam só
//! a LUZ — as camadas, o papel e o relevo ficam. A drenagem reacende a composição NÃO acesa que a última
//! recomposição inteira guardou, em vez de recompor a tela.
//!
//! A validade é por **lista branca de eventos**, nunca por versão (a recusa RC-77: «luz velha invisível»): só os
//! knobs deste ficheiro a mantêm viva; `invalidate_composite`, `mark_dirty`, o ramo parcial e a pista GPU (que
//! larga o `composited`) derrubam-na.

use super::Region;
use super::impasto_luz_dobra::LuzGuardada;
use crate::tool::{DrainBranch, PainterTool};
use std::sync::Arc;

impl PainterTool {
    /// Um knob da lista branca mudou: com a composição não acesa guardada basta reacender; sem ela, a de sempre.
    pub(super) fn so_a_lampada_mudou(&mut self) {
        // A pista da placa: nada por drenar nela além de knobs da lâmpada (os planos persistem lá).
        let placa = !self.gpu_lane_stale
            && match self.luz_placa {
                Some(t) => t == self.testemunha_da_placa(),
                None => !self.preview_dirty && self.dirty_rect.is_none(),
            };
        if self.luz_nao_aceso.is_some() && self.composited.is_some() {
            self.luz_so_lampada = true;
            self.preview_dirty = true;
            self.gpu_lane_stale = true;
        } else {
            self.invalidate_composite();
        }
        if placa {
            self.gpu_lane_stale = false;
            self.luz_placa = Some(self.testemunha_da_placa());
        }
    }

    fn testemunha_da_placa(&self) -> (u64, u64) {
        (self.layers_revision, self.pixel_clock)
    }

    /// A drenagem da pista GPU: só a lâmpada mudou desde a anterior (consome a marca).
    pub(crate) fn a_placa_so_reacende(&mut self) -> bool {
        self.luz_placa
            .take()
            .is_some_and(|t| t == self.testemunha_da_placa())
    }

    /// Qualquer evento fora da lista branca: a composição não acesa deixa de ser a da tela.
    pub(crate) fn larga_a_luz_guardada(&mut self) {
        self.luz_nao_aceso = None;
        self.luz_so_lampada = false;
        self.luz_placa = None;
    }

    /// O ramo inteiro da drenagem guarda a composição antes da luz (só quando a luz tem trabalho).
    pub(crate) fn guarda_o_nao_aceso(&mut self, nao_aceso: &[u8]) {
        self.luz_so_lampada = false;
        self.luz_nao_aceso = self
            .impasto_relief_present()
            .then(|| Arc::new(LuzGuardada::new(nao_aceso.to_vec())));
    }

    /// O ramo da lâmpada: `true` = a prévia saiu reacesa da composição guardada (`dirty` = o retângulo sujo).
    pub(crate) fn reacende_pela_luz_guardada(&mut self, dirty: Option<Region>) -> bool {
        if !std::mem::take(&mut self.luz_so_lampada) {
            return false;
        }
        let Some(guardada) = self
            .luz_nao_aceso
            .clone()
            .filter(|_| dirty.is_none() && self.composited.is_some())
        else {
            self.larga_a_luz_guardada();
            return false;
        };
        let (w, h) = self.source_size;
        let mut px = guardada.nao_aceso.clone();
        if !self.acende_pela_dobra(&guardada, &mut px) {
            self.apply_impasto_light(&mut px, Region { x: 0, y: 0, w, h });
        }
        self.apply_mask_overlay(&mut px);
        self.composited = Some(Arc::new(px));
        self.publica_o_envio(None, Vec::new());
        self.last_drain_branch = DrainBranch::Luz;
        true
    }

    /// Há relevo a acender, com Show ligado ou não (o Show volta a ligar-se pelo tique).
    fn impasto_relief_present(&self) -> bool {
        self.substrate().is_some()
            || !self.heights.is_empty()
            || !self.paint.relief.stroke_height.is_empty()
    }
}
