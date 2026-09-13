//! **Fase do quadro: O COMPOSTO, OS ENCAIXES E AS RÉGUAS** — o caminho composto, a regra de preenchimento, os encaixes (estado da FERRAMENTA) e as
//! réguas (estado do hero) (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct CompoundSnapRulersIntents {
    pub(super) pending_vec_compound: Option<bool>,
    pub(super) pending_vec_fill_rule: Option<bool>,
    pub(super) pending_vec_snap_on: Option<bool>,
    pub(super) pending_vec_snap_path: Option<bool>,
    pub(super) pending_vec_snap_cross: Option<bool>,
    pub(super) pending_vec_snap_guides: Option<bool>,
    pub(super) pending_rulers: Option<bool>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_compound_snap_rulers(&mut self, intents: CompoundSnapRulersIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let CompoundSnapRulersIntents {
            pending_vec_compound,
            pending_vec_fill_rule,
            pending_vec_snap_on,
            pending_vec_snap_path,
            pending_vec_snap_cross,
            pending_vec_snap_guides,
            pending_rulers,
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
        // ⚠️ A régua é estado do HERO, não da ferramenta: ela é chrome de canvas, aparece
        // com qualquer ferramenta na mão, e é o mesmo flag que a tecla/menu de vista
        // mexeria. O painel do vetor é só mais um lugar de onde se alcança o interruptor.
        if let Some(on) = pending_rulers {
            hero.view.rulers_visible = on;
        }
    }
}
