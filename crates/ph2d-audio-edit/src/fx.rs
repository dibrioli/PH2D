//! Offline effect processors for the editor (W3 §6 rack). Two families, split by
//! whether the output outlives the input:
//!
//! - [`Effect`] — **length-preserving** (filters, dynamics, character). Routes
//!   through [`crate::in_range`] just like the W2 ops, so applying to a selection
//!   processes only the selected samples (filter/compressor state starts fresh at
//!   the region edge).
//! - [`TailEffect`] — **tail-extending** (reverb, delay). Cannot use the
//!   length-preserving splice: it routes through [`crate::in_range_tail`], which
//!   rings the tail out over the audio after the range and grows the clip when the
//!   range touches the end.
//!
//! Both are control-thread only, so HR-3/HR-5 do not apply (they allocate and use
//! `tanh`/`exp` freely). Filters, dynamics, reverb and delay reuse the shared
//! `ph2d_audio::dsp` kit; character effects (saturate/bitcrush/width) are local
//! pure math.

use ph2d_audio::SampleData;
use ph2d_audio::dsp::{Biquad, BiquadCoeffs, Compressor, Delay, Reverb};

/// Channel count of the clip (≥1).
fn channels(data: &SampleData) -> usize {
    data.format().channel_count().max(1)
}

// Every effect must be a **byte-identical no-op** at its neutral parameters, so
// the editor can select it (and audition it) without touching the audio until the
// user actually turns something. "Almost identity" is not enough: a filter at the
// top of its range still phase-shifts, and a 1:1 compressor still rounds. Each
// neutral point below is therefore an explicit bypass, not an emergent one.

/// A low-pass at or above this cutoff passes the whole audible band → bypass.
const LOWPASS_BYPASS_HZ: f32 = 20_000.0;
/// A high-pass at or below this cutoff passes the whole audible band → bypass.
const HIGHPASS_BYPASS_HZ: f32 = 20.0;
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
    /// Feed-forward compressor (glue / level) + linear `makeup` gain.
    Compress {
        /// Level (linear 0..1) above which reduction starts.
        threshold: f32,
        /// Ratio (≥1).
        ratio: f32,
        /// Attack time (seconds).
        attack_secs: f32,
        /// Release time (seconds).
        release_secs: f32,
        /// Post-compression make-up gain (linear).
        makeup: f32,
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
            // Ratio 1:1 reduces nothing — and would still round the samples.
            Effect::Compress {
                threshold,
                ratio,
                attack_secs,
                release_secs,
                makeup,
            } if ratio > 1.0 => compress(data, threshold, ratio, attack_secs, release_secs, makeup),
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

    /// Whether this effect is at its neutral point and [`Effect::apply`] would
    /// return the input untouched.
    pub fn is_bypass(&self) -> bool {
        matches!(
            *self,
            Effect::LowPass { cutoff, .. } if cutoff >= LOWPASS_BYPASS_HZ)
            || matches!(*self, Effect::HighPass { cutoff, .. } if cutoff <= HIGHPASS_BYPASS_HZ)
            || matches!(*self, Effect::Compress { ratio, .. } if ratio <= 1.0)
            || matches!(*self, Effect::Saturate { drive } if drive < SATURATE_BYPASS_DRIVE)
            || matches!(*self, Effect::Bitcrush { bits, downsample }
                if bits >= BITCRUSH_BYPASS_BITS && downsample <= 1)
            || matches!(*self, Effect::StereoWidth { width } if (width - 1.0).abs() <= f32::EPSILON)
    }
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

/// Drive a **wet-only** stereo processor (the `dsp` kit's contract) across the
/// region then `tail_frames` of silence, crossfading dry/wet by `mix`. Mono clips
/// feed the processor `(x, x)` and collapse its stereo wet back to one channel.
fn render_wet(
    data: &SampleData,
    tail_frames: usize,
    mix: f32,
    mut wet: impl FnMut(f32, f32) -> (f32, f32),
) -> SampleData {
    let ch = channels(data);
    let frames = data.frame_count();
    let src = data.samples();
    let mix = mix.clamp(0.0, 1.0);
    let dry_gain = 1.0 - mix;
    let mut out = Vec::with_capacity((frames + tail_frames) * ch);
    for f in 0..frames + tail_frames {
        // Past the region the dry signal is silence — the processor keeps ringing.
        let (dry_l, dry_r) = if f < frames {
            let b = f * ch;
            if ch >= 2 {
                (src[b], src[b + 1])
            } else {
                (src[b], src[b])
            }
        } else {
            (0.0, 0.0)
        };
        let (wet_l, wet_r) = wet(dry_l, dry_r);
        if ch >= 2 {
            out.push((dry_l * dry_gain + wet_l * mix).clamp(-1.0, 1.0));
            out.push((dry_r * dry_gain + wet_r * mix).clamp(-1.0, 1.0));
        } else {
            let w = (wet_l + wet_r) * 0.5;
            out.push((dry_l * dry_gain + w * mix).clamp(-1.0, 1.0));
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// Run an independent [`Biquad`] per channel over the interleaved buffer.
fn biquad_all(data: &SampleData, coeffs: BiquadCoeffs) -> SampleData {
    let ch = channels(data);
    let mut filters: Vec<Biquad> = (0..ch).map(|_| Biquad::new(coeffs)).collect();
    let frames = data.frame_count();
    let mut out = data.samples().to_vec();
    for f in 0..frames {
        for (c, filt) in filters.iter_mut().enumerate() {
            let i = f * ch + c;
            out[i] = filt.process(out[i]);
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// One-shot / per-sample coefficient for a `secs` time-constant at `sr` Hz.
fn time_coeff(secs: f32, sr: f32) -> f32 {
    if secs <= 0.0 {
        return 1.0;
    }
    1.0 - (-1.0 / (secs * sr)).exp()
}

/// Stereo-linked compression (mono is duplicated then re-collapsed) + make-up.
fn compress(
    data: &SampleData,
    threshold: f32,
    ratio: f32,
    attack_secs: f32,
    release_secs: f32,
    makeup: f32,
) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let frames = data.frame_count();
    let mut comp = Compressor::default();
    comp.set_params(
        true,
        threshold,
        ratio,
        time_coeff(attack_secs, sr),
        time_coeff(release_secs, sr),
    );
    let mut out = data.samples().to_vec();
    for f in 0..frames {
        let base = f * ch;
        if ch >= 2 {
            let (l, r) = comp.process(out[base], out[base + 1]);
            out[base] = (l * makeup).clamp(-1.0, 1.0);
            out[base + 1] = (r * makeup).clamp(-1.0, 1.0);
        } else {
            let (l, _) = comp.process(out[base], out[base]);
            out[base] = (l * makeup).clamp(-1.0, 1.0);
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// `tanh` soft-clip normalized so full-scale in ≈ full-scale out.
fn saturate(data: &SampleData, drive: f32) -> SampleData {
    let k = drive.max(0.1);
    let norm = 1.0 / k.tanh();
    let out: Vec<f32> = data
        .samples()
        .iter()
        .map(|&x| (k * x).tanh() * norm)
        .collect();
    SampleData::from_interleaved(out, data.format())
}

/// Quantize to `bits` levels + hold each channel's value for `downsample` frames.
fn bitcrush(data: &SampleData, bits: u32, downsample: u32) -> SampleData {
    let ch = channels(data);
    let frames = data.frame_count();
    let hold = downsample.max(1) as usize;
    // Levels for a signed range [-1, 1]: 2^bits steps.
    let levels = (1u32 << bits.clamp(1, 24)) as f32;
    let quant = |x: f32| ((x * 0.5 + 0.5) * levels).round() / levels * 2.0 - 1.0;
    let src = data.samples();
    let mut out = src.to_vec();
    let mut held = vec![0.0f32; ch];
    for f in 0..frames {
        for c in 0..ch {
            if f % hold == 0 {
                held[c] = quant(src[f * ch + c]);
            }
            out[f * ch + c] = held[c].clamp(-1.0, 1.0);
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// Mid/Side width on stereo clips; mono is returned unchanged.
fn stereo_width(data: &SampleData, width: f32) -> SampleData {
    let ch = channels(data);
    if ch < 2 {
        return data.clone();
    }
    let frames = data.frame_count();
    let mut out = data.samples().to_vec();
    for f in 0..frames {
        let base = f * ch;
        let (l, r) = (out[base], out[base + 1]);
        let mid = (l + r) * 0.5;
        let side = (l - r) * 0.5 * width;
        out[base] = (mid + side).clamp(-1.0, 1.0);
        out[base + 1] = (mid - side).clamp(-1.0, 1.0);
    }
    SampleData::from_interleaved(out, data.format())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    fn stereo(v: Vec<f32>) -> SampleData {
        SampleData::from_interleaved(v, AudioFormat::stereo(48_000))
    }

    #[test]
    fn effects_preserve_length() {
        let d = stereo(vec![0.3, -0.4, 0.5, -0.6, 0.7, -0.8]); // 3 frames
        for fx in [
            Effect::LowPass {
                cutoff: 3_000.0,
                q: 0.707,
            },
            Effect::HighPass {
                cutoff: 150.0,
                q: 0.707,
            },
            Effect::Compress {
                threshold: 0.3,
                ratio: 4.0,
                attack_secs: 0.005,
                release_secs: 0.1,
                makeup: 1.5,
            },
            Effect::Saturate { drive: 3.0 },
            Effect::Bitcrush {
                bits: 6,
                downsample: 4,
            },
            Effect::StereoWidth { width: 1.6 },
        ] {
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
        for fx in [
            Effect::LowPass {
                cutoff: 20_000.0,
                q: 0.707,
            },
            Effect::HighPass {
                cutoff: 20.0,
                q: 0.707,
            },
            Effect::Compress {
                threshold: 0.3,
                ratio: 1.0,
                attack_secs: 0.005,
                release_secs: 0.1,
                makeup: 1.0,
            },
            Effect::Saturate { drive: 0.0 },
            Effect::Bitcrush {
                bits: 16,
                downsample: 1,
            },
            Effect::StereoWidth { width: 1.0 },
        ] {
            assert!(fx.is_bypass(), "{fx:?} should be neutral");
            assert_eq!(
                fx.apply(&d).samples(),
                d.samples(),
                "{fx:?} altered the audio"
            );
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

    #[test]
    fn saturate_is_monotonic_and_bounded() {
        let d = stereo(vec![0.0, 0.1, 0.5, 0.9, -0.5, -0.9]);
        let s = saturate(&d, 4.0);
        // Never exceeds unity, and a bigger input stays bigger (monotone).
        assert!(s.samples().iter().all(|x| x.abs() <= 1.0 + 1e-4));
        assert!(s.samples()[2] < s.samples()[3], "0.5 -> < 0.9 mapping");
        // Sign preserved.
        assert!(s.samples()[3] > 0.0 && s.samples()[5] < 0.0);
    }

    #[test]
    fn stereo_width_zero_is_mono_sum() {
        // width 0 collapses to mid on both channels.
        let d = stereo(vec![1.0, 0.0, 0.0, 1.0]);
        let m = stereo_width(&d, 0.0);
        assert_eq!(m.samples(), &[0.5, 0.5, 0.5, 0.5]);
        // width 1 is a passthrough.
        let p = stereo_width(&d, 1.0);
        assert_eq!(p.samples(), d.samples());
    }

    #[test]
    fn bitcrush_quantizes_and_holds() {
        // 1-bit crush snaps to the rail; downsample holds frame 0 across frame 1.
        let d = SampleData::from_interleaved(vec![0.7, 0.1], AudioFormat::mono(48_000));
        let c = bitcrush(&d, 1, 2);
        assert_eq!(c.samples()[0], c.samples()[1], "held across the decimation");
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

    #[test]
    fn lowpass_attenuates_nyquist_more_than_dc() {
        // A DC-ish ramp vs. an alternating (near-Nyquist) signal: the low-pass must
        // keep DC energy and kill the alternation.
        let alt: Vec<f32> = (0..64)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let d = SampleData::from_interleaved(alt, AudioFormat::mono(48_000));
        let lp = biquad_all(&d, BiquadCoeffs::lowpass(48_000.0, 1_000.0, 0.707));
        let energy: f32 = lp.samples().iter().skip(16).map(|x| x * x).sum();
        assert!(energy < 8.0, "near-Nyquist content strongly attenuated");
    }
}
