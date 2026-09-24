//! **Fase do quadro: O PAINTER: PERSISTIR, DESPACHAR E MEDIR** — persistir o trabalho do Painter antes da ponte, a ponte do Painter, a prévia da forma do
//! pincel e o relógio do `paint ms` (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_painter_dispatch(
        &mut self,
        window_size: ph2d_host::WindowSize,
        viewport: EditorRect,
    ) -> Option<bool> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        // ⭐ O Painter sobre a peça 3D prende a tela da vista e pousa-a ANTES da ponte da sprite,
        // que enquanto ela está presa não a toca (`ph2d_tool_painter::SCREEN_CANVAS_DOC`).
        #[cfg(feature = "sculpt3d")]
        ph2d_app_sculpt3d::painter_na_malha::quadro(
            gfx.sculpt3d.as_mut(),
            gfx.tools.active_mut().and_then(|t| {
                t.as_any_mut()
                    .downcast_mut::<ph2d_tool_painter::PainterTool>()
            }),
        );
        let FrameGfx {
            renderer,
            sim,
            present,
            camera,
            asset_db,
            theme,
            toasts,
            tools,
            vector_scene,
            text_system,
            hero_screen,
            atlas_asset_map,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        let paint_ctx = PaintCtx {
            theme: *theme,
            viewport,
            text: text_system,
        };
        // ── Persist painter work BEFORE the bridge rebinds / right after a deferred deactivation
        // (Enio 2026-06-24: paint must survive deselect / object-switch / closing painter mode).
        // Done HERE (not in the bridge) because the bake needs `&mut sim` and must run before the
        // bridge's source-push replaces the working canvas. ──
        {
            let painter_id = ph2d_editor_core::ToolId::new("painter");
            let painter_active = tools.active().map(|t| t.id()) == Some(painter_id.clone());
            if painter_active {
                // Selection moved off the bound sprite (incl. deselect) → bake it now.
                let sel = hero.gizmo.selection;
                if let Some(old) = self.last_painter_pushed_entity
                    && sel != Some(old)
                    && let Some(painter) = tools.active_mut().and_then(|t| {
                        t.as_any_mut()
                            .downcast_mut::<ph2d_tool_painter::PainterTool>()
                    })
                    && painter.has_unbaked_edits()
                {
                    crate::hero_intents::auto_commit_painter(
                        old,
                        sim,
                        renderer,
                        asset_db,
                        atlas_asset_map,
                        painter,
                        toasts,
                    );
                    self.last_painter_pushed_entity = None; // bridge re-pushes the new selection
                }
            } else if let Some(old) = self.last_painter_pushed_entity {
                if let Some(painter) = tools.tool_by_id_mut(&painter_id).and_then(|t| {
                    t.as_any_mut()
                        .downcast_mut::<ph2d_tool_painter::PainterTool>()
                }) && painter.take_deferred_bake()
                {
                    // The painter deactivated with unbaked edits → bake the kept canvas, then
                    // finish the teardown its `on_deactivate` deferred.
                    crate::hero_intents::auto_commit_painter(
                        old,
                        sim,
                        renderer,
                        asset_db,
                        atlas_asset_map,
                        painter,
                        toasts,
                    );
                    (painter as &mut dyn ph2d_editor_core::tool::RasterEditTool).deactivate();
                }
                // ⚠️ Cleared whether or not there was a bake to defer. The tool is not active, so
                // nothing is bound — and this memo is read downstream as "the doc the painter is
                // working on" (`on_active_doc` in the image-edit intents). Leaving it set on the
                // no-edits path left it naming a sprite the painter had already torn down: the
                // same stale-second-copy that made the canvas unreachable (Enio 2026-07-22).
                self.last_painter_pushed_entity = None;
            }
        }
        // Painter panel ⟷ tool bridge (W1 T1.5) — source push +
        // current_preview drain + pending_commit capture; on-canvas
        // overlay paints the canvas RGBA over the sprite footprint.
        // Sidebar Procreate-style lands in W2 (ph2d-panel-painter).
        let painter_dispatch_t0 = Instant::now();
        let painter_apply_committed = ph2d_app_painter::painter_bridge::dispatch(
            hero,
            tools,
            sim,
            // ⭐ O mundo de APRESENTAÇÃO: é lá que a malha posada da arte vive, e o chrome do canvas
            // desenha-se por cima dela (2026-09-15).
            present.world(),
            renderer,
            asset_db,
            atlas_asset_map,
            camera,
            window_size,
            vector_scene,
            paint_ctx.text,
            self.last_pointer,
            &mut self.last_painter_pushed_entity,
            &mut self.painter_preview,
            &mut self.painter_preview_gpu,
            &mut self.painter_gpu_preview,
            &mut self.painter_commit_requested,
            &mut self.painter_undo_requested,
            &mut self.painter_redo_requested,
            &mut self.donated_form,
            toasts,
            self.held_button.is_some(),
            crate::input_dispatch::fill_drag::fill_drag_armed(),
            // O funil de leitura de textura desta shell, entregue como fecho: a crate da
            // família não conhece o `texture_edit` nem o `SourceRead` dele.
            // PRECISION-READONLY: o fecho só entrega os pixels ao canvas de TRABALHO do Painter; a
            // escrita de volta na sprite é o Apply (`hero_intents::image_edit::painter`), por
            // `commit_edited_texture`, que avisa por dentro.
            |entity, sim, renderer, asset_db, atlas_asset_map| {
                crate::hero_intents::texture_edit::read_sprite_source(
                    entity,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                )
                .map(|src| {
                    let straight = src.image.into_straight();
                    (straight.pixels, straight.width, straight.height)
                })
            },
            &note_preview_px,
        );
        // Live-preview a non-selected sprite used as the brush Shape (so its opacity/blend remote-
        // control edits show in real time), into a SECOND preview slot/override.
        ph2d_app_painter::painter_bridge_shape_preview::drive_shape_source_preview(
            tools,
            renderer,
            &mut self.painter_shape_source_preview_gpu,
            toasts,
        );
        // Always measure (one Instant/frame) so the HUD's "paint ms" gauge is live, not gated on
        // the frame profiler. EWMA the painter CPU per frame = this frame's preview dispatch +
        // the coalesced re-stamp flush; publish reads it (1-frame lag — fine for a smoothed gauge).
        self.last_dispatch_us = painter_dispatch_t0.elapsed().as_micros() as u64;
        const PAINT_ALPHA: f32 = 0.1;
        let paint_ms_now = (self.last_dispatch_us + self.last_paint_stamp_us) as f32 / 1000.0;
        self.paint_ms_ewma = PAINT_ALPHA * paint_ms_now + (1.0 - PAINT_ALPHA) * self.paint_ms_ewma;
        if frame_prof_on() {
            FRAME_PROF_DISPATCH_US.with(|c| c.set(self.last_dispatch_us));
        }
        Some(painter_apply_committed)
    }
}
