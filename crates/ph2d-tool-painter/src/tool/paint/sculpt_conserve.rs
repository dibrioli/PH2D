//! **Which verbs conserve volume, and which way each one lets the surface travel.**
//!
//! Split from [`super::sculpt`] for the workspace file-LOC cap, and along a seam that was already there:
//! that module answers *what the artist chose*, and these two functions answer *what the physics owes* —
//! the pair the Conserve bite and the render both consult, and which must never be answered twice.

use super::sculpt::SculptMode;

impl SculptMode {
    /// The verbs the **Conserve** flag applies to — the whole plane family now.
    ///
    /// ## The law is one sentence, and Scrape was always a special case of it
    ///
    /// **Conserve means the stroke's net volume change is zero, and the rim settles the difference with
    /// the ledger's sign.** Scrape and the Chisel only ever remove, so their ledger is always negative and
    /// the rim always rises — which is why the first cut could hard-code a ridge and never notice it had
    /// assumed anything.
    ///
    /// Flatten and Fill were held back because *"conserving what ADDS needs a design for where the volume
    /// comes FROM"*. It does, and the answer is the mirror of where the removed volume goes: **from the
    /// rim**. Filling a hollow by dragging the surrounding paint into it leaves a moat, exactly as scraping
    /// a channel leaves a ridge. One law, one buffer, one kernel — the sign of `displaced` is the only
    /// thing that differs, and `bank_dab_push` was already sign-transparent (`plane[i] += k · scale`).
    ///
    /// ## ⚠️ What the measurement changed about what this flag MEANS
    ///
    /// Measured before building it (net volume of one stroke, ridge fixture, `loads·px²`):
    ///
    /// ```text
    ///              offset 0      +0,25      +0,5      −0,25
    ///   FLATTEN       +0,7       +741,7   +1482,7     −740,3
    ///   FILL         +59,7       +741,7   +1482,7       +0,0
    /// ```
    ///
    /// **Flatten at the centre of its Offset track is already conservative** (+1,2%), and not by luck: the
    /// least-squares plane passes through the weighted centroid, so `Σ w·(plane − h) = 0` by construction —
    /// what it takes off the peaks it has already put in the valleys. The residual is only the mismatch
    /// between the fit's weights and the render's `k = min(amount, 1)`.
    ///
    /// So on Flatten this flag is not *"stop deleting paint"*; it is **the Offset's counterweight**. The
    /// Offset is the volume knob — off centre it creates or destroys twelve times the whole redistribution
    /// — and Conserve is what makes the knob move paint around instead of conjuring it. That is why the
    /// flag is still worth offering on a verb that is neutral at rest: it is live exactly where the verb
    /// stops being neutral. (Fill needs no such caveat: it always adds.)
    pub(super) fn conserves(self) -> bool {
        matches!(
            self,
            SculptMode::Scrape | SculptMode::Chisel | SculptMode::Flatten | SculptMode::Fill
        )
    }

    /// Which way this verb lets the surface travel — the render's own `delta` clamp, published so the
    /// Conserve bite can measure the volume the render will actually move.
    ///
    /// **One door, asked twice.** `render_sculpt` clamps the delta and this reports the clamp; they must
    /// agree or the ledger would account for a stroke nobody made. (The non-plane verbs answer `Both`
    /// because nothing asks them — the bite only ever rides the plane family.)
    pub(super) fn travel(self) -> ph2d_painter_brush::sculpt::Travel {
        use ph2d_painter_brush::sculpt::Travel;
        match self {
            SculptMode::Scrape | SculptMode::Chisel => Travel::Down,
            SculptMode::Fill => Travel::Up,
            _ => Travel::Both,
        }
    }
}
