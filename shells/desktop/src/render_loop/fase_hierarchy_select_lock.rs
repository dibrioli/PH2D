//! **Fase do quadro: O CADEADO DO MODO NA SELECÇÃO DA HIERARQUIA** — num modo de criação (spec/06 F2), a
//! selecção pedida pela Hierarquia que trocaria o objecto em edição é recusada e consumida (OBRA 2 da
//! `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_hierarchy_select_lock(
        &mut self,
        mut hierarchy_select_intent: Option<hierarchy::HierarchySelectIntent>,
    ) -> Option<Option<hierarchy::HierarchySelectIntent>> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            toasts,
            hero_screen,
            hero_live,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // **O CADEADO DO MODO, na porta da HIERARQUIA** (Enio, 2026-08-19; do modo desde a F2 do
        // spec/06). Num modo de criação, clicar noutra linha não troca o objecto em edição:
        // recusa, e o aviso diz por onde sair.
        //
        // ⚠️ A intenção é **consumida** (posta a `None`), não saltada: deixá-la viva faria a
        // mesma recusa repetir-se no quadro seguinte, e o artista veria o aviso a piscar.
        if let Some(intent) = hierarchy_select_intent {
            let (target, additive) = match intent {
                hierarchy::HierarchySelectIntent::Row { row, modifier } => (
                    hero_live.as_ref().and_then(|l| l.bridge.entity_for(row)),
                    !matches!(
                        modifier,
                        ph2d_editor_core::action_bus::SelectModifier::Replace
                    ),
                ),
                // Um intervalo é aditivo por definição.
                hierarchy::HierarchySelectIntent::Range { .. } => (None, true),
            };
            if ph2d_editor_core::screens::hero::mode_drive::refused(hero, target, additive, toasts)
            {
                hierarchy_select_intent = None;
                self.title_dirty = true;
            }
        }
        Some(hierarchy_select_intent)
    }
}
