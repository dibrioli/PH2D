//! Per-property + structural MUTATORS — layer metadata setters (visibility /
//! opacity / blend / clipping / alpha-lock / reference), structural edits
//! (add / delete / duplicate / group / mask create + apply / reparent /
//! reorder), the active-target swap, and the multi-selection ops. Adjustment-
//! layer PARAM mutators live in the sibling `adjustments` module.
//! `impl PainterTool` (one of several blocks in this crate). Split out of the
//! former `tool/layers.rs` god-file (pure move).

use super::super::*;
use crate::layers::ReliefComposite;

impl PainterTool {
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

    /// Set a layer's **Impasto depth** (`-1..1`) — the live, permanent handle on everything ever
    /// sculpted on it (`Layer::impasto_depth`). No-op if `id` unknown.
    ///
    /// It composites, it does not re-sculpt: the height plane is untouched and `0` is a mute, not an
    /// erase. That is what lets it be the ONE knob that still reaches a stroke laid an hour ago — the
    /// brush's own Depth is baked into each stroke as it lands and afterwards only the last one is
    /// still live.
    pub fn set_layer_impasto_depth(&mut self, id: RtLayerId, depth: f32) {
        self.layers.set_impasto_depth(id, depth);
        self.invalidate_composite();
    }

    /// [`Self::set_layer_impasto_depth`] from a **bare slider's** `0..1` track — the panel's per-row
    /// sliders (opacity, and now this) store normalised and the tool maps to the domain, exactly like
    /// `set_brush_size_norm`. `0.5` is the zero: the two halves of the track mean opposite things.
    pub fn set_layer_impasto_depth_norm(&mut self, id: RtLayerId, norm: f32) {
        let depth = norm.clamp(0.0, 1.0).mul_add(2.0, -1.0); // CLAMP-OK: 0..1 track → -1..1 domain
        self.set_layer_impasto_depth(id, depth);
    }

    /// Set how a layer's relief meets the relief below it (`Add` / `Level`). No-op if `id` unknown.
    pub fn set_layer_impasto_composite(&mut self, id: RtLayerId, mode: ReliefComposite) {
        self.layers.set_impasto_composite(id, mode);
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
        if self.layers.get(id).is_none() {
            return false;
        }
        // The base sprite (bottom of root) is permanent — Apply bakes into it.
        if self.layers.root().last() == Some(&id) {
            return false;
        }
        let undo_before = self.snapshot_model();
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
                .map(|img| own_image(img).rgba8)
                .unwrap_or_else(|| vec![0u8; (w as usize) * (h as usize) * 4]);
            self.replace_canvas(Arc::new(buf));
        }
        self.prune_selection(); // drop the deleted subtree + mask from the highlight
        self.commit_structural_edit(undo_before);
        self.invalidate_composite();
        true
    }

    /// Duplicate `id` (a raster) above itself — clones the layer + its pixels,
    /// inserts above, and makes the copy the active edit target. No-op (`None`)
    /// mid-stroke, for a non-raster, or at the cap.
    pub fn duplicate_layer(&mut self, id: RtLayerId) -> Option<RtLayerId> {
        // Texture layers are raster-backed (pixels in `images[id]`/`canvas_rgba`), so the same
        // flush+clone path duplicates them; `LayerStack::duplicate` carries the cloned `TextureLayer`
        // spec, and the copy's buffer re-renders on its next edit.
        if !matches!(
            self.layers.get(id)?.kind,
            LayerKind::Raster(_) | LayerKind::Texture(_)
        ) {
            return None;
        }
        // Ensure the source pixels live in `images` (flush the active), copy them,
        // duplicate the model, then make the copy the active edit target.
        let undo_before = self.snapshot_model();
        self.flush_active_to_images();
        let (w, h) = self.source_size;
        let src_pixels = self
            .images
            .get(&id)
            .map(|img| img.rgba8.clone())
            .unwrap_or_else(|| vec![0u8; (w as usize) * (h as usize) * 4]);
        let new_id = self.layers.duplicate(id)?; // sets active = new_id
        self.images.remove(&new_id); // active lives in canvas_rgba
        self.replace_canvas(Arc::new(src_pixels));
        self.commit_structural_edit(undo_before);
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
                Arc::new(LayerImage {
                    width: w,
                    height: h,
                    rgba8: self.canvas_rgba.as_ref().clone(),
                }),
            );
        }
    }

    /// Add a new transparent raster layer at the top of the stack and make
    /// it active. The previous active layer's pixels are flushed to `images`;
    /// the new active starts as a fresh transparent `canvas_rgba`. Returns the
    /// new id, or `None` mid-stroke / before a source is set / at the cap.
    pub fn add_raster_layer(&mut self, name: impl Into<String>) -> Option<RtLayerId> {
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
        let undo_before = self.snapshot_model();
        self.flush_active_to_images();
        let id = self.layers.add_raster(name, w, h)?; // sets active = id (top)
        self.images.remove(&id); // active lives in canvas_rgba, not images
        self.replace_canvas(Arc::new(vec![0u8; (w as usize) * (h as usize) * 4]));
        self.commit_structural_edit(undo_before);
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
        let active = self.layers.active()?;
        // A raster OR a texture layer can take a mask (the compositor multiplies the layer's alpha by
        // the mask for both). A texture has no dims in its spec, so source the canvas dims here.
        if !matches!(
            self.layers.get(active)?.kind,
            LayerKind::Raster(_) | LayerKind::Texture(_)
        ) {
            return None; // not a group / mask / adjustment
        }
        let undo_before = self.snapshot_model();
        let (w, h) = self.source_size;
        let mask = self.layers.add_mask_for(active, w, h)?;
        // The parent (still active) flushes to images; the mask becomes the active edit target with a
        // fresh opaque-WHITE buffer (full visible).
        self.flush_active_to_images();
        self.images.remove(&mask); // active lives in canvas_rgba, not images
        self.replace_canvas(Arc::new(vec![255u8; (w as usize) * (h as usize) * 4]));
        self.layers.set_active(mask);
        self.commit_structural_edit(undo_before);
        self.reset_selection_to(mask);
        self.invalidate_composite();
        Some(mask)
    }

    /// Toggle the mask's grayscale-VIEW eye: show that mask's grayscale on canvas (eye open) ↔ the masked
    /// effect (eye closed, default). View-only + transient — flips `mask_view_grayscale` (one at a time)
    /// and invalidates the composite. No-op if `mask_id` isn't a mask.
    pub fn toggle_mask_view_grayscale(&mut self, mask_id: RtLayerId) {
        if !matches!(
            self.layers.get(mask_id).map(|l| &l.kind),
            Some(LayerKind::Mask(_))
        ) {
            return;
        }
        self.mask_view_grayscale = if self.mask_view_grayscale == Some(mask_id) {
            None
        } else {
            Some(mask_id)
        };
        self.invalidate_composite();
    }

    /// Toggle a mask's `Invert` flag (§2.7). The live compositor already honors
    /// it (`1 - value`), so this just flips the flag + invalidates the
    /// composite. No-op mid-stroke or if `mask_id` is not a mask.
    pub fn toggle_mask_inverted(&mut self, mask_id: RtLayerId) {
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
        let undo_before = self.snapshot_model();
        self.flush_active_to_images();
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let Some(mask_px) = self.images.get(&mask_id).map(|img| img.rgba8.clone()) else {
            return false;
        };
        let Some(parent_img) = self.images.get_mut(&parent).map(Arc::make_mut) else {
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
            .map(|img| own_image(img).rgba8)
            .unwrap_or_else(|| vec![0u8; n * 4]);
        self.replace_canvas(Arc::new(buf));
        self.commit_structural_edit(undo_before);
        self.reset_selection_to(new_active);
        self.invalidate_composite();
        true
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
        let undo_before = self.snapshot_model();
        self.flush_active_to_images();
        let (w, h) = self.source_size;
        let buf = self
            .images
            .remove(&id)
            .map(|img| own_image(img).rgba8)
            .unwrap_or_else(|| vec![0u8; (w as usize) * (h as usize) * 4]);
        self.replace_canvas(Arc::new(buf));
        self.layers.set_active(id);
        self.commit_structural_edit(undo_before);
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
        if self.layers.get(id).is_none() {
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
        if self.layers.get(id).is_none() {
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
        if self.layers.get(id).is_none() {
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

    /// Wrap ALL selected layers in a new group (multi-select Group). Creates a
    /// group, moves every selected non-base layer into it (in visible z-order,
    /// skipping the base sprite and anything the depth/cycle guards reject), and
    /// keeps the current active layer as the edit target (it is reparented, not
    /// removed, so it stays valid + paintable). The selection collapses to the
    /// new group. Falls back to `group_active` (the interim single-layer wrap)
    /// when fewer than two layers are selected. No-op (`None`) mid-stroke or if
    /// the group could not be created / nothing could be nested.
    pub fn group_selected(&mut self) -> Option<RtLayerId> {
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
        self.layers.move_up(id);
        self.invalidate_composite();
    }

    /// Move a layer one step toward the BACK (bottom of z-order) — layers panel
    /// ↓ reorder button. No-op mid-stroke or at the bottom. Invalidates the
    /// composite.
    pub fn move_layer_down(&mut self, id: RtLayerId) {
        self.layers.move_down(id);
        self.invalidate_composite();
    }
}
