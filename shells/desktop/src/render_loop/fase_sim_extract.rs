//! **Fase do quadro: O EXTRACT** — o tique do movimento de demo, a propagação de transforms e a emissão
//! das sprites e da ordem TOTAL do quadro, pelo `sim_extract::run` (OBRA 2 da `line/render-loop`,
//! 2026-09-12).
//!
//! ⚠️ Os cinco parâmetros são os que a `fase_extract_inputs` preparou neste quadro (o contexto
//! `ExtractInputs`), e esta fase corre DEPOIS da timeline, da física, dos sinais e da marca da receita
//! aberta — todos escrevem o que o extract lê.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_sim_extract(
        &mut self,
        dt: f32,
        preview_overrides: Vec<sim_extract::PreviewOverride>,
        sheet_preview: Option<ph2d_ecs::Entity>,
        ppm: f32,
        default_filter: ph2d_ecs::FilterMode,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            renderer,
            sim,
            present,
            camera,
            prop_state,
            worklist,
            sort_scratch,
            sort_inputs,
            frame_order,
            ..
        } = FrameGfx::of(gfx);

        sim_extract::run(
            dt,
            sim,
            present,
            renderer,
            prop_state,
            worklist,
            sort_scratch,
            sort_inputs,
            &preview_overrides,
            ppm,
            camera.cull_mask,
            default_filter,
            ph2d_ecs::RepeatMode::Disabled,
            sheet_preview,
            frame_order,
        );
    }
}
