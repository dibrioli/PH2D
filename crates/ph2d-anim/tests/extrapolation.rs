//! Per-track EXTRAPOLATION (crown-jewels plan §6) — the engine gates for
//! [`ph2d_anim::Extrap`] on [`ph2d_anim::Track`]. Loop / PingPong / Continue
//! reproduce the right value beyond the keys; the DEFAULT (`Hold/Hold`) is the
//! flat-clamp, byte-identical to the pre-feature engine (the fade fingerprint
//! pin lives in `ph2d-timeline/tests/fade_fingerprint.rs` — this file proves the
//! focused per-mode behaviour).

use ph2d_anim::{AnimValue, AttributeEvaluator, Extrap, Interp, Key, RationalTime, Track};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// A linear ramp `0 → 10` over `[0, 2]` (slope 5/s) — the canonical fixture: its
/// in-range value at time `t` is exactly `5·t`, so an extrapolated read has an
/// arithmetic oracle.
fn ramp() -> Track {
    Track::new(vec![
        Key {
            t: s(0.0),
            value: AnimValue::Float(0.0),
            interp: Interp::Linear,
        },
        Key {
            t: s(2.0),
            value: AnimValue::Float(10.0),
            interp: Interp::Linear,
        },
    ])
}

fn f(v: AnimValue) -> f64 {
    match v {
        AnimValue::Float(x) => f64::from(x),
        _ => panic!("expected scalar"),
    }
}

#[test]
fn default_is_hold_and_flat_clamps_both_ends() {
    let t = ramp();
    assert_eq!(t.pre(), Extrap::Hold);
    assert_eq!(t.post(), Extrap::Hold);
    // Before the first key holds the first value; after the last, the last.
    assert_eq!(t.sample(-5.0), AnimValue::Float(0.0));
    assert_eq!(t.sample(100.0), AnimValue::Float(10.0));
    // In-range is untouched.
    assert_eq!(t.sample(1.0), AnimValue::Float(5.0));
}

#[test]
fn hold_is_byte_identical_at_the_boundary() {
    // A non-Hold mode must NOT change the value AT the exact boundary — only
    // strictly outside. This is what lets Hold stay the byte-identical clamp.
    let mut t = ramp();
    t.set_post(Extrap::Loop);
    assert_eq!(t.sample(2.0), AnimValue::Float(10.0)); // exactly the last key
    t.set_pre(Extrap::Loop);
    assert_eq!(t.sample(0.0), AnimValue::Float(0.0)); // exactly the first key
}

#[test]
fn loop_repeats_the_range() {
    let mut t = ramp();
    t.set_post(Extrap::Loop);
    // period = 2s. t=2.5 -> 0.5 -> 2.5; t=3.0 -> 1.0 -> 5.0; t=4.5 -> 0.5 -> 2.5.
    assert!((f(t.sample(2.5)) - 2.5).abs() < 1e-9);
    assert!((f(t.sample(3.0)) - 5.0).abs() < 1e-9);
    assert!((f(t.sample(4.5)) - 2.5).abs() < 1e-9);
    // The sawtooth discontinuity: just before a period boundary reads near the
    // END, just after reads near the START.
    assert!(f(t.sample(3.999)) > 9.9);
    assert!(f(t.sample(4.001)) < 0.1);
}

#[test]
fn loop_wraps_the_pre_side_too() {
    let mut t = ramp();
    t.set_pre(Extrap::Loop);
    // t=-0.5 -> just before the end -> ~7.5 (value at 1.5).
    assert!((f(t.sample(-0.5)) - 7.5).abs() < 1e-9);
    // t=-2.0 is exactly one period back: (−2).rem_euclid(2) = 0 -> value at 0.
    assert!((f(t.sample(-2.0)) - 0.0).abs() < 1e-9);
}

#[test]
fn pingpong_reflects_the_range() {
    let mut t = ramp();
    t.set_post(Extrap::PingPong);
    // t=2.5 reflects to 1.5 -> 7.5; t=3.0 reflects to 1.0 -> 5.0; t=4.0 folds
    // back to the start -> 0.0; t=4.5 -> 0.5 -> 2.5.
    assert!((f(t.sample(2.5)) - 7.5).abs() < 1e-9);
    assert!((f(t.sample(3.0)) - 5.0).abs() < 1e-9);
    assert!((f(t.sample(4.0)) - 0.0).abs() < 1e-9);
    assert!((f(t.sample(4.5)) - 2.5).abs() < 1e-9);
}

#[test]
fn continue_extends_along_the_end_slope() {
    let mut t = ramp();
    t.set_post(Extrap::Continue);
    t.set_pre(Extrap::Continue);
    // slope 5/s. Post: 10 + 5·(t−2). Pre: 0 + 5·(t−0).
    assert!((f(t.sample(3.0)) - 15.0).abs() < 1e-9);
    assert!((f(t.sample(4.0)) - 20.0).abs() < 1e-9);
    assert!((f(t.sample(-1.0)) - (-5.0)).abs() < 1e-9);
    assert!((f(t.sample(-2.0)) - (-10.0)).abs() < 1e-9);
}

#[test]
fn pre_and_post_are_independent() {
    let mut t = ramp();
    t.set_pre(Extrap::Hold);
    t.set_post(Extrap::Continue);
    assert_eq!(t.sample(-5.0), AnimValue::Float(0.0)); // pre holds
    assert!((f(t.sample(3.0)) - 15.0).abs() < 1e-9); // post continues
}

#[test]
fn continue_holds_when_the_last_segment_is_stepped() {
    // The LAST SEGMENT's interp is `keys[n-2].interp` (the one leaving the
    // second-to-last key). A stepped segment has value_slope 0, so Continue has
    // no velocity to extend along and holds — a defensible, finite answer.
    let mut t = Track::new(vec![
        Key {
            t: s(0.0),
            value: AnimValue::Float(0.0),
            interp: Interp::Hold, // governs the segment [0, 2]
        },
        Key {
            t: s(2.0),
            value: AnimValue::Float(10.0),
            interp: Interp::Linear, // no segment after the last key — irrelevant
        },
    ]);
    t.set_post(Extrap::Continue);
    assert_eq!(t.sample(5.0), AnimValue::Float(10.0));
}

#[test]
fn a_zero_width_range_holds() {
    // Two keys at the same instant: no period to repeat, no span to slope. Every
    // non-Hold mode degrades to the boundary value rather than dividing by zero.
    let mut t = Track::new(vec![
        Key {
            t: s(1.0),
            value: AnimValue::Float(3.0),
            interp: Interp::Linear,
        },
        Key {
            t: s(1.0),
            value: AnimValue::Float(7.0),
            interp: Interp::Linear,
        },
    ]);
    for mode in [Extrap::Loop, Extrap::PingPong, Extrap::Continue] {
        t.set_post(mode);
        assert!(f(t.sample(5.0)).is_finite());
    }
}

#[test]
fn extrapolation_survives_the_serde_round_trip() {
    let mut t = ramp();
    t.set_pre(Extrap::PingPong);
    t.set_post(Extrap::Continue);
    // serde_json (not postcard) to match the crate's round-trip convention
    // (`tests/serde_roundtrip.rs`); the format is the document's choice.
    let json = serde_json::to_string(&t).expect("serialize");
    let back: Track = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.pre(), Extrap::PingPong);
    assert_eq!(back.post(), Extrap::Continue);
    // And it still samples the same beyond the range.
    assert_eq!(t.sample(3.0), back.sample(3.0));
}
