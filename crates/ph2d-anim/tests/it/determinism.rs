//! Determinism + rational-time exactness.
//!
//! Sampling the same `(track, t)` twice must yield **bit-identical** values, and
//! the cursor fast path must agree bit-for-bit with a cursor-cold reference —
//! otherwise playback would diverge from a random-access scrub.

use ph2d_anim::{AnimValue, AttributeEvaluator, Interp, Key, RationalTime, Track};

fn float_bits(v: AnimValue) -> u32 {
    match v {
        AnimValue::Float(x) => x.to_bits(),
        other => panic!("expected Float, got {other:?}"),
    }
}

fn ramp() -> Vec<Key> {
    // A non-trivial multi-segment ramp with mixed interpolation.
    (0..17)
        .map(|i| Key {
            t: RationalTime::from_frame(i, 24),
            value: AnimValue::Float((i * i) as f32 * 0.13 - 4.0),
            interp: if i % 3 == 0 {
                Interp::Linear
            } else if i % 3 == 1 {
                Interp::bezier(0.3, 0.1, 0.7, 0.9)
            } else {
                Interp::Hold
            },
        })
        .collect()
}

// ── from_frame is EXACT (integer num/den, not a rounded float). ──────────────
#[test]
fn from_frame_is_exact() {
    let one_sec = RationalTime::from_frame(24, 24);
    assert_eq!(one_sec.num(), 24);
    assert_eq!(one_sec.den(), 24);
    assert!((one_sec.to_seconds() - 1.0).abs() < 1e-15);

    let one_frame = RationalTime::from_frame(1, 24);
    assert_eq!(one_frame.num(), 1);
    assert_eq!(one_frame.den(), 24);
    // Exactly 1/24 s — matches the direct division bit-for-bit.
    assert_eq!(one_frame.to_seconds().to_bits(), (1.0f64 / 24.0).to_bits());
}

// ── to_seconds round-trips stably; equality is by normalized value. ──────────
#[test]
fn rational_time_roundtrip_stable() {
    let a = RationalTime::from_seconds(1.234_567);
    let b = RationalTime::from_seconds(1.234_567);
    assert_eq!(a, b); // same construction → equal
    assert_eq!(a.to_seconds().to_bits(), b.to_seconds().to_bits());
    assert!((a.to_seconds() - 1.234_567).abs() < 1e-6); // microsecond snap

    // Normalized equality: 2/4 == 1/2, 48/24 == 2/1.
    assert_eq!(RationalTime::new(2, 4), RationalTime::new(1, 2));
    assert_eq!(
        RationalTime::from_frame(48, 24),
        RationalTime::from_frame(2, 1)
    );
    assert!(RationalTime::from_frame(1, 24) < RationalTime::from_frame(1, 12));
}

// ── Same (track, t) → bit-identical every time. ──────────────────────────────
#[test]
fn repeated_sample_is_bit_identical() {
    let track = Track::new(ramp());
    for k in 0..200 {
        let t = k as f64 * 0.011;
        let first = float_bits(track.sample(t));
        for _ in 0..5 {
            assert_eq!(float_bits(track.sample(t)), first);
        }
    }
}

// ── Cursor fast path agrees with a cursor-cold reference (fresh Track). ───────
#[test]
fn cursor_path_matches_cold_reference() {
    let played = Track::new(ramp());

    // Warm the cursor by sweeping forward, then random-access.
    let sweep: Vec<f64> = (0..300).map(|i| i as f64 * 0.0047).collect();
    for &t in &sweep {
        let _ = played.sample(t);
    }

    // Now sample in a scrambled order and compare each against a cold Track.
    let order = [0.61, 0.02, 0.5, 0.199, 0.0, 0.7, 0.33, 0.6667, 0.48, 0.12];
    for &t in &order {
        let cold = Track::new(ramp()); // cursor at 0
        assert_eq!(
            float_bits(played.sample(t)),
            float_bits(cold.sample(t)),
            "cursor path diverged from cold reference at t = {t}"
        );
    }
}

// ── A cloned track samples identically (cursor is a hint, not state). ─────────
#[test]
fn cloned_track_samples_identically() {
    let a = Track::new(ramp());
    let _ = a.sample(0.5); // advance a's cursor
    let b = a.clone();
    for k in 0..100 {
        let t = k as f64 * 0.007;
        assert_eq!(float_bits(a.sample(t)), float_bits(b.sample(t)));
    }
}
