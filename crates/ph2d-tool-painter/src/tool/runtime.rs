//! See `tool/mod.rs` — dock toggle + accessors + preview-composite drive +
//! structural undo/redo, split out of the former `tool.rs` god-object.

use super::*;

impl PainterTool {
    // ── Dock toggle (mode C) ────────────────────────────────────────────

    /// Whether the layers panel occupies the shared right-dock slot. The shell
    /// `painter_bridge` reads this to compute `panel_visibility`.
    #[must_use]
    pub fn dock_shows_layers(&self) -> bool {
        self.dock_shows_layers
    }

    /// Flip the dock slot (the layers panel header toggle button).
    pub fn toggle_dock(&mut self) {
        self.dock_shows_layers = !self.dock_shows_layers;
    }

    /// Decode a per-row layers-panel widget [`NodeId`] back to its
    /// `(layer, kind)` by recomputing [`painter_layer_widget_id`] for every
    /// current layer × kind and matching. `None` if `id` isn't a per-row
    /// widget of any layer. (≤8 layers × kinds = cheap FNV hashes; the layers
    /// panel is not a hot path.)
    pub(crate) fn decode_layer_widget(
        &self,
        id: ph2d_a11y::NodeId,
    ) -> Option<(RtLayerId, ph2d_editor_core::ids::PainterLayerWidget)> {
        use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
        for layer in self.layers.all_ids() {
            for kind in PainterLayerWidget::ALL {
                if painter_layer_widget_id(layer.0, kind) == id {
                    return Some((layer, kind));
                }
            }
        }
        None
    }

    // ── Structural undo / redo (layer model) ────────────────────────────

    /// Undo the most recent structural layer edit, reinstalling the prior model.
    /// Returns `true` if an edit was undone. Driven by the shell's undo gesture /
    /// shortcut.
    pub fn undo_last(&mut self) -> bool {
        if let Some(model) = self.undo.undo() {
            self.restore_model(*model);
            true
        } else {
            false
        }
    }

    /// Redo the most recently undone structural layer edit. Returns `true` if an
    /// edit was redone.
    pub fn redo_last(&mut self) -> bool {
        if let Some(model) = self.undo.redo() {
            self.restore_model(*model);
            true
        } else {
            false
        }
    }

    /// `true` if there is at least one structural edit to undo.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.undo.can_undo()
    }

    /// `true` if there is at least one undone structural edit to redo.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.undo.can_redo()
    }

    // ── Apply / preview drive ───────────────────────────────────────────

    /// Requisita commit (Apply): the bridge fires `EditorAction::OneShotImageOp`
    /// next frame, which calls `run_full` to bake the layer composite into the
    /// sprite.
    pub fn request_commit(&mut self) {
        self.pending_commit = true;
    }

    /// Dimensions of the working canvas in pixels (`set_source` sets, `deactivate`
    /// zeroes).
    #[must_use]
    pub fn canvas_size(&self) -> (u32, u32) {
        self.source_size
    }

    /// **Zero-copy preview drain** via Arc clone (1 atomic increment). Drains
    /// `preview_dirty` and returns the SAME underlying `Arc<Vec<u8>>` for a
    /// trivial stack, or a freshly-composited cache for a multi-layer stack. The
    /// bridge stashes this Arc in its `painter_preview` cache.
    #[must_use]
    pub fn take_preview_arc(&mut self) -> Option<(Arc<Vec<u8>>, u32, u32)> {
        if !std::mem::take(&mut self.preview_dirty) || self.canvas_rgba.is_empty() {
            return None;
        }
        let (w, h) = self.source_size;
        // The trivial stack (single visible opaque Normal raster) stays the
        // zero-copy fast path. Any non-trivial stack (≥2 layers, or a layer with
        // opacity<1 / non-Normal blend / hidden / an adjustment) MUST be
        // composited here so per-layer opacity / blend / adjustments are visible.
        if self.is_trivial_stack() {
            // Carry the accumulated dirty bbox into the bridge so a stroke uploads
            // only the touched sub-rect (B.1 partial lane), NOT the whole canvas
            // each frame. `None` here forced a full clone + premul + full texture
            // upload per painted frame, O(W×H) regardless of the 10px dab — the
            // 300→150 fps stroke regression. The first drain (source-push, no paint
            // yet) has no `dirty_rect` → stays `None` → the bridge seeds the GPU
            // texture with one full upload; paint frames then patch just the dab
            // footprint. If the texture isn't seeded yet the bridge's guard falls
            // back to a full upload anyway, so an early bbox can never desync it.
            self.preview_upload_bbox = self.dirty_rect.take();
            return Some((Arc::clone(&self.canvas_rgba), w, h));
        }
        let active = self.layers.active().unwrap_or(RtLayerId(0));
        // Dirty-rect fast lane: when a valid full composite is cached AND only a
        // known bbox changed, recomposite ONLY that region and blit it into the
        // cache — O(N×bbox) vs O(N×W×H). Otherwise do a full recompose.
        let dirty = self.dirty_rect.take();
        let stroke_dirtied = dirty.is_some();
        match (self.composited.is_some(), dirty) {
            (true, Some(bbox)) => {
                let region = {
                    let src = ToolPixelSource {
                        active_id: active,
                        active_rgba: &self.canvas_rgba,
                        images: &self.images,
                    };
                    composite_region(&self.layers, &src, w, h, bbox)
                };
                self.compositor_cache.invalidate_from(active, &self.layers);
                self.adjustment_cache_pending = false;
                let cache = Arc::make_mut(self.composited.as_mut().expect("checked is_some"));
                blit_region(cache, w, &region, bbox);
                self.preview_upload_bbox = Some(bbox);
            }
            _ => {
                let src = ToolPixelSource {
                    active_id: active,
                    active_rgba: &self.canvas_rgba,
                    images: &self.images,
                };
                let composed =
                    if std::mem::take(&mut self.adjustment_cache_pending) && !stroke_dirtied {
                        // Adjustment slider-drag: restart from the cut-point cache —
                        // bit-identical to a full `composite` (gate
                        // `cache_matches_full_recompose`).
                        composite_with_cache(&self.layers, &src, w, h, &mut self.compositor_cache)
                    } else {
                        self.compositor_cache.invalidate_from(active, &self.layers);
                        composite(&self.layers, &src, w, h)
                    };
                self.composited = Some(Arc::new(composed));
                self.preview_upload_bbox = None;
            }
        }
        Some((
            Arc::clone(self.composited.as_ref().expect("just set")),
            w,
            h,
        ))
    }

    /// Drains the dirty bbox `(x, y, w, h)` of the LAST [`Self::take_preview_arc`]
    /// — `Some` iff that drain was a partial fast-lane update the bridge may
    /// upload as a sub-rect; `None` = upload the full texture.
    pub fn take_preview_upload_bbox(&mut self) -> Option<(u32, u32, u32, u32)> {
        self.preview_upload_bbox
            .take()
            .map(|r| (r.x, r.y, r.w, r.h))
    }

    /// Monotonic revision of the published layer structure. The bridge publishes
    /// the `LayerStack` snapshot only when this changes. Bumped by
    /// `invalidate_composite` (all structural/metadata edits) + `set_source`.
    #[must_use]
    pub fn layers_revision(&self) -> u64 {
        self.layers_revision
    }
}
