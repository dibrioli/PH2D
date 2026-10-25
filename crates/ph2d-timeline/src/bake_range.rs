//! **O intervalo de um bake** — a porta única: a física coze a simulação, as constraints a pilha
//! (`ph2d-app-constraints::bake`), e os dois perguntam aqui quanto da timeline cobrem.

/// How long a bake covers when the timeline has no opinion — no keys, no clip
/// duration, no armed loop.
///
/// **Measured, not chosen.** A body dropped from the top of the default view
/// (~4 m above the floor) touches down at ~0.9 s and has stopped bouncing by
/// ~2.5 s; a six-link chain settles by ~3.5 s. 5 s covers those with room, and
/// is the number a fresh scene bakes when the artist has not said otherwise.
/// It is a DEFAULT, not a ceiling: arming a loop in the transport overrides it,
/// and a document with keys uses its own extent.
pub const DEFAULT_BAKE_SECONDS: f64 = 5.0;

/// The window a Bake covers, `(start, end)` seconds, given the document and the
/// transport.
///
/// **The simulation still runs from tick 0** — it is a function of the tick, so
/// there is no starting it in the middle — but only the samples in `[start,
/// end]` become keys; the front `[0, start)` is simulated (the scene must be
/// advanced through it for any body riding a baked platform) and then
/// discarded. That discarding is the whole point: baking a `[2s, 5s]` loop used
/// to write keys across `[0, 5s]`, ignoring the start the artist had set.
///
/// The **loop is the control** for both ends, because it is the one the artist
/// already has for "this much of the timeline" — inventing a start/end field
/// beside the button would be a second way to say the same thing (the reasoning
/// [`DEFAULT_BAKE_SECONDS`] already gives for the end applies to the start).
/// So: an armed loop with a positive end gives `(loop_start, loop_end)`;
/// otherwise the window starts at 0 and ends at the document's own extent, or
/// [`DEFAULT_BAKE_SECONDS`] for a fresh scene. The range is shown ON the button,
/// so it is never a surprise.
#[must_use]
pub fn bake_range(doc: &crate::TimelineDoc, playhead: &ph2d_core::Playhead) -> (f64, f64) {
    if let Some((start, end)) = playhead.loop_range()
        && end > 0.0
    {
        // `set_loop_range` already clamps the start to `>= 0`; `max(0.0)` here is
        // belt-and-braces so a negative can never become a negative key time.
        return (start.max(0.0), end);
    }
    let extent = doc.end_seconds();
    if extent > 0.0 {
        return (0.0, extent);
    }
    (0.0, DEFAULT_BAKE_SECONDS)
}
