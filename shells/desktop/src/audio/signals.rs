//! Built-in test-signal generators for the "Play Test" path — pure
//! `AudioFormat -> SampleData` waveform builders, split out of `audio.rs` to
//! keep it under the HR-18 600-LOC shell cap. Ungated (part of the always-on
//! audio system, exercised by the panel's Play Test button).

use ph2d_audio::{AudioFormat, SampleData};

/// A mono sine tone with a short raised-linear fade in/out (so it never clicks).
pub(super) fn sine_tone(format: AudioFormat, freq_hz: f32, secs: f32, gain: f32) -> SampleData {
    let rate = format.sample_rate as f32;
    let n = (secs * rate) as usize;
    let fade = ((0.01 * rate) as usize).clamp(1, (n / 2).max(1));
    let omega = std::f32::consts::TAU * freq_hz;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / rate;
        let mut a = (omega * t).sin() * gain;
        if i < fade {
            a *= i as f32 / fade as f32;
        } else if i >= n - fade {
            a *= (n - i) as f32 / fade as f32;
        }
        samples.push(a);
    }
    SampleData::from_interleaved(samples, AudioFormat::mono(format.sample_rate))
}

/// A looping plucked note: fast attack then decay to silence, so it loops as a
/// repeating pluck (silence at the seam = no click). The decay gives transients
/// that make the meter's peak-hold marker jump then fall.
pub(super) fn pluck_loop(format: AudioFormat, freq_hz: f32, secs: f32, gain: f32) -> SampleData {
    let rate = format.sample_rate as f32;
    let n = (secs * rate) as usize;
    let attack = ((0.005 * rate) as usize).clamp(1, (n / 2).max(1));
    let omega = std::f32::consts::TAU * freq_hz;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / rate;
        let env = if i < attack {
            i as f32 / attack as f32
        } else {
            let d = (i - attack) as f32 / (n - attack).max(1) as f32; // 0..1
            (1.0 - d) * (1.0 - d) // decays to 0 at the loop seam
        };
        samples.push((omega * t).sin() * env * gain);
    }
    SampleData::from_interleaved(samples, AudioFormat::mono(format.sample_rate))
}

/// A looping "voice" blip: a short tone burst (with a `sin(π·phase)` window so it
/// starts/ends silent) followed by silence, so the loop is a periodic blip —
/// enough of a stand-in for dialogue to demo ducking.
pub(super) fn blip_loop(
    format: AudioFormat,
    freq_hz: f32,
    burst_secs: f32,
    period_secs: f32,
    gain: f32,
) -> SampleData {
    let rate = format.sample_rate as f32;
    let n = (period_secs * rate) as usize;
    let burst = ((burst_secs * rate) as usize).min(n);
    let omega = std::f32::consts::TAU * freq_hz;
    let mut samples = vec![0.0f32; n];
    for (i, s) in samples.iter_mut().enumerate().take(burst) {
        let t = i as f32 / rate;
        let window = (std::f32::consts::PI * (i as f32 / burst.max(1) as f32)).sin();
        *s = (omega * t).sin() * window * gain;
    }
    SampleData::from_interleaved(samples, AudioFormat::mono(format.sample_rate))
}

/// A looping tone under a slow amplitude swell (`sin(π·phase)` hump): silent at
/// both ends so it loops seamlessly regardless of frequency, pulsing gently.
pub(super) fn swell_loop(format: AudioFormat, freq_hz: f32, secs: f32, gain: f32) -> SampleData {
    let rate = format.sample_rate as f32;
    let n = (secs * rate) as usize;
    let omega = std::f32::consts::TAU * freq_hz;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / rate;
        let phase = i as f32 / n.max(1) as f32;
        let env = (std::f32::consts::PI * phase).sin(); // 0 → 1 → 0
        samples.push((omega * t).sin() * env * gain);
    }
    SampleData::from_interleaved(samples, AudioFormat::mono(format.sample_rate))
}
