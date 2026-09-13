//! **Fase do quadro: AS CENAS DO SPRITE INSPECTOR** — o 9-slice, as três formas de âncora, o que uma
//! âncora move e a §11 Animation a andar, cada uma a montar o seu sujeito no primeiro quadro em que o
//! atlas está em escopo (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_sprite_inspector_smokes(&mut self) {
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

        // **9-SLICE** (`PH2D_SLICE_SMOKE=1`, spec Sprite 03 §3.5): duas caixas do MESMO desenho,
        // esticadas ao mesmo tamanho — a da esquerda sem 9-slice (cantos redondos viram elipses),
        // a da direita com. ⚠️ A comparação É o smoke: uma caixa sozinha com a feature ligada
        // parece só «uma caixa».
        if let Some(hero) = hero_screen.as_mut()
            && crate::slice_smoke::enabled()
            && !std::mem::replace(&mut self.slice_smoke_done, true)
            && let Some(sliced) = crate::slice_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                next_import_cell,
                atlas_asset_map,
            )
        {
            hero.gizmo.replace_selection(Some(sliced));
            hero.bus
                .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                    kind: ph2d_editor_core::ViewFocusKind::Selected,
                });
            toasts.push(Toast::success(
                "9-Slice smoke: LEFT is plain (corners stretch), RIGHT is sliced — see the \
                 9-Slice section"
                    .to_string(),
            ));
            self.title_dirty = true;
        }

        // **AS TRÊS FORMAS DE ÂNCORA** (`PH2D_SOCKET_SMOKE=1`, ADR-0072): socket, slice e região
        // 9-slice numa sprite só. ⚠️ A §5 9-Slice e a §12 Sockets/Anchors partilham o vocabulário
        // do «miolo dentro de uma área» — ver as duas juntas é o que o explica.
        if let Some(hero) = hero_screen.as_mut()
            && crate::socket_smoke::enabled()
            && !std::mem::replace(&mut self.socket_smoke_done, true)
            && let Some(bits) = crate::socket_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                next_import_cell,
                hero.project.pixels_per_meter,
                atlas_asset_map,
            )
        {
            hero.gizmo.replace_selection(Some(bits));
            hero.bus
                .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                    kind: ph2d_editor_core::ViewFocusKind::Selected,
                });
            toasts.push(Toast::success(
                "Anchors smoke: open the Sockets / Anchors section to see the three marks"
                    .to_string(),
            ));
            self.title_dirty = true;
        }

        // **O QUE UMA ÂNCORA MOVE** (`PH2D_MOUNT_SMOKE=1`, ADR-0072 §2.6): um boneco com duas
        // âncoras e três filhos — dois montados e **um controlo que não monta em nada**. Sem o
        // controlo, uma cena com a montagem ignorada por completo pareceria igual.
        if let Some(hero) = hero_screen.as_mut()
            && crate::mount_smoke::enabled()
            && !std::mem::replace(&mut self.mount_smoke_done, true)
            && let Some(bits) = crate::mount_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                next_import_cell,
                hero.project.pixels_per_meter,
                atlas_asset_map,
            )
        {
            hero.gizmo.replace_selection(Some(bits));
            hero.bus
                .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                    kind: ph2d_editor_core::ViewFocusKind::Selected,
                });
            toasts.push(Toast::success(
                "Mount smoke: open Sockets / Anchors, pick hand_r and drag it — the red square \
                 follows, the grey one does not"
                    .to_string(),
            ));
            self.title_dirty = true;
        }

        // **A §11 ANIMATION A ANDAR** (`PH2D_ANIM_SMOKE=1`, spec Sprite 08): uma tira de 8 células
        // e TRÊS animações que se sobrepõem sobre ela — a tese do modelo do Aseprite (um pool,
        // intervalos nomeados), que N arrays separados não conseguem exprimir sem duplicar pixels.
        if let Some(hero) = hero_screen.as_mut()
            && crate::anim_smoke::enabled()
            && !std::mem::replace(&mut self.anim_smoke_done, true)
            && let Some(bits) = crate::anim_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                next_import_cell,
                hero.project.pixels_per_meter,
                atlas_asset_map,
            )
        {
            hero.gizmo.replace_selection(Some(bits));
            hero.bus
                .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                    kind: ph2d_editor_core::ViewFocusKind::Selected,
                });
            toasts.push(Toast::success(
                "Animation smoke: open the Animation section — it is playing walk (silent, even \
                 rhythm). Click idle: it HESITATES on one cell (per-frame timing) and fires a \
                 signal per lap. Click attack: it plays once, stays on the last cell, and \
                 announces the end"
                    .to_string(),
            ));
            self.title_dirty = true;
        }
    }
}
