//! **Fase do quadro: A EDIÇÃO DE IMAGEM E OS DESMONTES DO APPLY** — o dreno das ferramentas de imagem
//! (`image_edit::dispatch`) e, DEPOIS dele, o desmonte de cada ferramenta cujo Apply acabou de assar (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Os desmontes correm DEPOIS do dreno: o assar precisa da ferramenta ainda activa para ler o resultado.

use super::*;

/// Os pedidos das ferramentas de imagem que o dreno do barramento recolheu neste quadro — consumidos só
/// por esta fase, com os mesmos nomes do quadro.
pub(super) struct ImageEditIntents {
    pub(super) trim_entities: Vec<u64>,
    pub(super) make_square_entities: Vec<u64>,
    pub(super) real_size_entities: Vec<u64>,
    pub(super) rasterize_entities: Vec<u64>,
    pub(super) undo_image_edit: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    #[allow(clippy::too_many_arguments)] // os insumos do quadro, pelo nome que o corpo já usa
    pub(super) fn fase_image_edit_apply(
        &mut self,
        intents: ImageEditIntents,
        padding_apply: Option<(ph2d_tool_padding::PaddingSpec, bool, Vec<u64>)>,
        bgremoval_apply_committed: bool,
        color_equalization_apply: std::option::Option<std::vec::Vec<u64>>,
        equalize_sizes_apply: std::option::Option<std::vec::Vec<u64>>,
        upscale_apply: std::option::Option<std::vec::Vec<u64>>,
        painter_apply_committed: bool,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            renderer,
            sim,
            camera,
            asset_db,
            toasts,
            tools,
            vec_scene,
            hero_screen,
            next_import_cell,
            atlas_asset_map,
            image_edit_undo,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let ImageEditIntents {
            trim_entities,
            make_square_entities,
            real_size_entities,
            rasterize_entities,
            undo_image_edit,
        } = intents;
        // Image-edit drain phase + file-picker import — extracted
        // to sibling `image_edit.rs` as a free fn (Wave 3.2 stage A).
        // Returns whether any drain pushed a toast.
        // `padding_apply` carries a `Vec<u64>` (not `Copy`) — capture
        // the Apply-fired flag here so the teardown below can run
        // after the dispatch consumes the value.
        let padding_apply_fired = padding_apply.is_some();
        // Did a texture-RESIZING edit (rasterize / trim / make-square / real-size) hit the SELECTED
        // sprite? If so the Painter's working canvas is now the wrong resolution — reset the
        // push-tracker (below, after the lists are consumed) so `drive_source_push` re-reads the
        // sprite at its new size next frame, re-locking the brush / eyedropper / repeat-image.
        let painter_src_resized = hero.gizmo.selection.is_some_and(|sel| {
            rasterize_entities.contains(&sel)
                || trim_entities.contains(&sel)
                || make_square_entities.contains(&sel)
                || real_size_entities.contains(&sel)
        });
        if image_edit::dispatch(
            trim_entities,
            make_square_entities,
            real_size_entities,
            rasterize_entities,
            padding_apply,
            color_equalization_apply.clone(),
            equalize_sizes_apply.clone(),
            upscale_apply.clone(),
            undo_image_edit,
            hero,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
            toasts,
            image_edit_undo,
            tools,
            camera,
            next_import_cell,
            &mut self.bgremoval.last_pushed_entity,
            &mut self.last_painter_pushed_entity,
            vec_scene,
            &mut self.vec.entities,
        ) {
            self.title_dirty = true;
        }
        // A resize hit the selected sprite → force the Painter to re-read it at the new resolution
        // (see `painter_src_resized` above). The re-push replaces the now-invalid working canvas.
        if painter_src_resized {
            self.last_painter_pushed_entity = None;
        }
        // Apply teardown — runs AFTER the bake above (which needs
        // the BgRemovalTool still active to read the result). Now
        // that the committed alpha lives in the sprite texture,
        // deactivate the tool exactly like Cancel: the panel hides,
        // the sprite un-suppresses, the Inspector returns, and the
        // on-canvas preview overlay stops re-rendering on top of the
        // freshly baked sprite (that double-draw was the ghost edge
        // outline that appeared only while the image stayed selected).
        if bgremoval_apply_committed
            && let Some(default_id) = tools.default_tool_id()
            && tools.set_active(&default_id)
        {
            self.bgremoval.last_pushed_entity = None;
            self.bgremoval.preview = None;
            self.title_dirty = true;
        }
        // Padding Apply teardown — deactivate the tool so the panel
        // hides + the Inspector returns, exactly like Bg Removal.
        if padding_apply_fired
            && let Some(default_id) = tools.default_tool_id()
            && tools.set_active(&default_id)
        {
            self.title_dirty = true;
        }
        // Color Equalization Apply teardown — deactivate the tool
        // (panel hides, sprite returns to its un-edited live state
        // visually, multi-selection preserved). Mirror of Padding.
        if color_equalization_apply.is_some()
            && let Some(default_id) = tools.default_tool_id()
            && tools.set_active(&default_id)
        {
            self.last_color_equalization_pushed_entity = None;
            self.title_dirty = true;
        }
        // Equalize Sizes Apply teardown — same shape as Padding /
        // Color EQ: bake just ran, so switch back to the default
        // tool. The panel auto-hides because its `panel_visible`
        // gate keys off `tools.active().id() == "equalize_sizes"`
        // and the bridge clears the published snapshot on the
        // next frame.
        if equalize_sizes_apply.is_some()
            && let Some(default_id) = tools.default_tool_id()
            && tools.set_active(&default_id)
        {
            self.title_dirty = true;
        }
        // Upscale Apply teardown — mirror of Color EQ. Clear the
        // preview cache + push-tracker so re-activating starts
        // fresh against the new (post-bake) source.
        if upscale_apply.is_some()
            && let Some(default_id) = tools.default_tool_id()
            && tools.set_active(&default_id)
        {
            self.last_upscale_pushed_entity = None;
            self.upscale_preview = None;
            self.title_dirty = true;
        }
        // Painter Apply teardown (W1 T1.5) — same shape as BgR /
        // Upscale: deactivate the tool so the chrome returns to its
        // pre-painting state, and clear the preview/push-tracker so
        // re-activating starts fresh against the freshly-baked sprite.
        if painter_apply_committed
            && let Some(default_id) = tools.default_tool_id()
            && tools.set_active(&default_id)
        {
            self.last_painter_pushed_entity = None;
            self.painter_preview = None;
            self.title_dirty = true;
        }
    }
}
