//! **Authoring a strip's fades** — the inward ease and the outward lead, at either edge.
//!
//! Split from [`crate::intent_apply`] for the LOC cap; it is the two intents that write a
//! strip's four fade fields (`ease_in`/`ease_out`/`lead_in`/`lead_out`). Both run inside the
//! caller's `edit_at` bracket, so they take the `doc` and `host` the closure hands them and
//! just mutate.

use crate::StackHost;
use crate::doc::TimelineDoc;
use crate::stack::StripId;

/// The fade the strip authors for itself (B4) — the INWARD ease.
///
/// **The clamp is on the SUM, not on each edge.** `weight_at` is `mixIn(t) * mixOut(t)`
/// (Unity's shape), so two fades that OVERLAP multiply: give a 2 s strip a 2 s fade-in and a
/// 2 s fade-out — each perfectly legal on its own — and its weight peaks at **0.25**, never
/// reaching 1. On an `Override` lane that is a sprite permanently half-blended toward the
/// pose below, with nothing on screen to explain why. Unity clamps the sum for exactly this
/// reason, and so do we: a fade can take the whole strip, but the two of them cannot take it
/// twice. It also keeps the two grips from crossing on screen — the tips meet, never pass.
pub(crate) fn set_ease(
    doc: &mut TimelineDoc,
    host: StackHost,
    lane: usize,
    id: StripId,
    edge: u8,
    seconds: f64,
) {
    if let Some(s) = doc.strip_in_mut(host, lane, id) {
        let other = if edge == 0 { s.ease_out } else { s.ease_in };
        let room = (s.span() - other.max(0.0)).max(0.0);
        let v = seconds.clamp(0.0, room); // CLAMP-OK: 0..what the other fade left
        if edge == 0 {
            s.ease_in = v;
            // The inward fade-in and the OUTWARD one (`lead_in`) are the same handle on two
            // sides of the start edge — authoring one clears the other. Without this, dragging
            // the fade back in from the gap left a sliver of `lead_in`, and the grip (drawn
            // outward whenever `lead_in` is non-zero) stuck at the strip's tip and would not
            // come inside (Enio, 2026-07-16).
            s.lead_in = 0.0;
        } else {
            s.ease_out = v;
            s.lead_out = 0.0; // same exclusivity at the END edge (Enio, 2026-07-19)
        }
    }
}

/// The OUTWARD fade (the travel across the gap) — the LEAD.
///
/// Clamped to the gap on that side so it never overruns the neighbour's live span, and it
/// CLEARS the inward ease on the SAME edge: the fade handle authors one side of the edge or
/// the other, never both (the evaluator would blend against both a frozen and a live edge).
/// The gap is read from the LANE (it needs the neighbours), so the strip index is resolved
/// first.
pub(crate) fn set_lead(
    doc: &mut TimelineDoc,
    host: StackHost,
    lane: usize,
    id: StripId,
    edge: u8,
    seconds: f64,
) {
    let Some(l) = doc.host_stack_mut(host).and_then(|s| s.get_mut(lane)) else {
        return;
    };
    let Some(i) = l.index_of(id) else {
        return;
    };
    if edge == 0 {
        let gap = l.gap_before(i);
        let s = &mut l.strips[i];
        s.lead_in = seconds.clamp(0.0, gap); // CLAMP-OK: 0..the empty time before it
        s.ease_in = 0.0; // the inward fade-in and the outward one are exclusive
    } else {
        let gap = l.gap_after(i);
        let s = &mut l.strips[i];
        s.lead_out = seconds.clamp(0.0, gap); // CLAMP-OK: 0..the empty time after it
        s.ease_out = 0.0; // the inward fade-out and the outward one are exclusive
    }
}
