//! **Fase do quadro: AS CENAS DE SMOKE QUE PRECISAM DO ATLAS — a 1.ª metade** — o impasto, o substrato e
//! o card LINE, cada uma a pousar uma tela branca e a sentar a selecção nela no primeiro quadro em que o
//! atlas está em escopo (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! Partida da 2.ª metade (`fase_atlas_scene_smokes_late`) só porque o bake do sculpt3d corre entre as
//! duas, e a ordem é o contrato.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_atlas_scene_smokes(&mut self) {
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

        // Impasto smoke (`PH2D_IMPASTO_SMOKE=1`): one-shot, on the first frame where the atlas plumbing
        // is in scope — spawn a white canvas and SEAT the selection on it, so the artist lands on a
        // ready surface instead of assembling one. The brush itself is armed in `painter_bridge`, when
        // the painter first binds the document.
        if let Some(hero) = hero_screen.as_mut()
            && ph2d_app_painter::impasto_smoke::enabled()
            && !std::mem::replace(&mut self.impasto_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            if let Some(bits) = ph2d_app_painter::impasto_smoke::spawn_if_enabled(
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
                    "Impasto smoke: pick the Painter tool and drag".to_string(),
                ));
            }
        }

        // A cena do SUBSTRATO (`PH2D_SUBSTRATE_SMOKE=1`): a mesma dança do impasto, para o dente do
        // papel — que acende no DIGITAL, e por isso a cena não escolhe meio nenhum.
        if let Some(hero) = hero_screen.as_mut()
            && ph2d_app_painter::substrate_smoke::enabled()
            && !std::mem::replace(&mut self.substrate_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            if let Some(bits) = ph2d_app_painter::substrate_smoke::spawn_if_enabled(
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
                    "Substrate smoke: Painter -> secao Paper -> suba o Relief".to_string(),
                ));
            }
        }

        // A cena do card LINE (`PH2D_LINE_SMOKE=1`): a mesma dança, para os tipos de linha
        // procedural — que vivem no DIGITAL, e por isso a cena não escolhe meio nem tipo nenhum.
        if let Some(hero) = hero_screen.as_mut()
            && ph2d_app_painter::line_smoke::enabled()
            && !std::mem::replace(&mut self.line_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            if let Some(bits) = ph2d_app_painter::line_smoke::spawn_if_enabled(
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
                    "Line smoke: Painter -> card Line -> dropdown Type".to_string(),
                ));
            }
        }
    }
}
