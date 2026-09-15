//! **Fase do quadro: AS PONTES DAS FERRAMENTAS DE IMAGEM** — as pontes do Padding, da remoção de fundo, da equalização de cor, de igualar tamanhos e
//! do Upscale (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// O que as pontes das ferramentas de imagem devolvem ao quadro: os pedidos de aplicar de cada uma, para o dreno que os honra.
pub(super) struct ImageToolBridgesOut {
    pub(super) padding_apply: Option<(ph2d_tool_padding::PaddingSpec, bool, Vec<u64>)>,
    pub(super) bgremoval_apply_committed: bool,
    pub(super) color_equalization_apply: Option<Vec<u64>>,
    pub(super) equalize_sizes_apply: Option<Vec<u64>>,
    pub(super) upscale_apply: Option<Vec<u64>>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_image_tool_bridges(
        &mut self,
        window_size: ph2d_host::WindowSize,
    ) -> Option<ImageToolBridgesOut> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            renderer,
            sim,
            present,
            camera,
            asset_db,
            toasts,
            tools,
            vector_scene,
            hero_screen,
            atlas_asset_map,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // Padding panel ⟷ tool bridge — publishes the snapshot, draws
        // the live (non-destructive) canvas-bounds preview, and returns
        // the (selection, spec, pivot mode) to bake on Apply. Panel
        // events themselves are routed earlier in the frame via
        // `EditorAction::ToolPanelEvent` → `Tool::handle_panel_event`
        // (ADR-0040 TG-C). Sibling `padding_bridge.rs`.
        let padding_apply =
            padding_bridge::dispatch(hero, tools, sim, camera, window_size, vector_scene);
        // Bg Removal panel ⟷ tool bridge + on-canvas live preview
        // — extracted to sibling `bgremoval_preview.rs` (HR-18 LOC).
        // Panel events now flow through `EditorAction::ToolPanelEvent`
        // (drained above into `handle_panel_event` → `apply_ui_edit`);
        // the canvas-preview cache is gated on `BgRemovalTool::take_params_dirty`
        // instead of a per-frame edits vector (ADR-0040 TG-B).
        let bgremoval_apply_committed = bgremoval_preview::dispatch(
            hero,
            tools,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
            camera,
            window_size,
            vector_scene,
            &mut self.last_bgremoval_pushed_entity,
            &mut self.bgremoval_preview,
            &mut self.bgremoval_preview_gpu,
            present.world(),
            &mut self.bgremoval_tint_gpu,
            &mut self.bgremoval_tint_extra,
            toasts,
        );
        // Color Equalization panel ⟷ tool bridge: drives panel
        // visibility, refreshes the tool's source bitmap when the
        // primary changes, publishes the snapshot the panel paints,
        // and returns the multi-selection on Apply for the bake.
        let color_equalization_apply = color_equalization_bridge::dispatch(
            hero,
            tools,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
            camera,
            window_size,
            vector_scene,
            &mut self.last_color_equalization_pushed_entity,
            &mut self.color_equalization_previews,
            toasts,
        );
        // Equalize Sizes panel ⟷ tool bridge — multi-sprite, no
        // per-frame on-canvas preview (the visual effect is the
        // Apply bake; an interim transform-only preview is future
        // work). Returns the full `iter_selected()` on Apply for
        // the cross-sprite `run_full_resolution_multi` bake.
        let equalize_sizes_apply = equalize_sizes_bridge::dispatch(hero, tools);
        // Upscale panel ⟷ tool bridge — sabor 3 with on-canvas
        // live preview (algo + scale apply each frame the user
        // moves the slider). Mirror of `color_equalization_bridge`.
        let upscale_apply = upscale_bridge::dispatch(
            hero,
            tools,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
            camera,
            window_size,
            vector_scene,
            &mut self.last_upscale_pushed_entity,
            &mut self.upscale_preview,
        );
        Some(ImageToolBridgesOut {
            padding_apply,
            bgremoval_apply_committed,
            color_equalization_apply,
            equalize_sizes_apply,
            upscale_apply,
        })
    }
}
