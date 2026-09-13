//! **Fase do quadro: O CHROME LEGADO** — o ramo SEM `HeroScreen` (`PH2D_M5_DEMO=1`): o layout, a paleta de
//! ferramentas na zona CREATE, os toasts e as barras de trabalho (OBRA 2 da `line/render-loop`,
//! 2026-09-12).
//!
//! ⚠️ A paleta mapeia as ranhuras pela MESMA `palette_visible_tool_indices` do hit-test do clique, com
//! `mode_on = false` (este ramo não tem o modo Image Tools) — as duas nunca derivam.
//!
//! O `PaintCtx` é reconstruído aqui com os mesmos três campos do quadro: o `theme` copiado, o `viewport`
//! do quadro e o `text_system`. Ele não guarda estado nem tem `Drop`, e nada escreve o `theme` entre a
//! construção do quadro e esta fase.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_legacy_chrome(&mut self, viewport: EditorRect) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            theme,
            toasts,
            jobs,
            tools,
            layout,
            vector_scene,
            text_system,
            ..
        } = FrameGfx::of(gfx);
        let mut paint_ctx = PaintCtx {
            theme: *theme,
            viewport,
            text: text_system,
        };
        layout.paint(vector_scene, &mut paint_ctx);

        // Tool palette in the CREATE zone (top-right). Hidden in Zen
        // mode by virtue of `tool_palette_rects` returning empty.
        // This branch is the legacy no-hero (demo) path, so there is
        // no Image Tools mode → `mode_on = false`. Map slots through
        // the SAME `palette_visible_tool_indices` the click hit-test
        // uses so the two never drift (image tools filtered out when
        // off — no icon, no hit zone).
        let visible = crate::palette_visible_tool_indices(tools, false);
        let palette_rects = layout.tool_palette_rects(visible.len());
        let active_id = tools.active().map(|t| t.id());
        let palette_icons: Vec<(EditorRect, &str, bool)> = palette_rects
            .iter()
            .zip(visible.iter())
            .map(|(r, &i)| {
                let tool = &tools.tools()[i];
                let is_active = active_id.as_ref() == Some(&tool.id());
                (*r, tool.label(), is_active)
            })
            .collect();
        ph2d_editor_core::paint_tool_palette_icons(
            paint_ctx.text,
            vector_scene,
            &palette_icons,
            paint_ctx.theme,
        );

        // Legacy `FloatingPanel` paint retired (2026-05-17). Same
        // rationale as the live-mode branch above. Tool palette
        // chrome above remains because it's the click entrypoint
        // to switch tools; the per-tool panel itself is gone.
        toasts.paint(vector_scene, &mut paint_ctx);
        jobs.paint_below(toasts.len(), vector_scene, &mut paint_ctx);
    }
}
