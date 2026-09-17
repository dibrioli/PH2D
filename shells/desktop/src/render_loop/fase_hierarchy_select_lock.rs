//! **Fase do quadro: A TRAVA DO PAINTER NA SELECÇÃO DA HIERARQUIA** — com um documento do Painter aberto, a
//! selecção pedida pela Hierarquia que trocaria a sprite debaixo do pincel é recusada e consumida (OBRA 2 da
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
            tools,
            hero_screen,
            hero_live,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // Hierarchy intent dispatch phase — camera reset +
        // view-focus + 9 hierarchy intents (visibility_toggle /
        // reparent / duplicate / add_child / reset_transform /
        // delete / row_click / rename_seed / rename_commit).
        // Extracted to sibling `hierarchy.rs` as a free fn (Wave
        // 3.2 stage A).
        // **A TRAVA DO PAINTER, na porta da HIERARQUIA** (Enio, 2026-08-19). Enquanto o
        // Painter tem um documento aberto, clicar noutra linha não troca a sprite debaixo do
        // pincel: recusa, e o aviso diz por onde sair.
        //
        // ⚠️ A intenção é **consumida** (posta a `None`), não saltada: deixá-la viva faria a
        // mesma recusa repetir-se no quadro seguinte, e o artista veria o aviso a piscar.
        if let Some(intent) = hierarchy_select_intent {
            let locked = ph2d_app_painter::painter_lock::locked_entity(tools, hero);
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
            if ph2d_app_painter::painter_lock::decide(locked, target, additive)
                == ph2d_app_painter::painter_lock::Decision::Refuse
            {
                toasts.push(Toast::warning(ph2d_app_painter::painter_lock::REFUSAL.tr()));
                hierarchy_select_intent = None;
                self.title_dirty = true;
            }
        }
        Some(hierarchy_select_intent)
    }
}
