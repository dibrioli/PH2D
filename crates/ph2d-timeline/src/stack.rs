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
        if !self.covers(t) {
            return None;
        }
        let slice = self.slice();
        if slice <= 0.0 {
            return Some(self.src_in); // a zero-length slice is a pose, not a clip
        }
        let advanced = (t - self.t_start) * self.speed;
        let folded = match self.loop_mode {
            StripLoop::Once => advanced.clamp(0.0, slice),
            StripLoop::Loop => advanced.rem_euclid(slice),
            StripLoop::PingPong => {
                // Reflect over a period of two slices: forward, then backward.
                let u = advanced.rem_euclid(slice * 2.0);
                if u <= slice { u } else { slice * 2.0 - u }
            }
        };
        Some(self.src_in + folded)
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

    /// The blend window at the START of strip `i`: the overlap with the strip
    /// before it, or the authored `ease_in` when it has no neighbour there.
    #[must_use]
    pub fn blend_in(&self, i: usize) -> f64 {
        let s = &self.strips[i];
        let overlap = i
            .checked_sub(1)
            .map(|p| self.strips[p].t_end - s.t_start)
            .unwrap_or(0.0);
        if overlap > 0.0 {
            overlap.min(s.span()) // the overlap IS the blend (Unity's rule)
        } else {
            s.ease_in.max(0.0).min(s.span())
        }
    }

    /// The blend window at the END of strip `i`. Mirror of [`Self::blend_in`].
    #[must_use]
    pub fn blend_out(&self, i: usize) -> f64 {
        let s = &self.strips[i];
        let overlap = self
            .strips
            .get(i + 1)
            .map(|n| s.t_end - n.t_start)
            .unwrap_or(0.0);
        if overlap > 0.0 {
            overlap.min(s.span())
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
    #[must_use]
    pub fn weight_at(&self, i: usize, t: f64) -> f64 {
        let s = &self.strips[i];
        if !s.covers(t) {
            return 0.0;
        }
        let fade_in = ramp(t - s.t_start, self.blend_in(i));
        let fade_out = ramp(s.t_end - t, self.blend_out(i));
        fade_in * fade_out
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
