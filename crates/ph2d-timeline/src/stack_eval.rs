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

use ph2d_anim::{AnimTarget, AnimValue, AttributeEvaluator, Interp, RationalTime, Track};

use crate::doc::TimelineDoc;
use crate::prop::{Algebra, PropKind};
use crate::refusal::KeyRefusal;
use crate::stack::LaneMode;
use crate::stack_frames::{ActiveSource, ActiveStrip, StackScratch};

/// One channel of one entity — everything a stack query is *about*, as opposed to
/// the stack it is asked of. Four values that always travel together.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Query {
    /// Whose channel (ECS bits).
    pub entity: u64,
    /// Which track, in every clip that keys it.
    pub target: AnimTarget,
    /// Which property — it decides the blend algebra.
    pub prop: PropKind,
    /// The channel's captured base (R5): the value it held when it was first
    /// animated. It is what a lane fades in *from* when nothing is under it.
    /// Without it, "the object sits where I put it and the animation eases in"
    /// cannot be expressed — and blending toward a *type* default instead (0 for a
    /// position) would fling the sprite to its parent's origin. Rive shipped
    /// without this and had to add Capture Base State; Unreal calls it Base Pose.
    pub rest: f32,
}

/// The value `q.target` should hold at this instant, blended out of the whole
/// stack — or `None` when **no lane keys this channel**, in which case nothing is
/// written and the scene's own value stands (R2).
pub(crate) fn sample_stack(doc: &TimelineDoc, scratch: &StackScratch, q: Query) -> Option<f32> {
    sample_stack_probed(doc, scratch, q, None)
}

/// [`sample_stack`], but with one clip's track value **forced** to a probe.
///
/// The probe is how [`invert_stack`] measures the stack's response to the clip
/// you are keying — including a clip that has **no track yet** for this channel,
/// which is exactly the first-key case. A probed clip contributes whether or not
/// it has a track: that is what "if this clip held value v" means.
///
/// The probe is `(clip, value, key time)`. It carries the **time** because the
/// probe is a hypothetical *write*, and a write can move more than the value it
/// writes — see [`Probe`].
fn sample_stack_probed(
    doc: &TimelineDoc,
    scratch: &StackScratch,
    q: Query,
    probe: Option<Probe>,
) -> Option<f32> {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the scene value is f32; f64 is the blend's working precision"
    )]
    eval_frame(doc, scratch, 0, q, probe).map(|v| v as f32)
}

/// One frame's answer for `q.target`, blended out of ITS lanes — or `None` when no lane
/// of that frame keys the channel (R2, sparsity).
///
/// **The recursion is here and nowhere else.** A container strip's contribution is this
/// same function run on the frame its interior was resolved into: the interior normalizes
/// among its own lanes and hands up ONE value, exactly as a clip hands up one sample. That
/// is what makes a container reusable — its output does not depend on what sits under it in
/// the parent — and it is the precomp model (AE), not the flatten-into-the-parent model.
///
/// Depth is bounded by the container count, because the graph is acyclic by two gates.
fn eval_frame(
    doc: &TimelineDoc,
    scratch: &StackScratch,
    frame: usize,
    q: Query,
    probe: Option<Probe>,
) -> Option<f64> {
    let Query {
        entity,
        target,
        prop,
        rest,
    } = q;
    let algebra = prop.algebra();
    let mut acc = f64::from(rest);
    let mut touched = false;

    let info = *scratch.frames.get(frame)?;
    let lanes = match info.container {
        None => doc.stack(),
        Some(c) => doc.container_stack(c)?,
    };

    for (li, lane) in lanes.iter().enumerate() {
        if lane.muted {
            continue;
        }
        let additive = lane.mode == LaneMode::Additive;

        // ── inside the lane: normalize (see the module docs) ──
        let (mut num, mut den) = (0.0_f64, 0.0_f64);
        // **A lane pode FALAR deste canal e mesmo assim somar peso zero** — é o instante inicial
        // de um fade, ou a beirada de um fade-out. Silêncio (a lane não keya este canal) e peso
        // zero (ela keya, e agora não influi) NÃO são a mesma coisa, e confundi-los foi o bug:
        // ver o `den <= 0` lá embaixo.
        let mut speaks = false;
        // Only THIS frame's strips (see `StackFrame::first`). An interior must not blend
        // against its parent's siblings (ADR-0133 §1), and it must not pay to walk past them.
        let (first, count) = (info.first, info.count);
        for i in first..first + count {
            let a = scratch.active[i];
            if a.lane != li {
                continue;
            }
            let v = match a.source {
                ActiveSource::Container { frame: child, .. } => {
                    // The interior IS the value. A container that keys nothing here is
                    // silent, exactly like a clip with no track for this channel.
                    match eval_frame(doc, scratch, child, q, probe) {
                        Some(v) => v,
                        None => continue,
                    }
                }
                ActiveSource::Clip(clip) => {
                    let probed = probe.filter(|p| p.clip == clip);
                    // `.get`, not `[..]`: the index is cached in the scratch, and a scratch
                    // that outlived a DeleteClip would panic here rather than go quiet.
                    let track = doc.clips().get(clip).and_then(|c| c.clip.track(target));
                    // Sparsity (R2): a clip that does not key this channel contributes
                    // nothing — unless it is the probed one, whose whole purpose is to
                    // answer "what if it did".
                    match (probed, track) {
                        (Some(p), _) => p.value,
                        (None, Some(tr)) if !tr.is_empty() => {
                            let t_src = scratch.clocks[i].get(entity, a.t_clip);
                            let Some(v) = as_f64(tr.sample(t_src)) else {
                                continue;
                            };
                            v
                        }
                        _ => continue,
                    }
                }
            };
            let x = if additive {
                // **A write can move the reference it is measured against.** An
                // additive strip's reference is its clip's own value at `src_in`,
                // and the key we are about to insert may be the very key that
                // defines it (it is, whenever the animator poses at the strip's
                // first frame — which is where they start). Holding the reference
                // fixed made the solve report full influence where the truth is
                // none: the key was written, the delta came out zero, the pose was
                // thrown away, and every OTHER frame of that lane translated by the
                // value we had just invented. So the probe models the write.
                let base = match a.source {
                    // A container's reference is its interior read at the strip's FIRST
                    // frame — the same rule ("the clip's own value at `src_in`"), asked of
                    // the thing that is actually playing.
                    // The interior read at the strip's FIRST frame — the same rule a clip
                    // strip follows, asked of the thing that is actually playing.
                    ActiveSource::Container { reference, .. } => reference
                        .and_then(|r| eval_frame(doc, scratch, r, q, probe))
                        .unwrap_or(v),
                    ActiveSource::Clip(clip) => {
                        let probed = probe.filter(|p| p.clip == clip);
                        let track = doc.clips().get(clip).and_then(|c| c.clip.track(target));
                        match (probed, track) {
                            (Some(p), Some(tr)) => reference_after(tr, a.src_in, p),
                            (Some(p), None) => p.value, // no track: the key IS the curve
                            (None, Some(tr)) => reference(tr, a.src_in),
                            (None, None) => v,
                        }
                    }
                };
                contribution(v, base, algebra)
            } else {
                v
            };
            speaks = true; // há track para este canal — a lane TEM opinião, seja qual for o peso
            num += a.w * x;
            den += a.w;
        }
        if den <= 0.0 {
            // **Peso zero NÃO é silêncio.** Se a lane keya este canal mas os strips vivos somam
            // peso 0 (o primeiro instante de um fade-in, a última lasca de um fade-out), a
            // resposta dela é "influência 0" — ou seja, a pose de REPOUSO —, e não "não sei".
            //
            // Confundir os dois quebrava o invariante mais básico da animação: **a pose tem de
            // ser função do PLAYHEAD**. Lida como silêncio, ninguém escrevia, e o objeto segurava
            // o valor que o frame anterior tinha deixado — então o mesmo `t` dava poses
            // diferentes conforme o lado de onde você chegava nele (Enio, smoke: dois strips
            // encostados davam x=-3 vindo da esquerda e x=+3 vindo da direita, no mesmo t=3.0).
            //
            // `influence = 0` faz `acc` passar intacto em qualquer um dos três modos
            // (`lerp(acc, _, 0)` · `acc + _*0` · `acc * lerp(1, _, 0)`), então basta marcar que
            // esta lane se pronunciou: o `acc` que sai é o de repouso (ou o das lanes de baixo).
            //
            // Silêncio de verdade — a lane não keya este canal (esparsidade, R2) — continua
            // passando batido: é o que deixa o pose do artista em paz num canal que ninguém anima.
            touched |= speaks;
            continue;
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

    touched.then_some(acc)
}

/// A hypothetical **write**: "if clip `clip` held `value` at source time `t_key`".
///
/// It is not merely a value substitution, and that distinction is load-bearing. On
/// an additive lane the clip's own value at `src_in` is the reference the delta is
/// measured against — so a key can move the thing it is measured against, and a
/// probe that ignored the time could not see it.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Probe {
    /// Index of the clip being keyed.
    pub clip: usize,
    /// The value under test.
    pub value: f64,
    /// Where the key would land, in the clip's own time.
    pub t_key: f64,
}

/// The entity's clock for the solo (no-stack) path — the active clip at `t`.
pub(crate) fn solo_source_time(scratch: &StackScratch, entity: u64, t: f64) -> f64 {
    scratch.solo_clock().get(entity, t)
}

/// The one live strip playing `clip`, or `None` when there are **zero or several**.
///
/// Several is not a detail to paper over: if the clip you are editing is playing
/// twice at this instant, "key it here" has two answers, and picking one silently
/// would put the key somewhere the animator did not look. Blender hits the same
/// wall and says so — its keyframe remapping only works *"when the tweaked
/// strip's underlying action occurs once in the current frame"*.
pub(crate) fn sole_strip_of(
    scratch: &StackScratch,
    clip: usize,
) -> Result<ActiveStrip, KeyRefusal> {
    let mut found = None;
    for a in &scratch.active {
        // A HELD strip is over. It still shapes the pose (a fade crosses from it), but
        // "where is this clip playing right now" must not answer with a strip that has
        // ended — `K` would drop the key past the end of a strip nobody is watching.
        // **Every frame, not just the document's** (ADR-0133): under nesting a clip can
        // be playing inside a container — and twice, if the container itself plays twice.
        // "Where is this clip right now" has to look everywhere it could be, or `K` would
        // land a key in one of two places without saying there were two.
        if a.source.clip() != Some(clip) || a.held {
            continue;
        }
        if found.is_some() {
            return Err(KeyRefusal::PlaysTwice); // "here" names two places in it
        }
        found = Some(*a);
    }
    found.ok_or(KeyRefusal::NotPlaying)
}

/// The one live strip playing CONTAINER `c`, or `None` when there are zero or several.
///
/// The exact sibling of [`sole_strip_of`], and refusing for the exact same reason: a container
/// playing twice makes "where am I inside it" name two places, and a ruler that picked one
/// would draw the playhead at a second the animator is not at.
pub(crate) fn sole_container_strip_of(
    scratch: &StackScratch,
    c: usize,
) -> Result<ActiveStrip, KeyRefusal> {
    let mut found = None;
    for a in &scratch.active {
        let ActiveSource::Container { frame, .. } = a.source else {
            continue;
        };
        if a.held || scratch.frames.get(frame).and_then(|f| f.container) != Some(c) {
            continue;
        }
        if found.is_some() {
            return Err(KeyRefusal::PlaysTwice);
        }
        found = Some(*a);
    }
    found.ok_or(KeyRefusal::NotPlaying)
}

/// The clip time a strip is reading, run through that clip's own Time Remap for
/// this entity — the full outer-then-inner composition (R6).
pub(crate) fn strip_source_time(
    scratch: &StackScratch,
    strip: &ActiveStrip,
    entity: u64,
) -> Option<f64> {
    let i = scratch.active.iter().position(|a| {
        a.frame == strip.frame
            && a.lane == strip.lane
            && a.source == strip.source
            && a.t_clip.to_bits() == strip.t_clip.to_bits()
    })?;
    Some(scratch.clocks[i].get(entity, strip.t_clip))
}

/// What value `clip`'s track must hold so the whole stack lands on `want` — or
/// `None` when the clip **cannot reach it**, in which case the key is REFUSED.
///
/// # The stack is affine in the value you are keying, and that is the whole trick
///
/// Every operation the stack performs is affine in one clip's contribution:
/// `Override` is a `lerp`, additive `Sum` is an addition, and even additive
/// `Ratio` is `acc * (1 + inf*(v/base - 1))` — affine in `v`, because the
/// reference `base` is a fixed first-frame value, not a function of `v`.
/// Composing affine maps gives an affine map. So the stack, as a function of the
/// probed clip's value, is exactly `out(v) = A*v + B`, and **two evaluations
/// pin it down**: `B = out(0)`, `A = out(1) - B`. Then `v = (want - B) / A`.
///
/// Exact, not iterative — **where the stack really is affine**, which is not
/// everywhere. Put the same clip on an `Override` lane and a `Ratio` lane at once
/// and the composition is quadratic in `v`; let an additive reference move with the
/// key and a `Ratio` lane turns rational. So the solve does not assume: a third
/// probe checks the line it just drew, and a stack that fails the check is refused
/// (`AFFINE_TOL`). Two points pin a line through any two samples; they cannot tell
/// you what happened between them.
///
/// `A == 0` means the clip has no influence on what you are looking at — a full
/// `Override` lane above it, or an additive lane whose reference cancels it. The
/// pose you see is then simply not reachable by keying this clip, and the honest
/// move is to **refuse and say so**, never to write a key that moves the object.
/// (Blender's new layered system reaches the same conclusion: *"Blender will
/// simply reject keying and issue an error."*)
pub(crate) fn invert_stack(
    doc: &TimelineDoc,
    scratch: &StackScratch,
    q: Query,
    clip: usize,
    t_key: f64,
    want: f32,
) -> Option<f32> {
    let at = |value: f64| sample_stack_probed(doc, scratch, q, Some(Probe { clip, value, t_key }));
    let (b, one) = (f64::from(at(0.0)?), f64::from(at(1.0)?));
    let a = one - b;
    // Not "a != 0": a coefficient this small is a lever too long to pull — the key
    // would be astronomatical and the next frame's rounding would move the object.
    if a.abs() < 1e-6 {
        return None;
    }
    // **Verify the affinity; do not trust it.** Two points pin a line through ANY
    // two samples — they cannot tell you the function between them was a line. A
    // third probe costs one evaluation and refuses every case where it was not:
    // the same clip on an Override lane and a Ratio lane at once (the composition
    // is quadratic in `v`), or a Ratio lane whose reference the key itself moves.
    // Each of those would otherwise hand back a confident, wrong number and put the
    // object somewhere nobody asked for. A stack with no single answer has no
    // answer, and R9 says which way to fail.
    let half = f64::from(at(0.5)?);
    let scale = 1.0 + b.abs() + one.abs();
    if (half - (0.5 * a + b)).abs() > AFFINE_TOL * scale {
        return None;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the scene value is f32; f64 is the blend's working precision"
    )]
    Some(((f64::from(want) - b) / a) as f32)
}

/// How far a three-point probe may stray from the line its two endpoints define
/// before the stack is declared non-affine. Relative to the values in play, and a
/// few orders above `f32`'s round-trip noise (the tracks store `f32`).
const AFFINE_TOL: f64 = 1e-5;

/// The additive reference **after** the probed key is written.
///
/// Clones the track and inserts the key. That is not free — but it happens only on
/// the authoring path (auto-key / K), only under a stack, and only for the clip
/// being keyed. The alternative, modelling the insertion by hand, would be a second
/// implementation of interpolation that has to agree with the first forever:
/// [[feedback_derived_coordinate_seed_must_match_sample]] is the memory of what
/// that costs. Ask the real curve.
fn reference_after(tr: &Track, src_in: f64, p: Probe) -> f64 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the track stores f32; f64 is the probe's working precision"
    )]
    let v = p.value as f32;
    let mut hypothetical = tr.clone();
    hypothetical.upsert_key(
        RationalTime::from_seconds(p.t_key),
        AnimValue::Float(v),
        Interp::Linear,
    );
    as_f64(hypothetical.sample(src_in)).unwrap_or(0.0)
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
