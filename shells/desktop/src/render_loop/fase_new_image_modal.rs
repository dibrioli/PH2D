//! **Fase do quadro: O MODAL DE IMAGEM NOVA** (Cmd/Ctrl+N) — o pedido do botão Create é servido aqui,
//! onde o mundo, o renderizador e o atlas estão em escopo (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **Não é uma cena de smoke: é PRODUTO**, e por isso mora sozinha em vez de ao lado das cenas que
//! correm antes dela.

use super::*;
use ph2d_i18n::tr_with;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_new_image_modal(&mut self) {
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

        // New-image modal (Cmd/Ctrl+N) → spawn the chosen blank canvas. The modal's Create button set
        // `new_image_request`; service it here where `gfx` is destructured (sim/renderer/atlas access).
        if let Some(hero) = hero_screen.as_mut()
            && let Some((size, bg)) = hero.store.take_new_image_request()
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            match crate::image_import::spawn_blank_canvas(
                sim,
                renderer,
                asset_db,
                cell,
                size,
                bg,
                ph2d_core::Vec2::new(0.0, 0.0),
                ppm,
                atlas_asset_map,
            ) {
                Ok((label, bits)) => {
                    *next_import_cell = next_import_cell.saturating_add(1);
                    hero.gizmo.replace_selection(Some(bits));
                    hero.bus
                        .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                            kind: ph2d_editor_core::ViewFocusKind::Selected,
                        });
                    toasts.push(Toast::success(tr_with(
                        "shell.fase_new_image_modal.new_canvas",
                        &[("label", &label), ("size", &size)],
                    )));
                }
                Err(e) => {
                    toasts.push(Toast::error(tr_with(
                        "shell.fase_new_image_modal.new_canvas_failed",
                        &[("e", &e)],
                    )));
                }
            }
            self.title_dirty = true;
        }
    }
}
