//! **Where an open container's interior sits on the timeline** — the instance's map, in
//! BOTH directions (ADR-0133 §5).
//!
//! # Why a map and not just a time
//!
//! [`crate::container_playhead`] answers *"what second is the interior at?"*, and that was
//! enough while the ruler only had to DRAW the playhead. But the ruler is also a control:
//! the animator drags it, and the drag has to arrive as a time on the **timeline**, because
//! the timeline's playhead is the only one there is (ADR-0133 §1 — the parent owns the
//! clock). Reading in one clock and writing in another is the bug this module exists to
//! stop: with the instance starting at 4 s, dragging to the interior's second 1 seeked the
//! scene to second 1, silently leaving the container by three seconds.
//!
//! So the answer has to be the *relation*, not a sample of it — and once it is the relation,
//! the second ruler (the timeline's seconds over the interior's axis) is the same object read
//! the other way, which is why there is one of these rather than two.
//!
//! # When there is no map
//!
//! A map exists only where the instance is a **bijection** over its window:
//!
//! - the container has to be playing **exactly once** ([`crate::stack_eval`]'s refusal — two
//!   instances make "here" name two places);
//! - the strip must not **wrap**. Under [`StripLoop::Loop`]/[`StripLoop::PingPong`] one
//!   interior second recurs at several timeline seconds, and choosing one of them silently
//!   is precisely the class of guess this module refuses everywhere else.
//!
//! There is no message for the refusal, and that is deliberate: the ruler simply does not
//! scrub, the same way a strip too narrow to hold both grips is offered only the trim one.
//! A control that is not offered does not owe a sentence — a control that lies does.

use crate::doc::TimelineDoc;
use crate::stack::StripLoop;
use crate::stack_frames::{ActiveSource, ActiveStrip, StackScratch};

/// The affine relation between an open container's own seconds and the timeline's, over the
/// window where the instance actually plays.
///
/// `[u0, u1]` is the interior's window in its own clock; `[t0, t1]` is the same window in
/// timeline seconds. Both mappings **clamp** to it: outside the window the interior is not
/// on screen at all, so there is no honest answer to extrapolate to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContainerMap {
    /// Timeline seconds where the instance's playing window starts.
    pub t0: f64,
    /// Timeline seconds where it ends.
    pub t1: f64,
    /// The interior's own second at `t0`.
    pub u0: f64,
    /// The interior's own second at `t1`.
    pub u1: f64,
}

impl ContainerMap {
    /// The timeline second at which the interior reads `u` — what a scrub of the interior's
    /// ruler must seek to.
    #[must_use]
    pub fn host_time(&self, u: f64) -> f64 {
        remap(u, self.u0, self.u1, self.t0, self.t1)
    }

    /// The interior's own second at timeline second `t` — the inverse, and what labels the
    /// second ruler.
    #[must_use]
    pub fn local_time(&self, t: f64) -> f64 {
        remap(t, self.t0, self.t1, self.u0, self.u1)
    }
}

/// Linear remap of `x` from `[a0, a1]` onto `[b0, b1]`, clamped to the source window.
fn remap(x: f64, a0: f64, a1: f64, b0: f64, b1: f64) -> f64 {
    let d = a1 - a0;
    if d <= 0.0 {
        return b0; // a degenerate window is a single instant, not an axis
    }
    let f = ((x - a0) / d).clamp(0.0, 1.0); // CLAMP-OK: fraction of a measured window
    b0 + f * (b1 - b0)
}

/// **The open container's map**, or `None` when the instance is not a bijection (see the
/// module docs).
///
/// Reads the scratch, so the caller must have primed it at the current playhead — the same
/// precondition [`crate::container_playhead`] rides, and for the same reason.
#[must_use]
pub fn container_map(doc: &TimelineDoc, container: usize) -> Option<ContainerMap> {
    let scratch = doc.scratch();
    let a = crate::stack_eval::sole_container_strip_of(scratch, container).ok()?;
    let (mut t0, mut t1, u0, u1) = strip_window(doc, scratch, a)?;
    // Walk OUT to the timeline, composing one link at a time. At depth 1 the loop does not
    // run at all (the strip already lives in frame 0); deeper, each parent instance trims
    // and rescales the window its child described — which is the same composition the
    // evaluator does inward, read backwards.
    let mut frame = a.frame;
    while frame != 0 {
        let p = producer_of(scratch, frame)?;
        // Frames are appended breadth-first, so a producer always sits at a LOWER index than
        // what it produced. Asserting it here is what makes the walk terminate by structure
        // rather than by a counter that would need a limit nobody measured.
        if p.frame >= frame {
            return None;
        }
        let (pt0, pt1, pu0, pu1) = strip_window(doc, scratch, p)?;
        // `t0`/`t1` are in `frame`'s own clock — which is exactly what `p` reads.
        t0 = remap(t0, pu0, pu1, pt0, pt1);
        t1 = remap(t1, pu0, pu1, pt0, pt1);
        frame = p.frame;
    }
    (t1 > t0 && u1 > u0).then_some(ContainerMap { t0, t1, u0, u1 })
}

/// The strip that produced `frame`, i.e. the instance this interior belongs to.
fn producer_of(scratch: &StackScratch, frame: usize) -> Option<ActiveStrip> {
    scratch
        .active
        .iter()
        .copied()
        .find(|a| matches!(a.source, ActiveSource::Container { frame: f, .. } if f == frame))
}

/// One strip's playing window: `(t0, t1)` in the clock of the frame it LIVES in, and
/// `(u0, u1)` in the clock of what it READS. `None` when it is not a bijection.
fn strip_window(
    doc: &TimelineDoc,
    scratch: &StackScratch,
    a: ActiveStrip,
) -> Option<(f64, f64, f64, f64)> {
    let host = match scratch.frames.get(a.frame)?.container {
        Some(c) => crate::StackHost::Container(c),
        None => crate::StackHost::Document,
    };
    let s = doc
        .host_stack(host)?
        .get(a.lane)?
        .strips
        .iter()
        .find(|s| s.id == a.id)?;
    let (span, slice) = (s.span(), s.slice());
    if span <= 0.0 || slice <= 0.0 || s.speed <= 0.0 {
        // A zero-length slice is a POSE (`ClipStrip::fold` says so), and a pose has no axis
        // to scrub along. A non-positive speed is the same degeneracy from the other side.
        return None;
    }
    // Timeline seconds one full pass through the slice takes.
    let pass = slice / s.speed;
    match s.loop_mode {
        // `Once` CLAMPS past the slice, so the tail holds a still frame: the moving part is
        // the window, and the hold is honestly outside it.
        StripLoop::Once => {
            let t1 = s.t_start + pass.min(span);
            Some((
                s.t_start,
                t1,
                s.src_in,
                s.src_in + (t1 - s.t_start) * s.speed,
            ))
        }
        // A wrap that actually wraps has no inverse (module docs). One that does not reach
        // its own end never folds, so it is the `Once` case wearing another name.
        StripLoop::Loop | StripLoop::PingPong => {
            (span <= pass).then_some((s.t_start, s.t_end, s.src_in, s.src_in + span * s.speed))
        }
    }
}
