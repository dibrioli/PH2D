//! **Fase do quadro: O COMPOSTO E OS ENCAIXES** — o caminho composto, a regra de preenchimento e os
//! encaixes (estado da FERRAMENTA) (OBRA 2 da `line/render-loop`, 2026-09-13). As réguas eram daqui
//! por dois botões do painel do vetor que nada pintava; saíram (UIUX 16/10): a casa delas é o menu
//! View, que mexe no `hero.view` direto.

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct CompoundSnapRulersIntents {
    pub(super) pending_vec_compound: Option<bool>,
    pub(super) pending_vec_fill_rule: Option<bool>,
    pub(super) pending_vec_snap_on: Option<bool>,
    pub(super) pending_vec_snap_path: Option<bool>,
    pub(super) pending_vec_snap_cross: Option<bool>,
    pub(super) pending_vec_snap_guides: Option<bool>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_compound_snap_rulers(&mut self, intents: CompoundSnapRulersIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { vec_scene, .. } = FrameGfx::of(gfx);
        let CompoundSnapRulersIntents {
            pending_vec_compound,
            pending_vec_fill_rule,
            pending_vec_snap_on,
            pending_vec_snap_path,
            pending_vec_snap_cross,
            pending_vec_snap_guides,
        } = intents;
        if let Some(make) = pending_vec_compound {
            crate::input_dispatch::apply_vec_compound(vec_scene, &mut self.vec.pen, make);
        }
        if let Some(even_odd) = pending_vec_fill_rule {
            crate::input_dispatch::apply_vec_fill_rule(vec_scene, &self.vec.pen, even_odd);
        }
        // Snap settings are TOOL state, not document state — no undo step.
        if let Some(on) = pending_vec_snap_on {
            self.vec.snap.on = on;
        }
        if let Some(on) = pending_vec_snap_path {
            self.vec.snap.path = on;
        }
        if let Some(on) = pending_vec_snap_cross {
            self.vec.snap.crossings = on;
        }
        if let Some(on) = pending_vec_snap_guides {
            self.vec.snap.guides = on;
        }
    }
}
