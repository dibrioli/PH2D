//! Desktop audio backend — cpal owns the output device; the core mixer
//! (`ph2d-audio`) stays platform-agnostic (HR-1).
//!
//! The control-side [`AudioEngine`] lives on the `App` (main thread); the
//! [`AudioRenderer`] is moved into cpal's audio-callback thread. `cpal` is
//! confined to this module — it never enters `ph2d-audio`. If no device is
//! available the editor runs silently (degrade-gracefully, like `gilrs`).

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use ph2d_audio::{
    AudioEngine, AudioFormat, AudioRenderer, BusId, PlayParams, SUB_BUS_COUNT, SampleData,
};

/// The desktop audio system: the control handle + the live output stream.
/// Dropping it closes the stream and stops audio.
pub(crate) struct AudioSystem {
    engine: AudioEngine,
    format: AudioFormat,
    /// Last master gain pushed to the engine — so the per-frame bridge only
    /// sends a command when it actually changes (else it floods the ring).
    last_master_gain: std::cell::Cell<f32>,
    /// Same change-gate for the master filter cutoff.
    last_cutoff: std::cell::Cell<f32>,
    /// Change-gate for the master balance.
    last_master_pan: std::cell::Cell<f32>,
    /// Same change-gate, per sub-bus fader (index-aligned with `BusId::SUB_BUSES`).
    last_bus_gain: [std::cell::Cell<f32>; SUB_BUS_COUNT],
    /// Same change-gate, per sub-bus balance.
    last_bus_pan: [std::cell::Cell<f32>; SUB_BUS_COUNT],
    // Kept alive for the app's lifetime; the callback (which owns the renderer)
    // runs on cpal's thread until this drops. `cpal::Stream` is `!Send` on ALSA,
    // which is fine — `App` never leaves the main thread.
    _stream: cpal::Stream,
}

impl AudioSystem {
    /// Open the default output device and start the mixer. Returns `None`
    /// (logged) if there is no device or an unsupported sample format, so the
    /// editor keeps running without sound rather than crashing.
    pub(crate) fn new() -> Option<AudioSystem> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        let name = device.name().unwrap_or_else(|_| "<unknown>".to_string());
        let supported = match device.default_output_config() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("audio: no default output config ({e}); running silent");
                return None;
            }
        };
        let sample_format = supported.sample_format();
        let config: cpal::StreamConfig = supported.config();
        let rate = config.sample_rate.0;
        let dev_channels = config.channels as usize;
        let format = if dev_channels == 1 {
            AudioFormat::mono(rate)
        } else {
            AudioFormat::stereo(rate)
        };
        let (engine, renderer) = AudioEngine::new(format);
        let our_channels = format.channel_count();

        let built = match sample_format {
            cpal::SampleFormat::F32 => {
                build_stream::<f32>(&device, &config, renderer, dev_channels, our_channels)
            }
            cpal::SampleFormat::I16 => {
                build_stream::<i16>(&device, &config, renderer, dev_channels, our_channels)
            }
            cpal::SampleFormat::U16 => {
                build_stream::<u16>(&device, &config, renderer, dev_channels, our_channels)
            }
            other => {
                eprintln!("audio: unsupported sample format {other:?}; running silent");
                return None;
            }
        };
        let stream = match built {
            Ok(s) => s,
            Err(e) => {
                eprintln!("audio: failed to build output stream ({e}); running silent");
                return None;
            }
        };
        if let Err(e) = stream.play() {
            eprintln!("audio: failed to start stream ({e}); running silent");
            return None;
        }
        println!("audio: {name} @ {rate} Hz, {dev_channels} ch, {sample_format:?}");
        Some(AudioSystem {
            engine,
            format,
            last_master_gain: std::cell::Cell::new(1.0),
            last_cutoff: std::cell::Cell::new(20_000.0),
            last_master_pan: std::cell::Cell::new(0.0),
            last_bus_gain: std::array::from_fn(|_| std::cell::Cell::new(1.0)),
            last_bus_pan: std::array::from_fn(|_| std::cell::Cell::new(0.0)),
            _stream: stream,
        })
    }

    /// Current master output peak levels `[L, R]` for the mixer meter.
    pub(crate) fn levels(&self) -> [f32; 2] {
        self.engine.levels()
    }

    /// Current post-fader peak levels per sub-bus, for the strip meters.
    pub(crate) fn bus_levels(&self) -> [[f32; 2]; SUB_BUS_COUNT] {
        self.engine.bus_levels()
    }

    /// Set sub-bus `i`'s fader gain, change-gated per bus (mute is folded in by
    /// the caller sending `0.0`, mirroring the master strip).
    pub(crate) fn set_bus_gain(&self, i: usize, gain: f32) {
        if let Some(cell) = self.last_bus_gain.get(i)
            && (gain - cell.get()).abs() > f32::EPSILON
        {
            let _ = self.engine.set_bus_gain(BusId::SUB_BUSES[i], gain);
            cell.set(gain);
        }
    }

    /// Set the master stereo balance, change-gated.
    pub(crate) fn set_master_pan(&self, pan: f32) {
        if (pan - self.last_master_pan.get()).abs() > f32::EPSILON {
            let _ = self.engine.set_bus_pan(BusId::Master, pan);
            self.last_master_pan.set(pan);
        }
    }

    /// Set sub-bus `i`'s stereo balance, change-gated per bus.
    pub(crate) fn set_bus_pan(&self, i: usize, pan: f32) {
        if let Some(cell) = self.last_bus_pan.get(i)
            && (pan - cell.get()).abs() > f32::EPSILON
        {
            let _ = self.engine.set_bus_pan(BusId::SUB_BUSES[i], pan);
            cell.set(pan);
        }
    }

    /// Set the engine's master gain, but only enqueue a command when it changed
    /// (called every frame by the mixer bridge — avoid flooding the ring).
    pub(crate) fn set_master_gain(&self, gain: f32) {
        if (gain - self.last_master_gain.get()).abs() > f32::EPSILON {
            let _ = self.engine.set_master_gain(gain);
            self.last_master_gain.set(gain);
        }
    }

    /// Set the master low-pass cutoff (Hz), change-gated like the gain.
    pub(crate) fn set_master_cutoff(&self, hz: f32) {
        if (hz - self.last_cutoff.get()).abs() > f32::EPSILON {
            let _ = self.engine.set_master_cutoff(hz);
            self.last_cutoff.set(hz);
        }
    }

    /// Per-frame housekeeping: drop finished-sample `Arc`s on the main thread
    /// so they don't free on the RT audio thread (HR-3).
    pub(crate) fn poll(&self) {
        self.engine.collect_returns();
    }

    /// Queue a short 440 Hz test tone — the `PH2D_AUDIO_SMOKE` beep that proves
    /// the control → audio → device path end to end.
    pub(crate) fn play_test_tone(&mut self) {
        let tone = sine_tone(self.format, 440.0, 0.6, 0.4);
        let params = PlayParams {
            bus: BusId::Sfx,
            ..PlayParams::default()
        };
        match self.engine.play(tone, params) {
            Ok(_) => println!("audio: playing 440 Hz test tone on the SFX bus (PH2D_AUDIO_SMOKE)"),
            Err(e) => eprintln!("audio: test tone dropped ({e})"),
        }
    }

    /// Decode and loop-play an audio file (the `PH2D_AUDIO_FILE` smoke). The
    /// clip's own sample rate is resampled to the device rate by the voice.
    pub(crate) fn play_file(&mut self, path: &std::path::Path) {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("audio: cannot read {}: {e}", path.display());
                return;
            }
        };
        let data = match ph2d_audio_decode::decode(&bytes) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("audio: decode failed for {}: {e}", path.display());
                return;
            }
        };
        let fmt = data.format();
        let secs = fmt.frames_to_secs(data.frame_count() as u64);
        let params = PlayParams {
            looping: true,
            bus: BusId::Music,
            ..PlayParams::default()
        };
        match self.engine.play(data, params) {
            Ok(_) => println!(
                "audio: looping {} on the Music bus ({secs:.1}s, {} Hz, {:?})",
                path.display(),
                fmt.sample_rate,
                fmt.channels
            ),
            Err(e) => eprintln!("audio: play failed ({e})"),
        }
    }
}

/// Build the output stream for device sample type `T`. The mixer renders into a
/// reused `f32` scratch (mono/stereo per `our_channels`), which is then
/// converted + scattered into the device's `dev_channels` layout. Mirrors the
/// cpal `beep.rs` reference (DIRETIVA §1) for the `T::from_sample` conversion.
fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut renderer: AudioRenderer,
    dev_channels: usize,
    our_channels: usize,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: SizedSample + FromSample<f32>,
{
    let err_fn = |e| eprintln!("audio: stream error: {e}");
    // Owned by the callback; sized once (when the block size stabilizes), then
    // reused — no allocation in the warm hot path (HR-3).
    let mut scratch: Vec<f32> = Vec::new();
    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let frames = data.len() / dev_channels.max(1);
            let needed = frames * our_channels;
            if scratch.len() != needed {
                scratch.resize(needed, 0.0);
            }
            renderer.render(&mut scratch, frames);
            for f in 0..frames {
                for c in 0..dev_channels {
                    let s = if our_channels == 1 {
                        scratch[f]
                    } else if c < 2 {
                        scratch[f * 2 + c]
                    } else {
                        0.0
                    };
                    data[f * dev_channels + c] = T::from_sample(s);
                }
            }
        },
        err_fn,
        None,
    )
}

/// A mono sine tone with a short raised-linear fade in/out (so it never clicks).
fn sine_tone(format: AudioFormat, freq_hz: f32, secs: f32, gain: f32) -> SampleData {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::ChannelLayout;

    #[test]
    fn sine_tone_is_faded_bounded_and_audible() {
        let d = sine_tone(AudioFormat::stereo(48_000), 440.0, 0.1, 0.4);
        assert_eq!(d.frame_count(), 4_800, "0.1 s @ 48 kHz");
        assert_eq!(d.format().channels, ChannelLayout::Mono);
        let s = d.samples();
        // Fades to silence at both ends (no click).
        assert!(s[0].abs() < 0.05, "fade-in starts near zero");
        assert!(s[s.len() - 1].abs() < 0.05, "fade-out ends near zero");
        // Never exceeds the requested gain, and actually oscillates.
        assert!(s.iter().all(|&x| x.abs() <= 0.4 + 1e-4), "within gain");
        assert!(s.iter().any(|&x| x.abs() > 0.2), "tone has real amplitude");
    }
}
