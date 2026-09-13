//! **Fase do quadro: O FIM DO RAMO HERO** — os toasts e as barras de trabalho por cima dos painéis, e o
//! `hero_arena` do quadro esvaziado DEPOIS do despacho e da pintura (OBRA 2 da `line/render-loop`,
//! 2026-09-12).
//!
//! ⚠️ As barras partilham a coluna dos toasts e empilham-se POR BAIXO deles: recebem a CONTAGEM de
//! linhas já ocupadas, nunca a geometria. O `PaintCtx` é reconstruído com os três campos do quadro
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
            ..
        } = FrameGfx::of(gfx);
        let mut paint_ctx = PaintCtx {
            theme: *theme,
            viewport,
            text: text_system,
        };
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
        toasts.paint(vector_scene, &mut paint_ctx);
        // The job bars share the toasts' column and stack UNDER them, so they are handed
        // the number of rows already spoken for. The count, not the geometry: the column's
        // ruler lives in `progress::column_row` and neither the shell nor the toast painter
        // gets to have an opinion about where row N is.
        jobs.paint_below(toasts.len(), vector_scene, &mut paint_ctx);
        // Drain frame-local arena AFTER the dispatch + paint pass
        // so any events emitted earlier this frame are still alive
        // for downstream consumers — wired in Phase A+ (currently
        // events are logged, not acted on).
        hero_arena.reset();
    }
}
