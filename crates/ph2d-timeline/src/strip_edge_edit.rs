//! **What one edge gesture does to one strip** — the trim, the stretch, and the
//! change bar each leaves behind.
//!
//! A sibling of `intent_apply.rs` (which grew past the 700-LOC workspace cap) and a
//! unit in its own right: the router up there decides WHICH edit an intent is, and
//! everything here decides what that edit means to the strip's numbers. The two
//! questions have different readers.

/// The slowest and fastest a strip may play. Zero would freeze the source on one
/// frame forever — which `StripLoop::Once` past its end already expresses, and
/// honestly; a negative speed would run it backwards, which is `PingPong`'s job.
pub(crate) const MIN_STRIP_SPEED: f64 = 0.01;
/// Mirror of [`MIN_STRIP_SPEED`].
pub(crate) const MAX_STRIP_SPEED: f64 = 100.0;

/// Move one edge of a strip, taking the source slice with it.
///
/// **A trim is not a stretch.** The frames that stay visible must stay WHERE they
/// were on the timeline, so the span's edge and the slice's edge travel together
/// (by `speed`, which is what converts timeline seconds into clip seconds).
/// Dragging the start edge one second to the right hides the clip's first second
/// — it does not squeeze the whole clip into a shorter box.
///
/// Neither edge may cross the other: a strip of negative span is a strip that
/// covers no time and paints inside-out.
pub(crate) fn trim_strip(s: &mut crate::ClipStrip, edge: u8, t: f64) {
    let min_span = 1.0 / 240.0; // LITERAL-OK: a quarter of a frame at 60 fps
    if edge == 0 {
        let t_start = t.max(0.0).min(s.t_end - min_span);
        s.src_in += (t_start - s.t_start) * s.speed;
        s.t_start = t_start;
    } else {
        let t_end = t.max(s.t_start + min_span);
        s.src_out += (t_end - s.t_end) * s.speed;
        s.t_end = t_end;
    }
}

/// Record what this corner's edit did, for the change bar the panel draws over it.
///
/// Measured against `from` — where the edge sat when the gesture began — so every
/// frame of a drag lands on the SAME total, and the mark is a fact about the gesture
/// rather than about how many frames it took.
///
/// It reads the edge AFTER the edit, which gets the clamps for free: a corner that
/// ran into its limit records the change it actually made, not the one the pointer
/// asked for.
pub(crate) fn mark_edge(s: &mut crate::ClipStrip, stretch: bool, edge: u8, from: f64) {
    let now = if edge == 0 { s.t_start } else { s.t_end };
    s.marks[crate::mark_index(stretch, edge)] = from - now;
}

/// Move one edge of a strip WITHOUT touching its source slice — the retime.
///
/// The mirror image of [`trim_strip`], and the reason the two are separate
/// functions rather than one with a flag: a trim holds the *rate* and changes
/// *which frames play*; a stretch holds *which frames play* and changes the
/// *rate*. Every strip edit is one or the other, and an editor that blurs them is
/// an editor where an animator lengthens a strip and silently slows the animation
/// down (or shortens one and loses its tail) — the single most reported confusion
/// in every NLE that ever conflated them.
///
/// `speed = slice / span`, so the span is what actually gets clamped: the rate's
/// bounds are span bounds in disguise, and clamping the span (rather than the
/// speed, after the fact) is what keeps `speed` and the drawn box in agreement at
/// the limit. A zero-length slice is a pose, not a clip — it has no rate to change.
pub(crate) fn stretch_strip(s: &mut crate::ClipStrip, edge: u8, t: f64) {
    let slice = s.slice();
    if slice <= 0.0 {
        return;
    }
    // span = slice / speed, so the speed bounds ARE these span bounds.
    let (min_span, max_span) = (slice / MAX_STRIP_SPEED, slice / MIN_STRIP_SPEED);
    if edge == 0 {
        let span = (s.t_end - t.max(0.0)).clamp(min_span, max_span); // CLAMP-OK: derived bounds
        s.t_start = (s.t_end - span).max(0.0);
    } else {
        let span = (t - s.t_start).clamp(min_span, max_span); // CLAMP-OK: derived bounds
        s.t_end = s.t_start + span;
    }
    // Read back the span that actually landed, so the rate describes the box that
    // was drawn even where a clamp bit. Deriving it from the *requested* span is
    // how a strip ends up with a speed its own edges contradict.
    s.speed = slice / s.span();
}
