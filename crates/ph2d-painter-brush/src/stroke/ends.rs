//! **Where a stroke's ends are** — which ones exist at all, and what to do while the far one is not yet
//! known. One subject, two halves: [`TaperSpan`] answers the first, the tail hold the second.
//!
//! The tail hold is the mirror of [`super::warmup`], at the other end of the stroke.
//!
//! ## Why a hold at all
//!
//! [`crate::taper`] narrows a dab by how far it sits from each end of the stroke. The distance to the
//! START is known the instant a dab is laid; the distance to the END is not, and for a freehand stroke
//! it is not knowable at all — the artist has not decided yet. The engine cannot guess it and must not
//! pretend to.
//!
//! So the incremental methods (Space / Dots / Airbrush) keep the last `end_px` of arc in `tail_buf`.
//! A dab leaves the buffer the moment the cursor is further than the taper window past it — at which
//! point its end factor is provably exactly `1.0`, so it is released at full width and never revisited.
//! Whatever is still held when the pen lifts is *actually* the end, and [`Stroke::finish`] releases it
//! with the taper applied. **The taper is applied exactly once, in one place, to every dab.**
//!
//! ## The price, stated
//!
//! The wet end of the stroke trails the cursor by the taper length the artist dialled — that is the
//! whole reason [`crate::taper::MAX_TAPER_DIAMETERS`] exists, and it is a bound on *lag*, not on
//! arithmetic. Two things keep it honest: it is exactly zero when the end taper is off (the default),
//! and what is missing during the drag is the *thin* part of the stroke, because that is what the
//! taper makes it. The full-width body never waits.
//!
//! ⚠️ The alternative — lay the tail at full width and re-derive it at pen-up — needs the region
//! restored to its pre-tail state and every dab re-stamped, which the fluid (Wet Paint), the read-
//! modify tools (Smear / Blur / Clone) and the watercolor accumulators would each have to be taught.
//! That is a wave with its own acceptance, not a line inside this one.
//!
//! ## Order is the invariant
//!
//! Held dabs are always EARLIER in the stroke than whatever is in `out`, so both doors below release
//! them in front of the fresh batch. A deposit that is not order-independent (the fluid; the read-
//! modify tools) would otherwise see the stroke laid out of sequence, and no gate on width would ever
//! notice.

use super::{Dab, Stroke};
use crate::stroke_method::StrokeMethod;

/// What a stroke's [`crate::taper`] has to measure against.
///
/// The taper needs a distance to each end. Which ends exist — and whether the far one is knowable yet —
/// is a property of the stroke's GEOMETRY, not of the brush, so it is decided by whoever builds the path
/// and asked exactly once, in [`Stroke::dab_at`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum TaperSpan {
    /// An **open** path still being drawn: the start is known, the end is not (the artist has not
    /// decided). Only the start taper can apply here; the end is resolved by the tail hold below.
    OpenUnknownEnd,
    /// An **open** path whose total arc length was known before the first dab — the shape editors, which
    /// are handed the whole spine. Both tapers are exact and live.
    Open(f32),
    /// **No ends.** A closed loop (Ellipse / Polygon) or a single pinned stamp (Anchored / Drag Dot).
    ///
    /// ⚠️ This is not an omission: tapering a closed contour would thin it at the seam — an arbitrary
    /// point of the geometry, wherever the fill happened to start — and a circle with a notch in it is a
    /// defect, not a taper. A single stamp has no path to travel along at all.
    Closed,
}

impl TaperSpan {
    /// The span a stroke of `method` starts with. The whole-path fills overwrite it when they run; this
    /// is the answer for the methods that paint straight from `begin`/`extend`.
    pub(super) fn for_method(method: StrokeMethod) -> Self {
        match method {
            // One pinned stamp: `arc_len` grows with the drag but there is no stroke to taper.
            StrokeMethod::Anchored | StrokeMethod::DragDot => Self::Closed,
            _ => Self::OpenUnknownEnd,
        }
    }
}

impl Stroke {
    /// Move `out` into the tail buffer and release everything that has fallen out of the end-taper
    /// window. A no-op — one compare, nothing touched — unless this stroke holds its tail *or* still
    /// has dabs held from before a live spec edit switched the hold off.
    pub(super) fn tail_gate(&mut self, out: &mut Vec<Dab>) {
        if !self.holds_tail && self.tail_buf.is_empty() {
            return;
        }
        if self.holds_tail {
            self.tail_buf.append(out); // hold the new dabs (drains `out` → empty until release)
        }
        let end_px = self.spec.taper.end_px(2.0 * self.spec.clamped_radius());
        // A dab is out of the window once the stroke has travelled `end_px` beyond it. `arc_len` is the
        // arc at the cursor, `d.arc_len` the arc at the dab, and the buffer is in emission order — so
        // the released prefix is contiguous and the survivors keep theirs.
        let released = self
            .tail_buf
            .iter()
            .take_while(|d| !self.holds_tail || self.arc_len - d.arc_len >= end_px)
            .count();
        if released == 0 {
            return;
        }
        // Their end factor is 1.0 by construction here, so this applies the START taper only — the same
        // door, the same law, never a second answer to "how wide is this dab".
        let fresh = std::mem::take(out);
        for d in self.tail_buf.drain(..released).collect::<Vec<_>>() {
            out.push(self.tapered(d, f32::INFINITY));
        }
        out.extend(fresh);
    }

    /// Release the whole tail at [`Stroke::finish`], where the far end is finally a number: the stroke
    /// ends at `arc_len`, so a dab held at `d.arc_len` is `arc_len - d.arc_len` from it.
    ///
    /// ⚠️ The fresh dabs already in `out` are only folded into the tail **while the hold is on**. With
    /// the hold off they have already been tapered by [`Stroke::dab_at`], and passing them through here
    /// would scale them a second time — the same stroke thinned twice.
    pub(super) fn finish_tail(&mut self, out: &mut Vec<Dab>) {
        if self.holds_tail {
            self.tail_buf.append(out);
        } else if self.tail_buf.is_empty() {
            return;
        }
        let end = self.arc_len;
        let fresh = std::mem::take(out);
        for d in std::mem::take(&mut self.tail_buf) {
            out.push(self.tapered(d, (end - d.arc_len).max(0.0)));
        }
        out.extend(fresh);
    }

    /// Apply the taper to one held dab, given its distance `to_end` from the stroke's far end.
    fn tapered(&self, mut d: Dab, to_end: f32) -> Dab {
        let w = self
            .spec
            .taper
            .width(d.arc_len, to_end, 2.0 * self.spec.clamped_radius());
        crate::taper::scale_dab(&mut d, &self.spec.taper, w);
        d
    }
}

/// Whether a stroke with this spec must hold its tail: it wants an end taper AND its method lays dabs
/// incrementally, so nothing downstream can tell it where the stroke ends.
///
/// The whole-path fills measure their own spine ([`Stroke::fill_polyline_preview`]) and the single-stamp
/// methods have no path to taper along — asking
/// [`crate::stroke_method::StrokeMethod::is_incremental`] is what keeps this from becoming a private
/// list that drifts from the four other places already asking the same question.
pub(super) fn holds_tail(spec: &crate::spec::BrushSpec) -> bool {
    spec.taper.end > 0.0 && spec.stroke_method.is_incremental()
}
