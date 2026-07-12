//! Convolution reverb — put the sound in a **specific** room.
//!
//! The rack's other reverb (Freeverb) is *algorithmic*: a bank of comb and all-pass filters
//! tuned until it sounds like a plausible space. This one is a *measurement*. An impulse
//! response is what a real room did to a starter pistol — the early reflections off its
//! pillars, the diffuse tail, the particular way its stone eats the highs before the lows —
//! and convolving a dry sound with it puts that sound **in that room**. Not in an
//! approximation of one: in the cathedral you recorded.
//!
//! ## The one thing that has to be right: the IR is the tail
//!
//! Every other tail effect in the rack takes a `tail_secs` knob and rings out for that long.
//! This one does not, and it would be wrong to give it one: the impulse response *is* the
//! ring-out. Its length is how long that room takes to go quiet. Truncating it to a knob's
//! value would cut the cathedral off mid-decay — the reverb would stop, rather than end.
//!
//! So [`TailEffect::tail_frames`] for this variant is derived from the IR, and the clip grows
//! by exactly the room's own tail.
//!
//! The heavy lifting is in `ph2d-audio-spectral::convolve` (FFT overlap-add — the direct sum
//! is O(n·m) and would take minutes). That is the second thing the FFT dependency bought, and
//! it needed nothing new to buy it.

use std::sync::Arc;

use ph2d_audio::SampleData;

use crate::ops::channels;

/// One channel of the impulse response, deinterleaved.
///
/// A **mono** IR is one room heard from one point, and both channels of the clip go through
/// it. A **stereo** IR is the room heard in stereo — left and right captured separately — and
/// each channel goes through its own, which is what preserves the width of the space. An IR
/// with more channels than the clip has: the extras are ignored, and channel 0 is reused for
/// anything the IR does not cover, because a missing room is worse than a mismatched one.
fn ir_channel(ir: &[f32], ir_channels: usize, want: usize) -> Vec<f32> {
    let ch = ir_channels.max(1);
    let pick = if want < ch { want } else { 0 };
    ir.iter().skip(pick).step_by(ch).copied().collect()
}

/// Resample an impulse response to the clip's rate.
///
/// A room recorded at 44.1 kHz and convolved into a 48 kHz clip is a room **8 % wrong** — its
/// tail runs short and every resonance sits 8 % sharp. Nobody would hear that as a bug; they
/// would hear it as "this IR sounds a bit off", which is worse, because it is unfalsifiable.
///
/// Linear interpolation is enough here: an impulse response is broadband noise-like data, and
/// the artefacts of a linear resample sit far below the room's own diffuse tail. (The pitch
/// shifter next door needs better; this does not.)
fn resample(x: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || from == 0 || x.is_empty() {
        return x.to_vec();
    }
    let ratio = f64::from(to) / f64::from(from);
    let n = ((x.len() as f64) * ratio).round() as usize;
    (0..n)
        .map(|i| {
            let src = i as f64 / ratio;
            let j = src.floor() as usize;
            let t = (src - j as f64) as f32;
            let a = x.get(j).copied().unwrap_or(0.0);
            let b = x.get(j + 1).copied().unwrap_or(a);
            a + (b - a) * t
        })
        .collect()
}

/// Frames in an impulse response.
pub(super) fn ir_frames(ir: &[f32], ir_channels: usize) -> usize {
    ir.len() / ir_channels.max(1)
}

/// Convolve `data` with `ir`, crossfading dry→wet by `mix` and ringing out for `tail_frames`.
///
/// Mirrors `space::render_wet`'s contract exactly: inside the region the output is
/// `dry·(1−mix) + wet·mix`; past it the dry signal has ended, so only the room is left.
pub(super) fn render_convolution(
    data: &SampleData,
    ir: &Arc<[f32]>,
    ir_channels: usize,
    ir_rate: u32,
    mix: f32,
    tail_frames: usize,
) -> SampleData {
    let ch = channels(data);
    let frames = data.frame_count();
    let total = frames + tail_frames;
    let mix = mix.clamp(0.0, 1.0);
    let dry_gain = 1.0 - mix;
    let src = data.samples();

    // One allocation, not two (ADR-0117 D2). `build` and not `map_in_place`: the ring-out makes
    // the output LONGER than the input, so there is no input buffer to rewrite — the IR is the
    // tail, and the tail is new audio.
    SampleData::build(total * ch, data.format(), |out| {
        for c in 0..ch {
            let dry: Vec<f32> = (0..frames).map(|f| src[f * ch + c]).collect();
            // Unity-gain: a raw IR is a recording at whatever level the microphone saw, and an
            // unnormalised one would make the Mix knob a volume knob in disguise.
            let raw = resample(
                &ir_channel(ir, ir_channels, c),
                ir_rate,
                data.format().sample_rate,
            );
            let h = ph2d_audio_spectral::normalize_ir(&raw);
            let wet = ph2d_audio_spectral::convolve(&dry, &h);
            for f in 0..total {
                let d = if f < frames { dry[f] } else { 0.0 };
                let w = wet.get(f).copied().unwrap_or(0.0);
                out[f * ch + c] = (d * dry_gain + w * mix).clamp(-1.0, 1.0);
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::TailEffect;
    use ph2d_audio::{AudioFormat, ChannelLayout};

    const SR: f32 = 48_000.0;

    fn clip(n: usize) -> SampleData {
        let s: Vec<f32> = (0..n * 2)
            .map(|i| (std::f32::consts::TAU * 220.0 * (i as f32 / (SR * 2.0))).sin() * 0.5)
            .collect();
        SampleData::from_interleaved(
            s,
            AudioFormat {
                sample_rate: 48_000,
                channels: ChannelLayout::Stereo,
            },
        )
    }

    /// A decaying-noise IR — the shape of a real room's response.
    fn room(frames: usize) -> Arc<[f32]> {
        let mut s = 0x5EEDu64;
        let v: Vec<f32> = (0..frames)
            .map(|i| {
                s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let r = (s >> 40) as f32 / 8_388_608.0 - 1.0;
                r * (-(i as f32) / (frames as f32 * 0.3)).exp()
            })
            .collect();
        v.into()
    }

    fn fx(ir: Arc<[f32]>, mix: f32) -> TailEffect {
        TailEffect::Convolution {
            ir,
            ir_channels: 1,
            ir_rate: 48_000,
            mix,
        }
    }

    /// **A room recorded at another rate is resampled into this one.** Convolving a 44.1 kHz
    /// IR straight into a 48 kHz clip gives a room 8 % wrong — short tail, sharp resonances —
    /// which nobody hears as a bug, only as "this IR sounds off". The tail length is the
    /// visible half of that, and it is what this pins.
    #[test]
    fn an_ir_at_another_rate_is_resampled_into_the_clip() {
        let ir = room(44_100); // exactly 1 s at 44.1 kHz
        let f = TailEffect::Convolution {
            ir,
            ir_channels: 1,
            ir_rate: 44_100,
            mix: 1.0,
        };
        // At 48 kHz the room still has to last one second: ~48 000 frames, not 44 100.
        let tail = f.tail_frames(48_000);
        let data = clip(24_000);
        let out = f.render(&data, tail);
        let grew = out.frame_count() - data.frame_count();
        assert!(
            (47_000..49_500).contains(&grew),
            "a 1 s room at 44.1 kHz rang out for {grew} frames at 48 kHz — it should be ~48 000"
        );
    }

    /// **The IR is the tail.** The clip grows by exactly the room's own ring-out — not by
    /// whatever a knob says, because the room decides how long it takes to go quiet.
    #[test]
    fn the_clip_grows_by_the_rooms_own_tail() {
        let ir = room(12_000);
        let f = fx(ir, 0.5);
        let tail = f.tail_frames(48_000);
        assert_eq!(
            tail,
            12_000 - 1,
            "the tail is not the impulse response's length"
        );

        let data = clip(24_000);
        let out = f.render(&data, tail);
        assert_eq!(out.frame_count(), 24_000 + tail);
    }

    /// **Mix 0 is a byte-identical bypass, and it does not even ring out.** The rack's
    /// non-negotiable neutral point — and for a tail effect it has a second half: a dry
    /// reverb that still appended a silent tail would silently lengthen the clip.
    #[test]
    fn mix_zero_is_byte_identical_and_grows_nothing() {
        let f = fx(room(12_000), 0.0);
        assert!(f.is_bypass());
        assert_eq!(f.tail_frames(48_000), 0);
        let data = clip(4_800);
        let out = f.render(&data, 0);
        assert_eq!(data.samples(), out.samples());
    }

    /// An IR that is *nothing* is not a room, and the effect refuses to pretend: no IR loaded
    /// means bypass, whatever the Mix knob says. (Without this, turning Mix up with no IR
    /// would convolve with an empty buffer and silence the clip.)
    #[test]
    fn no_impulse_response_is_a_bypass_however_wet_the_mix() {
        let empty: Arc<[f32]> = Vec::new().into();
        let f = fx(empty, 1.0);
        assert!(f.is_bypass(), "a convolution with no room is not a reverb");
        assert_eq!(f.tail_frames(48_000), 0);
        let data = clip(4_800);
        assert_eq!(data.samples(), f.render(&data, 0).samples());
    }

    /// **The room actually rings.** After the input stops, the output keeps sounding — which
    /// is the entire proposition. A convolution that produced silence past the region would
    /// be a filter wearing a reverb's name.
    #[test]
    fn the_room_keeps_sounding_after_the_sound_stops() {
        let ir = room(12_000);
        let f = fx(ir, 1.0);
        let tail = f.tail_frames(48_000);
        let data = clip(24_000);
        let out = f.render(&data, tail);
        let s = out.samples();
        let energy: f32 = s[24_000 * 2..].iter().map(|v| v * v).sum();
        assert!(
            energy > 1e-4,
            "the room went silent the instant the sound did"
        );
    }

    /// A **fully wet** convolution is the room, and nothing of the dry sound is left dry: the
    /// output must differ from the input inside the region too, not merely after it.
    #[test]
    fn wet_changes_the_sound_inside_the_region_as_well() {
        let ir = room(8_000);
        let f = fx(ir, 1.0);
        let tail = f.tail_frames(48_000);
        let data = clip(24_000);
        let out = f.render(&data, tail);
        let moved = data
            .samples()
            .iter()
            .zip(out.samples())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            moved > 0.01,
            "a fully wet room left the sound alone: {moved}"
        );
    }

    /// A **stereo** IR gives each channel its own room — that is what a stereo capture is for,
    /// and collapsing it to mono would throw away the width of the space.
    ///
    /// Built so the two channels are unmistakably different (left is an impulse at 100, right
    /// at 3000), then checked that the left output leads the right.
    #[test]
    fn a_stereo_ir_puts_a_different_room_on_each_channel() {
        let mut v = vec![0.0f32; 8_000 * 2];
        v[100 * 2] = 1.0; // left: an early reflection
        v[3_000 * 2 + 1] = 1.0; // right: a late one
        let f = TailEffect::Convolution {
            ir: v.into(),
            ir_channels: 2,
            ir_rate: 48_000,
            mix: 1.0,
        };
        let tail = f.tail_frames(48_000);
        let data = clip(24_000);
        let out = f.render(&data, tail);
        let s = out.samples();
        // Around frame 500 the left channel has already been struck (its impulse was at 100);
        // the right has not (its impulse is at 3000).
        let l: f32 = (400..600).map(|f| s[f * 2].abs()).sum();
        let r: f32 = (400..600).map(|f| s[f * 2 + 1].abs()).sum();
        assert!(
            l > r * 10.0,
            "the two channels went through the same room: L {l:.3} vs R {r:.3}"
        );
    }
}
