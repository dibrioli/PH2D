//! **Reversing a track is a mirror, not a re-typing of key times.**
//!
//! The oracle here is the property, not the bookkeeping: the reversed curve sampled at
//! `span − t` must equal the original at `t`, everywhere — which is what "plays backwards"
//! actually means. A gate that only checked "the first key is now last" would stay green
//! while every ease-out silently became an ease-in
//! ([[reference_topic_oracle_discipline]]).

use ph2d_anim::{
    AnimValue, AttributeEvaluator, Easing, EasingFamily, EasingMode, Interp, RationalTime, Track,
};

fn f(v: AnimValue) -> f64 {
    match v {
        AnimValue::Float(x) => f64::from(x),
        other => panic!("not a scalar: {other:?}"),
    }
}

fn track(keys: &[(f64, f32, Interp)]) -> Track {
    let mut t = Track::new(Vec::new());
    for &(at, v, i) in keys {
        t.upsert_key(RationalTime::from_seconds(at), AnimValue::Float(v), i);
    }
    t
}

/// The whole point, as a property: sample both and compare across the span.
fn assert_mirrors(keys: &[(f64, f32, Interp)], span: f64) {
    let original = track(keys);
    let mut reversed = track(keys);
    reversed.reverse_about(span);
    for i in 0..=200 {
        let t = span * f64::from(i) / 200.0;
        let want = f(original.sample(t));
        let got = f(reversed.sample(span - t));
        assert!(
            (want - got).abs() < 1e-6,
            "t={t}: original {want} vs reversed@{} {got}",
            span - t
        );
    }
}

#[test]
fn a_reversed_linear_track_is_the_mirror_of_the_original() {
    assert_mirrors(
        &[
            (0.0, -3.0, Interp::Linear),
            (1.0, 2.0, Interp::Linear),
            (3.0, 3.0, Interp::Linear),
        ],
        3.0,
    );
}

/// **The half that a time-only mirror gets wrong.** An ease-out played backwards is an
/// ease-in; leave the interps where they sit and the values run backwards while every
/// acceleration stays forwards.
#[test]
fn a_reversed_eased_track_is_the_mirror_of_the_original() {
    assert_mirrors(
        &[
            (
                0.0,
                0.0,
                Interp::Eased(Easing {
                    family: EasingFamily::Cubic,
                    mode: EasingMode::Out,
                }),
            ),
            (2.0, 10.0, Interp::Linear),
        ],
        2.0,
    );
}

#[test]
fn a_reversed_bezier_track_is_the_mirror_of_the_original() {
    assert_mirrors(
        &[
            (0.0, 0.0, Interp::bezier(0.1, 0.9, 0.2, 1.0)),
            (2.0, 5.0, Interp::Linear),
        ],
        2.0,
    );
}

/// Weighted tangents carry ABSOLUTE value offsets from their anchors, and reversal swaps
/// the anchors — so the offsets travel with the handles instead of being negated. Includes
/// a FLAT segment, the case `dy` exists for (a normalized handle has nothing to scale).
#[test]
fn a_reversed_weighted_track_is_the_mirror_of_the_original() {
    assert_mirrors(
        &[
            (0.0, 1.0, Interp::bezier_w(0.25, 3.0, 0.75, -1.0)),
            (2.0, 1.0, Interp::Linear), // flat: only `dy` can curve it
        ],
        2.0,
    );
    assert_mirrors(
        &[
            (0.0, 0.0, Interp::bezier_w(0.1, 2.0, 0.9, 0.5)),
            (1.5, 4.0, Interp::bezier_w(0.3, -1.0, 0.4, 2.0)),
            (3.0, -2.0, Interp::Linear),
        ],
        3.0,
    );
}

/// A hold is a step; reversed it is still a step, at the other end of the segment. The
/// mirror property does not hold *at* the discontinuity, so this checks the shape.
#[test]
fn a_reversed_hold_is_still_a_hold_with_its_step_at_the_other_end() {
    let mut t = track(&[(0.0, 1.0, Interp::Hold), (2.0, 5.0, Interp::Linear)]);
    t.reverse_about(2.0);
    assert_eq!(f(t.sample(0.0)), 5.0, "the new first key");
    assert_eq!(f(t.sample(1.9)), 5.0, "held all the way across");
    assert_eq!(f(t.sample(2.0)), 1.0, "and steps at the end");
    assert_eq!(t.keys()[0].interp, Interp::Hold);
}

/// Reversing twice is the identity — the cheapest proof that nothing is lost or
/// double-applied on the way out and back.
///
/// Compared as a CURVE, not as bits: the reflection is `1 − x`, and `1 − (1 − 0.1)` is
/// `0.09999999999999998` in `f64`. Demanding bit-equality would fail on a handle that has
/// drifted by 2e-17 — a number no animator can see and no sample can measure. What must
/// come back exactly is the *kind* of each segment, because that is a discrete fact and a
/// silent `Hold → Linear` is a real bug.
#[test]
fn reversing_twice_returns_the_original() {
    let keys = [
        (0.0, -3.0, Interp::bezier(0.1, 0.9, 0.2, 1.0)),
        (1.0, 2.0, Interp::bezier_w(0.25, 3.0, 0.75, -1.0)),
        (2.0, 2.0, Interp::Hold),
        (3.0, 3.0, Interp::Linear),
    ];
    let before = track(&keys);
    let mut after = track(&keys);
    after.reverse_about(3.0);
    after.reverse_about(3.0);

    for i in 0..=300 {
        let t = 3.0 * f64::from(i) / 300.0;
        let (a, b) = (f(before.sample(t)), f(after.sample(t)));
        assert!((a - b).abs() < 1e-9, "t={t}: {a} came back as {b}");
    }
    for (a, b) in before.keys().iter().zip(after.keys()) {
        assert!((a.t.to_seconds() - b.t.to_seconds()).abs() < 1e-9);
        assert_eq!(f(a.value), f(b.value));
        assert_eq!(
            core::mem::discriminant(&a.interp),
            core::mem::discriminant(&b.interp),
            "the segment changed KIND on the round trip: {:?} -> {:?}",
            a.interp,
            b.interp
        );
    }
}

/// A key's identity survives the flip: `KeyId` names a key, not a position, so a
/// selection or an undo step still points at the same key afterwards.
#[test]
fn ids_ride_with_their_keys_through_the_flip() {
    let mut t = track(&[
        (0.0, 1.0, Interp::Linear),
        (1.0, 2.0, Interp::Linear),
        (3.0, 3.0, Interp::Linear),
    ]);
    let first = t.ids()[0];
    let value_of_first = f(t.key(first).unwrap().value);
    t.reverse_about(3.0);
    let moved = t.key(first).expect("the key kept its identity");
    assert_eq!(f(moved.value), value_of_first, "and its value");
    assert!(
        (moved.t.to_seconds() - 3.0).abs() < 1e-9,
        "the key that was at 0 is now at the span: {}",
        moved.t.to_seconds()
    );
}

#[test]
fn reversing_an_empty_track_is_a_no_op() {
    let mut t = Track::new(Vec::new());
    t.reverse_about(3.0);
    assert!(t.is_empty());
}
