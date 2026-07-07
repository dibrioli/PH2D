//! HR-3: the playback hot path ([`Track::sample`] / [`AnimCurve::at`]) must not
//! allocate.
//!
//! Uses the `dhat` heap-profiling allocator. Unlike `ph2d-ecs`'s propagation
//! test (which tolerates a small bevy-internal churn budget), sampling has **no**
//! internal growable buffer — the cursor is a stack/atomic hint and interpolation
//! is pure arithmetic — so the steady-state delta is a hard **zero**. Any
//! non-zero delta is a real regression (a hidden allocation on the tick).

use glam::Vec2;
use ph2d_anim::{
    AnimCurve, AnimValue, AnimationCurveSampler, AttributeEvaluator, Easing, EasingFamily,
    EasingMode, Interp, Key, RationalTime, Track,
};
use ph2d_color::OklchColor;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn f_track() -> Track {
    Track::new(
        (0..12)
            .map(|i| Key {
                t: RationalTime::from_frame(i, 24),
                value: AnimValue::Float(i as f32),
                interp: Interp::bezier(0.25, 0.1, 0.25, 1.0),
            })
            .collect(),
    )
}

fn v_track() -> Track {
    Track::new(
        (0..12)
            .map(|i| Key {
                t: RationalTime::from_frame(i, 24),
                value: AnimValue::Vec2(Vec2::new(i as f32, -(i as f32))),
                interp: Interp::Linear,
            })
            .collect(),
    )
}

fn c_track() -> Track {
    Track::new(
        (0..12)
            .map(|i| Key {
                t: RationalTime::from_frame(i, 24),
                value: AnimValue::Color(OklchColor::new(0.5, 0.1, i as f32 * 30.0, 1.0)),
                interp: Interp::Eased(Easing::new(EasingFamily::Cubic, EasingMode::InOut)),
            })
            .collect(),
    )
}

fn drain(v: AnimValue) -> f64 {
    match v {
        AnimValue::Float(x) => f64::from(x),
        AnimValue::Vec2(p) => f64::from(p.x),
        AnimValue::Color(c) => f64::from(c.h),
        _ => 0.0,
    }
}

#[test]
fn playback_sampling_is_zero_alloc() {
    let _profiler = dhat::Profiler::builder().testing().build();

    let ft = f_track();
    let vt = v_track();
    let ct = c_track();
    let curve = AnimCurve::ease(
        AnimValue::Float(0.0),
        AnimValue::Float(1.0),
        Easing::new(EasingFamily::Quint, EasingMode::Out),
    );

    let mut acc = 0.0f64;

    // Warm-up: first samples set the cursor; nothing here allocates, but we
    // measure the delta from a warm baseline to be strictly correct.
    for i in 0..16 {
        let t = f64::from(i) / 16.0;
        acc += drain(ft.sample(t)) + drain(vt.sample(t)) + drain(ct.sample(t));
        acc += drain(curve.at(t));
    }
    let warm = dhat::HeapStats::get();

    // Steady-state playback sweep, forward then a scrub backward.
    for i in 0..4000 {
        let t = (f64::from(i % 500)) / 499.0;
        acc += drain(ft.sample(t)) + drain(vt.sample(t)) + drain(ct.sample(t));
        acc += drain(curve.at(t));
    }
    let steady = dhat::HeapStats::get();
    std::hint::black_box(acc);

    let d_blocks = steady.total_blocks - warm.total_blocks;
    let d_bytes = steady.total_bytes - warm.total_bytes;
    assert_eq!(
        d_blocks, 0,
        "playback allocated {d_blocks} blocks ({d_bytes} bytes) — sample()/at() must be zero-alloc (HR-3)"
    );
    assert_eq!(
        d_bytes, 0,
        "playback allocated {d_bytes} bytes — expected 0 (HR-3)"
    );
}
