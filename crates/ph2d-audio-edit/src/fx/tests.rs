//! Surface-level tests for the `fx` effect enums — kept out of `fx.rs` so that
//! file stays under the 700-LOC workspace cap as the roster grows. A child module
//! of `fx`, so it reaches its private consts + helpers.

use super::*;
use ph2d_audio::AudioFormat;

fn stereo(v: Vec<f32>) -> SampleData {
    SampleData::from_interleaved(v, AudioFormat::stereo(48_000))
}

/// Every non-neutral `Effect`, for the surface-level invariants below. Keep in
/// sync with the enum — a variant missing here is a variant nobody proved
/// length-preserving.
fn tuned_effects() -> [Effect; 19] {
    [
        Effect::LowPass {
            cutoff: 3_000.0,
            q: 0.707,
        },
        Effect::HighPass {
            cutoff: 150.0,
            q: 0.707,
        },
        Effect::Peak {
            freq: 1_000.0,
            q: 1.0,
            gain_db: 6.0,
        },
        Effect::LowShelf {
            freq: 200.0,
            q: 0.707,
            gain_db: -6.0,
        },
        Effect::HighShelf {
            freq: 6_000.0,
            q: 0.707,
            gain_db: 4.0,
        },
        Effect::Compress {
            threshold: 0.3,
            ratio: 4.0,
            attack_secs: 0.005,
            release_secs: 0.1,
        },
        Effect::Gate {
            threshold: 0.05,
            ratio: 8.0,
            attack_secs: 0.002,
            release_secs: 0.1,
        },
        Effect::DeEss {
            freq: 6_000.0,
            threshold: 0.05,
            ratio: 6.0,
            release_secs: 0.05,
        },
        Effect::Limiter {
            ceiling_db: -1.0,
            release_secs: 0.02,
        },
        Effect::Saturate { drive: 3.0 },
        Effect::Bitcrush {
            bits: 6,
            downsample: 4,
        },
        Effect::StereoWidth { width: 1.6 },
        Effect::Chorus {
            rate: 1.0,
            depth_ms: 5.0,
            mix: 0.5,
        },
        Effect::Flanger {
            rate: 0.5,
            depth_ms: 3.0,
            feedback: 0.4,
            mix: 0.5,
        },
        Effect::Phaser {
            rate: 0.5,
            depth: 0.8,
            mix: 0.5,
        },
        Effect::Tremolo {
            rate: 5.0,
            depth: 0.8,
        },
        // The LPC/WSOLA end of the rack: block-based, so length preservation is a
        // real claim rather than a free one (overlap-add and a resample-then-stretch
        // both have to land back on exactly the frames they started with).
        Effect::FormantShift {
            semitones: -5.0,
            mix: 1.0,
        },
        Effect::Harmonizer {
            v1_st: 4.0,
            v2_st: 7.0,
            mix: 0.6,
        },
        Effect::DeClick {
            sensitivity: 0.8,
            width_secs: 0.001,
        },
    ]
}

/// Every neutral `Effect`. Keep in sync with `is_bypass`.
fn neutral_effects() -> [Effect; 19] {
    [
        Effect::LowPass {
            cutoff: 20_000.0,
            q: 0.707,
        },
        Effect::HighPass {
            cutoff: 20.0,
            q: 0.707,
        },
        Effect::Peak {
            freq: 1_000.0,
            q: 1.0,
            gain_db: 0.0,
        },
        Effect::LowShelf {
            freq: 200.0,
            q: 0.707,
            gain_db: 0.0,
        },
        Effect::HighShelf {
            freq: 6_000.0,
            q: 0.707,
            gain_db: 0.0,
        },
        Effect::Compress {
            threshold: 0.3,
            ratio: 1.0,
            attack_secs: 0.005,
            release_secs: 0.1,
        },
        Effect::Gate {
            threshold: 0.05,
            ratio: 1.0,
            attack_secs: 0.002,
            release_secs: 0.1,
        },
        Effect::DeEss {
            freq: 6_000.0,
            threshold: 0.05,
            ratio: 1.0,
            release_secs: 0.05,
        },
        Effect::Limiter {
            ceiling_db: 0.0,
            release_secs: 0.02,
        },
        Effect::Saturate { drive: 0.0 },
        Effect::Bitcrush {
            bits: 16,
            downsample: 1,
        },
        Effect::StereoWidth { width: 1.0 },
        Effect::Chorus {
            rate: 1.0,
            depth_ms: 5.0,
            mix: 0.0,
        },
        Effect::Flanger {
            rate: 0.5,
            depth_ms: 3.0,
            feedback: 0.4,
            mix: 0.0,
        },
        Effect::Phaser {
            rate: 0.5,
            depth: 0.8,
            mix: 0.0,
        },
        Effect::Tremolo {
            rate: 5.0,
            depth: 0.0,
        },
        Effect::FormantShift {
            semitones: 0.0,
            mix: 1.0,
        },
        Effect::Harmonizer {
            v1_st: 4.0,
            v2_st: 7.0,
            mix: 0.0,
        },
        Effect::DeClick {
            sensitivity: 0.0,
            width_secs: 0.001,
        },
    ]
}

#[test]
fn effects_preserve_length() {
    let d = stereo(vec![0.3, -0.4, 0.5, -0.6, 0.7, -0.8]); // 3 frames
    for fx in tuned_effects() {
        assert_eq!(
            fx.apply(&d).frame_count(),
            3,
            "{fx:?} must keep the frame count"
        );
    }
}

/// `is_bypass` must be exactly the set of parameters where the effect returns
/// its input untouched — the editor selects effects at these values, so an
/// "almost identity" here would smear the audio just by browsing the rack.
#[test]
fn is_bypass_implies_exact_identity() {
    let d = stereo(vec![0.3, -0.4, 0.5, -0.6, 0.7, -0.8]);
    for fx in neutral_effects() {
        assert!(fx.is_bypass(), "{fx:?} should be neutral");
        assert_eq!(
            fx.apply(&d).samples(),
            d.samples(),
            "{fx:?} altered the audio"
        );
        assert_eq!(fx.warmup_frames(48_000), 0, "{fx:?} asked for a pre-roll");
    }

    // ...and the converse: a tuned effect is NOT bypassed, or the rack would
    // silently ignore the knob the user just turned.
    for fx in tuned_effects() {
        assert!(!fx.is_bypass(), "{fx:?} reads as neutral while tuned");
    }

    // A fully-dry tail effect must not ring out NOR lengthen the clip.
    for fx in [
        TailEffect::Reverb {
            room_size: 0.7,
            damp: 0.5,
            mix: 0.0,
            tail_secs: 2.5,
        },
        TailEffect::Delay {
            time_secs: 0.25,
            feedback: 0.4,
            mix: 0.0,
            tail_secs: 2.0,
        },
    ] {
        assert!(fx.is_bypass(), "{fx:?} should be neutral");
        assert_eq!(
            fx.tail_frames(48_000),
            0,
            "{fx:?} would append a silent tail"
        );
        assert_eq!(fx.render(&d, 0).samples(), d.samples());
    }
}

/// The three EQ bands must actually shape the band they name, and only it: a
/// bell at 1 kHz that also lifts DC is a shelf with a bad haircut.
#[test]
fn the_eq_bands_shape_the_band_they_name() {
    let sr = 48_000.0;
    // Transfer-function magnitude at DC (z=1) and Nyquist (z=-1).
    let ends = |c: BiquadCoeffs| {
        (
            (c.b0 + c.b1 + c.b2) / (1.0 + c.a1 + c.a2),
            (c.b0 - c.b1 + c.b2) / (1.0 - c.a1 + c.a2),
        )
    };
    let boost = 10f32.powf(12.0 / 20.0);

    let (dc, nyq) = ends(BiquadCoeffs::peak(sr, 1_000.0, 1.0, 12.0));
    assert!(
        (dc - 1.0).abs() < 0.02 && (nyq - 1.0).abs() < 0.02,
        "bell tilted the ends"
    );

    let (dc, nyq) = ends(BiquadCoeffs::lowshelf(sr, 200.0, 0.707, 12.0));
    assert!((dc - boost).abs() < 0.1, "low shelf did not lift DC: {dc}");
    assert!((nyq - 1.0).abs() < 0.05, "low shelf moved the top: {nyq}");

    let (dc, nyq) = ends(BiquadCoeffs::highshelf(sr, 4_000.0, 0.707, 12.0));
    assert!(
        (nyq - boost).abs() < 0.2,
        "high shelf did not lift the top: {nyq}"
    );
    assert!((dc - 1.0).abs() < 0.05, "high shelf moved DC: {dc}");
}

/// The limiter's warm-up must cover its whole look-ahead window. Without it,
/// `in_range_warm` hands it a region whose gain curve starts flat at 1.0 and the
/// first peak inside a selection escapes.
#[test]
fn the_limiter_asks_for_its_whole_lookahead_as_warmup() {
    let fx = Effect::Limiter {
        ceiling_db: -1.0,
        release_secs: 0.02,
    };
    assert_eq!(fx.warmup_frames(48_000), (0.02 * 48_000.0) as usize);
    // Capped at one second, however long the release.
    let long = Effect::Limiter {
        ceiling_db: -1.0,
        release_secs: 10.0,
    };
    assert_eq!(long.warmup_frames(48_000), 48_000);
}

/// The tail must actually RING — and be long enough to contain the effect's
/// first output. Freeverb's shortest comb is ~25 ms, so a reverb rendered with
/// a 10 ms tail is pure silence: the preset's `tail_secs` has to clear that
/// latency (the shipped reverb preset uses 2.5 s).
#[test]
fn tail_effects_render_region_plus_a_ringing_tail() {
    let d = stereo(vec![0.8; 200]); // 100 frames of loud audio
    for fx in [
        TailEffect::Reverb {
            room_size: 0.7,
            damp: 0.5,
            mix: 0.35,
            tail_secs: 0.2, // > the ~25 ms first reflection
        },
        TailEffect::Delay {
            time_secs: 0.001,
            feedback: 0.4,
            mix: 0.5,
            tail_secs: 0.05,
        },
    ] {
        let tail = fx.tail_frames(48_000);
        let out = fx.render(&d, tail);
        assert_eq!(out.frame_count(), 100 + tail, "{fx:?} region + tail");
        let tail_energy: f32 = out.samples()[100 * 2..].iter().map(|x| x * x).sum();
        assert!(tail_energy > 1e-6, "{fx:?} tail must ring out, got silence");
        assert!(out.samples().iter().all(|x| x.abs() <= 1.0 + 1e-4));
    }
}

/// A tail shorter than the effect's own latency yields silence — the failure
/// mode that makes a preset feel "broken". Pinned so nobody shortens the
/// reverb preset's `tail_secs` without noticing.
#[test]
fn reverb_tail_shorter_than_first_reflection_is_silent() {
    let d = stereo(vec![0.8; 200]);
    let fx = TailEffect::Reverb {
        room_size: 0.7,
        damp: 0.5,
        mix: 1.0,
        tail_secs: 0.01, // 10 ms < ~25 ms shortest comb
    };
    let out = fx.render(&d, fx.tail_frames(48_000));
    let energy: f32 = out.samples().iter().map(|x| x * x).sum();
    assert!(
        energy < 1e-6,
        "reverb has not emitted its first reflection yet"
    );
}

#[test]
fn delay_tail_echo_lands_at_the_delay_time() {
    // Mono impulse-ish region, fully wet, no feedback → one echo one tap later.
    let d = SampleData::from_interleaved(vec![1.0, 0.0, 0.0, 0.0], AudioFormat::mono(1_000));
    let fx = TailEffect::Delay {
        time_secs: 0.005, // 5 frames @ 1 kHz
        feedback: 0.0,
        mix: 1.0,
        tail_secs: 0.01, // 10 frames
    };
    let out = fx.render(&d, fx.tail_frames(1_000));
    assert_eq!(out.frame_count(), 14);
    // Frame 5 carries the echo of the impulse at frame 0.
    assert!(
        out.samples()[5] > 0.4,
        "echo at the tap: {:?}",
        out.samples()
    );
}
