//! Interactive **drag-preview** stamping — the restore-then-re-stamp path that lets a moving/resizing/
//! growing preview (Drag Dot / Anchored / Line, and every shape editor) leave no trail, plus the dirty-rect
//! bookkeeping. Split from `paint.rs` for the workspace file-LOC cap; a child module of `paint`, so it keeps
//! access to `PaintState`'s module-private fields and the private `DragPreview` / `union_region` helpers.

use super::*;

/// A Drag Dot's restore record: the pristine pixels under the dab footprint (RGBA8 over `rect`), saved
/// before stamping so the next move can erase it and leave no trail.
pub(super) struct DragPreview {
    pub(super) rect: Region,
    pub(super) pixels: Vec<u8>,
}

impl PainterTool {
    /// Flag `rect` dirty for the next GPU preview upload + bump the active layer's pixel epoch.
    /// **Declara a região que um acesso de escrita de plano de fato ESCREVEU** — o canal que poupa o
    /// commit de undo de derivá-la varrendo os planos (`crate::undo::window`).
    ///
    /// ⚠️ Não confundir com [`Self::mark_dirty`], que declara onde a IMAGEM mudou. São perguntas
    /// diferentes: o Inflate escreve `target[gi] = next` em toda a janela do kernel e só marca sujo onde
    /// o relevo passou do `RELIEF_EPS`, e foi essa diferença que reprovou o atalho (doc 28 §5.17).
    /// `None` = o acesso não escreveu nada.
    pub(super) fn declare_wrote(&self, rect: Option<Region>) {
        let mut w = self.undo.write_state.get();
        w.mark(rect);
        self.undo.write_state.set(w);
    }

    pub(super) fn mark_dirty(&mut self, rect: Region) {
        #[cfg(test)]
        self.marks.push(rect);
        self.dirty_rect = Some(self.dirty_rect.map_or(rect, |acc| union_region(acc, rect)));
        self.undo_writes = self.undo_writes.wrapping_add(1);
        self.preview_dirty = true;
        self.edited_since_bind = true; // unbaked work — the shell auto-persists on leave/deactivate
        let active = self.layers.active();
        self.bump_layer_pixels(active);
    }

    /// Stamp an interactive preview batch (Drag Dot / Anchored = 1 dab, Line = N): restore the
    /// previous footprint's saved pixels, then save the pristine pixels under the new dabs' UNION
    /// bbox and stamp there — so the moving preview leaves no trail. Pen-up: `commit_drag_preview`.
    ///
    /// The stamp goes through the full [`Self::stamp_dabs`] dispatcher (NOT the bare brush route), so a
    /// **Composite Brush** runs all three layers here too. `stamp_dabs` tiles internally (so it takes
    /// the UNtiled dabs); the save-region bbox is measured over the tiled set so it still covers the
    /// wrapped copies (else the wrapped paint falls outside the restore region — a trail).
    pub(super) fn stamp_drag_preview(&mut self, dabs: &[Dab]) {
        // Doc 21 §F layer 2: a deposit_pass leak into a preview re-stamp is I2 resurrected.
        debug_assert!(!self.paint.wetpaint.deposit_pass);
        // (Watercolor render-path never reaches here — `stamp_stroke_dabs` short-circuits it before the
        // restore/save dance, since it deposits no pixels; see `super::watercolor_render`.)
        // In Selection **Edit** mode the native gizmo drives the SELECTION mask, not pixels — peel any
        // leftover preview and paint nothing (ADR-0103 Am.2). The mask refill runs off the pointer path.
        if self.paint.selection_edit_mode {
            self.peel_drag_preview();
            return;
        }
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        // The pixels were just restored to their pre-preview state and the WHOLE shape is about to be
        // re-stamped — so the relief has to be restored too. Without this the height envelope keeps the
        // ridge of every intermediate shape the artist dragged through, and a curve leaves a fan of
        // ghost relief behind the one it actually drew (`super::impasto`).
        self.reset_stroke_height();
        // …and the SCULPT has to be put back for the very same reason, one channel over. It differs from
        // the deposit in one way that matters here: the deposit builds its relief in a per-stroke envelope
        // and only merges it at commit, so clearing that envelope is enough — but the sculpt REWRITES the
        // layer's own plane live, so it has to actually restore it. Without this a Curve would carve deeper
        // on every pointer move while the artist merely LOOKED at it (`super::sculpt`, §4).
        self.restamp_reset_sculpt();
        // Coverage bbox over the wrapped Tiling copies (the stamp re-tiles them itself).
        let coverage_storage;
        let coverage: &[Dab] = if self.paint.tiling[0] || self.paint.tiling[1] {
            coverage_storage = tiling::tiled_dabs(dabs, self.source_size, self.paint.tiling);
            &coverage_storage
        } else {
            dabs
        };
        let bbox = coverage.iter().fold(None, |acc, d| {
            match (acc, self.dab_bbox(d.center, d.radius_px)) {
                (Some(a), Some(r)) => Some(union_region(a, r)),
                (a, r) => a.or(r),
            }
        });
        // Each preview frame re-stamps the WHOLE current batch onto the restored (pristine) canvas, so
        // a Composite Brush's Smear layer must chain fresh within THIS batch — clear the cross-batch
        // source (a Line's dabs then smear from the anchor; a single Drag-Dot dab simply has no source).
        self.paint.last_smear_pos = None;
        match bbox {
            Some(rect) => {
                let pixels = self.save_region(&rect);
                self.stamp_dabs(dabs);
                self.paint.drag_preview = Some(DragPreview { rect, pixels });
            }
            None => self.stamp_dabs(dabs),
        }
        // Doc 21 law B: record the batch this preview just painted — the
        // commit deposit IS the preview by construction (nothing re-derived
        // at commit: seed / offset / boolean-trace drift is structurally
        // impossible). The stash is the UNtiled parameter; the dispatcher
        // re-tiles at deposit exactly as it did here. Eraser: a flat erase
        // is its own commit — nothing to deposit. The re-arm marks the
        // re-stamp as an OWNED write so a live session survives authoring
        // (law D); one call covers restore + save + stamp in one dynamic
        // extent (single-threaded — no tick can interleave).
        if matches!(self.paint.paint_mode, PaintMode::WetPaint) && !self.paint.eraser {
            self.paint.wetpaint.pending_deposit.clear();
            self.paint.wetpaint.pending_deposit.extend_from_slice(dabs);
            self.wetpaint_rearm_after_own_write();
        }
    }

    /// Doc 21: peel the live flat preview back to pristine and clear the
    /// commit stash — the CANCEL door (Esc, a cancelled shape, the
    /// selection-edit escape). The peel is an OWNED write (the guard
    /// re-arms), so cancelling authoring over live water returns the water
    /// exactly as it was. ⚠️ The UNDO path (`restore_shape_overlay`) is
    /// deliberately NOT this door: an undo restore is a wholesale foreign
    /// swap and the guard's whole law is that undo kills the water.
    pub(super) fn peel_drag_preview(&mut self) {
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
            self.wetpaint_rearm_after_own_write();
        }
        self.paint.wetpaint.pending_deposit.clear();
    }

    /// Commit the interactive preview: drop the restore record so the last batch stays painted.
    /// Safe to call for any method (a no-op unless a preview is live).
    ///
    /// **This is where a drawing becomes canvas — so it is where the relief becomes layer** (Enio's
    /// smoke, 2026-07-12: *"smoothing nem sempre se aplica no fim do traço"*). The five SHAPE methods
    /// (Line / Arc / Ellipse / Polygon / Free Hand) deliberately keep the stroke OPEN at pen-up — the
    /// shape stays editable until Apply — so `close_stroke`, and with it `commit_stroke_height`, never
    /// ran for them. Three consequences, and Smoothing was only the visible one:
    ///
    ///   1. **Smoothing never applied.** The settle lives at commit; the light was reading the raw
    ///      envelope. (The freehand methods commit at pen-up and settle — hence "*nem sempre*".)
    ///   2. The whole live Body card (Depth / Body / Source) could not re-derive a shape's relief
    ///      either: the ingredients were never handed over.
    ///   3. Worst: the relief lived on in `stroke_height` with nothing owning it, so the **next
    ///      pen-down wiped it** (`reset_stroke_height`). Apply a curve, start another stroke, and the
    ///      thickness of the first simply evaporated — the pigment stayed, the body did not. Measured.
    ///
    /// Committing here fixes all three at one point, for every method, on Apply and Apply & Keep alike:
    /// the pigment and the body are two outputs of the same dab list, so they become permanent in the
    /// same breath. (The freehand path calls this too, just before `close_stroke` — whose own
    /// `commit_stroke_height` then finds the ingredients already taken and no-ops. One commit, not two.)
    ///
    /// **And the SCULPT has consequence #3 too, in a nastier form** (found 2026-07-13, by asking this
    /// comment's own question of the new channel). The sculpt writes the layer's relief LIVE, and a
    /// shape-editor re-stamp restores it from the frozen `pre` first (`restamp_reset_sculpt`) — so a
    /// session left open past its Apply is a loaded gun: the next shape's very first preview frame would
    /// restore over the applied shape's carving and **erase it**. The deposit only lost its relief; the
    /// sculpt would have quietly un-done work that was already on the canvas. Ending the session here is
    /// what closes it, at the same one point, for every method — and it is also where a sculpt gesture
    /// stops being editable, which is the point of `end_sculpt_session`'s docs.
    pub(super) fn commit_drag_preview(&mut self) {
        // Doc 21 law C: in Wet Paint the commit IS the deposit — peel the
        // flat sketch, replay the stash once through the full dispatcher,
        // let the sim resume ("the sketch melts"). The eraser and an empty
        // stash fall through to the flat commit below (a flat erase bakes;
        // an empty stash means nothing previewed).
        if matches!(self.paint.paint_mode, PaintMode::WetPaint)
            && !self.paint.eraser
            && !self.paint.wetpaint.pending_deposit.is_empty()
        {
            self.wetpaint_commit_deposit();
            self.paint.drag_preview = None;
            self.paint.wet_shape_active = false;
            return;
        }
        self.commit_stroke_height();
        self.end_sculpt_session(); // committed ⇒ the sculpt session dies (the card arms the NEXT stroke)
        // (The protection epoch does NOT die here — it belongs to the protection, not to the gesture.)
        self.paint.drag_preview = None;
        // #3: end any shape watercolor session so the ground (backdrop) rebuilds fresh for the next shape
        // or a freehand stroke. Harmless to the freehand brush, which never sets this flag.
        self.paint.wet_shape_active = false;
    }

    /// Watercolor variant of [`Self::stamp_drag_preview`] for the SHAPE editors (doc 13 #3): instead of a
    /// plain source-over deposit, composite the optical wash (frozen base + rim / Ragged-Edge warp /
    /// granulation) each preview frame. It reuses the SAME restore/save machinery, so the shape commit
    /// (`commit_drag_preview` keeps the last preview), cancel (peel `drag_preview`) and undo are all
    /// byte-unchanged — only HOW the preview pixels are produced differs. The wash GROUND (backdrop /
    /// substrate — a full layers-below composite, static within the session) is frozen ONCE via
    /// `wet_shape_active`; the base is re-pointed at the restored-pristine canvas each frame;
    /// `apply_watercolor(true)` bakes the settled look + drops the base, so the last preview IS the bake.
    pub(super) fn stamp_drag_preview_watercolor(&mut self, dabs: &[Dab]) {
        // Peel the previous frame's footprint → the pristine (pre-shape) canvas.
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        // Seamless Tiling (doc 13 #2): replicate the dabs across the wrapped sprite edges so the SHAPE
        // preview's wash ALSO forms on the opposite edge — the plain stroke does this in `stamp_dabs`, but
        // the shape editors re-stamp through HERE (a re-stamp preview, not the stroke lifecycle), so
        // without it a shape crossing the seam was cut at the border. Both the accumulate AND the
        // save/restore footprint use the wrapped list. Off ⇒ the raw dabs (byte-identical).
        let tiled;
        let wet_dabs: &[Dab] = if self.paint.tiling[0] || self.paint.tiling[1] {
            tiled = super::tiling::tiled_dabs(dabs, self.source_size, self.paint.tiling);
            &tiled
        } else {
            dabs
        };
        // Save the pristine pixels under the wash's INFLUENCE footprint (dab bbox + rim/warp reach), so the
        // next frame restores everything `apply_watercolor` will have written — else the rim leaves a trail.
        let Some(rect) = self.watercolor_preview_footprint(wet_dabs) else {
            return; // empty batch (no dabs) — the peel above already cleared the last preview
        };
        let pixels = self.save_region(&rect);
        self.paint.drag_preview = Some(DragPreview { rect, pixels });
        // Freeze the ground ONCE per shape session (expensive: `build_wet_backdrop` composites the layers
        // below; static within the session). Subsequent frames keep it; the base is re-pointed below.
        if !self.paint.wet_shape_active {
            self.freeze_watercolor_ground(false);
            self.paint.wet_shape_active = true;
        }
        // The optical base is THIS frame's restored-pristine canvas (cheap `Arc` clone); shapes are not a
        // wet session, so rebuild the whole shape's coverage fresh each frame.
        self.paint.watercolor_base = Some(Arc::clone(&self.canvas_rgba));
        self.paint.wet_session_base = None;
        self.clear_wet_coverage();
        self.clear_wet_color();
        self.accumulate_wet_coverage(wet_dabs);
        self.accumulate_wet_color(wet_dabs);
        self.apply_watercolor(true);
    }

    /// The save/restore footprint for a watercolor shape preview: each dab's bbox inflated by the wash's
    /// influence reach (rim `edge_spread` + Ragged-Edge `warp`), so the restore peels EVERYTHING
    /// `apply_watercolor` wrote (its `pad`). Over-inflation is safe — extra pristine pixels restore to a
    /// no-op; under-inflation would leave a rim trail. `None` when the batch is empty. With **Tiling** the
    /// wash spans the WHOLE tiled axis (`apply_watercolor` renders it there via `dab_batch_region`), so the
    /// footprint is forced full-axis to match — else the wrapped span leaves a trail on the next peel.
    fn watercolor_preview_footprint(&self, dabs: &[Dab]) -> Option<Region> {
        let b = &self.paint.brush;
        // Match `apply_watercolor`'s max reach (`spread * 2` when watered/soaked) + warp + slack.
        let pad = b.edge_spread.round().clamp(0.0, 48.0) * 2.0 + b.warp.max(0.0).ceil() + 4.0;
        let mut r = dabs.iter().fold(None, |acc, d| {
            match (acc, self.dab_bbox(d.center, d.radius_px + pad)) {
                (Some(a), Some(r)) => Some(union_region(a, r)),
                (a, r) => a.or(r),
            }
        })?;
        let (fw, fh) = self.source_size;
        if self.paint.tiling[0] {
            r.x = 0;
            r.w = fw;
        }
        if self.paint.tiling[1] {
            r.y = 0;
            r.h = fh;
        }
        Some(r)
    }

    /// Stamp the dabs a `begin`/`extend` produced. Drag Dot, Anchored AND Line are interactive
    /// preview methods: route their batch through the restore+re-stamp path so the moving/resizing/
    /// growing preview leaves no trail and `commit_drag_preview` keeps the last on pen-up. Every other
    /// method uses the cumulative stamp.
    pub(super) fn stamp_stroke_dabs(&mut self, dabs: &[Dab]) {
        // Watercolor render-path: no pixel deposit, so the drag-preview restore/save dance is unnecessary
        // (the composite reads the frozen base, giving a clean preview for free). Shape-preview methods
        // (Drag Dot / Anchored / Line) show a MOVING shape → rebuild coverage + colour each frame; the
        // cumulative methods accumulate along the path.
        if self.watercolor_render_active() {
            if matches!(
                self.paint.brush.stroke_method,
                StrokeMethod::DragDot | StrokeMethod::Anchored | StrokeMethod::Line
            ) {
                self.clear_wet_coverage();
                self.clear_wet_color();
            }
            self.stamp_dabs(dabs);
            return;
        }
        if matches!(
            self.paint.brush.stroke_method,
            StrokeMethod::DragDot | StrokeMethod::Anchored | StrokeMethod::Line
        ) {
            self.stamp_drag_preview(dabs);
        } else {
            self.stamp_dabs(dabs);
        }
    }
}
