//! The paint-stroke **lifecycle** — begin / extend / tick / end + close: the undo snapshot, the
//! engine `Stroke` drive, the watercolor recomposite calls and the per-stroke state resets. Split
//! from `paint.rs` for the workspace LOC cap; a child of `paint`, so it keeps access to
//! `PaintState`'s module-private fields.

use super::*;

impl PainterTool {
    /// `true` when the active layer can be painted and the working buffer is sized — a **Raster** layer
    /// OR a **Mask** (its coverage buffer is bound to `canvas_rgba` like a raster's, so painting writes
    /// Rec.601-luma coverage: black conceals, white reveals). Group/adjustment/texture aren't paintable.
    pub(super) fn paint_target_ready(&self) -> bool {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || self.canvas_rgba.is_empty() {
            return false;
        }
        self.layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| matches!(l.kind, LayerKind::Raster(_) | LayerKind::Mask(_)))
    }

    /// Begin a stroke at `ev` and stamp the first dab. Snapshots the model for undo
    /// **before** painting so the whole stroke restores to the pre-stroke pixels.
    pub(super) fn paint_begin(&mut self, ev: CanvasPointer) {
        // Mask tool: ensure the TRANSIENT scratch mask exists for the current layer (tool-side, no layer
        // created) before the stroke paints into it.
        if matches!(self.paint.paint_mode, PaintMode::Mask) {
            self.ensure_mask_scratch();
        }
        // Inpaint heal brush: start a fresh defect mask for this stroke (the previous stroke's mark was
        // consumed + cleared on its pen-up).
        if matches!(self.paint.paint_mode, PaintMode::Inpaint) {
            let (w, h) = self.source_size;
            let n = (w as usize) * (h as usize);
            if self.paint.inpaint_mask.len() == n {
                self.paint.inpaint_mask.iter_mut().for_each(|m| *m = 0);
            } else {
                self.paint.inpaint_mask = vec![0u8; n];
            }
        }
        let before = self.snapshot_model();
        self.paint.stroke_undo = Some(before);
        self.paint.drag_preview = None;
        self.paint.line_anchor = Some(ev.pos);
        // Reset the Accumulate-OFF cap mask (re-grown by the first dab) + the per-layer-colour
        // accumulation (so the recomposite snapshots THIS stroke's pre-pixels) — both per stroke.
        self.paint.stroke_mask.clear();
        self.paint.stroke_coverage.clear();
        self.paint.stroke_color.clear();
        self.paint.wet_frame_dirty = None;
        self.paint.wet_cum_dirty = None;
        self.paint.wet_smear_pos = None; // the Wet Mix true-smear chain restarts with the stroke
        // Watercolor render-path: freeze the pre-stroke canvas as the optical base (shared `Arc`, so O(1);
        // the first composite `make_mut` forks the live buffer, leaving this pristine) PLUS the real
        // ground (the composite of the layers below + document paper colour) the optics read the
        // Beer–Lambert base / rewet reference from. The wash is reconstructed over these every frame
        // instead of over-painting in place.
        self.freeze_watercolor_ground();
        self.paint.per_layer_stroke.reset();
        // Smear chains its source from the previous dab; a fresh stroke has none yet.
        self.paint.last_smear_pos = None;
        // Clone: establish the source→dest offset for this stroke (aligned keeps it across strokes,
        // non-aligned re-anchors to the sampled source each stroke). No-op unless a source is sampled.
        self.clone_begin_offset(ev.pos);
        // Pin the symmetry centre to the current canvas centre for the auto-centre modes before the
        // stroke captures the spec (the engine mirrors/rotates about `brush.symmetry.center`).
        self.resolve_symmetry_geometry();
        // Clone ignores Symmetry (its panel section is hidden): mirrored dabs would clone from mirrored
        // source positions, which is nonsensical — strip it from the captured spec so a leftover-enabled
        // flag can't silently mirror. Other modes keep Symmetry.
        let mut spec = self.paint.brush;
        if matches!(self.paint.paint_mode, PaintMode::Clone) {
            spec.symmetry.enabled = false;
        }
        let mut stroke = Stroke::new(spec, self.paint.dynamics, self.paint.seed);
        // Seed the texture RNG from this stroke's seed, decorrelated from the jitter stream so the
        // two don't lock-step (HR-5: deterministic per stroke).
        self.paint.tex_rng = self.paint.seed ^ 0x7465_7874_7572_6573;
        self.paint.seed = self.paint.seed.wrapping_add(1);
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.begin(
            StrokePoint {
                pos: ev.pos,
                pressure: ev.pressure,
            },
            &mut dabs,
        );
        self.stamp_stroke_dabs(&dabs);
        // Watercolor render-path: reconstruct the wash optically over the frozen base (live; grows with
        // the stroke). Replaces the per-dab deposit that `stamp_dabs` skipped in watercolor mode.
        if self.watercolor_render_active() {
            self.apply_watercolor(false);
        }
        self.paint.dabs = dabs;
        self.paint.stroke = Some(stroke);
    }

    /// Extend the in-progress stroke to `ev`, stamping any dabs the spacing emits.
    /// Returns `false` if no stroke is active (a stray Move).
    pub(super) fn paint_extend(&mut self, ev: CanvasPointer) -> bool {
        let Some(mut stroke) = self.paint.stroke.take() else {
            return false;
        };
        // Line + Alt: snap the cursor to a 45° increment around the press point (Blender
        // `constrain_line`). Tool-side so the engine's Line fill stays a plain anchor→cursor segment.
        let pos = match (self.paint.brush.stroke_method, self.paint.line_anchor) {
            (StrokeMethod::Line, Some(anchor)) if self.paint.line_constrain => {
                brush_settings::snap_to_45(anchor, ev.pos)
            }
            _ => ev.pos,
        };
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.extend(
            StrokePoint {
                pos,
                pressure: ev.pressure,
            },
            &mut dabs,
        );
        self.stamp_stroke_dabs(&dabs);
        // Watercolor render-path: recomposite over the frozen base (no overlay peel — the base is
        // untouched, so each frame recomposites cleanly from the grown coverage + colour).
        if self.watercolor_render_active() {
            self.apply_watercolor(false);
        }
        self.paint.dabs = dabs;
        self.paint.stroke = Some(stroke);
        self.paint.moved_this_frame = true;
        true
    }

    /// Per-frame heartbeat while a stroke is held (`dt_s` = wall time since the last frame); clears the
    /// move flag and drives two time-based behaviours (both no-op when their method doesn't apply):
    /// - **Airbrush timer:** deposit dabs at the brush's Rate, moving OR parked (Blender fires it on a
    ///   timer, not on motion — builds up when held, sparse when swept). No-op for non-Airbrush.
    /// - **Stabilizer catch-up:** when parked, walk the lagged path to the cursor (`settle` is
    ///   Space-only) so a high-stabilizer stroke arrives without waiting for pointer-up.
    pub(crate) fn paint_tick(&mut self, dt_s: f32) {
        // Decay the transient in-gizmo Stencil preview (armed by panel param changes); runs every frame
        // even with no open stroke, so it fades out shortly after the user stops changing the params.
        if self.paint.stencil_preview_s > 0.0 {
            self.paint.stencil_preview_s = (self.paint.stencil_preview_s - dt_s).max(0.0);
        }
        // Keep the auto-centre symmetry pivot on the canvas centre every frame (also no-op when idle),
        // so the dashed overlay guide stays correct after a resize / fresh-sprite bind without paint.
        self.resolve_symmetry_geometry();
        let parked = !self.paint.moved_this_frame;
        self.paint.moved_this_frame = false;
        let Some(mut stroke) = self.paint.stroke.take() else {
            return;
        };
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.tick(dt_s, &mut dabs);
        // Watercolor render-path: accumulate the airbrush/settle dabs, then recomposite over the frozen
        // base once — but only when a dab actually landed (a held stroke ticks every frame).
        let wet = self.watercolor_render_active();
        let mut stamped = !dabs.is_empty();
        self.stamp_dabs(&dabs);
        if parked {
            stroke.settle(&mut dabs); // clears `dabs` first
            stamped |= !dabs.is_empty();
            self.stamp_dabs(&dabs);
        }
        // Water dwell: the heartbeat pours soak under the nib (parked OR moving) — a lingering wet
        // brush deepens/widens its own bleed. When the soak grew with no new dab, fold its disc into
        // the frame dirty and recomposite anyway, so the growth is visible live while parked.
        if wet && let Some(r) = self.grow_wet_soak(dt_s) {
            self.paint.wet_frame_dirty = Some(match self.paint.wet_frame_dirty {
                Some(f) => union_region(f, r),
                None => r,
            });
            self.paint.wet_cum_dirty = Some(match self.paint.wet_cum_dirty {
                Some(c) => union_region(c, r),
                None => r,
            });
            stamped = true;
        }
        if wet && stamped {
            self.apply_watercolor(false);
        }
        self.paint.dabs = dabs;
        self.paint.stroke = Some(stroke);
    }

    /// Finish the stroke at `ev` (stamp the final segment, flush the freehand smoother's tail so
    /// the stroke reaches the release point, then close + record undo).
    pub(super) fn paint_end(&mut self, ev: CanvasPointer) {
        self.paint_extend(ev);
        if let Some(mut stroke) = self.paint.stroke.take() {
            let mut dabs = std::mem::take(&mut self.paint.dabs);
            stroke.finish(&mut dabs);
            self.stamp_dabs(&dabs);
            self.paint.dabs = dabs;
            self.paint.stroke = Some(stroke);
        }
        // Drag Dot: the dab at the release point is the commit — keep it (drop the restore record).
        self.commit_drag_preview();
        // Inpaint heal brush: reconstruct the marked defect BEFORE close_stroke, so the structural-undo
        // entry captures pre-stroke → healed as a single Cmd+Z step.
        if matches!(self.paint.paint_mode, PaintMode::Inpaint) {
            self.heal_inpaint();
        }
        // Watercolor render-path: bake the final optical composite over the frozen base (`commit` drops
        // the base). BEFORE close_stroke so pre-stroke → wash is one undo step (mirror of heal_inpaint).
        if self.watercolor_render_active() {
            self.apply_watercolor(true);
        }
        self.close_stroke();
    }

    /// Finalize the current stroke: drop the in-progress state and push one undo
    /// entry (pre-stroke → current) so the whole stroke undoes/redoes as a unit. No-op when no stroke
    /// is open. Reuses the structural-undo stack (a full-canvas snapshot per stroke; tile delta later).
    pub(super) fn close_stroke(&mut self) {
        self.paint.stroke = None;
        self.paint.line_anchor = None;
        self.paint.last_smear_pos = None;
        self.paint.watercolor_base = None; // defensive: the render-path drops it on commit already
        self.paint.wet_backdrop = None; // the ground is per-stroke (the stack below may change)
        self.paint.wet_soak_pos = None;
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
    }
}
