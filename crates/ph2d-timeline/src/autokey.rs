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

use ph2d_anim::{AttributeEvaluator, RationalTime};

use crate::doc::TimelineDoc;
use crate::prop::PropKind;

/// One selected sprite's six animatable values this frame, in [`PropKind::ALL`]
/// order (`TranslationX, TranslationY, Rotation, ScaleX, ScaleY, Opacity`).
/// `None` where the sprite lacks the backing component (e.g. no `Sprite` for
/// opacity), which never keys.
pub type PoseSample = [Option<f32>; 6];

/// Which of a sprite's properties auto-key should write at `t`, given its live
/// pose (`world`), the pose it had last frame (`baseline`, for unbound
/// first-touch), and whether new tracks may be created this frame
/// (`allow_create` — the shell ties this to the timeline panel being open, so
/// casual editing with the panel closed never sprays tracks).
///
/// Returns `(prop, value)` pairs the caller upserts. Empty when nothing moved off
/// its curve — the common case, so a still scene keys nothing.
#[must_use]
pub fn autokey_props(
    doc: &TimelineDoc,
    entity: u64,
    t: RationalTime,
    world: &PoseSample,
    baseline: &PoseSample,
    allow_create: bool,
) -> Vec<(PropKind, f32)> {
    let t_secs = t.to_seconds();
    let mut out = Vec::new();
    for (i, &prop) in PropKind::ALL.iter().enumerate() {
        let Some(v) = world[i] else { continue };
        match curve_value(doc, entity, prop, t_secs) {
            // Bound: key when the pose left the curve it is drawn on.
            Some(sampled) => {
                if v != sampled {
                    out.push((prop, v));
                }
            }
            // Unbound: first-touch create, only if the UI actually moved it since
            // last frame and creation is allowed here.
            None => {
                if allow_create && baseline[i].is_some_and(|b| b != v) {
                    out.push((prop, v));
                }
            }
        }
    }
    out
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
        let got = autokey_props(&st.doc, E, s(0.5), &pose(&[(TX, 7.0)]), &pose(&[]), true);
        assert_eq!(got, vec![(PropKind::TranslationX, 7.0)]);
    }

    #[test]
    fn a_bound_prop_sitting_on_its_curve_is_not_keyed() {
        // THE anti-feedback case: after an undo/paste/scrub the apply pass writes
        // the curve value to the world, so world == curve — auto-key must be silent
        // or it would re-key what the document just produced, fighting the undo.
        let st = doc_with_tx_track();
        let got = autokey_props(&st.doc, E, s(0.5), &pose(&[(TX, 5.0)]), &pose(&[]), true);
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
            s(0.5),
            &pose(&[(ROT, 0.5)]),
            &pose(&[(ROT, 0.0)]),
            true,
        );
        assert_eq!(got, vec![(PropKind::Rotation, 0.5)]);
    }

    #[test]
    fn an_unbound_prop_that_did_not_move_creates_nothing() {
        let st = doc_with_tx_track();
        let got = autokey_props(
            &st.doc,
            E,
            s(0.5),
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
            s(0.5),
            &pose(&[(ROT, 9.0)]),
            &pose(&[(ROT, 0.0)]),
            false,
        );
        assert!(got.is_empty());
        // But a BOUND prop still auto-keys with creation off — updating an
        // existing channel is always allowed.
        let got = autokey_props(&st.doc, E, s(0.5), &pose(&[(TX, 7.0)]), &pose(&[]), false);
        assert_eq!(got, vec![(PropKind::TranslationX, 7.0)]);
    }

    #[test]
    fn an_unbound_prop_with_no_baseline_yet_creates_nothing() {
        // First frame an entity is selected: no baseline → nothing to compare, so
        // its mere selection never mints a key.
        let st = doc_with_tx_track();
        let got = autokey_props(&st.doc, E, s(0.5), &pose(&[(ROT, 9.0)]), &pose(&[]), true);
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
            s(0.5),
            &pose(&[(TX, 3.0)]),
            &pose(&[(TX, 0.0)]),
            true,
        );
        assert_eq!(got, vec![(PropKind::TranslationX, 3.0)]);
    }

    #[test]
    fn upserting_the_returned_props_leaves_the_pose_on_its_curve() {
        // End to end: key what autokey_props returns, and next frame the same pose
        // is ON the curve → nothing more to key. This is the loop that must close.
        let mut st = doc_with_tx_track();
        let t = s(0.5);
        let got = autokey_props(&st.doc, E, t, &pose(&[(TX, 7.0)]), &pose(&[]), true);
        for (prop, v) in got {
            st.doc
                .upsert_key(E, prop, t, AnimValue::Float(v), Interp::Linear);
        }
        let again = autokey_props(&st.doc, E, t, &pose(&[(TX, 7.0)]), &pose(&[]), true);
        assert!(
            again.is_empty(),
            "the keyed pose is now on its own curve: {again:?}"
        );
    }
}
