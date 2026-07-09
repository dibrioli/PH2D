//! Offline effect processors for the editor (W3 §6 rack). Two families, split by
//! whether the output outlives the input:
//!
//! - [`Effect`] — **length-preserving** (filters, EQ, dynamics, character). Routes
//!   through [`crate::in_range_warm`] just like the W2 ops, so applying to a
//!   selection processes only the selected samples — with a pre-roll of the audio
//!   before it, so a filter enters the region already settled instead of clicking.
//! - [`TailEffect`] — **tail-extending** (reverb, delay). Cannot use the
//!   length-preserving splice: it routes through [`crate::in_range_tail`], which
//!   rings the tail out over the audio after the range and grows the clip when the
//!   range touches the end.
//!
//! Both are control-thread only, so HR-3/HR-5 do not apply (they allocate and use
//! `tanh`/`exp` freely). Filters, dynamics, reverb and delay reuse the shared
//! `ph2d_audio::dsp` kit; character effects (saturate/bitcrush/width) and the
//! true-peak limiter are local.
//!
//! Implementations live in the sibling modules ([`tone`], [`dynamics`], [`space`]);
//! this file is the two enums and the neutral points they promise.

mod dynamics;
mod space;
mod tone;

use ph2d_audio::SampleData;
use ph2d_audio::dsp::{BiquadCoeffs, Delay, Reverb};

use dynamics::{compress, limit};
use space::render_wet;
use tone::{biquad_all, bitcrush, saturate, stereo_width};

// Every effect must be a **byte-identical no-op** at its neutral parameters, so
// the editor can select it (and audition it) without touching the audio until the
// user actually turns something. "Almost identity" is not enough: a filter at the
// top of its range still phase-shifts, and a 1:1 compressor still rounds. Each
// neutral point below is therefore an explicit bypass, not an emergent one.

/// A low-pass at or above this cutoff passes the whole audible band → bypass.
const LOWPASS_BYPASS_HZ: f32 = 20_000.0;
/// A high-pass at or below this cutoff passes the whole audible band → bypass.
const HIGHPASS_BYPASS_HZ: f32 = 20.0;
/// An EQ band this close to flat is inaudible — and a "0 dB" RBJ section is only
/// *algebraically* an identity: its coefficients still round every sample.
const EQ_BYPASS_GAIN_DB: f32 = 1e-3;
/// A limiter whose ceiling is at (or above) full scale has nothing to catch that a
/// `[-1, 1]` buffer could contain in its samples. Dial it below 0 dBFS to work —
/// the mastering convention is −1 dBTP.
const LIMITER_BYPASS_CEILING_DB: f32 = 0.0;
/// Drive below this is inaudible (and `tanh(k)` underflows the normalizer).
const SATURATE_BYPASS_DRIVE: f32 = 1e-3;
/// Bit depth at or above this, with no decimation, is transparent → bypass.
const BITCRUSH_BYPASS_BITS: u32 = 16;

/// A single length-preserving offline effect. Each variant has a **neutral point**
/// at which [`Effect::apply`] returns its input untouched (see
/// [`Effect::is_bypass`]) — that is where the editor's sliders start, so selecting
/// an effect never alters the audio until the user turns something.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    /// 2nd-order low-pass (muffle / warm) at `cutoff` Hz, `q` resonance.
    LowPass { cutoff: f32, q: f32 },
    /// 2nd-order high-pass (thin / de-rumble) at `cutoff` Hz, `q` resonance.
    HighPass { cutoff: f32, q: f32 },
    /// Peaking bell: `gain_db` boost/cut centred on `freq` Hz, width set by `q`.
    /// Leaves DC and Nyquist alone. Neutral at `gain_db` 0.
    Peak { freq: f32, q: f32, gain_db: f32 },
    /// Low shelf: `gain_db` applied below the `freq` Hz corner. Neutral at 0 dB.
    LowShelf { freq: f32, q: f32, gain_db: f32 },
    /// High shelf: `gain_db` applied above the `freq` Hz corner. Neutral at 0 dB.
    HighShelf { freq: f32, q: f32, gain_db: f32 },
    /// Feed-forward compressor (glue / level). Make-up gain is **automatic and
    /// peak-preserving**: the compressed region is scaled back so its peak matches
    /// the input's, so raising `ratio` reduces the dynamic range (quiet parts come
    /// up) without ever raising the waveform's amplitude. Neutral at `ratio` 1.
    Compress {
        /// Level (linear 0..1) above which reduction starts.
        threshold: f32,
        /// Ratio (≥1).
        ratio: f32,
        /// Attack time (seconds).
        attack_secs: f32,
        /// Release time (seconds).
        release_secs: f32,
    },
    /// Look-ahead **true-peak** limiter: holds the *reconstructed* waveform under
    /// `ceiling_db`, not just the samples. Neutral at 0 dBFS.
    Limiter {
        /// Ceiling in dBFS (≤ 0). `−1.0` is the mastering convention.
        ceiling_db: f32,
        /// Look-ahead and recovery radius, in seconds.
        release_secs: f32,
    },
    /// `tanh` soft-clip saturation (warmth / drive). Neutral at `drive` 0.
    Saturate { drive: f32 },
    /// Lo-fi bit-depth reduction to `bits` + sample-hold decimation by
    /// `downsample` (≥1 = no decimation).
    Bitcrush { bits: u32, downsample: u32 },
    /// Mid/Side stereo width (`1.0` = unchanged, `>1` wider, `0` mono). No-op on
    /// mono clips.
    StereoWidth { width: f32 },
}

impl Effect {
    /// Render `data` through this effect, returning a fresh, **same-length** clip.
    pub fn apply(&self, data: &SampleData) -> SampleData {
        let sr = data.format().sample_rate as f32;
        match *self {
            // Each arm short-circuits at its neutral point (see the bypass consts):
            // the returned clip is then the SAME `Arc<[f32]>`, not a re-render.
            Effect::LowPass { cutoff, q } if cutoff < LOWPASS_BYPASS_HZ => {
                biquad_all(data, BiquadCoeffs::lowpass(sr, cutoff, q))
            }
            Effect::HighPass { cutoff, q } if cutoff > HIGHPASS_BYPASS_HZ => {
                biquad_all(data, BiquadCoeffs::highpass(sr, cutoff, q))
            }
            Effect::Peak { freq, q, gain_db } if gain_db.abs() > EQ_BYPASS_GAIN_DB => {
                biquad_all(data, BiquadCoeffs::peak(sr, freq, q, gain_db))
            }
            Effect::LowShelf { freq, q, gain_db } if gain_db.abs() > EQ_BYPASS_GAIN_DB => {
                biquad_all(data, BiquadCoeffs::lowshelf(sr, freq, q, gain_db))
            }
            Effect::HighShelf { freq, q, gain_db } if gain_db.abs() > EQ_BYPASS_GAIN_DB => {
                biquad_all(data, BiquadCoeffs::highshelf(sr, freq, q, gain_db))
            }
            // Ratio 1:1 reduces nothing — and would still round the samples.
            Effect::Compress {
                threshold,
                ratio,
                attack_secs,
                release_secs,
            } if ratio > 1.0 => compress(data, threshold, ratio, attack_secs, release_secs),
            Effect::Limiter {
                ceiling_db,
                release_secs,
            } if ceiling_db < LIMITER_BYPASS_CEILING_DB => limit(data, ceiling_db, release_secs),
            Effect::Saturate { drive } if drive >= SATURATE_BYPASS_DRIVE => saturate(data, drive),
            Effect::Bitcrush { bits, downsample }
                if bits < BITCRUSH_BYPASS_BITS || downsample > 1 =>
            {
                bitcrush(data, bits, downsample)
            }
            // Width 1.0 is algebraically identity, but `mid ± side` still rounds.
            Effect::StereoWidth { width } if (width - 1.0).abs() > f32::EPSILON => {
                stereo_width(data, width)
            }
            _ => data.clone(),
        }
    }

    /// Frames of *preceding* audio this effect needs to enter a mid-clip region
    /// already settled — see [`crate::in_range_warm`]. Zero for memoryless effects
    /// and for the compressor, which settles its own envelope via `prime()`.
    ///
    /// A 2nd-order section rings for roughly `Q / (π·f0)` seconds per time
    /// constant, so a low cutoff at high `Q` needs a long pre-roll; the cap keeps
    /// the extra render bounded. The limiter needs its whole look-ahead window, or
    /// its gain curve starts flat at 1.0 and lets the region's first peak through.
    pub fn warmup_frames(&self, sample_rate: u32) -> usize {
        /// Time constants of pre-roll: `1 - e^-8` ≈ 0.9997 of the way settled.
        const TAUS: f32 = 8.0;
        if self.is_bypass() {
            return 0;
        }
        let cap = sample_rate as usize; // 1 s
        match *self {
            Effect::LowPass { cutoff, q } | Effect::HighPass { cutoff, q } => {
                biquad_warmup(cutoff, q, sample_rate, TAUS).min(cap)
            }
            Effect::Peak { freq, q, .. }
            | Effect::LowShelf { freq, q, .. }
            | Effect::HighShelf { freq, q, .. } => {
                biquad_warmup(freq, q, sample_rate, TAUS).min(cap)
            }
            Effect::Limiter { release_secs, .. } => {
                ((release_secs.max(0.0) * sample_rate as f32) as usize).min(cap)
            }
            _ => 0,
        }
    }

    /// Whether this effect is at its neutral point and [`Effect::apply`] would
    /// return the input untouched.
    pub fn is_bypass(&self) -> bool {
        matches!(
            *self,
            Effect::LowPass { cutoff, .. } if cutoff >= LOWPASS_BYPASS_HZ)
            || matches!(*self, Effect::HighPass { cutoff, .. } if cutoff <= HIGHPASS_BYPASS_HZ)
            || matches!(
                *self,
                Effect::Peak { gain_db, .. }
                    | Effect::LowShelf { gain_db, .. }
                    | Effect::HighShelf { gain_db, .. }
                    if gain_db.abs() <= EQ_BYPASS_GAIN_DB)
            || matches!(*self, Effect::Compress { ratio, .. } if ratio <= 1.0)
            || matches!(*self, Effect::Limiter { ceiling_db, .. }
                if ceiling_db >= LIMITER_BYPASS_CEILING_DB)
            || matches!(*self, Effect::Saturate { drive } if drive < SATURATE_BYPASS_DRIVE)
            || matches!(*self, Effect::Bitcrush { bits, downsample }
                if bits >= BITCRUSH_BYPASS_BITS && downsample <= 1)
            || matches!(*self, Effect::StereoWidth { width } if (width - 1.0).abs() <= f32::EPSILON)
    }
}

/// Frames a 2nd-order section needs to settle: `TAUS` time constants of `Q/(π·f0)`.
fn biquad_warmup(freq: f32, q: f32, sample_rate: u32, taus: f32) -> usize {
    let tau_frames = q.max(0.1) * sample_rate as f32 / (std::f32::consts::PI * freq.max(1.0));
    (taus * tau_frames) as usize
}

/// A **tail-extending** offline effect: its output rings on after the input stops,
/// so it renders `region + tail` frames and splices via [`crate::in_range_tail`]
/// (never [`crate::in_range`], which would truncate the tail).
///
/// `mix` crossfades dry→wet inside the region (`0` = dry, `1` = fully wet); the
/// tail is pure wet, since the dry signal has ended there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TailEffect {
    /// Stereo Freeverb room reverb.
    Reverb {
        /// Decay length (0..1).
        room_size: f32,
        /// High-frequency absorption (0..1).
        damp: f32,
        /// Dry→wet crossfade (0..1).
        mix: f32,
        /// Ring-out rendered past the region, in seconds.
        tail_secs: f32,
    },
    /// Feedback delay (echo). `time_secs` is clamped to the kit's 1 s line.
    Delay {
        /// Echo tap time (seconds, < 1.0).
        time_secs: f32,
        /// Repeat feedback (0..1, clamped below unity so echoes decay).
        feedback: f32,
        /// Dry→wet crossfade (0..1).
        mix: f32,
        /// Ring-out rendered past the region, in seconds.
        tail_secs: f32,
    },
}

impl TailEffect {
    /// Dry→wet crossfade of either variant.
    fn mix(&self) -> f32 {
        match *self {
            TailEffect::Reverb { mix, .. } | TailEffect::Delay { mix, .. } => mix,
        }
    }

    /// Whether this effect is at its neutral point (fully dry). Then it must NOT
    /// even ring out: appending a silent tail would lengthen the clip for nothing.
    pub fn is_bypass(&self) -> bool {
        self.mix() <= 0.0
    }

    /// How many frames of ring-out this effect needs at `sample_rate`. **Zero when
    /// bypassed**, which is what keeps a fully-dry reverb from growing the clip.
    pub fn tail_frames(&self, sample_rate: u32) -> usize {
        if self.is_bypass() {
            return 0;
        }
        let secs = match *self {
            TailEffect::Reverb { tail_secs, .. } | TailEffect::Delay { tail_secs, .. } => tail_secs,
        };
        (secs.max(0.0) * sample_rate as f32) as usize
    }

    /// Render `data` followed by `tail_frames` of ring-out. Always returns
    /// `data.frame_count() + tail_frames` frames.
    pub fn render(&self, data: &SampleData, tail_frames: usize) -> SampleData {
        if self.is_bypass() {
            return data.clone(); // fully dry: `tail_frames` is 0, so lengths match
        }
        let sr = data.format().sample_rate;
        match *self {
            TailEffect::Reverb {
                room_size,
                damp,
                mix,
                ..
            } => {
                let mut rv = Reverb::new(sr);
                rv.set_params(room_size, damp);
                render_wet(data, tail_frames, mix, move |l, r| rv.process(l, r))
            }
            TailEffect::Delay {
                time_secs,
                feedback,
                mix,
                ..
            } => {
                let mut dl = Delay::new(sr);
                dl.set_params(time_secs, feedback);
                render_wet(data, tail_frames, mix, move |l, r| dl.process(l, r))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    fn stereo(v: Vec<f32>) -> SampleData {
        SampleData::from_interleaved(v, AudioFormat::stereo(48_000))
    }

    /// Every non-neutral `Effect`, for the surface-level invariants below. Keep in
    /// sync with the enum — a variant missing here is a variant nobody proved
    /// length-preserving.
    fn tuned_effects() -> [Effect; 10] {
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
        ]
    }

    /// Every neutral `Effect`. Keep in sync with `is_bypass`.
    fn neutral_effects() -> [Effect; 10] {
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
}
