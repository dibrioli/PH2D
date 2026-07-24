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
        // A LIVE Deform Transform owns undo while it's up: step the gizmo back through its gestures (and
        // finally un-lift), all WITHOUT touching the structural timeline — the whole transform is one
        // structural entry, committed only when it ends (Enio 2026-07-04).
        if self.transform_undo_step() {
            return true;
        }
        // ONE unified timeline: shape authoring (create / point-edit / reshape / Offset) and pixel bakes
        // (Apply / Apply & Keep) are all `ModelSnapshot` entries, so undo walks them in reverse chronological
        // order regardless of kind. Each entry carries the open-shape editor state, so a restore reinstates
        // the overlay together with the pixels — they can never desync (Enio 2026-06-28). First close any
        // coalesced Offset drag still in flight so it undoes as its own step.
        self.flush_shape_txn();
        if let Some(model) = self.undo.undo() {
            self.restore_model(*model);
            true
        } else {
            false
        }
    }

    /// Redo the most recently undone edit on the unified timeline. Returns `true` if an edit was redone.
    pub fn redo_last(&mut self) -> bool {
        // A live / just-un-lifted Transform owns redo: re-lift the gizmo (recreate it) or step a gizmo pose
        // FORWARD — mirroring transform_undo_step, WITHOUT touching the structural timeline (Enio 2026-07-04).
        if self.transform_redo_step() {
            return true;
        }
        // While a Transform float is live with nothing left to redo, the structural timeline is frozen (the
        // transform is one pending entry) — swallow redo so it can't reinstate a stale structural state.
        if self.deform_transform_live() {
            return false;
        }
        self.flush_shape_txn();
        if let Some(model) = self.undo.redo() {
            self.restore_model(*model);
            true
        } else {
            false
        }
    }

    /// `true` if there is at least one edit to undo (an open in-flight shape transaction counts).
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.has_pending_shape_txn() || self.undo.can_undo()
    }

    /// `true` if there is at least one undone edit to redo.
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

    /// `true` when the working canvas/composite has edits not yet baked into the sprite (any stroke,
    /// layer op or adjustment since the last `set_source` bind). The shell uses this to auto-persist
    /// the painting when the selection leaves the sprite or the tool deactivates (Enio 2026-06-24).
    #[must_use]
    pub fn has_unbaked_edits(&self) -> bool {
        self.edited_since_bind && !self.canvas_rgba.is_empty()
    }

    /// Mark the working canvas as fully baked into the sprite — clears the unbaked-edits flag without
    /// touching the canvas (the shell calls this right after a successful auto-bake).
    pub fn mark_baked(&mut self) {
        self.edited_since_bind = false;
    }

    /// Take (and clear) the "deactivation deferred a bake" flag set by [`crate::tool::PainterTool`]'s
    /// `on_deactivate` when it kept the canvas. `true` → the shell must bake the kept canvas back to
    /// the last-bound sprite, then call `deactivate()` to finish the teardown.
    pub fn take_deferred_bake(&mut self) -> bool {
        std::mem::take(&mut self.deferred_bake)
    }

    /// Dimensions of the working canvas in pixels (`set_source` sets, `deactivate`
    /// zeroes).
    #[must_use]
    pub fn canvas_size(&self) -> (u32, u32) {
        self.source_size
    }

    /// Composite the live layers to a flat `Rec.601` luminance WITHOUT baking — the source for "Use as
    /// Brush Grain" on the active document, so the grain reflects the LIVE painting without re-pushing
    /// the sprite (a re-push runs `set_source`, which resets the layer stack and would destroy the
    /// user's layers). `None` for an empty canvas. Read-only (never mutates the document).
    #[must_use]
    pub fn composite_to_lum(&self) -> Option<(Vec<u8>, u32, u32)> {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return None;
        }
        let active = self.layers.active().unwrap_or(RtLayerId(0));
        let src = ToolPixelSource {
            active_id: active,
            active_rgba: &self.canvas_rgba,
            images: &self.images,
        };
        let rgba = composite(&self.layers, &src, w, h);
        let lum = rgba
            .chunks_exact(4)
            .map(|p| {
                ((u32::from(p[0]) * 77 + u32::from(p[1]) * 150 + u32::from(p[2]) * 29) >> 8) as u8
            })
            .collect();
        Some((lum, w, h))
    }

    /// Sample the visible layer COMPOSITE at normalized `(u, v)` ∈ `[0, 1]` → straight-sRGB8 RGBA —
    /// the displayed pixel, integrated with the layer stack (opacity / blend / masks / adjustments).
    /// Drives the colour-picker eyedropper so it reads the painted colour, not the transparent Vello
    /// overlay the generic GPU readback hits. Composites only the 1×1 pixel (the eyedrop is a one-off).
    #[must_use]
    pub fn sample_composite_at_uv(&self, u: f32, v: f32) -> Option<[u8; 4]> {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || self.canvas_rgba.is_empty() {
            return None;
        }
        let ix = ((u.clamp(0.0, 1.0) * w as f32) as u32).min(w - 1);
        let iy = ((v.clamp(0.0, 1.0) * h as f32) as u32).min(h - 1);
        let src = ToolPixelSource {
            active_id: self.layers.active().unwrap_or(RtLayerId(0)),
            active_rgba: &self.canvas_rgba,
            images: &self.images,
        };
        let px = composite_region(
            &self.layers,
            &src,
            w,
            h,
            Region {
                x: ix,
                y: iy,
                w: 1,
                h: 1,
            },
        );
        (px.len() >= 4).then(|| [px[0], px[1], px[2], px[3]])
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
        // Past the guard we are returning a fresh preview, so the CONTENT changed since the last drain:
        // bump the version the shell keys its upload on. This is the ONE place a Some is produced, so a
        // single bump here covers the trivial, composite and mask-view returns below. (See the field doc
        // on `preview_version`: the version replaces the shell's old `Arc::as_ptr` compare, which forced a
        // whole-canvas copy per move.)
        self.preview_version += 1;
        let (w, h) = self.source_size;
        // A mask row's grayscale-VIEW eye is open → show that mask's grayscale instead of the composite.
        if let Some(gray) = self.mask_grayscale_view_pixels() {
            self.composited = Some(Arc::new(gray));
            self.preview_upload_bbox = None;
            return Some((
                Arc::clone(self.composited.as_ref().expect("just set")),
                w,
                h,
            ));
        }
        // The trivial stack (single visible opaque Normal raster) stays the
        // zero-copy fast path. Any non-trivial stack (≥2 layers, or a layer with
        // opacity<1 / non-Normal blend / hidden / an adjustment) MUST be
        // composited here so per-layer opacity / blend / adjustments are visible.
        // Impasto forces the composite path: this lane hands back the RAW `canvas_rgba` Arc, which the
        // light pass must never write into (those are the artist's PIXELS, not a preview buffer) — and a
        // single-layer document is the common case, so without this guard the most ordinary way to use
        // Impasto would show no relief at all.
        if self.is_trivial_stack() && !self.mask_scratch_active() && !self.impasto_visible() {
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
        // While the Mask brush's scratch is live the composite carries the protection-overlay TINT over the
        // whole frame; a partial-region blit can't re-tint the untouched area, so force the full recompose
        // path (a single-layer recompose is cheap — not the 100k-sprite hot path).
        let force_full = self.mask_scratch_active();
        match (self.composited.is_some() && !force_full, dirty) {
            (true, Some(bbox)) => {
                let region = {
                    let src = ToolPixelSource {
                        active_id: active,
                        active_rgba: &self.canvas_rgba,
                        images: &self.images,
                    };
                    composite_region(&self.layers, &src, w, h, bbox)
                };
                // Impasto: light the FRESHLY-composited region (never the cache — lighting is not
                // idempotent, and re-lighting the cached pixels would compound the shading a little
                // more every frame). The normal reads across the region's edge into the full height
                // field, so the border is lit exactly as a full recompose would light it.
                let mut region = region;
                self.apply_impasto_light(&mut region, bbox);
                let region = region;
                self.compositor_cache.invalidate_from(active, &self.layers);
                self.adjustment_cache_pending = false;
                let cache = Arc::make_mut(self.composited.as_mut().expect("checked is_some"));
                blit_region(cache, w, &region, bbox);
                self.preview_upload_bbox = Some(bbox);
            }
            _ => {
                // Mask brush: the active layer composites NORMALLY (fully visible) — the protection scratch
                // never hides it; only `apply_mask_overlay` below tints the frozen region.
                let src = ToolPixelSource {
                    active_id: active,
                    active_rgba: &self.canvas_rgba,
                    images: &self.images,
                };
                let mut composed =
                    if std::mem::take(&mut self.adjustment_cache_pending) && !stroke_dirtied {
                        // Adjustment slider-drag: restart from the cut-point cache —
                        // bit-identical to a full `composite` (gate
                        // `cache_matches_full_recompose`).
                        composite_with_cache(&self.layers, &src, w, h, &mut self.compositor_cache)
                    } else {
                        self.compositor_cache.invalidate_from(active, &self.layers);
                        composite(&self.layers, &src, w, h)
                    };
                // Impasto: light the whole freshly-composited canvas (see the dirty-rect lane above).
                self.apply_impasto_light(&mut composed, Region { x: 0, y: 0, w, h });
                // Mask overlay: tint the composite by the active mask's coverage (no-op otherwise).
                self.apply_mask_overlay(&mut composed);
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

    /// Monotonic CONTENT version of the preview, bumped once per dirty
    /// [`Self::take_preview_arc`]. The shell keys its GPU-slot upload on this
    /// instead of the drained `Arc`'s pointer, so it never has to hold a clone of
    /// `canvas_rgba` across the frame — which is what forced `stamp_dabs` to copy
    /// the whole canvas per move. Read it right after the drain that produced the
    /// preview (its value pairs with the `Arc` that drain returned).
    #[must_use]
    pub fn canvas_version(&self) -> u64 {
        self.preview_version
    }
}
