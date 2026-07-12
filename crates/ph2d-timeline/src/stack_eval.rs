//! Evaluating the clip stack: lanes in, one value out (ADR-0115 R2-R5).
//!
//! Two levels, and they answer different questions:
//!
//! 1. **Inside a lane** the strips are *alternatives in time* — at most a
//!    crossfade between two of them. So the lane **normalizes**: its value is the
//!    weighted mean of its live strips, `sum(w*v) / sum(w)`. That division is the
//!    single most important line in this file. Without it a lane at half weight
//!    would hand out *half a value*, and the stack below would make up the
//!    difference from a default — which for an absolute channel like
//!    `TranslationX` means the sprite lurches toward its parent's origin. Unity
//!    ships an entire `AnimationOutputWeightProcessor` to stop exactly this.
//!
//!    What the lane *says* (the normalized value) and how much it *asserts* (its
//!    coverage, `min(1, sum(w))`) are therefore two different numbers.
//!
//! 2. **Across lanes**, bottom to top, coverage becomes influence and the lane
//!    enters by its mode: `Override` mixes toward its value, `Additive` applies
//!    its **delta**.
//!
//! **Sparsity is the mask** (R2). A lane only touches the channels its clips key.
//! A channel no lane touches is never written, and the scene keeps it. This is
//! why there is no Avatar Mask, no bone filter, no channel list: the keyed set is
//! the mask, exactly as in Rive and Spine — and unlike them, it costs nothing to
//! get right, because the crossfade weights are complementary (`stack.rs`).

use ph2d_anim::{AnimTarget, AnimValue, AttributeEvaluator, Track};

use crate::clock::ClockIndex;
use crate::doc::TimelineDoc;
use crate::prop::{Algebra, PropKind};
use crate::stack::LaneMode;

/// One strip that is live at the current time, with everything the per-binding
/// loop needs — resolved **once per frame**, not once per binding.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ActiveStrip {
    /// Which lane it belongs to (its index in the stack).
    pub lane: usize,
    /// Which clip it plays.
    pub clip: usize,
    /// Its blend weight at this instant (the crossfade ramp).
    pub w: f64,
    /// The clip time it is reading.
    pub t_clip: f64,
    /// The first frame of its slice — the reference an additive lane measures
    /// its delta against.
    pub src_in: f64,
}

/// Per-frame scratch: which strips are live, and each one's entity clocks.
///
/// The clocks are per **strip**, not per document, because two strips can play
/// two different clips, and an entity's Time Remap track lives *inside* a clip.
/// One index per live strip is the price of that, and it is a small price: live
/// strips are counted on one hand.
///
/// Zero-alloc in steady state (HR-3): both vectors are cleared and refilled, and
/// the clock indices are retained so their own buffers stay warm.
#[derive(Debug, Default, Clone)]
pub(crate) struct StackScratch {
    active: Vec<ActiveStrip>,
    /// Parallel to `active`.
    clocks: Vec<ClockIndex>,
}

/// Scratch is not identity — see [`ClockIndex`]'s own note.
impl PartialEq for StackScratch {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl StackScratch {
    /// Resolve the live strips at `t` and every remapped entity's clock inside
    /// each one. Call once per frame, after the liveness pass.
    ///
    /// With an empty stack this leaves exactly one entry: the active clip, read
    /// at the playhead. That is not a special case bolted on — it is what "no
    /// stack" *means*, and it keeps the single-clip path on the same code.
    pub(crate) fn rebuild(&mut self, doc: &TimelineDoc, t: f64) {
        self.active.clear();
        if doc.stack().is_empty() {
            self.active.push(ActiveStrip {
                lane: 0,
                clip: doc.active_index(),
                w: 1.0,
                t_clip: t,
                src_in: 0.0,
            });
        } else {
            for (li, lane) in doc.stack().iter().enumerate() {
                if lane.muted {
                    continue; // muting REMOVES the lane; it is not a zero weight
                }
                for (si, strip) in lane.strips.iter().enumerate() {
                    let w = lane.weight_at(si, t);
                    if w <= 0.0 {
                        continue;
                    }
                    let (Some(t_clip), true) = (
                        strip.source_time(t),
                        (strip.clip as usize) < doc.clips().len(),
                    ) else {
                        continue; // outside the strip, or a clip that was deleted
                    };
                    self.active.push(ActiveStrip {
                        lane: li,
                        clip: strip.clip as usize,
                        w,
                        t_clip,
                        src_in: strip.src_in,
                    });
                }
            }
        }

        // Grow (never shrink) the clock pool, then refill it in place.
        while self.clocks.len() < self.active.len() {
            self.clocks.push(ClockIndex::default());
        }
        for (i, a) in self.active.iter().enumerate() {
            let clip = &doc.clips()[a.clip].clip;
            self.clocks[i].rebuild(doc, clip, a.t_clip);
        }
    }

    /// The one live strip of the empty-stack path (the active clip at the
    /// playhead), and the entity clock resolved inside it.
    pub(crate) fn solo_clock(&self) -> &ClockIndex {
        &self.clocks[0]
    }
}

/// The value `target` should hold at this instant, blended out of the whole
/// stack — or `None` when **no lane keys this channel**, in which case nothing is
/// written and the scene's own value stands (R2).
///
/// `rest` is the channel's captured base (R5): the value the property held when
/// it was first animated. It is what a lane fades in *from* when nothing is under
/// it. Without it, "the object sits where I put it and the animation eases in"
/// cannot be expressed — and blending toward a *type* default instead (0 for a
/// position) would fling the sprite to its parent's origin. Rive shipped without
/// this and had to add Capture Base State; Unreal calls it the Base Pose.
pub(crate) fn sample_stack(
    doc: &TimelineDoc,
    scratch: &StackScratch,
    entity: u64,
    target: AnimTarget,
    prop: PropKind,
    rest: f32,
) -> Option<f32> {
    let algebra = prop.algebra();
    let mut acc = f64::from(rest);
    let mut touched = false;

    for (li, lane) in doc.stack().iter().enumerate() {
        if lane.muted {
            continue;
        }
        let additive = lane.mode == LaneMode::Additive;

        // ── inside the lane: normalize (see the module docs) ──
        let (mut num, mut den) = (0.0_f64, 0.0_f64);
        for (i, a) in scratch.active.iter().enumerate() {
            if a.lane != li {
                continue;
            }
            let Some(track) = doc.clips()[a.clip].clip.track(target) else {
                continue; // this clip does not key this channel: sparsity (R2)
            };
            if track.is_empty() {
                continue;
            }
            let t_src = scratch.clocks[i].get(entity, a.t_clip);
            let Some(v) = as_f64(track.sample(t_src)) else {
                continue;
            };
            let x = if additive {
                contribution(v, reference(track, a.src_in), algebra)
            } else {
                v
            };
            num += a.w * x;
            den += a.w;
        }
        if den <= 0.0 {
            continue; // the lane is silent on this channel: it passes through
        }

        // ── across lanes: coverage becomes influence ──
        let x_lane = num / den;
        let influence = den.min(1.0) * lane.weight.clamp(0.0, 1.0);
        acc = match (additive, algebra) {
            (false, _) => lerp(acc, x_lane, influence),
            (true, Algebra::Sum) => acc + x_lane * influence,
            // `lerp(1, ratio, influence)` and not `ratio.powf(influence)`: the
            // pow is the textbook weighting, and it is a transcendental (HR-5
            // bans them — they are not bit-reproducible across platforms). The
            // lerp is what Blender's own MULTIPLY blend uses, and at influence 1
            // (the case that matters) the two agree exactly.
            (true, Algebra::Ratio) => acc * lerp(1.0, x_lane, influence),
        };
        touched = true;
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "the scene value is f32; f64 is the blend's working precision"
    )]
    touched.then_some(acc as f32)
}

/// The entity's clock for the solo (no-stack) path — the active clip at `t`.
pub(crate) fn solo_source_time(scratch: &StackScratch, entity: u64, t: f64) -> f64 {
    scratch.solo_clock().get(entity, t)
}

/// What an additive strip measures its delta against: its clip's value at the
/// **first frame of the slice it uses**. Slice the clip elsewhere and the
/// reference moves with it — otherwise a trimmed strip would jump on entry.
///
/// Read in **clip time**, deliberately NOT through the entity's Time Remap: this
/// is a reference pose, and a reference that moves with a clock is not a
/// reference. (With the identity remap — every document that never touches the
/// feature — the two are the same time anyway.)
fn reference(track: &Track, src_in: f64) -> f64 {
    as_f64(track.sample(src_in)).unwrap_or(0.0)
}

/// An additive strip's contribution: how far it moved from its own first frame.
///
/// A clip holding a **constant pose contributes nothing** — that is the whole
/// point of an additive lane, and it is the test that catches "I summed the
/// absolute value".
fn contribution(v: f64, base: f64, algebra: Algebra) -> f64 {
    match algebra {
        Algebra::Sum => v - base,
        // A zero reference has no ratio. Rather than divide by it, contribute
        // nothing: neutral is 1.
        Algebra::Ratio if base.abs() > f64::EPSILON => v / base,
        Algebra::Ratio => 1.0,
    }
}

fn as_f64(v: AnimValue) -> Option<f64> {
    match v {
        AnimValue::Float(f) => Some(f64::from(f)),
        _ => None,
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
