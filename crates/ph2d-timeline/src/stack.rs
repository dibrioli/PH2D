//! The clip stack: **lanes** of clip **strips** (ADR-0115).
//!
//! A strip is one *instance* of a clip placed on the timeline — where it plays,
//! which slice of the clip it uses, how fast, and what the source does when it
//! runs out. A lane is an ordered row of strips; lanes stack bottom to top.
//!
//! This module owns the two things a strip knows how to answer, and nothing
//! else — the blend across lanes lives in the evaluator:
//!
//! - **Where in the clip am I?** ([`ClipStrip::source_time`]) — the strip's time
//!   map. It maps *timeline* time to *clip* time. It does NOT touch the entity's
//!   Time Remap: that is the clip's own clock and composes **inside** this one
//!   (ADR-0115 R6, the AE precomp model). One map, one direction, no second
//!   clock invented alongside it.
//! - **How much do I count?** ([`ClipLane::weight_at`]) — the ease curve.
//!
//! **The gesture** (ADR-0115 R1, and the one thing Blender's NLA cannot do):
//! *overlapping two strips on a lane IS the crossfade.* The overlap's width is
//! the blend's duration — nobody types a number, and there are no two numbers to
//! keep in agreement. An authored `ease_in`/`ease_out` only applies where a strip
//! has no neighbour to blend against; where it has one, the overlap wins. This is
//! Unity's rule (the field is literally relabelled "Blend" and greyed out when an
//! overlap defines it), and it is what makes ease and blend the same curve rather
//! than two systems that must agree.
//!
//! The crossfade is **exactly complementary** — `w_a + w_b == 1` through the
//! whole overlap — because smoothstep satisfies `s(1 - u) == 1 - s(u)`. That is
//! not a nicety: complementary weights need **no base value**, so the crossfade
//! is immune to the "sag toward the default pose" that Unity ships a whole
//! `AnimationOutputWeightProcessor` to prevent. It is proved in the tests.

use serde::{Deserialize, Serialize};

/// What a strip's source does once it runs past its slice.
///
/// There is deliberately **no "Nothing"** variant (Blender's, which stops the
/// strip contributing while its span still covers the time). A strip that spans
/// time it cannot fill is a mis-trimmed strip, not a feature — and a strip whose
/// coverage silently drops to zero mid-span is exactly the hole that lets the
/// stack fall back to a default value and yank the sprite. Trim the strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StripLoop {
    /// Play the slice once, then hold its last value for the rest of the span.
    #[default]
    Once,
    /// Wrap back to the slice's start.
    Loop,
    /// Reflect: play forward, then backward, then forward…
    PingPong,
}

/// How a lane's value enters the stack below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LaneMode {
    /// Mix toward this lane's value (`lerp`). The lane *replaces* what is under
    /// it, by its coverage and weight.
    #[default]
    Override,
    /// Add this lane's **delta** — its value measured against the first frame of
    /// its own clip. A clip holding a constant pose therefore contributes
    /// nothing, which is the whole point: an additive lane carries *change*, not
    /// position. (Maya: "evaluates the clip relative to its first frame"; Unity's
    /// additive reference pose is frame 0 of the clip.)
    Additive,
}

/// A strip's stable identity, for as long as the document lives.
///
/// A strip cannot be addressed by its index: the lane keeps its strips **sorted
/// by start time**, so dragging one past its neighbour renumbers both. A drag
/// anchored on an index would silently grab the other strip at the exact moment
/// they crossed — which is the moment the animator is looking hardest. Selection
/// and undo have the same problem. Mirrors `KeyId`, for the same reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StripId(pub u64);

/// One placement of a clip on the timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClipStrip {
    /// Stable identity (see [`StripId`]). Allocated by the document.
    pub id: StripId,
    /// Index into the document's clips.
    pub clip: u16,
    /// Timeline seconds: where the strip starts.
    pub t_start: f64,
    /// Timeline seconds: where it ends (exclusive).
    pub t_end: f64,
    /// Clip seconds: the first frame of the slice used.
    pub src_in: f64,
    /// Clip seconds: the end of the slice used.
    pub src_out: f64,
    /// Source playback rate. 1.0 = real time; 2.0 = twice as fast.
    pub speed: f64,
    /// What the source does past its slice.
    pub loop_mode: StripLoop,
    /// Authored fade-in, in seconds. **Ignored where an overlap defines the
    /// blend** — the overlap is the blend (see the module docs).
    pub ease_in: f64,
    /// Authored fade-out, in seconds. Same rule.
    pub ease_out: f64,
    /// **Outward fade-in ("lead-in"), in seconds** — the fade that lives in the GAP
    /// *before* the strip, not inside it (Enio, 2026-07-16). Where `ease_in` blends
    /// against this clip while it PLAYS (its opening is spent in the crossfade), the
    /// lead-in blends against the clip's FROZEN first frame: the object travels from
    /// the previous strip's held pose to this clip's start pose during the gap, and
    /// then the clip plays from frame 0 untouched. It reaches back from `t_start` and
    /// is mutually exclusive with `ease_in` (the fade-in grip is on one side of the
    /// edge or the other). Appended (`DOC_VERSION` 5 -> 6); `0.0` is the old
    /// behaviour byte-for-byte.
    pub lead_in: f64,
    /// **What each corner's last edit DID, in seconds — the change bar** (Enio,
    /// 2026-07-16). Index it with [`mark_index`]; the value is signed, and it is
    /// `edge_before_the_gesture - edge_now`.
    ///
    /// The panel draws the interval between the edge and `edge + mark` in the
    /// corner's own colour, so **which way it points falls out of the sign** rather
    /// than being a second rule to keep in step: pull a start edge outward and the
    /// span it gained is now INSIDE the strip; push it inward and the span it lost
    /// is OUTSIDE. Same formula at both edges, both operations.
    ///
    /// It is a **delta, not a time**, which is what lets it survive a slide: move
    /// the whole strip and the mark travels with the edge it describes, because it
    /// never referred to an absolute moment in the first place.
    ///
    /// It lives in the document (rather than in the panel's drag state) because
    /// "permanently visible" has to mean *after you let go, after an undo, and
    /// after a reload* — a mark that evaporates on load would be a mark that lies
    /// about being permanent. Appended (`DOC_VERSION` 6 -> 7); all-zero is the old
    /// behaviour, and zero draws nothing.
    pub marks: [f64; 4],
}

/// Index into [`ClipStrip::marks`] for one corner: `stretch` picks the GREEN top
/// pair over the RED bottom pair, `edge` is the document's usual `0` = start,
/// `1` = end.
///
/// One door, because three places ask this question — the trim apply, the stretch
/// apply, and the painter — and a corner whose mark is written at one index and
/// read at another is a mark that silently describes the wrong edit.
#[must_use]
pub const fn mark_index(stretch: bool, edge: u8) -> usize {
    (if stretch { 2 } else { 0 }) + (edge != 0) as usize
}

impl ClipStrip {
    /// A strip playing all of `clip` over `[t_start, t_end)`, at speed 1, no ease.
    ///
    /// The id is left at zero: authoring goes through [`crate::TimelineDoc`], which
    /// allocates one. (Two strips sharing an id would confuse a drag, not corrupt
    /// the document — the evaluator never reads the id.)
    #[must_use]
    pub fn new(clip: u16, t_start: f64, t_end: f64, src_len: f64) -> Self {
        Self {
            id: StripId(0),
            clip,
            t_start,
            t_end,
            src_in: 0.0,
            src_out: src_len,
            speed: 1.0,
            loop_mode: StripLoop::Once,
            ease_in: 0.0,
            ease_out: 0.0,
            lead_in: 0.0,
            marks: [0.0; 4],
        }
    }

    /// Builder: stamp the identity the document allocated.
    #[must_use]
    pub fn with_id(mut self, id: StripId) -> Self {
        self.id = id;
        self
    }

    /// How long the strip occupies the timeline.
    #[must_use]
    pub fn span(&self) -> f64 {
        (self.t_end - self.t_start).max(0.0)
    }

    /// How much of the clip it uses.
    #[must_use]
    pub fn slice(&self) -> f64 {
        (self.src_out - self.src_in).max(0.0)
    }

    /// `true` while the strip covers `t`.
    #[must_use]
    pub fn covers(&self, t: f64) -> bool {
        t >= self.t_start && t < self.t_end
    }

    /// The **clip** time this strip reads at timeline time `t`, or `None` when it
    /// does not cover `t`.
    ///
    /// This is the strip's whole contract with time. What it hands back is a time
    /// in the clip's own frame — the entity's Time Remap track (which lives in
    /// that clip) then maps it to the entity's source time. Outer map, inner map,
    /// one direction: never a second clock running beside the first.
    #[must_use]
    pub fn source_time(&self, t: f64) -> Option<f64> {
        self.covers(t).then(|| self.fold(t - self.t_start))
    }

    /// Where the outward fade-in reaches back to — `t_start - lead_in`, clamped so a
    /// lead never starts before time zero.
    #[must_use]
    pub fn lead_start(&self) -> f64 {
        (self.t_start - self.lead_in.max(0.0)).max(0.0)
    }

    /// The clip time this strip reads INCLUDING its outward lead-in.
    ///
    /// In the lead-in window `[lead_start, t_start)` it returns `src_in` — the clip's
    /// FROZEN first frame, the pose the object travels TO across the gap — regardless
    /// of loop mode (the travel is to the first frame, not to a wrapped one). Inside
    /// the span it is [`Self::source_time`]; outside both, `None`. This is the door the
    /// evaluator samples through, so the lead-in window contributes a still pose, not a
    /// negative/extrapolated time.
    #[must_use]
    pub fn source_time_with_lead(&self, t: f64) -> Option<f64> {
        if self.lead_in > 0.0 && t >= self.lead_start() && t < self.t_start {
            Some(self.src_in) // the frozen first frame — the travel target
        } else {
            self.source_time(t)
        }
    }

    /// **The clip time this strip HOLDS once it is over** — its reading at the very
    /// end of its span.
    ///
    /// A strip's pose does not evaporate at its edge: it persists until something
    /// else takes over, which is what makes a lone fade-in *cross* from the previous
    /// strip instead of from the rest pose (Blender's `Hold` extrapolation, Unity's
    /// clip extrapolation). See [`ClipLane::hold_at`].
    ///
    /// The limit of [`Self::source_time`] as `t → t_end`, not a second opinion about
    /// it: both fold through [`Self::fold`], so a looping strip holds exactly the
    /// frame it was on and cannot disagree with the frame it was showing a moment
    /// before.
    #[must_use]
    pub fn hold_source_time(&self) -> f64 {
        self.fold(self.span())
    }

    /// How far into the clip a strip that has been running for `elapsed` seconds of
    /// TIMELINE time is reading — rate, then whatever the source does past its slice.
    ///
    /// The one place the folding lives. [`Self::source_time`] asks it for an instant
    /// inside the span and [`Self::hold_source_time`] for the span's end.
    fn fold(&self, elapsed: f64) -> f64 {
        let slice = self.slice();
        if slice <= 0.0 {
            return self.src_in; // a zero-length slice is a pose, not a clip
        }
        let advanced = elapsed * self.speed;
        let folded = match self.loop_mode {
            StripLoop::Once => advanced.clamp(0.0, slice),
            StripLoop::Loop => advanced.rem_euclid(slice),
            StripLoop::PingPong => {
                // Reflect over a period of two slices: forward, then backward.
                let u = advanced.rem_euclid(slice * 2.0);
                if u <= slice { u } else { slice * 2.0 - u }
            }
        };
        self.src_in + folded
    }
}

/// A row of strips. Lanes stack bottom to top; the strips inside one are ordered
/// by start time and **may overlap** — that is how a crossfade is authored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClipLane {
    /// Author-visible name.
    pub name: String,
    /// A muted lane contributes nothing at all — which is **not** the same as a
    /// weight of zero. Zero weight still asserts the lane's coverage and mixes
    /// toward it; muting removes the lane from the stack. (Blender's own layered
    /// design draws this distinction explicitly, having learned it the hard way.)
    pub muted: bool,
    /// The lane's influence over the stack below it, `[0, 1]`.
    pub weight: f64,
    /// How it enters the stack.
    pub mode: LaneMode,
    /// Ordered by `t_start` (see [`ClipLane::insert`]).
    pub strips: Vec<ClipStrip>,
}

impl ClipLane {
    /// A fresh, empty lane at full weight.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            muted: false,
            weight: 1.0,
            mode: LaneMode::Override,
            strips: Vec::new(),
        }
    }

    /// Add a strip, keeping the lane ordered by start time. The order is an
    /// invariant: [`Self::weight_at`] reads a strip's neighbours to find the
    /// overlap that defines its blend, and neighbours only mean something in a
    /// sorted row.
    pub fn insert(&mut self, strip: ClipStrip) -> usize {
        let at = self.strips.partition_point(|s| s.t_start <= strip.t_start);
        self.strips.insert(at, strip);
        at
    }

    /// Where the strip with this identity currently sits, if it is here.
    ///
    /// An index is a *position*, not a name: a drag holds the [`StripId`] and asks
    /// this each time, because moving a strip past its neighbour renumbers both.
    #[must_use]
    pub fn index_of(&self, id: StripId) -> Option<usize> {
        self.strips.iter().position(|s| s.id == id)
    }

    /// Restore the sort after a strip's start time changed — the invariant that
    /// [`Self::weight_at`] rests on (a neighbour only means something in order).
    pub fn resort(&mut self) {
        self.strips
            .sort_by(|a, b| a.t_start.total_cmp(&b.t_start).then(a.id.cmp(&b.id)));
    }

    /// The blend window at the START of strip `i`: how far into it another strip is
    /// still playing, or the authored `ease_in` when none is.
    ///
    /// **Every other strip live at that edge is asked, not just `strips[i-1]`.** The
    /// neighbour in sort order is the right strip to ask only when the lane is a
    /// staircase, and nothing makes it one: a body drag can drop a short strip
    /// *inside* a long one, or leave two overlapping strips with a third between
    /// them. Asking the wrong strip is how a lane's coverage silently collapsed —
    /// a strip would fade out against a neighbour that had already ended, `den`
    /// would fall toward zero, and the sprite would crawl back to its rest pose in
    /// the middle of a clip that never moves. (The bug the audit found; the tests
    /// only ever built staircases.)
    #[must_use]
    pub fn blend_in(&self, i: usize) -> f64 {
        let s = &self.strips[i];
        let reach = self.neighbour_reach_in(i);
        if reach > 0.0 {
            reach.min(s.span()) // the overlap IS the blend (Unity's rule)
        } else {
            s.ease_in.max(0.0).min(s.span())
        }
    }

    /// How far another strip still plays INTO the start of strip `i` — `0.0` when
    /// nothing does. The overlap, before it is capped by the span.
    ///
    /// This is the ONE place that answers *"whose window is this?"*, and both callers
    /// need the same answer: [`Self::blend_in`] uses it to pick between the overlap and
    /// the authored `ease_in`, and the panel uses it to decide whether the ease handle is
    /// draggable or **read-only** (Unity greys the field out when an overlap defines it).
    /// Two copies of this test would be two ways to disagree about who owns an edge, and
    /// the artist would find the disagreement by dragging a handle that does nothing.
    #[must_use]
    pub fn neighbour_reach_in(&self, i: usize) -> f64 {
        let s = &self.strips[i];
        self.strips
            .iter()
            .enumerate()
            .filter(|(j, o)| *j != i && o.covers(s.t_start))
            .map(|(_, o)| o.t_end - s.t_start)
            .fold(0.0_f64, f64::max)
    }

    /// Mirror of [`Self::neighbour_reach_in`] at the end — and it asks the *other*
    /// question: only a strip that is still there when this one ENDS (`o.t_end >=
    /// s.t_end`) shortens its tail. One that ends earlier makes a hump in the middle,
    /// and the middle is not an edge.
    #[must_use]
    pub fn neighbour_reach_out(&self, i: usize) -> f64 {
        let s = &self.strips[i];
        self.strips
            .iter()
            .enumerate()
            .filter(|(j, o)| *j != i && o.t_start < s.t_end && o.t_end >= s.t_end)
            .map(|(_, o)| s.t_end - o.t_start)
            .fold(0.0_f64, f64::max)
    }

    /// The blend window at the END of strip `i`. Mirror of [`Self::blend_in`].
    ///
    /// A strip only fades OUT against something that is still there when it ends —
    /// `o.t_end >= s.t_end`. A strip that ends *before* this one does not shorten
    /// this one's tail; it makes a hump in its middle, and the middle is not an
    /// edge. That distinction is the whole of the containment fix.
    #[must_use]
    pub fn blend_out(&self, i: usize) -> f64 {
        let s = &self.strips[i];
        let reach = self.neighbour_reach_out(i);
        if reach > 0.0 {
            reach.min(s.span())
        } else {
            s.ease_out.max(0.0).min(s.span())
        }
    }

    /// Strip `i`'s weight at `t`: the product of its fade-in and fade-out ramps,
    /// zero where it does not cover `t`.
    ///
    /// The product is Unity's shape (`mixIn(t) * mixOut(t)`), and the ramp is
    /// smoothstep — an S, not a line, because a linear crossfade has a visible
    /// corner in velocity at both ends. Transcendental-free (HR-5).
    ///
    /// **The fade-in reaches OUTWARD by `lead_in`.** The ramp runs `0 -> 1` over
    /// `[lead_start, t_start + blend_in]`, so the strip is "present" in its lead-in
    /// window (in the gap), and [`Self::hold_at`] gives the complement to the previous
    /// strip's held pose: the object travels from that pose to this clip's frozen first
    /// frame ([`ClipStrip::source_time_with_lead`]) across the gap, and reaches full
    /// weight exactly at `t_start` — where the clip starts playing clean. With no lead
    /// (`lead_in == 0`) the window is `[t_start, ...]` and this is byte-for-byte the old
    /// behaviour.
    #[must_use]
    pub fn weight_at(&self, i: usize, t: f64) -> f64 {
        let s = &self.strips[i];
        let lead = s.lead_in.max(0.0);
        if t < s.lead_start() || t >= s.t_end {
            return 0.0;
        }
        let fade_in = ramp(t - s.lead_start(), lead + self.blend_in(i));
        let fade_out = ramp(s.t_end - t, self.blend_out(i));
        fade_in * fade_out
    }

    /// The empty span before strip `i` — from the end of the nearest strip that ends
    /// at or before it (or time 0 if none) up to its start. This is how far a lead-in
    /// may reach without overrunning a neighbour's live span: the outward fade lives in
    /// the GAP, and the gap ends where the previous strip does.
    #[must_use]
    pub fn gap_before(&self, i: usize) -> f64 {
        let s = &self.strips[i];
        let prev_end = self
            .strips
            .iter()
            .enumerate()
            .filter(|(j, o)| *j != i && o.t_end <= s.t_start)
            .map(|(_, o)| o.t_end)
            .fold(0.0_f64, f64::max);
        (s.t_start - prev_end).max(0.0)
    }

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
    /// **Forward only.** A strip does not reach back before it starts: fading in from
    /// the rest pose at the top of a timeline is a real thing to want, and there is
    /// nothing behind the first strip to hold anyway.
    ///
    /// The weight is the complement of what is live, which is exactly what turns the
    /// normalized mix into a plain `lerp(held, incoming, w)` — see the tests.
    #[must_use]
    pub fn hold_at(&self, t: f64) -> Option<(&ClipStrip, f64)> {
        let live: f64 = (0..self.strips.len()).map(|i| self.weight_at(i, t)).sum();
        let w = 1.0 - live;
        if w <= 0.0 {
            return None;
        }
        // The most recently ENDED strip. A scan, not `strips.last()`: the lane is
        // sorted by START time, and a long strip can begin before a short one and
        // outlive it.
        let held = self
            .strips
            .iter()
            .filter(|s| s.t_end <= t)
            .max_by(|a, b| a.t_end.total_cmp(&b.t_end))?;
        Some((held, w))
    }
}

/// The fade curve: `smoothstep(elapsed / window)`, and 1 where there is no window.
///
/// `smoothstep(1 - u) == 1 - smoothstep(u)`, which is why two strips sharing an
/// overlap sum to exactly 1 through it (proved in the tests). Complementary
/// weights need no base value to blend against — that property is what keeps the
/// crossfade immune to sagging toward a default pose.
fn ramp(elapsed: f64, window: f64) -> f64 {
    if window <= 0.0 {
        return 1.0;
    }
    let u = (elapsed / window).clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}
