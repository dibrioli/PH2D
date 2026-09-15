//! **Fase do quadro: O EXTRACT** — o tique do movimento de demo, a propagação de transforms e a emissão
//! das sprites e da ordem TOTAL do quadro, pelo `sim_extract::run` (OBRA 2 da `line/render-loop`,
//! 2026-09-12) — e a MALHA de cada imagem presa ao esqueleto, posta na instância que o extract emitiu
//! (plano `docs/Skeleton/03`, W2, 2026-09-13).
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
        // A escolha do artista, lida antes do empréstimo do `gfx`.
        let pele_suave = ph2d_app_vec::pele_suave(self.vec.draw_config.skin_deform);
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            renderer,
            sim,
            present,
            camera,
            tools,
            toasts,
            prop_state,
            worklist,
            sort_scratch,
            sort_inputs,
            frame_order,
            surface,
            hero_screen,
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

        // ⭐⭐⭐ **A MALHA DAS IMAGENS PRESAS AO ESQUELETO** — DEPOIS do extract, porque a sujeita é a
        // INSTÂNCIA que ele emitiu: uma sprite escondida não tem instância e não recebe malha, e o
        // passe de sprites troca o quad de cada uma pela malha posada.
        //
        // ⚠️ **O `ppm` é o MESMO do extract:** a régua da imagem e o quad resolvem a mesma âncora.
        // ⚠️ E a escala da câmera é a da CENA (`scene_camera_window`, a porta que o split do Motion
        // pede a todo mapeamento mundo↔ecrã), que é onde a tolerância do `Smooth` é medida.
        let px_por_metro = ph2d_app_motion::field_gizmo::scene_px_per_world(
            camera,
            hero_screen.as_ref().map(|h| h.view.center_split),
            surface.size(),
        );
        // ⭐⭐⭐ **PINTAR ACHATA A ARTE** (ordem do dono, 2026-09-15): a sprite que o Painter edita
        // não recebe malha, e o desenho, o ponteiro e o chrome voltam ao quad de uma vez. Sair
        // devolve a deformação no quadro seguinte — ver [`ph2d_app_painter::skin_suspend`].
        let achatada = ph2d_app_painter::skin_suspend::achata_e_avisa(
            tools,
            hero_screen.as_ref().and_then(|h| h.gizmo.selection),
            |e| ph2d_skeleton_live::skin_image::is_skinned_image(sim.world(), e),
            toasts,
        );
        crate::skeleton_skin_image::attach_skin_meshes(
            sim,
            present,
            ppm,
            pele_suave,
            px_por_metro,
            achatada,
        );
    }
}
