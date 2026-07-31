//! **O valor que uma KEY precisa guardar** — a rota INVERSA do [`super`].
//!
//! Compor um frame é ir para a frente: as lanes entram, um valor sai. Keyar sob uma pilha é
//! a pergunta oposta — *que número este clip tem de guardar para que a COMPOSIÇÃO mostre a
//! pose que o artista acabou de fazer?* — e ela se resolve sondando a resposta do stack a
//! um valor hipotético ([`super::Probe`]) e invertendo a relação afim.
//!
//! Mora ao lado por isso, e porque alcança o `sample_stack_probed` privado do irmão.

use ph2d_anim::AnimTarget;

use super::{
    AFFINE_TOL, AnimSource, Probe, Query, cast_f32, clip_anim_source, sample_stack_probed, seed_of,
};
use crate::doc::TimelineDoc;
use crate::frame_solve::LinkFrame;
use crate::frame_solve::eval_expr;
use crate::refusal::KeyRefusal;
use crate::stack_frames::StackScratch;

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
        // channel (`ExpressionDriven`, ADR-0152 W5), else the lane stack (`Overridden`).
        Err(AffineFail::NoInfluence | AffineFail::NonAffine) => {
            Err(refusal_for(doc, clip, q.target))
        }
    }
}

/// **Invert the ACTIVE clip's affine expression on the NON-STACKED path (C3, ADR-0152 W5).**
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
/// FORMULA drives this channel (the fix is the formula, ADR-0152 W5), else `Overridden` (the
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
