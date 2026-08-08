//! **The far end of a freehand stroke, resolved at pen-up.**
//!
//! The head of a taper is free: a dab knows how far it is from the pen-down the instant it is made. The
//! far end is the whole problem — while the artist is still drawing, the stroke has not ended yet.
//!
//! ⛔ The first cut of the taper answered that by **holding the tail back** until the cursor had
//! travelled past the window. It is exact, and it was rejected on sight (Enio, 2026-08-08: *"o traço não
//! pode ter nenhum delay e nenhum stabilize"*) — the mark stops following the hand, which reads as a
//! heavy stabilizer. So nothing is ever withheld, and the plain drag simply laid its tail at full width.
//!
//! This module is the other half: at **pen-up**, when the far end finally exists, the last window's
//! worth of ink is **put back and laid again**, tapered. Nothing changes while the artist draws — the
//! ink is under the cursor the whole time — and the tip narrows at the instant they lift.
//!
//! ## Why a restore is unavoidable
//!
//! Paint only ever goes ON. Re-stamping a smaller dab over a bigger one does not shrink the bigger one,
//! and the relief is a `max` envelope, which is the same fact one plane over. The only way to make the
//! tail thinner is to put the region back to what it was before the stroke and lay it again — and the
//! "before" is already in hand: [`crate::undo::ModelSnapshot`], captured at pen-down for the undo.
//!
//! ## What is put back, and why each one
//!
//! - `canvas_rgba` — the pixels.
//! - `heights` / `covers` / `mats` of the active layer — the relief. A stroke's colour and its body are
//!   one fact and they roll back together (the lesson the undo snapshot itself paid for).
//! - `stroke_coverage` — the per-stroke Accumulate cap. It only ever grows, so leaving it would make the
//!   replayed dabs land against a ceiling they already reached, and the tail would come back thin AND
//!   pale.
//! - the relief **envelope** (`stroke_height` and the four ingredient planes beside it). It is a `max`
//!   per texel: a smaller dab cannot lower it, so it has to be cleared where we are laying again.
//!
//! ## The window, and why body dabs are replayed too
//!
//! The rect is the union of the footprints of the dabs inside the end window — but the dabs REPLAYED
//! into it are every recorded dab whose footprint touches it, not just those. A stroke that curves back
//! across its own tail has body ink inside that rect, and restoring would take it away; replaying only
//! the tail would not bring it back.
//!
//! ## Where this does NOT run, and it is named rather than hidden
//!
//! Only the plain deposit (`PaintMode::Paint`, not the eraser). The watercolor wash reconstructs itself
//! from its own accumulators and Wet Paint runs a fluid that cannot be rewound, so putting their pixels
//! back would not put their STATE back. Smear / Blur / Clone read the canvas as they go, so a replay
//! reads a different ground than the original pass did. Those keep the head taper and a blunt tail.

use super::PaintMode;
use super::PainterTool;
use ph2d_painter_brush::Dab;

/// A recorded dab plus the taper factor it was stamped with.
///
/// The factor is kept rather than re-derived at replay time because it is what the correction is a
/// RATIO of: the dab already carries the head taper, and what the tail owes it is the difference.
#[derive(Clone, Copy, Debug)]
pub(super) struct StampedDab {
    pub(super) dab: Dab,
    /// The width factor this dab was emitted with (head taper only — the far end was unknown).
    pub(super) emitted_w: f32,
}

/// Below this the dab is already a point: its radius has hit the engine's own `0.5 px` clamp, so the
/// ratio that would shrink it further is `0/0`. Nothing to correct.
const MIN_EMITTED_W: f32 = 1e-4;

impl PainterTool {
    /// Whether this stroke is going to want its tail resolved — asked once per batch, and the answer is
    /// what keeps the recording free for every stroke that is not tapered.
    ///
    /// ⚠️ `stroke_method.is_incremental()` is the whole point: the shape editors already know their own
    /// geometry, so their far end is exact and live, and recording their dabs would be paying for an
    /// answer they already have.
    pub(super) fn taper_tail_wanted(&self) -> bool {
        let b = &self.paint.brush;
        b.taper.end > 0.0
            && b.stroke_method.is_incremental()
            && matches!(self.paint.paint_mode, PaintMode::Paint)
            && !self.paint.eraser
            && !self.watercolor_render_active()
            && !self.paint.wetpaint.armed
        // ⛔ IMPASTO IS OUT, and the number is why (Enio 2026-08-08: *"o mouse up do Taper retira a
        // tinta e deixa só o relevo"*). Ablated: with the restore, an impasto stroke measures **0
        // inked rows** afterwards; without it, 20. So the restore takes the paint and the replay
        // does NOT put it back — under impasto the canvas colour is not simply what each dab
        // deposited, and re-running the live relief render after the replay does not bring it back
        // either (measured). Losing the artist's paint is far worse than a blunt tail, so the resolve
        // stays out of the mode until that is understood rather than guessed at.
            && !self.paint.brush.impasto
    }

    /// Record what a batch stamped, so the tail can be laid again once the far end exists.
    ///
    /// ⚠️ Called from the ONE stamp door, and it records what that door was HANDED — before Tiling and
    /// the per-copy expansion inside it. Replaying through the same door re-derives those, so recording
    /// them here would lay each wrapped copy twice.
    pub(super) fn note_taper_dabs(&mut self, dabs: &[Dab]) {
        if self.paint.taper_replaying || !self.taper_tail_wanted() {
            return;
        }
        let diameter = 2.0 * self.paint.brush.clamped_radius();
        let taper = self.paint.brush.taper;
        self.paint
            .taper_dabs
            .extend(dabs.iter().map(|d| StampedDab {
                dab: *d,
                emitted_w: taper.width(d.arc_len, f32::INFINITY, diameter),
            }));
    }

    /// Forget the recording — a new gesture. (A stroke that resolves its tail takes the list with
    /// `mem::take`; an undo throws away the canvas the list describes, and the next `paint_begin`
    /// clears it before anything can read it.)
    pub(super) fn drop_taper_record(&mut self) {
        self.paint.taper_dabs.clear();
    }

    /// **Lay the last window's worth of ink again, tapered.** Runs at pen-up, before the stroke's undo
    /// entry is recorded, so the artist sees one stroke and one undo step.
    pub(super) fn resolve_taper_tail(&mut self) {
        let rec = std::mem::take(&mut self.paint.taper_dabs);
        if rec.is_empty() || !self.taper_tail_wanted() {
            return;
        }
        let Some(before) = self.paint.stroke_undo.clone() else {
            return; // no "before" to put back — nothing honest to do
        };
        let taper = self.paint.brush.taper;
        let diameter = 2.0 * self.paint.brush.clamped_radius();
        let total = rec.iter().fold(0.0f32, |m, s| m.max(s.dab.arc_len));
        let end_px = taper.end_px(diameter);
        if end_px <= 0.0 {
            return;
        }

        // ⚠️ The window is the WHOLE stroke, not just the end, and that is not laziness: the relief
        // envelope (`reset_stroke_height`) is per-STROKE — a `max` per texel plus a push/wave ledger
        // that is a fact about the dab LIST — so it cannot be cleared for a rectangle without lying
        // about the rest. Clearing it and replaying only the tail would take the body's relief with it.
        // So the stroke is put back and laid again in full, which is exactly what a shape editor does
        // every preview frame. The cost is one extra stamp of the stroke, once, at pen-up.
        let mut rect: Option<super::Region> = None;
        for s in &rec {
            if let Some(r) = self.dab_bbox(s.dab.center, s.dab.radius_px) {
                rect = Some(match rect {
                    Some(a) => super::union_region(a, r),
                    None => r,
                });
            }
        }
        let Some(rect) = rect else { return };

        self.restore_before(&before, rect);

        let mut batch: Vec<Dab> = Vec::with_capacity(rec.len());
        for s in &rec {
            let mut d = s.dab;
            let w_final = taper.width(d.arc_len, (total - d.arc_len).max(0.0), diameter);
            // The dab already wears the HEAD taper, so what the far end owes it is the RATIO. A dab that
            // was emitted at essentially zero width is already a point and there is nothing to divide.
            if s.emitted_w > MIN_EMITTED_W && w_final < s.emitted_w {
                let r = w_final / s.emitted_w;
                d.radius_px =
                    (d.radius_px * r).clamp(0.5, ph2d_painter_brush::spec::MAX_BRUSH_RADIUS_PX);
                let cov_emit = taper.coverage_mult(s.emitted_w);
                if cov_emit > MIN_EMITTED_W {
                    d.coverage =
                        (d.coverage * taper.coverage_mult(w_final) / cov_emit).clamp(0.0, 1.0);
                }
            }
            batch.push(d);
        }

        self.paint.taper_replaying = true;
        self.reset_stroke_height();
        self.paint.stroke_coverage.clear();
        self.stamp_dabs(&batch);
        self.paint.taper_replaying = false;
    }

    /// Put `rect` back to what the pre-stroke snapshot holds — the pixels AND the three relief planes
    /// of the active layer, because a stroke's colour and its body are one fact and they roll back
    /// together (the lesson [`crate::undo::ModelSnapshot`] itself paid for).
    ///
    /// ⚠️ Through the **fork doors**, never `Arc::make_mut`: they are what tells the undo journal which
    /// plane these bytes belong to and which region is about to change. A restore that skipped them
    /// would put the pixels back and leave the step's journal describing bytes that are no longer there.
    fn restore_before(&mut self, before: &crate::undo::ModelSnapshot, rect: super::Region) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let spans = |per: usize, len: usize| -> Vec<(usize, usize)> {
            (0..rect.h)
                .filter_map(|row| {
                    let y = (rect.y + row) as usize;
                    let a = (y * w as usize + rect.x as usize) * per;
                    let b = a + rect.w as usize * per;
                    (b <= len).then_some((a, b))
                })
                .collect()
        };
        if before.canvas_rgba.len() == self.canvas_rgba.len() {
            let src = before.canvas_rgba.clone();
            let sp = spans(4, src.len());
            let dst = super::plane_fork::fork_canvas(
                &mut self.canvas_rgba,
                &self.undo.write_state,
                w,
                Some(rect),
            );
            for (a, b) in sp {
                dst[a..b].copy_from_slice(&src[a..b]);
            }
        }
        let Some(layer) = self.layers.active() else {
            return;
        };
        if let (Some(src), true) = (before.heights.get(&layer).cloned(), true)
            && self
                .heights
                .get(&layer)
                .is_some_and(|d| d.len() == src.len())
        {
            let sp = spans(1, src.len());
            let slot = self.heights.get_mut(&layer).expect("checked above");
            let dst = super::plane_fork::fork_heights(
                slot,
                &self.undo.write_state,
                layer,
                (w, h),
                Some(rect),
            );
            for (a, b) in sp {
                dst[a..b].copy_from_slice(&src[a..b]);
            }
        }
        if let Some(src) = before.covers.get(&layer).cloned()
            && self
                .covers
                .get(&layer)
                .is_some_and(|d| d.len() == src.len())
        {
            let sp = spans(1, src.len());
            let slot = self.covers.get_mut(&layer).expect("checked above");
            let dst = super::plane_fork::fork_covers(
                slot,
                &self.undo.write_state,
                layer,
                (w, h),
                Some(rect),
            );
            for (a, b) in sp {
                dst[a..b].copy_from_slice(&src[a..b]);
            }
        }
        if let Some(src) = before.mats.get(&layer).cloned()
            && self.mats.get(&layer).is_some_and(|d| d.len() == src.len())
        {
            let sp = spans(1, src.len());
            let slot = self.mats.get_mut(&layer).expect("checked above");
            let dst = super::plane_fork::fork_mats(
                slot,
                &self.undo.write_state,
                layer,
                (w, h),
                Some(rect),
            );
            for (a, b) in sp {
                dst[a..b].copy_from_slice(&src[a..b]);
            }
        }
        self.mark_dirty(rect);
    }
}
