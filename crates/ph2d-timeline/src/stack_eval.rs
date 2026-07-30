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
use ph2d_expr::Expr;

use crate::doc::TimelineDoc;
use crate::frame_solve::{LinkFrame, eval_expr};
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

/// What a clip contributes to a channel this frame: its keyed track, or a per-clip
/// EXPRESSION over it (ADR-0146 W1). Deciding the source ONCE is load-bearing — the value
/// path and the additive-reference path must read the SAME answer, or a per-clip expr that
/// faded the value but not the reference would leak on an additive lane.
enum AnimSource<'a> {
    /// A non-empty keyed track — sampled directly (the pre-expression behaviour, and the
    /// only source a formula-free document ever produces, so the fade stays byte-stable).
    Track(&'a Track),
    /// A per-clip formula whose `value` is `value_track`'s sample (or rest when the channel
    /// has no keys — a PURE expression that still covers the channel, §4). The `Expr` is
    /// owned (parsed this frame); caching was measured and rejected (`expr_pass.rs`).
    Expr {
        ir: Expr,
        value_track: Option<&'a Track>,
    },
}

/// Resolve clip `clip`'s source for channel `target`: a per-clip expression if the clip
/// carries one and it parses, else its non-empty keyed track, else `None` (sparsity — the
/// clip does not touch this channel). A parse error falls back to the raw track (the
/// property keeps its keyed value, the same fallback the retired post-pass gave).
fn clip_anim_source(doc: &TimelineDoc, clip: usize, target: AnimTarget) -> Option<AnimSource<'_>> {
    let named = doc.clips().get(clip)?;
    let track = named.clip.track(target).filter(|tr| !tr.is_empty());
    // ⚠️ **A channel being PREVIEWED goes quiet here**, and this is the one line that
    // keeps the preview from doubling the artist's own work: the Expression card seeds
    // itself from the formula this channel already carries (`TrackView::expr`, which
    // reads exactly this map), so the sheet ALREADY contains it. Leave it composing and
    // the live pass would apply it to a `value` that already includes it — the artist
    // would be tuning `f(f(x))` and every number would read double.
    //
    // It goes at THIS door because both sample sites resolve through it (see the module
    // docs), so the suppression cannot apply on the stacked path and miss the solo one.
    let previewed = crate::expr_live::live_expr().is_some_and(|l| l.target == target.get());
    if !previewed
        && let Some(src) = named.expr.get(&target)
        && let Ok(ir) = ph2d_expr_parse::parse(src)
    {
        return Some(AnimSource::Expr {
            ir,
            value_track: track,
        });
    }
    track.map(AnimSource::Track)
}

/// This channel's wiggle seed — DELEGATED to the one door, so `wiggle` phases identically
/// wherever it is asked (the blend, the post-pass, the card's ribbon, the census).
fn seed_of(target: AnimTarget) -> f64 {
    f64::from(crate::frame_solve::seed_of_target(target.get()))
}

/// **The SECOND sample site (ADR-0146 W2, C1).** The value the ACTIVE clip `clip`
/// contributes to `target` at its own clip time `t`, on the NON-STACKED path — a keyed
/// sample or a per-clip EXPRESSION over it. `None` when the clip neither keys nor drives
/// the channel (sparsity), so a just-bound property is never forced to a default.
///
/// The blend's [`eval_frame`] resolves this for a STACKED document; a document with no
/// strips (the common keyed animation) never reaches the blend, so its per-clip expression
/// would go undriven in silence without this door. Both sites resolve through the SAME
/// [`clip_anim_source`], so a per-clip expr drives identically whether or not a stack sits
/// over it.
pub(crate) fn solo_source_value(
    doc: &TimelineDoc,
    clip: usize,
    target: AnimTarget,
    t: f64,
    rest: f32,
    links: &LinkFrame,
) -> Option<f32> {
    let v: f64 = match clip_anim_source(doc, clip, target)? {
        AnimSource::Track(tr) => as_f64(tr.sample(t))?,
        AnimSource::Expr { ir, value_track } => {
            // `value` is the keyed sample of this clip (or rest for a pure expression that
            // still covers the channel, §4), exactly as the stacked path's Expr arm.
            let value = value_track
                .and_then(|tr| as_f64(tr.sample(t)))
                .unwrap_or_else(|| f64::from(rest));
            f64::from(eval_expr(&ir, value, t, seed_of(target), links))
        }
    };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the scene value is f32; f64 is the blend's working precision"
    )]
    Some(v as f32)
}

/// The value `q.target` should hold at this instant, blended out of the whole
/// stack — or `None` when **no lane keys this channel**, in which case nothing is
/// written and the scene's own value stands (R2).
pub(crate) fn sample_stack(
    doc: &TimelineDoc,
    scratch: &StackScratch,
    q: Query,
    links: &LinkFrame,
) -> Option<f32> {
    sample_stack_probed(doc, scratch, q, None, links)
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
    links: &LinkFrame,
) -> Option<f32> {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the scene value is f32; f64 is the blend's working precision"
    )]
    eval_frame(doc, scratch, 0, q, probe, links).map(|v| v as f32)
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
    links: &LinkFrame,
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
                    match eval_frame(doc, scratch, child, q, probe, links) {
                        Some(v) => v,
                        None => continue,
                    }
                }
                ActiveSource::Clip(clip) => {
                    let probed = probe.filter(|p| p.clip == clip);
                    // The clip's source for this channel: its keyed track, or a per-clip
                    // EXPRESSION over it (ADR-0146 W1). `.get`, not `[..]`: the index is
                    // cached in the scratch, and a scratch that outlived a DeleteClip would
                    // panic rather than go quiet.
                    let source = clip_anim_source(doc, clip, target);
                    // Sparsity (R2): a clip that does not key this channel contributes
                    // nothing — unless it is the probed one, whose whole purpose is to
                    // answer "what if it did".
                    match (probed, &source) {
                        // Keying an EXPRESSION-driven clip substitutes the probe into the
                        // formula's `value` INPUT and lets `E(value)` enter the blend (ADR-0146
                        // W5), so `invert_stack` solves for the STORED value: `value + g(time)`
                        // keys and pre-compensates; `wiggle`/`value*value` refuse (A~0 / the
                        // 3rd probe). `t_src` is this strip's own source time — the same clock
                        // the un-probed Expr arm below reads.
                        (Some(p), Some(AnimSource::Expr { ir, .. })) => {
                            let t_src = scratch.clocks[i].get(entity, a.t_clip);
                            f64::from(eval_expr(ir, p.value, t_src, seed_of(target), links))
                        }
                        // Keying a keyed/track clip (or one with no track yet) forces the value
                        // directly — the invert path measures the raw stack response.
                        (Some(p), _) => p.value,
                        (None, Some(AnimSource::Track(tr))) => {
                            let t_src = scratch.clocks[i].get(entity, a.t_clip);
                            let Some(v) = as_f64(tr.sample(t_src)) else {
                                continue;
                            };
                            v
                        }
                        (None, Some(AnimSource::Expr { ir, value_track })) => {
                            // The strip contributes E(t_src), with `value` = this strip's
                            // keyed sample (or rest). This flows through the SAME num/den
                            // normalization, crossfade, and additive delta as a track — so
                            // the expression FADES, crosses, and sums for free.
                            let t_src = scratch.clocks[i].get(entity, a.t_clip);
                            let value = value_track
                                .and_then(|tr| as_f64(tr.sample(t_src)))
                                .unwrap_or_else(|| f64::from(rest));
                            f64::from(eval_expr(ir, value, t_src, seed_of(target), links))
                        }
                        (None, None) => continue,
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
                    // the thing that is actually playing. The frame is clamped to the
                    // interior's first voice at resolve time (`stack_frames::resolve`),
                    // mirroring a track's clamp before its first key.
                    //
                    // A channel the clamped frame STILL cannot answer (a lane deeper in
                    // the interior that starts later) measures against `rest` — which is
                    // what soloing the container shows there: nothing written, the scene's
                    // captured base standing. Falling back to `v` was the bug this comment
                    // replaces: `v - v` is an exact zero, so the whole strip went silently
                    // inert (Enio, 2026-07-23).
                    ActiveSource::Container { reference, .. } => reference
                        .and_then(|r| eval_frame(doc, scratch, r, q, probe, links))
                        .unwrap_or_else(|| f64::from(rest)),
                    ActiveSource::Clip(clip) => {
                        let probed = probe.filter(|p| p.clip == clip);
                        // The SAME source the value path resolved — a per-clip expr must fade
                        // its reference exactly as it fades its value (ADR-0146).
                        let source = clip_anim_source(doc, clip, target);
                        match (probed, &source) {
                            (Some(p), Some(AnimSource::Track(tr))) => {
                                reference_after(tr, a.src_in, p)
                            }
                            // W5: the reference of an expression-driven additive strip under a
                            // probe is `E` at the strip's FIRST frame, `value` = the (post-key)
                            // keyed reference there — or the probe value when it has no track.
                            // Affine in `p.value` iff `E` is affine, the guarantee the solve
                            // needs; a non-affine one is refused by the third probe.
                            (Some(p), Some(AnimSource::Expr { ir, value_track })) => {
                                let ref_val = value_track
                                    .map_or(p.value, |tr| reference_after(tr, a.src_in, p));
                                f64::from(eval_expr(ir, ref_val, a.src_in, seed_of(target), links))
                            }
                            (Some(p), _) => p.value, // no track / expr under probe: key IS the curve
                            (None, Some(AnimSource::Track(tr))) => reference(tr, a.src_in),
                            (None, Some(AnimSource::Expr { ir, value_track })) => {
                                // The reference is E at the strip's FIRST frame, `value` = the
                                // keyed reference there (or rest). A CONSTANT expression then
                                // contributes 0 (Sum) / 1 (Ratio) — the additive invariant,
                                // exactly as a constant track does.
                                let ref_val = value_track
                                    .map(|tr| reference(tr, a.src_in))
                                    .unwrap_or_else(|| f64::from(rest));
                                f64::from(eval_expr(ir, ref_val, a.src_in, seed_of(target), links))
                            }
                            (None, None) => v, // unreachable: the value path continued on None
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
    links: &LinkFrame,
) -> Result<f32, KeyRefusal> {
    let at = |value: f64| {
        sample_stack_probed(doc, scratch, q, Some(Probe { clip, value, t_key }), links)
            .map(f64::from)
    };
    match solve_affine(at, f64::from(want)) {
        Ok(v) => Ok(cast_f32(v)),
        // A probe FORCES the clip to contribute, so `NoValue` here means the clip is in no
        // live lane at all (degenerate) — read as no influence.
        Err(AffineFail::NoValue) => Err(KeyRefusal::Overridden),
        // `A ~ 0` or a non-affine composition: the fix is the FORMULA if one drives this
        // channel (`ExpressionDriven`, ADR-0146 W5), else the lane stack (`Overridden`).
        Err(AffineFail::NoInfluence | AffineFail::NonAffine) => {
            Err(refusal_for(doc, clip, q.target))
        }
    }
}

/// **Invert the ACTIVE clip's affine expression on the NON-STACKED path (C3, ADR-0146 W5).**
/// Without a stack the scene value IS the expression `E(stored, t)` (`solo_source_value`), so
/// keying `want` stores the `v` with `E(v, t) == want` — `value + g(time)` pre-compensates by
/// `want - g(t)`. `None` when the active clip does not DRIVE `target` by an expression (the
/// caller then stores `want` verbatim, the track being the scene); `Err(ExpressionDriven)` when
/// the formula is pure or non-linear. Uses the SAME [`solve_affine`] as the stacked path, so
/// the two fail and pre-compensate by one rule.
pub(crate) fn invert_active_clip_expr(
    doc: &TimelineDoc,
    target: AnimTarget,
    t: f64,
    want: f32,
    links: &LinkFrame,
) -> Option<Result<f32, KeyRefusal>> {
    let AnimSource::Expr { ir, .. } = clip_anim_source(doc, doc.active_index(), target)? else {
        return None; // a keyed/track channel keys verbatim — the caller's early `Ok(want)`
    };
    let e = |v: f64| Some(f64::from(eval_expr(&ir, v, t, seed_of(target), links)));
    Some(match solve_affine(e, f64::from(want)) {
        Ok(v) => Ok(cast_f32(v)),
        Err(_) => Err(KeyRefusal::ExpressionDriven),
    })
}

/// The refusal reason when the affine solve is degenerate / non-affine: `ExpressionDriven` if a
/// FORMULA drives this channel (the fix is the formula, ADR-0146 W5), else `Overridden` (the
/// fix is the lane stack).
fn refusal_for(doc: &TimelineDoc, clip: usize, target: AnimTarget) -> KeyRefusal {
    if matches!(
        clip_anim_source(doc, clip, target),
        Some(AnimSource::Expr { .. })
    ) {
        KeyRefusal::ExpressionDriven
    } else {
        KeyRefusal::Overridden
    }
}

/// **Invert an affine-in-`v` sampler by THREE probes** — `B = f(0)`, `A = f(1) - B`, and a
/// third `f(0.5)` that VERIFIES the line (two points pin a line through ANY two samples; they
/// cannot tell you the function between them was one). Returns the `v` with `f(v) == want`.
///
/// The one solve, shared by the stacked keying ([`invert_stack`]) and the non-stacked
/// expression keying ([`invert_active_clip_expr`], C3), so both refuse and pre-compensate by
/// the same rule and there is no second copy to drift ([[feedback_two_doors_to_the_same_question_diverge]]).
fn solve_affine(mut f: impl FnMut(f64) -> Option<f64>, want: f64) -> Result<f64, AffineFail> {
    let b = f(0.0).ok_or(AffineFail::NoValue)?;
    let one = f(1.0).ok_or(AffineFail::NoValue)?;
    let a = one - b;
    // Not "a != 0": a coefficient this small is a lever too long to pull — the key would be
    // astronomical and the next frame's rounding would move the object.
    if a.abs() < 1e-6 {
        return Err(AffineFail::NoInfluence);
    }
    // **Verify the affinity; do not trust it.** A third probe refuses every case where the
    // function between the two endpoints was not a line: the same clip on an Override lane and
    // a Ratio lane at once (quadratic in `v`), a Ratio lane whose reference the key moves, or a
    // non-linear formula (`value*value`). Each would otherwise hand back a confident, wrong
    // number and put the object somewhere nobody asked for.
    let half = f(0.5).ok_or(AffineFail::NoValue)?;
    let scale = 1.0 + b.abs() + one.abs();
    if (half - (0.5 * a + b)).abs() > AFFINE_TOL * scale {
        return Err(AffineFail::NonAffine);
    }
    Ok((want - b) / a)
}

/// Why [`solve_affine`] could not name a stored value.
enum AffineFail {
    /// A probe produced no value: no lane keys the channel even under the probe.
    NoValue,
    /// `A ~ 0`: the output does not depend on the stored value (a full Override above, or a
    /// value-independent formula like `wiggle`).
    NoInfluence,
    /// The third probe strayed off the line: the composition is non-affine in the stored value
    /// (Override + Ratio at once, or a non-linear formula like `value*value`).
    NonAffine,
}

/// The scene value is f32; f64 is the blend's working precision.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the scene value is f32; f64 is the blend's working precision"
)]
fn cast_f32(v: f64) -> f32 {
    v as f32
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
