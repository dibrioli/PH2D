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
        //
        // ⛔ **A escala da CÂMERA saiu daqui em 2026-09-17**, com a fileira `Deform`: ela existia
        // para converter a tolerância do refinamento por quadro para pixels de ecrã, e não há
        // refinamento por quadro — a densidade é uma decisão do BIND.
        // ⭐⭐⭐ **EDITAR PIXELS ACHATA A ARTE** (regra do dono, F6-s): a sprite que a ferramenta
        // edita não recebe malha — a tabela e a exceção do Liquify vivem na porta.
        let achatadas = ph2d_app_painter::skin_suspend::achata_e_avisa(
            tools,
            hero_screen
                .as_ref()
                .into_iter()
                .flat_map(|h| h.gizmo.iter_selected()),
            |e| ph2d_skeleton_live::skin_image::is_skinned_image(sim.world(), e),
            toasts,
        );
        crate::skeleton_skin_image::attach_skin_meshes(sim, present, ppm, &achatadas);
    }
}
