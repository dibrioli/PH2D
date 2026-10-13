//! **Fase do quadro: O FIM DO RAMO HERO** — os toasts e as barras de trabalho por cima dos painéis, e o
//! `hero_arena` do quadro esvaziado DEPOIS do despacho e da pintura (OBRA 2 da `line/render-loop`,
//! 2026-09-12).
//!
//! ⚠️ As barras partilham a coluna dos toasts e empilham-se POR BAIXO deles: recebem a CONTAGEM de
//! linhas da coluna que os balões ocuparam (`toast::paint_top_column`), nunca o número de balões. O
//! `PaintCtx` é reconstruído com os três campos do quadro
//! (`theme` copiado, `viewport`, `text_system`) — sem `Drop` nem estado, e ninguém escreve o `theme`.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_hero_chrome_tail(&mut self, viewport: EditorRect) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            theme,
            toasts,
            jobs,
            tools,
            vector_scene,
            text_system,
            hero_arena,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // ⭐ Os toasts e as barras são chrome: pintam-se na escala da interface (ecrã × `ui_scale`).
        let escala = hero_screen
            .as_ref()
            .map_or_else(Default::default, |h| h.escala());
        // Legacy `FloatingPanel` Procreate-style paint was retired
        // here (2026-05-17). The pink/magenta tab-strip + Accent
        // toggle decoration was inconsistent with the canonical
        // dark-glass surface used by Inspector / Hierarchy /
        // Widget Gallery. `Tool::build_panel()` still exists for
        // event dispatch but the visual is dropped; per-tool
        // chrome rewires through the new panel style in a
        // follow-up wave (BgRemoval especially needs its preview
        // panel re-painted; Move/Brush were stubs anyway).
        let _ = tools;
        ph2d_editor_core::ui_scale::pintar_no_chrome(escala, viewport, vector_scene, |vp, cena| {
            let mut paint_ctx = PaintCtx {
                theme: *theme,
                viewport: vp,
                text: text_system,
            };
            // The job bars share the toasts' column and stack UNDER them by the ROWS the toasts
            // took (a long toast spans several) — one door paints both, so neither the shell nor
            // either painter has an opinion about where row N is.
            ph2d_editor_core::toast::paint_top_column(toasts, jobs, cena, &mut paint_ctx);
        });
        // Drain frame-local arena AFTER the dispatch + paint pass
        // so any events emitted earlier this frame are still alive
        // for downstream consumers — wired in Phase A+ (currently
        // events are logged, not acted on).
        hero_arena.reset();
    }
}
