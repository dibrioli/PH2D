//! The general-timeline seam, headless: advancing the engine [`Playhead`] drives
//! a [`Clip`] sample. This is the "it's alive" proof the visual smoke shows a
//! human — asserted here without a GPU: the sampled value *changes* as the
//! playhead advances, *freezes* on pause, and is *deterministic*.

use ph2d_anim::{AnimTarget, AnimValue, Clip, Interp, Key, RationalTime, Track};
use ph2d_core::Playhead;

const DT: f64 = 1.0 / 60.0;

fn ramp_clip() -> (Clip, AnimTarget) {
    let target = AnimTarget::new(0);
    let clip = Clip::new(RationalTime::from_seconds(2.0)).with_track(
        target,
        Track::new(vec![
            Key {
                t: RationalTime::from_seconds(0.0),
                value: AnimValue::Float(0.0),
                interp: Interp::Linear,
            },
            Key {
                t: RationalTime::from_seconds(2.0),
                value: AnimValue::Float(10.0),
                interp: Interp::Hold,
            },
        ]),
    );
    (clip, target)
}

#[test]
fn advancing_playhead_moves_the_sample() {
    let (clip, target) = ramp_clip();
    let mut ph = Playhead::new(DT);

    let s0 = clip.sample(target, ph.time());
    assert_eq!(s0, Some(AnimValue::Float(0.0)));

    for _ in 0..30 {
        ph.advance(); // 0.5 s
    }
    let s_half = clip.sample(target, ph.time());
    // Live: the sample changed as the playhead advanced…
    assert_ne!(s0, s_half);
    // …and it's the exact linear value at 0.5 s of a 0→10 ramp over 2 s.
    match s_half {
        Some(AnimValue::Float(v)) => assert!((v - 2.5).abs() < 1e-5, "got {v}, expected 2.5"),
        other => panic!("expected Float, got {other:?}"),
    }
}

#[test]
fn pause_freezes_the_sample() {
    let (clip, target) = ramp_clip();
    let mut ph = Playhead::new(DT);
    for _ in 0..15 {
        ph.advance();
    }
    let frozen = clip.sample(target, ph.time());
    ph.pause();
    for _ in 0..60 {
        ph.advance();
    }
    assert_eq!(clip.sample(target, ph.time()), frozen);
}

#[test]
fn seek_scrubs_to_an_exact_value() {
    let (clip, target) = ramp_clip();
    let mut ph = Playhead::new(DT);
    ph.seek(1.0); // midpoint of the 2 s ramp → 5.0
    assert_eq!(clip.sample(target, ph.time()), Some(AnimValue::Float(5.0)));
}

#[test]
fn playhead_driven_sampling_is_deterministic() {
    let (clip, target) = ramp_clip();
    let mut a = Playhead::new(DT);
    let mut b = Playhead::new(DT);
    for _ in 0..37 {
        a.advance();
        b.advance();
    }
    // Same advance sequence → bit-identical time → identical sample.
    assert_eq!(a.time().to_bits(), b.time().to_bits());
    assert_eq!(clip.sample(target, a.time()), clip.sample(target, b.time()));
}
