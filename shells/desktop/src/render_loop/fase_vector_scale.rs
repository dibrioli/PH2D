//! **Fase do quadro: A ESCALA DO DESENHO VECTORIAL** — a ferramenta vectorial activa e as unidades de mundo por pixel de ecrã (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_scale(
        &mut self,
        window_size: ph2d_host::WindowSize,
    ) -> Option<(bool, f64)> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { camera, tools, .. } = FrameGfx::of(gfx);
        // ADR-0108 cutover: the Vector drawing tool. `AppGfx.vec_scene` is
        // document artwork — render it into the shared Vello scene EVERY
        // frame (not gated on the active tool; no per-tool branch). The
        // `vector_bridge` reflects the active tool's Style into the shell
        // Pen + recolours the selection; the edit gizmos draw ONLY while the
        // Vector tool is active (mirror of how the pen input is gated).
        let vector_active = tools
            .active()
            .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("vector"));
        // World units per screen pixel (1px delta) — lets the bridge convert
        // the tool's px stroke width into the selected path's world width.
        let vw0 = camera.screen_to_world((0.0, 0.0), window_size);
        let vw1 = camera.screen_to_world((1.0, 0.0), window_size);
        let vec_px_to_world =
            (((vw1[0] - vw0[0]).powi(2) + (vw1[1] - vw0[1]).powi(2)).sqrt()) as f64;
        Some((vector_active, vec_px_to_world))
    }
}
