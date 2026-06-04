//! See `tool/mod.rs` — this is W3 layer model — layers / selection / masks / adjustments / groups,
//! split out of the former `tool.rs` god-object (pure mechanical move).

use super::*;

/// Max control points per Curves channel (ADR-0045 §2.6 `ControlPoints` cap).
/// The bespoke editor's add-point button stops here.
pub const MAX_CURVE_POINTS_PER_CHANNEL: usize = 8;

impl PainterTool {
    // ── W3 layer model (runtime canon, ADR-0046-amд-1 Option A) ─────────

    /// Read-only view of the runtime layer stack — what the docked layers
    /// panel renders (the shell snapshots `layers().clone()` per frame).
    #[must_use]
    pub fn layers(&self) -> &LayerStack {
        &self.layers
    }

    /// Canvas dimensions `(width, height)` of the current source — what the GPU
    /// preview composites at. `(0, 0)` before any `set_source`.
    #[must_use]
    pub fn source_size(&self) -> (u32, u32) {
        self.source_size
    }

    /// Bump `id`'s pixel CONTENT version (the GPU preview compositor's cache
    /// key — see [`Self::layer_pixel_versions`]). Call from every canvas
    /// pixel-write chokepoint (stroke stamp, undo/redo, mask flatten, fresh
    /// source). No-op when `id` is `None` (degenerate / empty stack).
    pub(crate) fn bump_layer_pixels(&mut self, id: Option<RtLayerId>) {
        if let Some(id) = id {
            self.pixel_clock = self.pixel_clock.wrapping_add(1);
            self.layer_pixel_versions.insert(id, self.pixel_clock);
        }
    }

    /// Borrow one layer's straight-sRGB8 pixels + its content version for the
    /// GPU preview compositor (the shell bridge adapts this to
    /// `ph2d_render::LayerPixelProvider`; the tool stays decoupled from the
    /// render crate). The ACTIVE layer reads the live `canvas_rgba` working
    /// buffer (always current — strokes mutate it in place); every other layer
    /// reads its `images` entry. `version` changes iff the layer's PIXELS
    /// changed (see [`Self::layer_pixel_versions`]), so the compositor
    /// re-uploads a slice only on a real pixel edit. `None` for an unknown key
    /// or an empty active buffer (the bridge then falls back to the CPU
    /// compositor). W3 GPU preview (ADR-0045 Phase 3, step 2).
    #[must_use]
    pub fn preview_layer_pixels(&self, key: u64) -> Option<(u64, &[u8])> {
        let id = RtLayerId(key);
        let version = self.layer_pixel_versions.get(&id).copied().unwrap_or(0);
        if self.layers.active() == Some(id) {
            let buf = self.canvas_rgba.as_ref().as_slice();
            if buf.is_empty() {
                return None;
            }
            Some((version, buf))
        } else {
            self.images
                .get(&id)
                .map(|img| (version, img.rgba8.as_slice()))
        }
    }

    /// Drain the `preview_dirty` flag WITHOUT compositing — the GPU preview
    /// path's equivalent of the dirty gate inside [`Self::take_preview_arc`],
    /// for when the shell composites on the GPU instead of the CPU. Returns
    /// `true` iff the preview changed since the last drain (so the bridge
    /// recomposites this frame). Mirrors `take_preview_arc`'s empty-canvas
    /// guard so an un-sourced tool reports clean.
    #[must_use]
    pub fn take_preview_dirty(&mut self) -> bool {
        if self.canvas_rgba.is_empty() {
            return false;
        }
        std::mem::take(&mut self.preview_dirty)
    }

    /// `true` when the stack is a single visible, opaque, Normal raster with
    /// no mask/clip — i.e. the composite is byte-identical to `canvas_rgba`,
    /// so `current_preview` skips compositing entirely (T1.5 fast path).
    pub(crate) fn is_trivial_stack(&self) -> bool {
        let root = self.layers.root();
        if root.len() != 1 {
            return false;
        }
        match self.layers.get(root[0]) {
            Some(l) => {
                matches!(l.kind, LayerKind::Raster(_))
                    && l.visible
                    && l.opacity >= 1.0
                    && l.blend_mode == BlendMode::Normal
                    && l.mask.is_none()
                    && !l.clipping
            }
            None => true, // empty/degenerate → nothing to composite
        }
    }

    /// Mark the composite stale so the next `current_preview` recomputes.
    pub(crate) fn invalidate_composite(&mut self) {
        self.composited = None;
        // Drop any accumulated dirty-rect: a structural edit (opacity / blend /
        // visibility / reorder / add / select) can change the composite OUTSIDE
        // the stamped region, so the next drain must do a FULL recompose.
        self.dirty_rect = None;
        self.preview_dirty = true;
        // W5: a structural edit (add/remove/reorder/visibility/opacity/blend/
        // select) changes the composite below some adjustment → every cut is
        // potentially stale. Conservative-correct: drop them all (they cold-
        // refill on the next slider-drag). The id is ignored by `invalidate_from`.
        self.compositor_cache
            .invalidate_from(RtLayerId(0), &self.layers);
        // B.5: every structural/metadata edit funnels through here, so this is
        // the single chokepoint that bumps the publish revision (set_source is
        // the only structural reset that bypasses it — bumped there too).
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// Drop undo/redo history at a layer switch. **Audit W3 (data-loss):**
    /// the undo controller snapshots `canvas_rgba` = whatever layer is active
    /// NOW; it is NOT yet layer-keyed. Without this reset, undo after switching
    /// layers would blit one layer's pre-image onto a DIFFERENT layer's working
    /// buffer. Clearing on switch makes undo not cross a layer change — safe
    /// v1; per-layer (transactional) undo is the Coordinator's follow-up.
    /// Mirror of the reset `set_source` does.
    pub(crate) fn reset_undo_after_layer_switch(&mut self) {
        self.undo.clear();
        self.undo_redo_records.clear();
        self.pending_pre_stroke = None;
    }

    /// Set a layer's visibility (layers panel edit). No-op if `id` unknown.
    ///
    /// Unlike the STRUCTURAL edits (`add_raster_layer`/`select_layer`/
    /// `move_layer_*`), the three metadata setters below are intentionally NOT
    /// guarded by `!stroke_active`: they only touch `Layer` flags (no buffer
    /// swap, no `canvas_rgba`/`images`/undo reshuffle), so they cannot corrupt
    /// an in-flight stroke. `invalidate_composite` just drops the cache, which a
    /// mid-stroke edit recovers from on the next drain.
    pub fn set_layer_visible(&mut self, id: RtLayerId, visible: bool) {
        self.layers.set_visible(id, visible);
        self.invalidate_composite();
    }

    /// Set a layer's opacity (0..=1). No-op if `id` unknown.
    pub fn set_layer_opacity(&mut self, id: RtLayerId, opacity: f32) {
        self.layers.set_opacity(id, opacity);
        self.invalidate_composite();
    }

    /// Set a layer's blend mode. No-op if `id` unknown.
    pub fn set_layer_blend_mode(&mut self, id: RtLayerId, mode: BlendMode) {
        self.layers.set_blend_mode(id, mode);
        self.invalidate_composite();
    }

    /// Toggle a layer's clipping-mask modifier (§2.8) — non-destructive (the
    /// painting is preserved; only the composite changes). No-op if `id` unknown.
    pub fn set_layer_clipping(&mut self, id: RtLayerId, clipping: bool) {
        self.layers.set_clipping(id, clipping);
        self.invalidate_composite();
    }

    /// Toggle a layer's alpha-lock modifier (§2.10). Future paint into this
    /// layer is then clamped to its existing alpha. No-op if `id` unknown.
    pub fn set_layer_alpha_locked(&mut self, id: RtLayerId, locked: bool) {
        self.layers.set_alpha_locked(id, locked);
        self.invalidate_composite();
    }

    /// Toggle a layer's reference modifier (§2.9) — exclusive (setting one
    /// clears the others). No-op if `id` unknown. Full ColorDrop behavior is W7;
    /// this is the non-destructive flag + UI badge.
    pub fn set_layer_reference(&mut self, id: RtLayerId, is_reference: bool) {
        self.layers.set_reference(id, is_reference);
        self.invalidate_composite();
    }

    /// Add an empty group at the top of the stack (§2.1). No-op (`None`)
    /// mid-stroke or at the hard cap.
    pub fn add_group(&mut self) -> Option<RtLayerId> {
        if self.stroke_active {
            return None;
        }
        let id = self.layers.add_group("Group")?;
        self.invalidate_composite();
        Some(id)
    }

    /// Wrap the ACTIVE layer in a new group and return the group id. Interim
    /// behavior until multi-select lands (group-the-selection): for now a single
    /// active layer is nested so the "Group" button does something visible. The
    /// active layer stays the edit target (its `canvas_rgba` is untouched). No-op
    /// (`None`) mid-stroke, with no active layer, on the base sprite, or at cap.
    pub fn group_active(&mut self) -> Option<RtLayerId> {
        if self.stroke_active {
            return None;
        }
        let active = self.layers.active()?;
        // The base sprite is pinned at root bottom — don't nest it.
        if self.layers.root().last() == Some(&active) {
            return None;
        }
        let g = self.layers.add_group("Group")?;
        if !self.layers.move_into_group(active, g) {
            self.layers.remove(g); // couldn't nest (depth/cycle) — drop the empty group
            return None;
        }
        self.layers.set_active(active); // keep painting the layer, not the group
        self.selection.clear();
        self.selection.insert(g);
        self.selection.insert(active);
        self.invalidate_composite();
        Some(g)
    }

    /// Apply a drag-drop reparent/reorder emitted by the dispatch
    /// (`WidgetEvent::PainterLayerReparent`). Reverses the dragged + target
    /// `NodeId`s to `LayerId`s via the per-row widget id, then dispatches to
    /// `move_into_group` (drop INTO a group) / `move_to_sibling_of` (before /
    /// after) / `move_to_root_bottom_above_base` (drop at the end). The
    /// LayerStack guards (base pinned, `MAX_GROUP_DEPTH`, cycle) reject unsafe
    /// drops. No-op mid-stroke or if `dragged` doesn't resolve to a layer.
    pub fn handle_layer_reparent(
        &mut self,
        dragged: ph2d_a11y::NodeId,
        drop: ph2d_editor_core::interaction::PainterLayerDrop,
    ) {
        use ph2d_editor_core::interaction::PainterLayerDrop;
        if self.stroke_active {
            return;
        }
        let Some(d) = self.decode_layer_widget(dragged).map(|(l, _)| l) else {
            return;
        };
        // Masks are owner-attached (not in the z-order) — never reparent one
        // (the row is selectable, but dragging it must not touch the stack).
        if matches!(
            self.layers.get(d).map(|l| &l.kind),
            Some(LayerKind::Mask(_))
        ) {
            return;
        }
        let moved = match drop {
            PainterLayerDrop::Inside(t) => match self.decode_layer_widget(t) {
                // Middle band: drop INTO a group folder. If the target isn't a
                // group (or the nest is rejected — depth cap / cycle),
                // `move_into_group` returns `false` WITHOUT mutating (its guards
                // precede the detach), so we fall back to a sibling insert ABOVE
                // the target. This kills the dead 40% middle band on normal layer
                // rows: every position over a row now resolves to a meaningful
                // move, and the panel's drop indicator mirrors it (box for a
                // group, before-line for a leaf). Photoshop/Procreate semantics:
                // you only nest into a folder; over a leaf it's always before/after.
                Some((tgt, _)) => {
                    self.layers.move_into_group(d, tgt)
                        || self.layers.move_to_sibling_of(d, tgt, false)
                }
                None => false,
            },
            PainterLayerDrop::Before(t) => match self.decode_layer_widget(t) {
                Some((tgt, _)) => self.layers.move_to_sibling_of(d, tgt, false),
                None => false,
            },
            PainterLayerDrop::After(t) => match self.decode_layer_widget(t) {
                Some((tgt, _)) => self.layers.move_to_sibling_of(d, tgt, true),
                None => false,
            },
            PainterLayerDrop::End => self.layers.move_to_root_bottom_above_base(d),
        };
        if moved {
            self.invalidate_composite();
        }
    }

    /// Remove `id` (and its subtree + mask) and clean up the tool buffers (the
    /// `images` entries + the active `canvas_rgba` + undo). The base sprite (root
    /// bottom) is NOT removable. No-op (`false`) mid-stroke, if `id` is unknown,
    /// or if `id` is the base.
    pub fn delete_layer(&mut self, id: RtLayerId) -> bool {
        if self.stroke_active || self.layers.get(id).is_none() {
            return false;
        }
        // The base sprite (bottom of root) is permanent — Apply bakes into it.
        if self.layers.root().last() == Some(&id) {
            return false;
        }
        let was_active = self.layers.active() == Some(id);
        self.layers.remove(id); // drops subtree + mask, repoints active
        // Drop `images` entries for any layer that no longer exists.
        let alive: std::collections::BTreeSet<RtLayerId> = self.layers.all_ids().collect();
        self.images.retain(|lid, _| alive.contains(lid));
        if was_active {
            // The deleted layer's working buffer is discarded with it; load the
            // NEW active's pixels into `canvas_rgba` (transparent if none).
            let (w, h) = self.source_size;
            let buf = self
                .layers
                .active()
                .and_then(|a| self.images.remove(&a))
                .map(|img| img.rgba8)
                .unwrap_or_else(|| vec![0u8; (w as usize) * (h as usize) * 4]);
            self.canvas_rgba = Arc::new(buf);
        }
        self.prune_selection(); // drop the deleted subtree + mask from the highlight
        self.reset_undo_after_layer_switch();
        self.invalidate_composite();
        true
    }

    /// Duplicate `id` (a raster) above itself — clones the layer + its pixels,
    /// inserts above, and makes the copy the active edit target. No-op (`None`)
    /// mid-stroke, for a non-raster, or at the cap.
    pub fn duplicate_layer(&mut self, id: RtLayerId) -> Option<RtLayerId> {
        if self.stroke_active {
            return None;
        }
        if !matches!(self.layers.get(id)?.kind, LayerKind::Raster(_)) {
            return None;
        }
        // Ensure the source pixels live in `images` (flush the active), copy them,
        // duplicate the model, then make the copy the active edit target.
        self.flush_active_to_images();
        let (w, h) = self.source_size;
        let src_pixels = self
            .images
            .get(&id)
            .map(|img| img.rgba8.clone())
            .unwrap_or_else(|| vec![0u8; (w as usize) * (h as usize) * 4]);
        let new_id = self.layers.duplicate(id)?; // sets active = new_id
        self.images.remove(&new_id); // active lives in canvas_rgba
        self.canvas_rgba = Arc::new(src_pixels);
        self.reset_undo_after_layer_switch();
        self.reset_selection_to(new_id);
        self.invalidate_composite();
        Some(new_id)
    }

    /// Snapshot the ACTIVE layer's working buffer (`canvas_rgba`) into
    /// `images` before a layer switch — so the layer we're leaving keeps its
    /// pixels. (The active layer is otherwise NOT stored in `images`.)
    pub(crate) fn flush_active_to_images(&mut self) {
        if let Some(active) = self.layers.active() {
            let (w, h) = self.source_size;
            self.images.insert(
                active,
                LayerImage {
                    width: w,
                    height: h,
                    rgba8: self.canvas_rgba.as_ref().clone(),
                },
            );
        }
    }

    /// Add a new transparent raster layer at the top of the stack and make
    /// it active. The previous active layer's pixels are flushed to `images`;
    /// the new active starts as a fresh transparent `canvas_rgba`. Returns the
    /// new id, or `None` mid-stroke / before a source is set / at the cap.
    pub fn add_raster_layer(&mut self, name: impl Into<String>) -> Option<RtLayerId> {
        if self.stroke_active {
            return None; // lifecycle: no structural edits mid-stroke
        }
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return None;
        }
        // Cap check BEFORE flushing (audit W3): otherwise a cap-hit add would
        // leave the just-flushed active layer stranded in `images`, breaking
        // the "active is never in images" invariant.
        if self.layers.len() >= crate::layers::HARD_CAP_LAYERS {
            return None;
        }
        self.flush_active_to_images();
        let id = self.layers.add_raster(name, w, h)?; // sets active = id (top)
        self.images.remove(&id); // active lives in canvas_rgba, not images
        self.canvas_rgba = Arc::new(vec![0u8; (w as usize) * (h as usize) * 4]);
        self.reset_undo_after_layer_switch();
        self.reset_selection_to(id);
        self.invalidate_composite();
        Some(id)
    }

    /// Create a grayscale mask on the ACTIVE raster layer (§2.7) and make the
    /// mask the edit target — a fresh opaque-WHITE buffer (fully visible). The
    /// brush then paints luminance into it (`effective_active_color` grays the
    /// stroke). No-op (`None`) mid-stroke, if the active layer isn't a raster,
    /// if it already has a mask, or at the hard cap.
    pub fn add_mask_to_active(&mut self) -> Option<RtLayerId> {
        if self.stroke_active {
            return None;
        }
        let active = self.layers.active()?;
        if !matches!(self.layers.get(active)?.kind, LayerKind::Raster(_)) {
            return None; // only a raster takes a mask (not a group / another mask)
        }
        let mask = self.layers.add_mask(active)?;
        // The parent raster (still active) flushes to images; the mask becomes
        // the active edit target with a fresh opaque-WHITE buffer (full visible).
        self.flush_active_to_images();
        let (w, h) = self.source_size;
        self.images.remove(&mask); // active lives in canvas_rgba, not images
        self.canvas_rgba = Arc::new(vec![255u8; (w as usize) * (h as usize) * 4]);
        self.layers.set_active(mask);
        self.sync_mask_brush_color(); // entering a mask → black brush (hide)
        self.reset_undo_after_layer_switch();
        self.reset_selection_to(mask);
        self.invalidate_composite();
        Some(mask)
    }

    /// Toggle a mask's `Invert` flag (§2.7). The live compositor already honors
    /// it (`1 - value`), so this just flips the flag + invalidates the
    /// composite. No-op mid-stroke or if `mask_id` is not a mask.
    pub fn toggle_mask_inverted(&mut self, mask_id: RtLayerId) {
        if self.stroke_active {
            return;
        }
        let inverted = match self.layers.get(mask_id).map(|l| &l.kind) {
            Some(LayerKind::Mask(m)) => m.inverted,
            _ => return,
        };
        self.layers.set_mask_inverted(mask_id, !inverted);
        self.invalidate_composite();
    }

    /// **Apply mask (§2.7) — destructive bake.** Multiply each parent texel's
    /// straight alpha by the mask coverage (Rec.601 luma, `1 - v` when the mask
    /// is inverted — EXACTLY the live compositor's `mask_value` path, so the
    /// baked result equals what was previewed), then remove the mask. The parent
    /// becomes the active edit target carrying the baked pixels (mirror of mask
    /// deletion). No-op (`false`) mid-stroke, if `mask_id` is not a mask, its
    /// parent is gone, or a buffer is missing/short.
    pub fn apply_mask(&mut self, mask_id: RtLayerId) -> bool {
        if self.stroke_active {
            return false;
        }
        // Resolve the mask + its invert flag, then the owning parent raster
        // (the layer whose `.mask == Some(mask_id)`).
        let inverted = match self.layers.get(mask_id).map(|l| &l.kind) {
            Some(LayerKind::Mask(m)) => m.inverted,
            _ => return false,
        };
        let Some(parent) = self
            .layers
            .all_ids()
            .find(|&p| self.layers.get(p).and_then(|l| l.mask) == Some(mask_id))
        else {
            return false;
        };
        // Flush the live active buffer so BOTH parent + mask pixels are in
        // `images` regardless of which (if either) is currently active.
        self.flush_active_to_images();
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let Some(mask_px) = self.images.get(&mask_id).map(|img| img.rgba8.clone()) else {
            return false;
        };
        let Some(parent_img) = self.images.get_mut(&parent) else {
            return false;
        };
        if parent_img.rgba8.len() < n * 4 || mask_px.len() < n * 4 {
            return false; // degenerate buffers — refuse rather than index OOB
        }
        for idx in 0..n {
            let v = crate::compositor::mask_value(&mask_px, idx);
            let cov = if inverted { 1.0 - v } else { v };
            let a = parent_img.rgba8[idx * 4 + 3] as f32;
            parent_img.rgba8[idx * 4 + 3] = (a * cov).round().clamp(0.0, 255.0) as u8;
        }
        // Remove the mask (this also scrubs `parent.mask` back to None + repoints
        // active off the mask) and drop its now-baked buffer.
        let was_active = self.layers.active();
        self.layers.remove(mask_id);
        self.images.remove(&mask_id);
        // GPU preview: the parent's alpha was baked → bump its content version.
        self.bump_layer_pixels(Some(parent));
        // The parent takes over as the active edit target iff the mask (or
        // nothing) was active; an unrelated active layer is left untouched.
        let new_active = match was_active {
            Some(a) if a == mask_id => parent,
            Some(a) => a,
            None => parent,
        };
        self.layers.set_active(new_active);
        // Reload `canvas_rgba` from `images[new_active]` (the baked parent when
        // `new_active == parent`), restoring the "active not in images" invariant.
        let buf = self
            .images
            .remove(&new_active)
            .map(|img| img.rgba8)
            .unwrap_or_else(|| vec![0u8; n * 4]);
        self.canvas_rgba = Arc::new(buf);
        self.sync_mask_brush_color(); // leaving the mask → restore the real color
        self.reset_undo_after_layer_switch();
        self.reset_selection_to(new_active);
        self.invalidate_composite();
        true
    }

    /// Create a non-destructive adjustment layer of `kind` at the top of the
    /// stack and SELECT it (highlight) WITHOUT changing the paint target — an
    /// adjustment has no pixel buffer, so the previously active raster stays the
    /// edit target (mirror of the group guard, [`Self::set_active_layer`]).
    /// No-op (`None`) mid-stroke or at the layer cap. W4 T4.3 (HSB Day-4 smoke).
    pub fn add_adjustment_layer(
        &mut self,
        kind: ph2d_painter_brush::adjustments::AdjustmentKind,
    ) -> Option<RtLayerId> {
        if self.stroke_active {
            return None;
        }
        let prev_active = self.layers.active();
        let id = self.layers.add_adjustment(kind)?; // LayerStack sets active = adj
        // Bespoke Curves editor: seed EVERY channel (master + R/G/B) with 5
        // evenly-spaced identity handles so the curve canvas — and each R/G/B tab —
        // opens with draggable control points. The data-model default stays empty
        // (a bit-exact identity for persisted / programmatic layers); a user-created
        // Curves layer gets editable handles. All-diagonal seeds = identity output.
        if kind == ph2d_painter_brush::adjustments::AdjustmentKind::Curves
            && let Some(adj) = self.layers.adjustment_mut(id)
            && let ph2d_painter_brush::adjustments::AdjustmentParams::Curves(c) = &mut adj.params
        {
            let identity: Vec<[f32; 2]> = (0..5)
                .map(|i| {
                    let t = i as f32 / 4.0;
                    [t, t]
                })
                .collect();
            c.points_rgb.points = identity.clone();
            c.points_r.points = identity.clone();
            c.points_g.points = identity.clone();
            c.points_b.points = identity;
        }
        // Not paintable — restore the prior raster as the edit target. The
        // canvas_rgba is untouched (add_adjustment does not flush/load), so a
        // plain `set_active` (no buffer dance) keeps it consistent.
        if let Some(p) = prev_active {
            self.layers.set_active(p);
        }
        self.reset_selection_to(id); // highlight the new adjustment row
        self.invalidate_composite();
        Some(id)
    }

    /// Set slider `slot` of adjustment layer `id` from a normalized `0..1`
    /// value (the panel's per-slot sliders). The per-kind mapping lives in
    /// `adjustments::set_adjustment_slider_param` (HSB / Brightness-Contrast /
    /// …). No-op mid-stroke or if `id` is not an adjustment. Invalidates the
    /// composite so the live preview re-renders.
    pub fn set_adjustment_param(&mut self, id: RtLayerId, slot: usize, slider01: f32) {
        if self.stroke_active {
            return;
        }
        let Some(adj) = self.layers.adjustment_mut(id) else {
            return;
        };
        ph2d_painter_brush::adjustments::set_adjustment_slider_param(
            &mut adj.params,
            slot,
            slider01,
        );
        // W5 slider-drag hot path: a param-only change leaves every layer BELOW
        // this adjustment untouched, so keep their cuts and drop only the cuts
        // ABOVE it; the next drain restarts from this adjustment's cut via
        // `composite_with_cache` instead of recomposing the whole stack. (NOT the
        // structural `invalidate_composite`, which would clear all cuts + force a
        // cold full recompose every drag frame — the exact cost we're killing.)
        self.compositor_cache.invalidate_above(id, &self.layers);
        self.composited = None;
        self.dirty_rect = None;
        self.adjustment_cache_pending = true;
        self.preview_dirty = true;
        // The adjustment's params live in the published LayerStack, so the panel
        // snapshot must republish (mirror of `invalidate_composite`'s bump).
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// Flip toggle `slot` of adjustment layer `id` (the panel's per-slot switches,
    /// e.g. Photo Filter's "Preserve Luminosity"). The params are the single
    /// source of truth, so the panel forwards a bare click and the tool reads the
    /// current value + inverts it (mirror of [`Self::toggle_mask_inverted`]). The
    /// per-kind mapping lives in `adjustments::{adjustment_toggle_params,
    /// set_adjustment_toggle_param}`. No-op mid-stroke or if `id` is not an
    /// adjustment. Routes the preview through the same cut-cache fast lane as a
    /// slider edit ([`Self::set_adjustment_param`]).
    pub fn flip_adjustment_toggle(&mut self, id: RtLayerId, slot: usize) {
        if self.stroke_active {
            return;
        }
        let Some(adj) = self.layers.adjustment_mut(id) else {
            return;
        };
        let cur = ph2d_painter_brush::adjustments::adjustment_toggle_params(&adj.params)
            .get(slot)
            .map(|(_, on)| *on)
            .unwrap_or(false);
        ph2d_painter_brush::adjustments::set_adjustment_toggle_param(&mut adj.params, slot, !cur);
        // Same param-only hot lane as `set_adjustment_param`: keep the cuts below
        // this adjustment, restart from its cut, republish for the panel snapshot.
        self.compositor_cache.invalidate_above(id, &self.layers);
        self.composited = None;
        self.dirty_rect = None;
        self.adjustment_cache_pending = true;
        self.preview_dirty = true;
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// Select option `option` of adjustment layer `id`'s segmented param (the
    /// panel's segment-button row, e.g. Color Balance's tonal range). The per-kind
    /// mapping lives in `adjustments::set_adjustment_segment_param`. No-op
    /// mid-stroke or if `id` is not an adjustment. Routes the preview through the
    /// same cut-cache fast lane as a slider edit ([`Self::set_adjustment_param`]).
    pub fn set_adjustment_segment(&mut self, id: RtLayerId, option: usize) {
        if self.stroke_active {
            return;
        }
        let Some(adj) = self.layers.adjustment_mut(id) else {
            return;
        };
        ph2d_painter_brush::adjustments::set_adjustment_segment_param(&mut adj.params, option);
        // Same param-only hot lane as `set_adjustment_param` (a scope change only
        // rebuilds this adjustment's transfer; layers below keep their cuts).
        self.compositor_cache.invalidate_above(id, &self.layers);
        self.composited = None;
        self.dirty_rect = None;
        self.adjustment_cache_pending = true;
        self.preview_dirty = true;
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// Move control point `point_index` of adjustment `id`'s `channel` curve to
    /// normalized `(x01, y01)` (both `0..=1`). `channel`: 0 = master (RGB),
    /// 1 = R, 2 = G, 3 = B. The bespoke curve-editor UI calls this on a point
    /// drag (the Curves analogue of [`Self::set_adjustment_param`] — Curves does
    /// not fit the generic ≤6-slider rack). No-op mid-stroke, for a non-Curves
    /// layer, an out-of-range channel, or a missing point index.
    ///
    /// X is **clamped between the two neighbours** (not re-sorted): the free-2D
    /// editor binds a stable `point_index` per handle, so a sort here would make
    /// the next drag frame grab a *different* point as soon as a point crossed its
    /// neighbour. Clamping keeps the points ordered (the spline eval needs
    /// ascending x) AND the index stable for the whole gesture. Routes the preview
    /// through the same cut-point cache fast lane as a slider drag (so the GPU LUT
    /// path re-renders Curves in real time).
    pub fn set_curve_point(
        &mut self,
        id: RtLayerId,
        channel: u8,
        point_index: usize,
        x01: f32,
        y01: f32,
    ) {
        if self.stroke_active {
            return;
        }
        let Some(adj) = self.layers.adjustment_mut(id) else {
            return;
        };
        let ph2d_painter_brush::adjustments::AdjustmentParams::Curves(c) = &mut adj.params else {
            return;
        };
        let pts = match channel {
            0 => &mut c.points_rgb,
            1 => &mut c.points_r,
            2 => &mut c.points_g,
            3 => &mut c.points_b,
            _ => return,
        };
        let n = pts.points.len();
        if point_index >= n {
            return;
        }
        // Clamp X into the neighbours' span so the points stay ordered without a
        // sort (stable index across the drag — see the doc comment). Endpoints are
        // free to the [0,1] domain edge on their outer side.
        let left = if point_index == 0 {
            0.0
        } else {
            pts.points[point_index - 1][0]
        };
        let right = if point_index + 1 == n {
            1.0
        } else {
            pts.points[point_index + 1][0]
        };
        let p = &mut pts.points[point_index];
        p[0] = x01.clamp(0.0, 1.0).clamp(left, right);
        p[1] = y01.clamp(0.0, 1.0);
        self.after_curve_edit(id);
    }

    /// Cut-cache restart + republish after any curve edit (move/add/remove a
    /// point): a curve change is param-only relative to the layers BELOW the
    /// adjustment, so keep their cuts and restart from this adjustment's cut via
    /// `composite_with_cache` (the GPU LUT path then re-renders in real time —
    /// same hot-path lane as `set_adjustment_param`).
    fn after_curve_edit(&mut self, id: RtLayerId) {
        self.compositor_cache.invalidate_above(id, &self.layers);
        self.composited = None;
        self.dirty_rect = None;
        self.adjustment_cache_pending = true;
        self.preview_dirty = true;
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// Insert a control point on `channel`'s curve of adjustment `id` at the
    /// midpoint of its widest X-gap, with Y sampled ON the current curve (so the
    /// rendered output is unchanged until the new point is dragged). Returns the
    /// inserted index, or `None` (no-op) mid-stroke, for a non-Curves layer, an
    /// out-of-range channel, a degenerate (<2-point) curve, or at the ≤8-point cap.
    pub fn add_curve_point(&mut self, id: RtLayerId, channel: u8) -> Option<usize> {
        if self.stroke_active {
            return None;
        }
        let adj = self.layers.adjustment_mut(id)?;
        let ph2d_painter_brush::adjustments::AdjustmentParams::Curves(c) = &mut adj.params else {
            return None;
        };
        let pts = match channel {
            0 => &mut c.points_rgb,
            1 => &mut c.points_r,
            2 => &mut c.points_g,
            3 => &mut c.points_b,
            _ => return None,
        };
        let n = pts.points.len();
        if !(2..MAX_CURVE_POINTS_PER_CHANNEL).contains(&n) {
            return None;
        }
        let mut best_gap = -1.0_f32;
        let mut new_x = 0.5_f32;
        let mut insert_at = n;
        for i in 0..n - 1 {
            let gap = pts.points[i + 1][0] - pts.points[i][0];
            if gap > best_gap {
                best_gap = gap;
                new_x = (pts.points[i][0] + pts.points[i + 1][0]) * 0.5;
                insert_at = i + 1;
            }
        }
        let new_y = ph2d_painter_brush::adjustments::curve_value_at(&pts.points, new_x);
        pts.points.insert(insert_at, [new_x, new_y]);
        self.after_curve_edit(id);
        Some(insert_at)
    }

    /// Remove control point `index` of `channel`'s curve of adjustment `id`. No-op
    /// mid-stroke, for a non-Curves layer, an out-of-range channel/index, or when
    /// only the two endpoints remain (a curve needs ≥2 points).
    pub fn remove_curve_point(&mut self, id: RtLayerId, channel: u8, index: usize) {
        if self.stroke_active {
            return;
        }
        let Some(adj) = self.layers.adjustment_mut(id) else {
            return;
        };
        let ph2d_painter_brush::adjustments::AdjustmentParams::Curves(c) = &mut adj.params else {
            return;
        };
        let pts = match channel {
            0 => &mut c.points_rgb,
            1 => &mut c.points_r,
            2 => &mut c.points_g,
            3 => &mut c.points_b,
            _ => return,
        };
        if pts.points.len() <= 2 || index >= pts.points.len() {
            return;
        }
        pts.points.remove(index);
        self.after_curve_edit(id);
    }

    /// `true` when the active edit target is a grayscale mask.
    #[must_use]
    pub fn active_is_mask(&self) -> bool {
        self.layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| matches!(l.kind, LayerKind::Mask(_)))
    }

    /// `true` when the active layer has its alpha locked — paint is then
    /// restricted to the layer's existing alpha (§2.10).
    #[must_use]
    pub fn active_alpha_locked(&self) -> bool {
        self.layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| l.alpha_locked)
    }

    /// The active stroke color, forced ACHROMATIC (chroma 0) when the edit
    /// target is a mask so brush paint lands as luminance (white = reveal,
    /// black = hide, §2.7). Unchanged for a normal raster.
    pub(crate) fn effective_active_color(&self) -> OklchColor {
        let c = self.params.active_color;
        if self.active_is_mask() {
            OklchColor { c: 0.0, ..c }
        } else {
            c
        }
    }

    /// Sync the brush color with mask-edit mode: entering a mask defaults the
    /// brush to BLACK — a mask starts WHITE (all visible), so the expected first
    /// action is to HIDE (white still reveals). Saves the user's real color and
    /// restores it on leaving the mask. (The default brush is a LIGHT orange,
    /// which grayed for a mask barely hides — masking looked like a no-op.)
    /// Call after every `set_active`.
    pub(crate) fn sync_mask_brush_color(&mut self) {
        let now_mask = self
            .layers
            .active()
            .and_then(|a| self.layers.get(a))
            .is_some_and(|l| matches!(l.kind, LayerKind::Mask(_)));
        if now_mask {
            if self.color_before_mask.is_none() {
                self.color_before_mask = Some(self.params.active_color);
            }
            self.params.active_color = OklchColor {
                l: 0.0,
                c: 0.0,
                h: 0.0,
                a: 1.0,
            };
        } else if let Some(c) = self.color_before_mask.take() {
            self.params.active_color = c;
        }
    }

    /// The first raster layer (depth-first pre-order) inside group `id`, or
    /// `None` for an empty group / a group containing only groups. Lets
    /// activating a group "enter" its top paintable layer (a group itself has no
    /// pixel buffer). Masks are owner-attached (not in `children`), so this only
    /// ever returns a raster.
    pub(crate) fn first_paintable_descendant(&self, id: RtLayerId) -> Option<RtLayerId> {
        let children = match self.layers.get(id).map(|l| &l.kind) {
            Some(LayerKind::Group(g)) => g.children.clone(),
            _ => return None,
        };
        for child in children {
            match self.layers.get(child).map(|l| &l.kind) {
                Some(LayerKind::Raster(_)) => return Some(child),
                Some(LayerKind::Group(_)) => {
                    if let Some(found) = self.first_paintable_descendant(child) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Make `id` the active edit target: flush the current active's pixels to
    /// `images`, load `id`'s pixels into `canvas_rgba` (transparent if it has
    /// none yet), swap the mask brush color, reset undo, invalidate. No buffer
    /// work if `id` is already active. Shared by `select_layer` / the multi-
    /// select `select_single` / `select_additive` / `select_range`. Caller owns
    /// the `selection` set bookkeeping. A Group id is resolved to its first
    /// paintable descendant (a group can never be the paint target).
    pub(crate) fn set_active_layer(&mut self, id: RtLayerId) {
        // A Group has NO pixel buffer — making it the paint target would load a
        // throwaway transparent `canvas_rgba` (the composite never reads it) and
        // silently swallow strokes. Resolve a group to its first paintable
        // descendant so clicking a group "enters" it; an empty group (or unknown
        // id) leaves the active target unchanged.
        let target = match self.layers.get(id).map(|l| &l.kind) {
            Some(LayerKind::Group(_)) => match self.first_paintable_descendant(id) {
                Some(t) => t,
                None => return,
            },
            Some(_) => id,
            None => return,
        };
        if self.layers.active() == Some(target) {
            return;
        }
        let id = target;
        self.flush_active_to_images();
        let (w, h) = self.source_size;
        let buf = self
            .images
            .remove(&id)
            .map(|img| img.rgba8)
            .unwrap_or_else(|| vec![0u8; (w as usize) * (h as usize) * 4]);
        self.canvas_rgba = Arc::new(buf);
        self.layers.set_active(id);
        self.sync_mask_brush_color(); // mask ↔ normal layer: swap to/from black
        self.reset_undo_after_layer_switch();
        self.invalidate_composite();
    }

    /// Collapse the multi-selection to a single layer (selection bookkeeping
    /// only — does NOT touch the active edit target). Called by the structural
    /// ops (add / duplicate) that already set the active layer themselves, so a
    /// stale multi-selection does not linger as a phantom highlight.
    pub(crate) fn reset_selection_to(&mut self, id: RtLayerId) {
        self.selection.clear();
        self.selection.insert(id);
    }

    /// Drop any selection members that no longer exist (after a delete/remove).
    pub(crate) fn prune_selection(&mut self) {
        self.selection.retain(|id| self.layers.get(*id).is_some());
    }

    /// Make `id` the active layer and collapse the multi-selection to it (a
    /// plain row click). No-op mid-stroke or unknown id. (Kept under the
    /// historic name; callers that just want a single active layer still work.)
    pub fn select_layer(&mut self, id: RtLayerId) {
        self.select_single(id);
    }

    /// Plain row click — replace the selection with `id` and make it active.
    /// No-op mid-stroke or for an unknown id.
    pub fn select_single(&mut self, id: RtLayerId) {
        if self.stroke_active || self.layers.get(id).is_none() {
            return;
        }
        self.set_active_layer(id);
        self.selection.clear();
        self.selection.insert(id);
    }

    /// Cmd/Ctrl-click — toggle `id` in the selection. Adding it makes it the
    /// active edit target; removing the active member repoints active to
    /// another member. Never empties the selection (a toggle-off of the lone
    /// member is ignored). No-op mid-stroke or for an unknown id.
    pub fn select_additive(&mut self, id: RtLayerId) {
        if self.stroke_active || self.layers.get(id).is_none() {
            return;
        }
        // The current active is always a selected row — fold it in so the FIRST
        // Cmd-click extends the active layer rather than replacing it.
        if let Some(a) = self.layers.active() {
            self.selection.insert(a);
        }
        if self.selection.contains(&id) {
            if self.selection.len() == 1 {
                return; // keep at least one member selected
            }
            self.selection.remove(&id);
            if self.layers.active() == Some(id)
                && let Some(&next) = self.selection.iter().next()
            {
                self.set_active_layer(next);
            }
        } else {
            self.selection.insert(id);
            self.set_active_layer(id);
        }
    }

    /// Shift-click — select the contiguous run of layers between the current
    /// active (anchor) and `id` along the visible row order, and make `id`
    /// active. Falls back to `select_single` if either endpoint is not in the
    /// visible z-order run (e.g. a mask sub-row, or no active anchor). No-op
    /// mid-stroke or for an unknown id.
    pub fn select_range(&mut self, id: RtLayerId) {
        if self.stroke_active || self.layers.get(id).is_none() {
            return;
        }
        let order = self.visible_row_order();
        let bi = order.iter().position(|&x| x == id);
        let ai = self
            .layers
            .active()
            .and_then(|a| order.iter().position(|&x| x == a));
        match (ai, bi) {
            (Some(ai), Some(bi)) => {
                let (lo, hi) = if ai <= bi { (ai, bi) } else { (bi, ai) };
                self.selection.clear();
                self.selection.extend(order[lo..=hi].iter().copied());
                self.set_active_layer(id);
            }
            _ => self.select_single(id),
        }
    }

    /// The visible layer rows in panel order (top-to-bottom pre-order): each
    /// root entry, descending into non-collapsed groups. Masks are owner-
    /// attached sub-rows (not in the z-order) and are excluded — they are not
    /// range-selectable or groupable. Drives `select_range` + `group_selected`.
    pub(crate) fn visible_row_order(&self) -> Vec<RtLayerId> {
        fn walk(stack: &LayerStack, ids: &[RtLayerId], out: &mut Vec<RtLayerId>) {
            for &id in ids {
                out.push(id);
                if let Some(LayerKind::Group(g)) = stack.get(id).map(|l| &l.kind)
                    && !g.collapsed
                {
                    walk(stack, &g.children, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.layers, self.layers.root(), &mut out);
        out
    }

    /// The current multi-selection folded with the active layer — the set of
    /// rows the panel highlights. Published each frame by the bridge
    /// (`set_current_selection`). Always includes the active layer (so a fresh
    /// tool with an empty `selection` still highlights its active row).
    #[must_use]
    pub fn selection(&self) -> std::collections::BTreeSet<RtLayerId> {
        let mut s = self.selection.clone();
        if let Some(a) = self.layers.active() {
            s.insert(a);
        }
        s
    }

    /// Wrap ALL selected layers in a new group (multi-select Group). Creates a
    /// group, moves every selected non-base layer into it (in visible z-order,
    /// skipping the base sprite and anything the depth/cycle guards reject), and
    /// keeps the current active layer as the edit target (it is reparented, not
    /// removed, so it stays valid + paintable). The selection collapses to the
    /// new group. Falls back to `group_active` (the interim single-layer wrap)
    /// when fewer than two layers are selected. No-op (`None`) mid-stroke or if
    /// the group could not be created / nothing could be nested.
    pub fn group_selected(&mut self) -> Option<RtLayerId> {
        if self.stroke_active {
            return None;
        }
        // Collect selected layers in stable visible order; the base sprite
        // (pinned at root bottom) is never groupable.
        let base = self.layers.root().last().copied();
        let targets: Vec<RtLayerId> = self
            .visible_row_order()
            .into_iter()
            .filter(|id| self.selection.contains(id) && Some(*id) != base)
            .collect();
        if targets.len() < 2 {
            return self.group_active();
        }
        let g = self.layers.add_group("Group")?;
        let mut moved_any = false;
        for &id in &targets {
            if self.layers.move_into_group(id, g) {
                moved_any = true;
            }
        }
        if !moved_any {
            self.layers.remove(g); // could not nest anything — drop the empty group
            return None;
        }
        // Active is unchanged by grouping (its layer was reparented, not moved
        // out of focus). Collapse the selection to the new group for a clear cue.
        self.selection.clear();
        self.selection.insert(g);
        if let Some(a) = self.layers.active() {
            self.selection.insert(a);
        }
        self.invalidate_composite();
        Some(g)
    }

    /// Move a layer one step toward the FRONT (top of z-order) — layers panel
    /// ↑ reorder button. No-op mid-stroke (structural-edit lifecycle, mirror of
    /// `select_layer`) or at the top. Invalidates the composite.
    pub fn move_layer_up(&mut self, id: RtLayerId) {
        if self.stroke_active {
            return;
        }
        self.layers.move_up(id);
        self.invalidate_composite();
    }

    /// Move a layer one step toward the BACK (bottom of z-order) — layers panel
    /// ↓ reorder button. No-op mid-stroke or at the bottom. Invalidates the
    /// composite.
    pub fn move_layer_down(&mut self, id: RtLayerId) {
        if self.stroke_active {
            return;
        }
        self.layers.move_down(id);
        self.invalidate_composite();
    }
}
