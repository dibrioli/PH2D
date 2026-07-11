//! W0.T4 — the persistable timeline data round-trips through serde (JSON here;
//! the real save format is the document's choice). Covers every `Interp` and
//! `AnimValue` kind, the `Track`/`Clip`/`AnimCurve` proxies, and the version marker.

use glam::{Vec2, Vec3};
use ph2d_anim::{
    AnimCurve, AnimTarget, AnimValue, AnimationCurveSampler, AttributeEvaluator, Clip, Easing,
    EasingFamily, EasingMode, Interp, Key, RationalTime, SCHEMA_VERSION, Track,
};
use ph2d_color::OklchColor;

fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(v: &T) -> T {
    let json = serde_json::to_string(v).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn sample_track() -> Track {
    Track::new(vec![
        Key {
            t: s(0.0),
            value: AnimValue::Float(1.0),
            interp: Interp::Linear,
        },
        Key {
            t: RationalTime::from_frame(24, 24),
            value: AnimValue::Vec2(Vec2::new(3.0, -4.0)),
            interp: Interp::Hold,
        },
        Key {
            t: s(2.0),
            value: AnimValue::Color(OklchColor::new(0.5, 0.1, 120.0, 1.0)),
            interp: Interp::Eased(Easing::new(EasingFamily::Cubic, EasingMode::InOut)),
        },
        Key {
            t: s(3.0),
            value: AnimValue::Vec3(Vec3::new(1.0, 2.0, 3.0)),
            interp: Interp::bezier(0.42, 0.0, 0.58, 1.0),
        },
        Key {
            t: s(3.5),
            value: AnimValue::Float(2.5),
            interp: Interp::bezier_w(0.25, 3.0, 0.75, -1.5),
        },
        Key {
            t: s(4.0),
            value: AnimValue::Bool(true),
            interp: Interp::Linear,
        },
        Key {
            t: s(5.0),
            value: AnimValue::Enum(7),
            interp: Interp::Hold,
        },
    ])
}

#[test]
fn track_roundtrips_every_value_and_interp_kind() {
    let tr = sample_track();
    let back = roundtrip(&tr);
    assert_eq!(back.len(), tr.len());
    for (a, b) in tr.keys().iter().zip(back.keys()) {
        assert_eq!(a.value, b.value, "value survives");
        assert_eq!(a.interp, b.interp, "interp survives");
        assert_eq!(a.t, b.t, "time survives (normalized)");
    }
    // Behaviour is identical (f32 round-trips exactly through serde_json/ryu).
    for &t in &[-1.0, 0.5, 1.0, 2.5, 3.5, 4.5, 100.0] {
        assert_eq!(tr.sample(t), back.sample(t), "sample identical at {t}");
    }
}

#[test]
fn clip_roundtrips() {
    let clip = Clip::new(RationalTime::from_frame(48, 24))
        .with_track(AnimTarget::new(3), sample_track())
        .with_track(AnimTarget::new(7), Track::constant(AnimValue::Float(9.0)));
    let back = roundtrip(&clip);
    assert_eq!(back.len(), 2);
    assert_eq!(back.duration(), clip.duration());
    assert_eq!(
        back.sample(AnimTarget::new(3), 2.5),
        clip.sample(AnimTarget::new(3), 2.5)
    );
    assert_eq!(
        back.sample(AnimTarget::new(7), 0.0),
        Some(AnimValue::Float(9.0))
    );
    assert_eq!(back.sample(AnimTarget::new(999), 0.0), None);
}

#[test]
fn anim_curve_roundtrips_and_stays_sorted() {
    let curve = AnimCurve::bezier(
        AnimValue::Float(0.0),
        AnimValue::Float(10.0),
        0.25,
        0.1,
        0.25,
        1.0,
    );
    let back = roundtrip(&curve);
    for &t in &[0.0, 0.25, 0.5, 0.75, 1.0] {
        assert_eq!(curve.at(t), back.at(t), "curve sample identical at {t}");
    }
}

#[test]
fn schema_version_is_stable() {
    assert_eq!(SCHEMA_VERSION, 1);
}
