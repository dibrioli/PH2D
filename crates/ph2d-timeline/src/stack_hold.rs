//! **The lane's HOLD** — which strip fills the coverage its live strips leave, and the pose
//! a fade crosses TO or FROM.
//!
//! Split from [`crate::stack`] for the LOC cap, but it is one unit: the complement of
//! [`ClipLane::weight_at`], answering *"what is under the fade"* for a gap, a loop wrap, or a
//! lone fade at either edge (Enio's fixes, 2026-07-16 … 2026-07-19). It reads the lane's
//! geometry (`weight_at`, `blend_in`/`blend_out`, `gap_*`) and the strips' time map
//! (`fold`, `hold_source_time`), and writes nothing.

use crate::stack::{ClipLane, ClipStrip};

impl ClipLane {
    /// **Which strip is HOLDING at `t`, and how strongly** — the lane's answer for
    /// whatever coverage its live strips do not account for.
    ///
    /// `None` when the live strips already sum to a full 1 (a strip mid-span, or two
    /// crossfading through their overlap: the overlap sums to exactly 1, so nothing
    /// is held and the crossfade is untouched), or when nothing has ended yet.
    ///
    /// # A strip's pose does not evaporate at its edge
    ///
    /// Before this, a lane's answer where no strip covered was *silence* — nobody
    /// wrote, and the object simply kept the pose it had. But a strip covering `t`
    /// with weight 0 (the first instant of a fade-in) answered **rest**. The two
    /// disagreed across one pixel of ruler, so a fade-in against nothing began with a
    /// jump: the sprite sat where the previous strip left it, then snapped to the rest
    /// pose to start the ramp it was supposed to start from *where it was* (Enio,
    /// 2026-07-16: *"a sprite não faz a transição a partir de onde está mas pula para
    /// mais perto da posição inicial da outra strip"* — measured at 3 units in one
    /// frame).
    ///
    /// The fix is not to silence weight zero — that is what made the pose depend on
    /// which side the playhead arrived from. It is that **the gap was never silence**:
    /// the previous strip is still asserting its last frame, and the incoming fade
    /// crossfades against *that*. This is Blender's `Hold` extrapolation and Unity's
    /// clip extrapolation, and it is what makes the lone fade behave like the overlap
    /// the animator already trusts.
    ///
    /// **Forward only — UNLESS a loop makes the lane cyclic.** A strip does not reach
    /// back before it starts: fading in from the rest pose at the top of a timeline is
    /// a real thing to want, and there is nothing behind the first strip to hold.
    ///
    /// That last clause is true of a timeline you play once, and **false of one you
    /// loop**, where the ruler's ends are neighbours: what is "before the first strip"
    /// is "after the last", and what is "after the last" is "before the first". So a
    /// fade at EITHER edge of the loop crosses to the pose the loop shows at the seam
    /// (the closing edge asks [`Self::seam_source`]; the opening edge, where nothing is
    /// live at the head to own it, is always the loop's end), never to the rest pose or
    /// a strip that ended before it:
    ///
    /// - the **opening** fade-in (nothing has ended yet) put the object at the last
    ///   strip's pose one frame and at the rest pose the next — a jump (Enio,
    ///   2026-07-16);
    /// - the **closing** fade-out (the loop's last content fading out toward the wrap)
    ///   reached the previous strip's held frame instead of the first strip's start —
    ///   also a jump (Enio, 2026-07-19). It is the trailing ramp of the strip that ends
    ///   latest AT or before the loop end; a strip that straddles the end fades out past
    ///   the wrap, where the loop never reaches.
    ///
    /// Outside the loop range nothing wraps and the fade-from-rest above is untouched.
    ///
    /// Returns the held strip, **the clip second it is asserting**, and the weight. The
    /// time is returned rather than re-derived by the caller because the cases answer it
    /// differently, and a caller that picked would be a second opinion about which frame
    /// is being held.
    ///
    /// The weight is the complement of what is live, which is exactly what turns the
    /// normalized mix into a plain `lerp(held, incoming, w)` — see the tests.
    #[must_use]
    pub fn hold_at(
        &self,
        t: f64,
        loop_range: Option<(f64, f64)>,
    ) -> Option<(&ClipStrip, f64, f64)> {
        let live: f64 = (0..self.strips.len()).map(|i| self.weight_at(i, t)).sum();
        let w = 1.0 - live;
        if w <= 0.0 {
            return None;
        }
        // **CLOSING edge of a loop.** The strip that ends latest (at or before the loop
        // end) in its fade-OUT ramp — or the gap after it, before the wrap — crosses to
        // the pose the loop shows at the seam ([`Self::seam_source`]), not to the strip
        // that ended before it (Enio, 2026-07-19). A strip that STRADDLES the loop end
        // fades out past the wrap, where the loop never reaches, so `t_end <= b` gates
        // it out. This runs BEFORE the mid-timeline hold below, which is exactly the
        // answer it overrides: at the trailing edge the previous strip is not the pose
        // to reveal.
        if let Some((a, b)) = loop_range
            && t >= a
            && t < b
        {
            let closing = self
                .strips
                .iter()
                .enumerate()
                .max_by(|(_, x), (_, y)| x.t_end.total_cmp(&y.t_end))
                .is_some_and(|(li, last)| {
                    let bo = self.blend_out(li);
                    bo > 0.0 && last.t_end <= b && t >= last.t_end - bo
                });
            if closing && let Some((strip, t_clip)) = self.seam_source(a, b) {
                return Some((strip, t_clip, w));
            }
        }
        // **FADE-OUT toward the NEXT strip (no loop needed).** A strip in its fade-out ramp
        // — and the gap AFTER it, up to where the next strip starts — crosses to the NEXT
        // strip's START, not to the rest pose (Enio, 2026-07-19: without this the object
        // sagged to rest during the fade, then JUMPED back to the strip's held pose in the
        // gap, then jumped again into the next strip). Now it travels to the next pose while
        // it fades, HOLDS it through the gap, and the next strip plays from it seamlessly.
        //
        // Runs BEFORE the mid-timeline hold below and overrides it: the hold reveals the
        // PREVIOUS strip (correct for a fade-IN, wrong for a fade-OUT, which reveals where
        // the object is GOING). It only fires when the strip actually faded out
        // (`blend_out > 0`, inside `fade_out_target`) — a hard cut with no fade keeps the
        // gap-holds-previous behaviour, which is the author's choice.
        if let Some(nxt) = self.fade_out_target(t) {
            return Some((nxt, nxt.fold(0.0), w));
        }
        // The most recently ENDED strip. A scan, not `strips.last()`: the lane is
        // sorted by START time, and a long strip can begin before a short one and
        // outlive it. This is the pose a fade-IN crosses FROM, and what a plain gap
        // (previous strip did not fade out) holds.
        if let Some(held) = self
            .strips
            .iter()
            .filter(|s| s.t_end <= t)
            .max_by(|a, b| a.t_end.total_cmp(&b.t_end))
        {
            return Some((held, held.hold_source_time(), w));
        }
        // Nothing has ended yet — the OPENING edge. Under a loop that brackets `t`,
        // wrap: the pose the object is coming FROM is the one the loop's end leaves
        // behind. It keeps its own expression rather than calling `seam_source`: when
        // nothing has ended, nothing is live at the head to own the seam, so the seam
        // is always the loop's end — which is `seam_source`'s own fallback.
        let (a, b) = loop_range?;
        if t < a || t >= b {
            return None;
        }
        let last = self
            .strips
            .iter()
            .max_by(|x, y| x.t_end.total_cmp(&y.t_end))?;
        let elapsed = (b - last.t_start).clamp(0.0, last.span()); // CLAMP-OK: span() >= 0
        Some((last, last.fold(elapsed), w))
    }

    /// **What the loop shows at the seam** (`b` ≡ `a`) — the pose the object rests on
    /// across the wrap, as a `(strip, clip-time)` the evaluator can sample.
    ///
    /// A seamless loop needs the fade on BOTH sides of the wrap to cross to the *same*
    /// pose, or they disagree and the loop jumps. That pose is whichever end OWNS the
    /// seam:
    ///
    /// - if a strip is fully live at the head `a` (no fade-in there), its own pose there
    ///   is the restart pose — the closing fade-out crosses to it and the loop lands on
    ///   it;
    /// - otherwise the head itself is fading in, and the object crosses from what the
    ///   loop's END leaves asserting: the last strip read at `b`. This is exactly the
    ///   opening wrap's own answer — the two share this door on purpose, so the fade-in
    ///   and the fade-out cannot disagree about the seam.
    ///
    /// When both ends fade, neither owns the seam and the last strip's end is the
    /// consistent choice both wraps reveal — the loop settles there while the weights
    /// dip through the wrap.
    fn seam_source(&self, a: f64, b: f64) -> Option<(&ClipStrip, f64)> {
        let head_live: f64 = (0..self.strips.len()).map(|i| self.weight_at(i, a)).sum();
        if head_live >= 1.0
            && let Some(first) = self.strips.iter().find(|s| s.covers(a))
        {
            let elapsed = (a - first.t_start).clamp(0.0, first.span()); // CLAMP-OK: span() >= 0
            return Some((first, first.fold(elapsed)));
        }
        // The head fades in (or is a gap): cross from what the loop's END leaves
        // asserting — the last strip read at the wrap. This is the SAME pose the opening
        // wrap reveals (`hold_at`'s last branch), so a fade-in and fade-out that meet at
        // the seam agree on it.
        let last = self
            .strips
            .iter()
            .max_by(|x, y| x.t_end.total_cmp(&y.t_end))?;
        let elapsed = (b - last.t_start).clamp(0.0, last.span()); // CLAMP-OK: span() >= 0
        Some((last, last.fold(elapsed)))
    }

    /// The strip that starts NEXT after time `end` — the smallest `t_start >= end`, with
    /// its index. `None` when nothing starts after (`end` is past the last strip).
    ///
    /// A strip that *overlaps* `end` (`t_start < end`) is not "next": that is a
    /// crossfade, and [`Self::weight_at`] already handles it with complementary weights.
    fn next_after(&self, end: f64) -> Option<(usize, &ClipStrip)> {
        self.strips
            .iter()
            .enumerate()
            .filter(|(_, o)| o.t_start >= end)
            .min_by(|(_, a), (_, b)| a.t_start.total_cmp(&b.t_start))
    }

    /// **What a fade-OUT at `t` crosses TO** — the next strip, or `None`.
    ///
    /// It fires while a strip is in its fade-out ramp AND through the gap after it, up to
    /// where the next strip's OWN fade-in ends:
    /// `t ∈ [s.t_end - blend_out(s), next.t_start + blend_in(next))`. Two conditions gate
    /// it, and both are the point:
    ///
    /// - `blend_out(s) > 0` — the strip actually has a fade-out. A hard cut (no fade) is
    ///   the author saying "hold and jump", and the gap keeps holding the PREVIOUS strip.
    /// - a `next` strip exists — there is somewhere to cross TO. The LAST strip's fade-out
    ///   with nothing after is the loop's job (`hold_at`'s closing branch) or a fade to
    ///   rest, not this.
    ///
    /// The crossed-to pose is the next strip's FROZEN first frame (`next.fold(0.0)`), the
    /// same pose the clip shows when it starts playing — so holding it through the gap and
    /// then playing it are the same value, and the entry is seamless.
    ///
    /// **The window reaches THROUGH the next strip's fade-in** (`+ blend_in(next)`), not
    /// just up to its start. When BOTH strips fade (this one out, the next one in), the
    /// object crosses to the next start and STAYS there while the next eases in — so the
    /// next strip eases from its own start instead of snapping back to the previous strip
    /// one frame after the gap. With no fade-in on the next strip, `blend_in` is 0 and the
    /// window is exactly the gap.
    fn fade_out_target(&self, t: f64) -> Option<&ClipStrip> {
        self.strips
            .iter()
            .enumerate()
            .filter(|(i, s)| {
                let bo = self.blend_out(*i);
                // Fades out either INWARD (`ease_out`, window starts at `t_end - bo`) or
                // OUTWARD (`lead_out`, in the gap from `t_end`). Both reach a next strip.
                (bo > 0.0 || s.lead_out > 0.0) && t >= s.t_end - bo
            })
            .filter_map(|(_, s)| {
                let (ni, nxt) = self.next_after(s.t_end)?;
                (t < nxt.t_start + self.blend_in(ni)).then_some(nxt)
            })
            .min_by(|a, b| a.t_start.total_cmp(&b.t_start))
    }
}
