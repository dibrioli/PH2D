//! Golden / behavioural tests — the acceptance suite for `ph2d-anim` (this
//! crate has no UI, so the seam/behavioural-test gate does not apply; the test
//! suite *is* the DoD, in the spirit of a node golden test).

use glam::Vec2;
use ph2d_anim::{
    AnimCurve, AnimValue, AnimationCurveSampler, AttributeEvaluator, Clip, Easing, EasingFamily,
    EasingMode, Interp, Key, RationalTime, Track,
};
use ph2d_color::OklchColor;

const EPS: f64 = 1e-4;

fn as_float(v: AnimValue) -> f32 {
    match v {
        AnimValue::Float(x) => x,
        other => panic!("expected Float, got {other:?}"),
    }
}

fn as_color(v: AnimValue) -> OklchColor {
    match v {
        AnimValue::Color(c) => c,
        other => panic!("expected Color, got {other:?}"),
    }
}

// ── DoD 1: a constant track returns the same value at any time. ──────────────
#[test]
fn constant_track_is_constant() {
    let track = Track::constant(AnimValue::Float(3.5));
    for &t in &[-1000.0, -1.0, 0.0, 0.5, 1.0, 1e6] {
        assert_eq!(track.sample(t), AnimValue::Float(3.5));
    }
}

// ── DoD 2: linear 2-key segment sampled at the midpoint is the exact average. ─
#[test]
fn linear_midpoint_is_exact_float() {
    let track = Track::new(vec![
        Key {
            t: RationalTime::from_seconds(0.0),
            value: AnimValue::Float(0.0),
            interp: Interp::Linear,
        },
        Key {
            t: RationalTime::from_seconds(1.0),
            value: AnimValue::Float(10.0),
            interp: Interp::Hold,
        },
    ]);
    assert_eq!(track.sample(0.5), AnimValue::Float(5.0));
    assert_eq!(track.sample(0.25), AnimValue::Float(2.5));
}

#[test]
fn linear_midpoint_is_exact_vec2() {
    let track = Track::new(vec![
        Key {
            t: RationalTime::from_seconds(0.0),
            value: AnimValue::Vec2(Vec2::new(0.0, 0.0)),
            interp: Interp::Linear,
        },
        Key {
            t: RationalTime::from_seconds(1.0),
            value: AnimValue::Vec2(Vec2::new(10.0, 20.0)),
            interp: Interp::Hold,
        },
    ]);
    assert_eq!(track.sample(0.5), AnimValue::Vec2(Vec2::new(5.0, 10.0)));
}

// ── DoD 3: easing — a known bezier golden + every preset passes through 0 and 1.
#[test]
fn cubic_bezier_symmetric_midpoint_is_half() {
    // (.42, 0, .58, 1) is point-symmetric about (0.5, 0.5) → f(0.5) == 0.5.
    let v = Interp::bezier(0.42, 0.0, 0.58, 1.0).remap(0.5);
    assert!((v - 0.5).abs() < EPS, "bezier midpoint = {v}, expected 0.5");
    assert!(Interp::bezier(0.42, 0.0, 0.58, 1.0).remap(0.0).abs() < EPS);
    assert!((Interp::bezier(0.42, 0.0, 0.58, 1.0).remap(1.0) - 1.0).abs() < EPS);
}

#[test]
fn every_easing_preset_passes_through_endpoints() {
    for family in EasingFamily::ALL {
        for mode in EasingMode::ALL {
            let e = Easing::new(family, mode);
            let at0 = e.eval(0.0);
            let at1 = e.eval(1.0);
            assert!(
                at0.abs() < 1e-9,
                "{family:?}/{mode:?} e(0) = {at0}, expected 0"
            );
            assert!(
                (at1 - 1.0).abs() < 1e-9,
                "{family:?}/{mode:?} e(1) = {at1}, expected 1"
            );
        }
    }
}

#[test]
fn determinism_flag_matches_family_class() {
    assert!(Easing::new(EasingFamily::Cubic, EasingMode::InOut).is_deterministic());
    assert!(Easing::new(EasingFamily::Back, EasingMode::Out).is_deterministic());
    assert!(Easing::new(EasingFamily::Bounce, EasingMode::Out).is_deterministic());
    assert!(!Easing::new(EasingFamily::Sine, EasingMode::In).is_deterministic());
    assert!(!Easing::new(EasingFamily::Expo, EasingMode::In).is_deterministic());
    assert!(!Easing::new(EasingFamily::Circ, EasingMode::In).is_deterministic());
    assert!(!Easing::new(EasingFamily::Elastic, EasingMode::In).is_deterministic());
    // Bezier is polynomial-solved → deterministic.
    assert!(Interp::bezier(0.3, 0.0, 0.7, 1.0).is_deterministic());
    assert!(!Interp::Eased(Easing::new(EasingFamily::Sine, EasingMode::In)).is_deterministic());
}

// ── DoD 4: Hold/stepped holds the previous key's value until the next key. ────
#[test]
fn hold_is_stepped() {
    let track = Track::new(vec![
        Key {
            t: RationalTime::from_seconds(0.0),
            value: AnimValue::Float(0.0),
            interp: Interp::Hold,
        },
        Key {
            t: RationalTime::from_seconds(1.0),
            value: AnimValue::Float(10.0),
            interp: Interp::Hold,
        },
    ]);
    // Anywhere inside the first segment holds the start value…
    assert_eq!(track.sample(0.001), AnimValue::Float(0.0));
    assert_eq!(track.sample(0.999), AnimValue::Float(0.0));
    // …and the next key snaps to its value.
    assert_eq!(track.sample(1.0), AnimValue::Float(10.0));
}

// ── DoD 5: sampling outside the key range clamps to the end values. ──────────
#[test]
fn ends_are_clamped() {
    let track = Track::new(vec![
        Key {
            t: RationalTime::from_seconds(0.0),
            value: AnimValue::Float(2.0),
            interp: Interp::Linear,
        },
        Key {
            t: RationalTime::from_seconds(1.0),
            value: AnimValue::Float(8.0),
            interp: Interp::Hold,
        },
    ]);
    assert_eq!(track.sample(-5.0), AnimValue::Float(2.0));
    assert_eq!(track.sample(100.0), AnimValue::Float(8.0));
}

// ── DoD 6: colour tracks take the OKLCH short hue arc (350° → 10° = 20°). ─────
#[test]
fn color_track_takes_short_hue_arc() {
    let track = Track::new(vec![
        Key {
            t: RationalTime::from_seconds(0.0),
            value: AnimValue::Color(OklchColor::new(0.5, 0.1, 350.0, 1.0)),
            interp: Interp::Linear,
        },
        Key {
            t: RationalTime::from_seconds(1.0),
            value: AnimValue::Color(OklchColor::new(0.5, 0.1, 10.0, 1.0)),
            interp: Interp::Hold,
        },
    ]);
    let mid = as_color(track.sample(0.5));
    // Short arc through 360°/0°, not the long way through 180°.
    assert!(
        mid.h.abs() < 1e-3 || (mid.h - 360.0).abs() < 1e-3,
        "hue = {} (expected ~0)",
        mid.h
    );
    assert!((mid.l - 0.5).abs() < 1e-6);
    assert!((mid.c - 0.1).abs() < 1e-6);
}

// ── DoD 9: AnimCurve is a real AnimationCurveSampler (2nd mock replaced). ─────
#[test]
fn anim_curve_linear_samples() {
    let curve = AnimCurve::linear(AnimValue::Float(0.0), AnimValue::Float(4.0));
    assert_eq!(curve.at(0.0), AnimValue::Float(0.0));
    assert_eq!(curve.at(0.5), AnimValue::Float(2.0));
    assert_eq!(curve.at(1.0), AnimValue::Float(4.0));
    // Flat-clamped ends.
    assert_eq!(curve.at(-1.0), AnimValue::Float(0.0));
    assert_eq!(curve.at(2.0), AnimValue::Float(4.0));
}

#[test]
fn anim_curve_matches_mock_linear_semantics() {
    use ph2d_vector_traits::MockAnimationCurveSampler;
    let from = AnimValue::Float(1.0);
    let to = AnimValue::Float(3.0);
    let real = AnimCurve::linear(from, to);
    let mock = MockAnimationCurveSampler::new(from, to);
    for &t in &[0.0, 0.1, 0.5, 0.9, 1.0] {
        assert!((as_float(real.at(t)) - as_float(mock.at(t))).abs() < 1e-6);
    }
}

#[test]
fn anim_curve_eased_endpoints() {
    let curve = AnimCurve::ease(
        AnimValue::Float(0.0),
        AnimValue::Float(1.0),
        Easing::new(EasingFamily::Cubic, EasingMode::InOut),
    );
    assert!((as_float(curve.at(0.0)) - 0.0).abs() < 1e-6);
    assert!((as_float(curve.at(1.0)) - 1.0).abs() < 1e-6);
    // Symmetric ease-in-out crosses 0.5 at the midpoint.
    assert!((as_float(curve.at(0.5)) - 0.5).abs() < 1e-6);
}

// ── Clip binds tracks to opaque targets and samples per-target. ──────────────
#[test]
fn clip_samples_per_target() {
    use ph2d_anim::AnimTarget;
    let opacity = AnimTarget::new(1);
    let rotation = AnimTarget::new(2);
    let clip = Clip::new(RationalTime::from_frame(24, 24))
        .with_track(opacity, Track::constant(AnimValue::Float(0.5)))
        .with_track(
            rotation,
            Track::new(vec![
                Key {
                    t: RationalTime::from_seconds(0.0),
                    value: AnimValue::Float(0.0),
                    interp: Interp::Linear,
                },
                Key {
                    t: RationalTime::from_seconds(1.0),
                    value: AnimValue::Float(90.0),
                    interp: Interp::Hold,
                },
            ]),
        );
    assert_eq!(clip.sample(opacity, 0.5), Some(AnimValue::Float(0.5)));
    assert_eq!(clip.sample(rotation, 0.5), Some(AnimValue::Float(45.0)));
    assert_eq!(clip.sample(AnimTarget::new(999), 0.5), None);
    assert!((clip.duration().to_seconds() - 1.0).abs() < 1e-9);
    assert_eq!(clip.len(), 2);
}

// ── Thread-safety: Track/Clip/AnimCurve satisfy the future bridge bound. ──────
#[test]
fn types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Track>();
    assert_send_sync::<Clip>();
    assert_send_sync::<AnimCurve>();

    // The exact bound the future motion bridge pins at the use site.
    let _eval: Box<dyn AttributeEvaluator + Send + Sync> =
        Box::new(Track::constant(AnimValue::Float(1.0)));
    let _curve: Box<dyn AnimationCurveSampler + Send + Sync> = Box::new(AnimCurve::linear(
        AnimValue::Float(0.0),
        AnimValue::Float(1.0),
    ));
}
