//! **Fase do quadro: AS CENAS DO PINCEL DO PAINTER** — o taper (o Touch Taper do Procreate) e a tinta
//! molhada, a mesma dança do impasto (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_painter_brush_smokes(&mut self) {
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

        // Taper smoke (`PH2D_TAPER_SMOKE=1`): the same dance for the Procreate Touch Taper. Nothing but
        // the canvas is staged — the taper opens OFF, because the first thing this scene asks is
        // whether an untouched build still paints what it painted yesterday.
        if let Some(hero) = hero_screen.as_mut()
            && ph2d_app_painter::taper_smoke::enabled()
            && !std::mem::replace(&mut self.taper_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            if let Some(bits) = ph2d_app_painter::taper_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                cell,
                ppm,
                atlas_asset_map,
            ) {
                *next_import_cell = next_import_cell.saturating_add(1);
                hero.gizmo.replace_selection(Some(bits));
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor_core::ViewFocusKind::Selected,
                    });
                toasts.push(Toast::success(
                    "Taper smoke: brush panel -> TAPER, under the Falloff".to_string(),
                ));
            }
        }

        // Wet Paint smoke (`PH2D_WETPAINT_SMOKE=1`): the impasto smoke's exact dance for the fluid
        // mode (ADR-0134 W1) — spawn, seat the selection, arm in `painter_bridge`.
        if let Some(hero) = hero_screen.as_mut()
            && ph2d_app_painter::wetpaint_smoke::enabled()
            && !std::mem::replace(&mut self.wetpaint_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            if let Some(bits) = ph2d_app_painter::wetpaint_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                cell,
                ppm,
                atlas_asset_map,
            ) {
                *next_import_cell = next_import_cell.saturating_add(1);
                hero.gizmo.replace_selection(Some(bits));
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor_core::ViewFocusKind::Selected,
                    });
                toasts.push(Toast::success(
                    "Wet Paint smoke: pick the Painter tool and drag".to_string(),
                ));
            }
        }
    }
}
