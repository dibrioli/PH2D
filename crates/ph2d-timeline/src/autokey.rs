//! Auto-key decision (W4.T1/T2) — the pure core of "the user moved something,
//! so record a key at the playhead".
//!
//! The shell has no single choke point for UI-driven Transform edits: the gizmo,
//! the Inspector fields, the Inspector/Hierarchy "Reset", and the tint picker all
//! write a sprite's pose through different, mostly bespoke paths (enumerated in
//! the W4 handoff). Rather than hook each one — fragile, and blind to any path
//! added later — the shell samples each selected sprite's pose once per frame and
//! asks THIS function which properties to key. It observes the *result*, so it
//! cannot miss a path.
//!
//! The comparison is deliberately **against the animated value, not against last
//! frame**. A bound property is auto-keyed only when the world differs from what
//! the document already produces at the playhead — i.e. the user pushed it off
//! its own curve. That is what makes it immune to the feedback loop a
//! "changed-since-last-frame" test would create: an undo, a paste, or a
//! dope-sheet key move all change the document, the apply pass writes that back
//! to the world, and world-equals-curve means **no** spurious re-key.
//!
//! An **unbound** property has no curve to compare to, so first-touch auto-create
//! (W4.T2) falls back to last frame's value — safe precisely because nothing but
//! the UI ever writes an unbound property (the apply pass skips it), so there is
//! no feedback loop to guard against.

use ph2d_anim::AttributeEvaluator;

use crate::doc::TimelineDoc;
use crate::prop::PropKind;

/// One selected sprite's six animatable values this frame, in [`PropKind::ALL`]
/// order (`TranslationX, TranslationY, Rotation, ScaleX, ScaleY, Opacity`).
/// `None` where the sprite lacks the backing component (e.g. no `Sprite` for
/// opacity), which never keys.
pub type PoseSample = [Option<f32>; 6];

/// Which of a sprite's properties auto-key should write, given its live pose
/// (`world`), the pose it had last frame (`baseline`, for unbound first-touch),
/// and whether new tracks may be created this frame (`allow_create` — the shell
/// ties this to the timeline panel being open, so casual editing with the panel
/// closed never sprays tracks).
///
/// Returns `(prop, value)` pairs the caller upserts. Empty when nothing moved off
/// its curve — the common case, so a still scene keys nothing.
///
/// # `t_secs` is an `f64`, and that is the whole point
///
/// It must be **byte-identical to the time [`crate::apply_from_doc`] sampled** to
/// write the pose this function is about to judge. It took a `RationalTime`, and
/// `RationalTime` quantises to the microsecond ([`RationalTime::from_seconds`]) —
/// so the diff read the curve at a time the apply never used. A playhead that
/// advanced by `1/60 s`, or that a ruler drag parked anywhere, is off that grid by
/// a fraction of a microsecond; on a track moving 10 units/s that is ~3e-6 of
/// value, which is a DIFFERENT `f32`, and the comparison below is exact. So
/// scrubbing an animated object with auto-key armed minted a key every frame — the
/// pose WAS the curve, the diff just read it somewhere else (Enio, 2026-07-12).
///
/// It never showed in a test because a test scrubs to `0.5`, and `0.5` survives
/// the round-trip. [[feedback_derived_coordinate_seed_must_match_sample]]: whatever
/// writes a derived coordinate and whatever reads it must be the SAME function.
#[must_use]
pub fn autokey_props(
    doc: &TimelineDoc,
    entity: u64,
    t_secs: f64,
    world: &PoseSample,
    baseline: &PoseSample,
    allow_create: bool,
) -> AutokeyPlan {
    autokey_props_in(doc, entity, t_secs, world, baseline, allow_create, false)
}

/// **The Keys/solo view's auto-key** — the same decision, taken against the
/// SOLOED clip instead of the stack's blend.
///
/// The sibling of `key_authoring_solo` in the shell's K flow, and it exists for
/// the same bug (Enio, 2026-07-22: *"a partir do momento que eu crio uma strip
/// numa lane, não consigo mais criar keys com autokey"*): the moment ONE strip
/// exists, [`autokey_props`] starts reading the STACK — the blend at the
/// TIMELINE clock — while the Keys view drives the scene from the active clip
/// at the CLIP clock ([`crate::apply_active_clip`]). Diffing one against the
/// other saw a phantom move every frame and then asked the strip's map where to
/// key it, which refused (`NotPlaying`) whenever the scene's strips were
/// elsewhere — the toast wall.
///
/// In solo there is no blend: `t_secs` is the CLIP playhead's instant (the same
/// clock the solo apply sampled), the shown value is the clip's own curve, the
/// stored value is the pose itself, and there is **nothing to refuse** — soloing
/// the clip is precisely what guarantees "key it here" one answer.
#[must_use]
pub fn autokey_props_solo(
    doc: &TimelineDoc,
    entity: u64,
    t_secs: f64,
    world: &PoseSample,
    baseline: &PoseSample,
    allow_create: bool,
) -> AutokeyPlan {
    autokey_props_in(doc, entity, t_secs, world, baseline, allow_create, true)
}

/// The shared core — `solo` picks which scene the diff believes in: the stack's
/// blend (Arrange) or the active clip alone (Keys). One function, two named
/// doors, so the two views cannot drift in what "moved" means.
fn autokey_props_in(
    doc: &TimelineDoc,
    entity: u64,
    t_secs: f64,
    world: &PoseSample,
    baseline: &PoseSample,
    allow_create: bool,
    solo: bool,
) -> AutokeyPlan {
    let mut plan = AutokeyPlan::default();
    for (i, &prop) in PropKind::ALL.iter().enumerate() {
        let Some(v) = world[i] else { continue };

        // What the apply actually WROTE at this instant — the value the animator
        // is looking at. With no stack (or soloed) that is the active clip's
        // curve; under a stack it is the blend, and comparing against the active
        // clip's curve instead would see a difference every single frame and mint
        // a key every single frame. The diff must read what the eye reads.
        let shown = if solo {
            curve_value(doc, entity, prop, t_secs)
        } else {
            shown_value(doc, entity, prop, t_secs)
        };

        let moved = match shown {
            // Bound: key when the pose left the curve it is drawn on.
            Some(sampled) => v != sampled,
            // Unbound: first-touch create, only if the UI actually moved it since
            // last frame and creation is allowed here.
            None => allow_create && baseline[i].is_some_and(|b| b != v),
        };
        if !moved {
            continue;
        }

        if solo {
            // Soloed, the pose you see IS the clip's value (`key_authoring_solo`
            // stores the same way): no blend to invert, nothing to refuse.
            plan.keys.push((prop, v));
            continue;
        }
        // What must the ACTIVE clip's track hold so the scene lands on `v`? With
        // no stack, `v` itself. Under a stack, the inverse of the blend — and
        // sometimes there is no inverse (ADR-0115 R9), in which case we refuse.
        // Never write a key that moves the object.
        match key_value_in_active_clip(doc, entity, prop, v) {
            Some(stored) => plan.keys.push((prop, stored)),
            None => plan.refused.push(prop),
        }
    }
    plan
}

/// What auto-key decided: the keys to write, and the ones it **would not** write.
///
/// Refusals are a first-class result, not an empty list. Under a clip stack a
/// pose can be genuinely unreachable by keying the clip you are editing — an
/// `Override` lane at full weight above it owns the channel — and the only honest
/// answers are "refuse" or "silently move the object". Handing the caller a
/// refusal lets it say so.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AutokeyPlan {
    /// `(prop, value)` pairs the caller upserts into the active clip.
    pub keys: Vec<(PropKind, f32)>,
    /// Properties whose pose the active clip cannot express right now.
    pub refused: Vec<PropKind>,
}

impl AutokeyPlan {
    /// `true` when there is nothing to write and nothing to complain about.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty() && self.refused.is_empty()
    }
}

/// The value the scene is showing for `prop` right now — the same number the
/// apply pass wrote. `None` when nothing drives it (the unbound path).
fn shown_value(doc: &TimelineDoc, entity: u64, prop: PropKind, t_secs: f64) -> Option<f32> {
    if doc.stack().is_empty() {
        return curve_value(doc, entity, prop, t_secs);
    }
    let b = doc.binding_for(entity, prop)?;
    crate::stack_eval::sample_stack(
        doc,
        doc.scratch(),
        crate::stack_eval::Query {
            entity,
            target: b.target,
            prop,
            rest: b.rest.unwrap_or(0.0),
        },
    )
}

/// **The number to store in the active clip's track so the scene shows `want`.**
///
/// Without a stack this is the identity — the track IS the scene. With one, the
/// track is one voice in a blend, and the key must be pre-compensated for the
/// rest of it ([`crate::stack_eval::invert_stack`]). `None` = **refuse**: the pose
/// is not reachable by keying this clip (an `Override` lane at full weight above
/// owns the channel), and the alternatives are to refuse or to move the object
/// behind the animator's back.
///
/// Every path that authors a key goes through here — auto-key, performing, and
/// the manual `K` — because a value written by one rule and read by another is
/// the bug this module has now shipped three times.
#[must_use]
pub fn key_value_in_active_clip(
    doc: &TimelineDoc,
    entity: u64,
    prop: PropKind,
    want: f32,
) -> Option<f32> {
    if doc.stack().is_empty() {
        return Some(want);
    }
    let scratch = doc.scratch();
    // The clip being edited must be playing exactly once right now, or "key it
    // here" has no single answer (see `stack_eval::sole_strip_of`).
    let strip = crate::stack_eval::sole_strip_of(scratch, doc.active_index()).ok()?;
    // WHERE the key lands, in the clip's own time. The probe needs it: on an
    // additive lane a key can move the reference its own delta is measured against,
    // and a probe that only substituted a value could not see that.
    let t_key = crate::stack_eval::strip_source_time(scratch, &strip, entity)?;
    // An unbound property has no target anywhere yet, so no clip can key it — a
    // target no clip holds is exactly the right answer, and the probe supplies the
    // active clip's contribution regardless. Targets are allocated up from 0, so
    // this one is unreachable by construction.
    let target = doc
        .binding_for(entity, prop)
        .map_or(ph2d_anim::AnimTarget::new(u64::MAX), |b| b.target);
    let rest = doc
        .binding_for(entity, prop)
        .and_then(|b| b.rest)
        .unwrap_or(want);
    crate::stack_eval::invert_stack(
        doc,
        scratch,
        crate::stack_eval::Query {
            entity,
            target,
            prop,
            rest,
        },
        doc.active_index(),
        t_key,
        want,
    )
}

/// The scalar `prop`'s track produces at `t_secs` for `entity`, or `None` when no
/// non-empty track backs it (so the caller takes the unbound path). An empty
/// track — a binding with no keys — counts as unbound: there is nothing to
/// sample against.
fn curve_value(doc: &TimelineDoc, entity: u64, prop: PropKind, t_secs: f64) -> Option<f32> {
    let target = doc.binding_for(entity, prop)?.target;
    let track = doc.active_clip().track(target)?;
    if track.is_empty() {
        return None;
    }
    // Every `PropKind` animates a scalar `Float`; a non-scalar would be a v1
    // impossibility, and treating it as unbound is the safe fallback.
    match track.sample(t_secs) {
        ph2d_anim::AnimValue::Float(v) => Some(v),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::TimelineState;
    use crate::{TimelineIntent as I, apply_intent};
    use ph2d_anim::RationalTime;
    use ph2d_anim::{AnimValue, Interp};
    use ph2d_core::Playhead;

    const E: u64 = 1;
    /// PoseSample index for each PropKind we probe.
    const TX: usize = 0;
    const ROT: usize = 2;

    fn s(t: f64) -> RationalTime {
        RationalTime::from_seconds(t)
    }

    /// A doc with a TranslationX track keyed 0→10 over 0..1 s.
    fn doc_with_tx_track() -> TimelineState {
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        for (t, v) in [(0.0, 0.0), (1.0, 10.0)] {
            apply_intent(
                &mut st,
                &mut ph,
                I::AddKey {
                    entity: E,
                    prop: PropKind::TranslationX,
                    t: s(t),
                    value: AnimValue::Float(v),
                    interp: Interp::Linear,
                },
            );
        }
        st
    }

    fn pose(vals: &[(usize, f32)]) -> PoseSample {
        let mut p: PoseSample = [None; 6];
        for &(i, v) in vals {
            p[i] = Some(v);
        }
        p
    }

    #[test]
    fn a_bound_prop_off_its_curve_is_keyed() {
        // At t = 0.5 the curve says x = 5. The world is at 7 (the user dragged it):
        // key it. The other props are None → never keyed.
        let st = doc_with_tx_track();
        let got = autokey_props(&st.doc, E, 0.5, &pose(&[(TX, 7.0)]), &pose(&[]), true);
        assert_eq!(got.keys, vec![(PropKind::TranslationX, 7.0)]);
    }

    #[test]
    fn a_bound_prop_sitting_on_its_curve_is_not_keyed() {
        // THE anti-feedback case: after an undo/paste/scrub the apply pass writes
        // the curve value to the world, so world == curve — auto-key must be silent
        // or it would re-key what the document just produced, fighting the undo.
        let st = doc_with_tx_track();
        let got = autokey_props(&st.doc, E, 0.5, &pose(&[(TX, 5.0)]), &pose(&[]), true);
        assert!(got.is_empty(), "on-curve poses key nothing: {got:?}");
    }

    #[test]
    fn an_unbound_prop_that_moved_since_last_frame_auto_creates() {
        // Rotation has no track. It moved from 0 (last frame) to 0.5 (now), and
        // creation is allowed → key it (the shell will upsert, which binds+creates).
        let st = doc_with_tx_track();
        let got = autokey_props(
            &st.doc,
            E,
            0.5,
            &pose(&[(ROT, 0.5)]),
            &pose(&[(ROT, 0.0)]),
            true,
        );
        assert_eq!(got.keys, vec![(PropKind::Rotation, 0.5)]);
    }

    #[test]
    fn an_unbound_prop_that_did_not_move_creates_nothing() {
        let st = doc_with_tx_track();
        let got = autokey_props(
            &st.doc,
            E,
            0.5,
            &pose(&[(ROT, 0.5)]),
            &pose(&[(ROT, 0.5)]), // same as last frame
            true,
        );
        assert!(
            got.is_empty(),
            "an unchanged unbound prop must not spray a track"
        );
    }

    #[test]
    fn an_unbound_prop_never_auto_creates_when_creation_is_off() {
        // Panel closed → the shell passes allow_create = false → casual editing
        // never sprays new tracks, however far the object moved.
        let st = doc_with_tx_track();
        let got = autokey_props(
            &st.doc,
            E,
            0.5,
            &pose(&[(ROT, 9.0)]),
            &pose(&[(ROT, 0.0)]),
            false,
        );
        assert!(got.is_empty());
        // But a BOUND prop still auto-keys with creation off — updating an
        // existing channel is always allowed.
        let got = autokey_props(&st.doc, E, 0.5, &pose(&[(TX, 7.0)]), &pose(&[]), false);
        assert_eq!(got.keys, vec![(PropKind::TranslationX, 7.0)]);
    }

    #[test]
    fn an_unbound_prop_with_no_baseline_yet_creates_nothing() {
        // First frame an entity is selected: no baseline → nothing to compare, so
        // its mere selection never mints a key.
        let st = doc_with_tx_track();
        let got = autokey_props(&st.doc, E, 0.5, &pose(&[(ROT, 9.0)]), &pose(&[]), true);
        assert!(got.is_empty());
    }

    #[test]
    fn an_empty_track_counts_as_unbound() {
        // A binding with no keys has no curve to compare to — treat it as unbound
        // so the first edit still creates a key rather than being lost.
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        apply_intent(
            &mut st,
            &mut ph,
            I::Bind {
                entity: E,
                prop: PropKind::TranslationX,
            },
        );
        let got = autokey_props(
            &st.doc,
            E,
            0.5,
            &pose(&[(TX, 3.0)]),
            &pose(&[(TX, 0.0)]),
            true,
        );
        assert_eq!(got.keys, vec![(PropKind::TranslationX, 3.0)]);
    }

    #[test]
    fn upserting_the_returned_props_leaves_the_pose_on_its_curve() {
        // End to end: key what autokey_props returns, and next frame the same pose
        // is ON the curve → nothing more to key. This is the loop that must close.
        let mut st = doc_with_tx_track();
        let t = s(0.5);
        let got = autokey_props(
            &st.doc,
            E,
            t.to_seconds(),
            &pose(&[(TX, 7.0)]),
            &pose(&[]),
            true,
        );
        for (prop, v) in got.keys {
            st.doc
                .upsert_key(E, prop, t, AnimValue::Float(v), Interp::Linear);
        }
        let again = autokey_props(
            &st.doc,
            E,
            t.to_seconds(),
            &pose(&[(TX, 7.0)]),
            &pose(&[]),
            true,
        );
        assert!(
            again.is_empty(),
            "the keyed pose is now on its own curve: {again:?}"
        );
    }
}
