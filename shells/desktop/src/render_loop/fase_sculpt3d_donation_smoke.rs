//! **Fase do quadro: A CENA DA DOAÇÃO** (`PH2D_SCULPT3D_SMOKE=2`) — a mesma dança do impasto, para a
//! tela em que a forma vai acender a tinta (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **Atrás da `feature` `sculpt3d`** (o `mod` e a chamada no quadro), porque o statement já estava.
//! Partida do objeto misto (`fase_sculpt3d_bake`), que corre a seguir, porque é outro assunto: esta
//! ENCENA uma tela, aquela ASSA e re-autora a luz.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_sculpt3d_donation_smoke(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            renderer,
            sim,
            asset_db,
            toasts,
            hero_screen,
            next_import_cell,
            atlas_asset_map,
            ..
        } = FrameGfx::of(gfx);

        // A cena da DOAÇÃO (`PH2D_SCULPT3D_SMOKE=2`): a mesma dança do impasto, para a tela em que a
        // forma vai acender a tinta. A esfera nasce em `sculpt3d_smoke`; aqui nasce o que pintar.
        #[cfg(feature = "sculpt3d")]
        if let Some(hero) = hero_screen.as_mut()
            && !std::mem::replace(&mut self.sculpt3d_req.canvas_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            // ⚠️ **Duas chamadas, e o corte é o da regra 2** (W2/L3-B): a família diz SE uma
            // tela é precisa e COMO ela tem de ser (`canvas_wanted`, com os três números e a
            // razão de cada); quem sabe FAZER uma é o `image_import`, folha desta shell com 41
            // consumidores de famílias diferentes. Ela nunca foi da escultura.
            if let Some(bits) = ph2d_app_sculpt3d::donation::canvas_wanted().and_then(|q| {
                ph2d_app_sculpt3d::donation::canvas_born(crate::image_import::spawn_blank_canvas(
                    sim,
                    renderer,
                    asset_db,
                    cell,
                    q.edge,
                    q.bg,
                    q.center,
                    ppm,
                    atlas_asset_map,
                ))
            }) {
                *next_import_cell = next_import_cell.saturating_add(1);
                // ⭐⭐ **A `=52` quer um CATAVENTO nesta tela** — a decisão é da família (ela
                // escreveu a cena), o gesto é desta shell (ela tem o mundo). Em toda outra cena a
                // porta devolve `None` e esta linha é inerte.
                if let Some(catavento) = ph2d_app_sculpt3d::donation::catavento_pedido() {
                    sim.world_mut()
                        .entity_mut(ph2d_ecs::Entity::from_bits(bits))
                        .insert(catavento);
                }
                hero.gizmo.replace_selection(Some(bits));
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor_core::ViewFocusKind::Selected,
                    });
                toasts.push(Toast::success(
                    "Sculpt3d: esculpa, aperte D ate ler LUZ, e pinte".to_string(),
                ));
            }
        }
    }
}
