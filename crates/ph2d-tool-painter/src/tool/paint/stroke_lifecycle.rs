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
        // Wet Paint: arm the live-gesture gate — the deposit route refuses dabs outside a real
        // pen-down..pen-up gesture (shape re-stamps against a fluid would pile paint; see `wetpaint`).
        if matches!(self.paint.paint_mode, PaintMode::WetPaint) {
            self.paint.wetpaint.live_gesture = true;
        }
        let before = self.snapshot_model();
        self.paint.stroke_undo = Some(before);
        self.paint.drag_preview = None;
        self.paint.wetpaint.pending_deposit.clear(); // doc 21: a new gesture invalidates the stash
        self.paint.line_anchor = Some(ev.pos);
        // EDGE-1 wet session: while the paper is still WET and the canvas is untouched since OUR
        // last bake, consecutive watercolor strokes are ONE wash — keep the union buffers + cum
        // rect + session base, so the bake re-renders the union with a single rim (the previous
        // stroke's inner rim melts). Anything else (dried, foreign edit, mode/layer change) starts
        // a fresh session over the current canvas.
        let wet_session = self.wet_session_continues();
        // Reset the Accumulate-OFF cap mask (re-grown by the first dab) + the per-layer-colour
        // accumulation (so the recomposite snapshots THIS stroke's pre-pixels) — both per stroke.
        self.paint.stroke_mask.clear();
        self.reset_stroke_height(); // Impasto: this stroke's relief starts empty (see `super::impasto`)
        // Sculpt: belt-and-braces. A committed gesture already ended its own session, so this normally
        // finds nothing — but any path that leaves one open (a shape abandoned without Cancel) would
        // otherwise have THIS stroke re-render from a `pre` frozen for a different one.
        self.end_sculpt_session();
        if !wet_session {
            self.paint.stroke_coverage.clear();
            self.paint.stroke_color.clear();
            self.paint.stroke_density.clear();
            self.paint.stroke_deplete.clear();
            self.paint.wet_styles.clear();
            self.paint.stroke_water = Vec::new();
            self.paint.wet_cum_dirty = None;
        }
        self.paint.wet_frame_dirty = None;
        // THIS-stroke footprint restarts every stroke (even continuing a wet session): only what THIS
        // stroke paints re-wets the moisture map, so earlier washes keep their own drying clocks (#4).
        self.paint.wet_stroke_dirty = None;
        self.paint.wet_smear_pos = None; // the Wet Mix true-smear chain restarts with the stroke
        self.reset_wet_mix(); // the mixer reservoir starts fresh (no pickup) each stroke
        // Watercolor render-path: freeze the pre-stroke canvas as the optical base (shared `Arc`, so O(1);
        // the first composite `make_mut` forks the live buffer, leaving this pristine) PLUS the real
        // ground (the composite of the layers below + document paper colour) the optics read the
        // Beer–Lambert base / rewet reference from. The wash is reconstructed over these every frame
        // instead of over-painting in place. In a CONTINUING session this refrozen base (which now
        // includes the union baked so far) feeds only the mixer pickup + rewet; the composite keeps
        // reading the SESSION base below.
        self.freeze_watercolor_ground(wet_session);
        if !wet_session {
            // Fresh session: composite base = this pen-down's frozen canvas; the guard Arc is set
            // at the bake. A stale wet map from a broken session keeps drying on its own.
            self.paint.wet_session_base = self.paint.watercolor_base.clone();
            self.paint.wet_session_canvas = None;
        }
        if self.watercolor_render_active() {
            // EDGE-1 per-stroke style (doc 13 topo): capture THIS stroke's wash params at pen-down
            // — the union re-bake resolves them per pixel by the owner map, so an already-painted
            // wash keeps ITS Concentration/Edge/water instead of being re-styled by the current
            // brush (Enio 2026-07-09).
            let forced_wet = self.paint.wet_session_wetness;
            self.paint
                .wet_styles
                .push_capture(&self.paint.brush, forced_wet);
        }
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
            self.pour_canvas_wet(); // #2: moisture is laid LIVE (the damp shows during the stroke, not at pen-up)
        }
        self.paint.dabs = dabs;
        self.paint.stroke = Some(stroke);
    }

    /// The cursor moved over the canvas with **no button down** (image px). Advances the hover heading so
    /// the brush-cursor ring already wears the orientation the next dab will use — a calligraphic nib is
    /// aimed on the way in, not after you commit (Enio 2026-07-19).
    ///
    /// ⚠️ Deliberately **not** a `PointerPhase::Move`: a Move with no stroke open is a stray event that the
    /// paint paths, the shape editors and the deform gizmo all have their own opinions about. This touches
    /// exactly one field and paints nothing.
    ///
    /// It runs the SAME filter at the SAME length scale as the engine's own heading
    /// ([`ph2d_painter_brush::heading::advance`] / `smooth_len`), so the ring does not jump when the stroke
    /// takes over. Only maintained when a slot follows the stroke — a plain brush pays a bool test.
    pub fn on_canvas_hover(&mut self, pos: [f32; 2]) {
        if !self.paint.brush.follows_stroke() {
            // Nothing reads it; keep the state cold so a plain brush is untouched.
            self.paint.hover_pos = None;
            self.paint.hover_heading = [0.0, 0.0];
            return;
        }
        let prev = self.paint.hover_pos.replace(pos);
        let Some(prev) = prev else { return };
        let step = [pos[0] - prev[0], pos[1] - prev[1]];
        let len = (step[0] * step[0] + step[1] * step[1]).sqrt();
        if len <= f32::EPSILON {
            return; // no motion: hold the established heading (the engine holds too)
        }
        let smooth =
            ph2d_painter_brush::heading::smooth_len(2.0 * self.paint.brush.clamped_radius());
        self.paint.hover_heading = ph2d_painter_brush::heading::advance(
            self.paint.hover_heading,
            [step[0] / len, step[1] / len],
            len,
            smooth,
        );
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
            self.pour_canvas_wet(); // #2: moisture is laid LIVE (the damp shows during the stroke, not at pen-up)
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
        // EDGE-1: the paper dries every frame too (stroke open or not — same class as the decay
        // above): the persistent moisture poured at each bake fades over the drying window.
        self.dry_canvas_wet(dt_s);
        // Wet Paint: the 40 Hz fluid sim heartbeat (paused while a stroke is down — the engine's own
        // gate); a plain no-op with no live session, like the two decays above.
        self.wetpaint_tick(dt_s);
        // Keep the auto-centre symmetry pivot on the canvas centre every frame (also no-op when idle),
        // so the dashed overlay guide stays correct after a resize / fresh-sprite bind without paint.
        self.resolve_symmetry_geometry();
        // Live-editable wash (Enio 2026-07-11): while the paper is still wet and no stroke is open, a moved
        // Grain/Paper texture param re-renders the WHOLE committed wash (central + every Tiling copy) so the
        // wet wash reflects the new texture live — not only the next stroke. Inert unless a param changed.
        if self.paint.stroke.is_none()
            && self.watercolor_render_active()
            && self.paint.wet_editable_base.is_some()
            && self.paint.wet_editable_tex != Some(self.wet_editable_sig())
        {
            self.rerender_editable_wash();
        }
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
        // the frame dirty so a composite picks it up.
        //
        // PARKED ⇒ recomposite NOW (the visible "bleed deepens under the held nib" — the whole point
        // of the dwell). MOVING ⇒ do NOT force a composite here: the pointer-Move flush already
        // recomposited this frame's window, and compositing it AGAIN for the soak DOUBLED the
        // per-frame watercolor cost mid-gesture (frame profiler 2026-07-07: `stamps` + `tool-tick`
        // both carried a full composite). The folded dirty carries the soak into the next composite
        // (≤1 frame later, mid-gesture — imperceptible; a sweeping nib pours almost no local dwell),
        // and the pen-up bake reads the full soak field regardless ⇒ the painted result is
        // byte-identical. Airbrush/settle dabs stamped by THIS tick still composite (`stamped` above).
        if wet && let Some(r) = self.grow_wet_soak(dt_s) {
            self.paint.wet_frame_dirty = Some(match self.paint.wet_frame_dirty {
                Some(f) => union_region(f, r),
                None => r,
            });
            self.paint.wet_cum_dirty = Some(match self.paint.wet_cum_dirty {
                Some(c) => union_region(c, r),
                None => r,
            });
            self.paint.wet_stroke_dirty = Some(match self.paint.wet_stroke_dirty {
                Some(c) => union_region(c, r),
                None => r,
            });
            stamped |= parked;
        }
        if wet && stamped {
            self.apply_watercolor(false);
            self.pour_canvas_wet(); // #2: live moisture on the held/settling heartbeat too
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
        // Wet Paint: close the engine's direct stroke (the sim resumes; the session — the water —
        // stays alive across strokes). BEFORE close_stroke so the undo entry sees the final state.
        if matches!(self.paint.paint_mode, PaintMode::WetPaint) {
            self.wetpaint_stroke_end();
        }
        // Inpaint heal brush: reconstruct the marked defect BEFORE close_stroke, so the structural-undo
        // entry captures pre-stroke → healed as a single Cmd+Z step.
        if matches!(self.paint.paint_mode, PaintMode::Inpaint) {
            self.heal_inpaint();
        }
        // Watercolor render-path: bake the final optical composite over the frozen base (`commit` drops
        // the base). BEFORE close_stroke so pre-stroke → wash is one undo step (mirror of heal_inpaint).
        if self.watercolor_render_active() {
            // Keep the pre-wash BASE + frozen GROUND the bake composites over, so the wash stays
            // re-renderable while the paper is wet (live Grain/Paper edits, Enio 2026-07-11): the commit
            // drops the live base and `close_stroke` drops the ground, so capture them first.
            let editable_base = self
                .paint
                .wet_session_base
                .clone()
                .or_else(|| self.paint.watercolor_base.clone());
            let editable_backdrop = self.paint.wet_backdrop.clone();
            let region = self.apply_watercolor(true);
            // EDGE-1: pour the wash into the persistent moisture map AFTER the bake, then arm the
            // session guard — the exact canvas Arc our bake produced. A stroke landing while the
            // paper is still wet AND the guard still matches continues this wash (union re-bake).
            self.pour_canvas_wet();
            self.paint.wet_session_canvas = Some(Arc::clone(&self.canvas_rgba));
            // Arm the live-editable wash over the committed footprint (already full-axis on a tiled axis).
            if let (Some(base), Some(region)) = (editable_base, region) {
                self.paint.wet_editable_base = Some(base);
                self.paint.wet_editable_backdrop = editable_backdrop;
                self.paint.wet_editable_region = Some(region);
                self.paint.wet_editable_tex = Some(self.wet_editable_sig());
            }
        }
        self.close_stroke();
    }

    /// Finalize the current stroke: drop the in-progress state and push one undo
    /// entry (pre-stroke → current) so the whole stroke undoes/redoes as a unit. No-op when no stroke
    /// is open. Reuses the structural-undo stack (a full-canvas snapshot per stroke; tile delta later).
    pub(super) fn close_stroke(&mut self) {
        // Impasto: fold this stroke's relief into the layer BEFORE the undo entry is recorded, so the
        // step captures the height together with the pigment that made it — one Ctrl+Z takes both.
        self.commit_stroke_height();
        // Sculpt: the stroke is finished, so the session is finished — free it whole (Enio's smoke,
        // 2026-07-13). It used to be PARKED here, keeping Radius / Smooth↔Sharpen live on the stroke you
        // had just made; but a sculpt is an operation, not a substance, and picking Sharpen to sharpen
        // somewhere ELSE turned the Smooth behind you into its opposite (`super::sculpt::SculptState`).
        // The relief is already in `heights`, and `heights` is in the `ModelSnapshot` — so the undo entry
        // the line below records takes the carving with the paint that was under it. (No-op after
        // `commit_drag_preview`, which the freehand pen-up already ran: one death, not two.)
        self.end_sculpt_session();
        // Smear: the knife's warp session is per STROKE (unlike Deform's, which spans them for
        // Reconstruct). The result stays on the canvas and becomes the next stroke's baseline.
        self.end_smear_session();
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
