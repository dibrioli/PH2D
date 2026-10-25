//! **Fase do quadro: A SELECÇÃO DA HIERARQUIA NUM MODO** — num modo de criação (spec/06 F2), escolher a
//! linha de OUTRO objecto selecciona-o e volta a Object (dono 24/10; revoga o cadeado de 03/10).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_hierarchy_select_lock(
        &mut self,
        hierarchy_select_intent: Option<hierarchy::HierarchySelectIntent>,
    ) -> Option<Option<hierarchy::HierarchySelectIntent>> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            hero_screen,
            hero_live,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        if let Some(intent) = hierarchy_select_intent {
            let (target, range) = match intent {
                hierarchy::HierarchySelectIntent::Row { row, .. } => (
                    hero_live.as_ref().and_then(|l| l.bridge.entity_for(row)),
                    false,
                ),
                hierarchy::HierarchySelectIntent::Range { .. } => (None, true),
            };
            ph2d_editor_core::screens::hero::mode_drive::chosen(hero, target, range);
        }
        Some(hierarchy_select_intent)
    }
}
