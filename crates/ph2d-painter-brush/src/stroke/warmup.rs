//! The texture-**Rake warm-up**: a stroke holds its opening dabs until it has travelled far enough to
//! define a confident heading, then releases them all stamped with that settled heading — so a Rake
//! stroke starts already at the correct angle instead of laying its first dab at the rest Angle and
//! snapping once motion begins. A child module of [`super`] (`stroke`) so it keeps private access to
//! `Stroke`'s warm-up fields; split out to keep `stroke.rs` under the workspace LOC cap.
//!
//! Eligibility + state live in [`super::Stroke`] (set in `begin`); the gate is run after every emission
//! (`begin`/`extend`/`finish`/`settle`). `pub(super)` so those sibling call sites can reach it. Only the
//! emission TIMING changes — the heading EMA still computes live; a non-Rake brush never warms.

use super::*;

impl Stroke {
    /// The warm-up gate, run after each emission while `warming`: hold the just-emitted dabs in
    /// `warm_buf` until the cursor has travelled [`crate::heading::warmup_len`] *and* a heading exists,
    /// then release the whole batch at that settled heading. A no-op once warmed / for a non-Rake brush,
    /// so the common path is untouched (dabs flow straight through, the heading EMA stamps them live).
    pub(super) fn warmup_gate(&mut self, out: &mut Vec<Dab>) {
        if !self.warming {
            return;
        }
        self.warm_buf.append(out); // hold the new dabs (drains `out` → empty until release)
        let w = crate::heading::warmup_len(2.0 * self.spec.clamped_radius());
        if self.warm_dist >= w && self.heading != [0.0, 0.0] {
            self.release_warmup(out);
        }
    }

    /// Flush the held start-up dabs into `out`, stamping the settled `heading` on each so the stroke opens
    /// already at the correct Rake angle, and end the warm-up (later dabs flow with the live heading).
    pub(super) fn release_warmup(&mut self, out: &mut Vec<Dab>) {
        for mut d in self.warm_buf.drain(..) {
            d.dir = self.heading;
            out.push(d);
        }
        self.warming = false;
    }
}
