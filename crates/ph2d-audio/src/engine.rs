//! The two public halves of the engine, split along the thread boundary:
//! [`AudioEngine`] (control thread — allocates, returns handles) and
//! [`AudioRenderer`] (audio thread — `Send`, no-alloc, no-free).
//!
//! [`AudioEngine::new`] builds both and the rings between them; the caller keeps
//! the engine on the game thread and ships the renderer to the shell's cpal
//! callback, which calls [`AudioRenderer::render`] each block.

use std::sync::Arc;

use crate::AudioError;
use crate::buffer::{MixScratch, SampleData};
use crate::command::{AudioCommand, AudioReturn, Consumer, PlayParams, Producer, ring};
use crate::dsp::BiquadCoeffs;
use crate::format::{AudioFormat, ChannelLayout, Sample};
use crate::meter::AudioMeter;
use crate::mixer::Mixer;
use crate::voice::VoiceId;
use crate::{CMD_CAPACITY, MAX_VOICES, RETURN_CAPACITY};

/// Control-side handle. Lives on the game thread; every method just enqueues a
/// command (allocation happens here, never on the audio thread). Returns opaque
/// [`VoiceId`]s (HR-8).
pub struct AudioEngine {
    commands: Producer<AudioCommand>,
    returns: Consumer<AudioReturn>,
    meter: Arc<AudioMeter>,
    next_id: u64,
    format: AudioFormat,
}

impl AudioEngine {
    /// Build the engine + its renderer + the rings between them.
    pub fn new(format: AudioFormat) -> (AudioEngine, AudioRenderer) {
        let (cmd_tx, cmd_rx) = ring::<AudioCommand>(CMD_CAPACITY);
        let (ret_tx, ret_rx) = ring::<AudioReturn>(RETURN_CAPACITY);
        let meter = Arc::new(AudioMeter::default());
        let engine = AudioEngine {
            commands: cmd_tx,
            returns: ret_rx,
            meter: Arc::clone(&meter),
            next_id: 0,
            format,
        };
        let renderer = AudioRenderer::new(format, MAX_VOICES, cmd_rx, ret_tx, meter);
        (engine, renderer)
    }

    /// The most recent output block's peak level as `[left, right]` (0.0 =
    /// silence; > 1.0 = clipping). For metering UIs — read once per frame.
    pub fn levels(&self) -> [f32; 2] {
        self.meter.peaks()
    }

    /// The output format the renderer was built for.
    pub fn format(&self) -> AudioFormat {
        self.format
    }

    fn send(&self, cmd: AudioCommand) -> Result<(), AudioError> {
        self.commands
            .push(cmd)
            .map_err(|_| AudioError::QueueFull(self.commands.capacity()))
    }

    /// Start playing `data` under `params`, returning its handle. `Err` only if
    /// the command ring is full (the sample is dropped here, on the control
    /// thread). The returned id can be used to stop/modify the voice until it
    /// ends on its own.
    pub fn play(&mut self, data: SampleData, params: PlayParams) -> Result<VoiceId, AudioError> {
        self.next_id += 1;
        let id = VoiceId(self.next_id);
        self.send(AudioCommand::Play {
            voice: id,
            data,
            params,
        })?;
        Ok(id)
    }

    /// Stop a voice immediately (no envelope release).
    pub fn stop(&self, voice: VoiceId) -> Result<(), AudioError> {
        self.send(AudioCommand::Stop { voice })
    }

    /// Trigger the voice's envelope release (note-off); it fades then ends.
    pub fn release(&self, voice: VoiceId) -> Result<(), AudioError> {
        self.send(AudioCommand::Release { voice })
    }

    /// Set a voice's target gain (smoothed).
    pub fn set_voice_gain(&self, voice: VoiceId, gain: f32) -> Result<(), AudioError> {
        self.send(AudioCommand::SetVoiceGain { voice, gain })
    }

    /// Set a voice's stereo pan (`-1.0..=1.0`).
    pub fn set_voice_pan(&self, voice: VoiceId, pan: f32) -> Result<(), AudioError> {
        self.send(AudioCommand::SetVoicePan { voice, pan })
    }

    /// Set the master output gain (smoothed).
    pub fn set_master_gain(&self, gain: f32) -> Result<(), AudioError> {
        self.send(AudioCommand::SetMasterGain { gain })
    }

    /// Set the master low-pass filter cutoff in Hz. At/near Nyquist the filter
    /// is effectively open, sent as identity (true bypass). Coefficients are
    /// computed here on the control thread — no transcendentals on the RT thread.
    pub fn set_master_cutoff(&self, cutoff_hz: f32) -> Result<(), AudioError> {
        let sr = self.format.sample_rate as f32;
        let coeffs = if cutoff_hz >= sr * 0.5 * 0.9 {
            BiquadCoeffs::identity()
        } else {
            BiquadCoeffs::lowpass(sr, cutoff_hz.max(20.0), std::f32::consts::FRAC_1_SQRT_2)
        };
        self.send(AudioCommand::SetMasterFilter { coeffs })
    }

    /// Drain and drop finished samples returned by the audio thread. Call once
    /// per game frame so their `Arc`s free on the control thread, not the RT one.
    pub fn collect_returns(&self) {
        while let Some(AudioReturn::FinishedSample(_sample)) = self.returns.pop() {
            // Dropping `_sample` here frees its `Arc` on the control thread (HR-3).
        }
    }

    /// Number of finished-sample returns awaiting [`AudioEngine::collect_returns`].
    pub fn pending_returns(&self) -> usize {
        self.returns.len()
    }
}

/// Audio-side renderer. `Send`, so the shell moves it to the cpal callback
/// thread. [`AudioRenderer::render`] is the hot path: no allocation, no free.
pub struct AudioRenderer {
    mixer: Mixer,
    commands: Consumer<AudioCommand>,
    returns: Producer<AudioReturn>,
    meter: Arc<AudioMeter>,
    scratch: MixScratch,
    format: AudioFormat,
}

impl AudioRenderer {
    fn new(
        format: AudioFormat,
        max_voices: usize,
        commands: Consumer<AudioCommand>,
        returns: Producer<AudioReturn>,
        meter: Arc<AudioMeter>,
    ) -> Self {
        Self {
            mixer: Mixer::new(format, max_voices),
            commands,
            returns,
            meter,
            scratch: MixScratch::new(),
            format,
        }
    }

    /// Fill `out` (interleaved, matching the engine format) with `frames` frames
    /// of mixed audio. Called from the device callback. Drains control commands,
    /// mixes voices, applies master gain, clamps, and returns finished samples to
    /// the control thread. HR-3: zero allocation once warm.
    pub fn render(&mut self, out: &mut [Sample], frames: usize) {
        let Self {
            mixer,
            commands,
            returns,
            meter,
            scratch,
            format,
        } = self;

        let mut on_finished = |data: SampleData| {
            // Return ring full only if the control thread stalled badly; dropping
            // here (a free on the RT thread) is the rare last resort.
            let _ = returns.push(AudioReturn::FinishedSample(data));
        };

        // 1. Apply queued control commands.
        while let Some(cmd) = commands.pop() {
            mixer.apply(cmd, &mut on_finished);
        }

        // 2. Zero the stereo scratch for this block (reuses capacity when warm).
        scratch.reset(frames * 2);
        let master = scratch.master_mut();

        // 3. Mix active voices + master gain.
        mixer.render(master, frames, &mut on_finished);

        // 4. Publish this block's peak level (pre-clamp, so clipping reads > 1).
        let mut peak_l = 0.0f32;
        let mut peak_r = 0.0f32;
        for f in 0..frames {
            peak_l = peak_l.max(master[2 * f].abs());
            peak_r = peak_r.max(master[2 * f + 1].abs());
        }
        meter.store(peak_l, peak_r);

        // 5. Write to the device buffer in the output layout, clamped to [-1, 1].
        write_out(out, master, frames, *format);
    }

    /// The output format.
    pub fn format(&self) -> AudioFormat {
        self.format
    }

    /// Currently sounding voices.
    pub fn active_voices(&self) -> usize {
        self.mixer.active_voices()
    }

    /// Voice-pool capacity (never changes).
    pub fn voice_capacity(&self) -> usize {
        self.mixer.voice_capacity()
    }

    /// Mix-scratch capacity — the HR-3 no-alloc gate reads this across warm blocks.
    pub fn scratch_capacity(&self) -> usize {
        self.scratch.capacity()
    }
}

/// Write the interleaved-stereo `master` scratch into the device `out` buffer,
/// down-mixing to mono if the output layout is mono, clamped to `[-1, 1]`.
fn write_out(out: &mut [Sample], master: &[Sample], frames: usize, format: AudioFormat) {
    match format.channels {
        ChannelLayout::Stereo => {
            let n = (frames * 2).min(out.len());
            for i in 0..n {
                out[i] = master[i].clamp(-1.0, 1.0);
            }
        }
        ChannelLayout::Mono => {
            let n = frames.min(out.len());
            for f in 0..n {
                out[f] = (0.5 * (master[2 * f] + master[2 * f + 1])).clamp(-1.0, 1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _assert_send<T: Send>() {}

    #[test]
    fn renderer_is_send() {
        // The renderer must move to the audio thread.
        _assert_send::<AudioRenderer>();
    }

    #[test]
    fn play_returns_distinct_handles() {
        let (mut engine, _r) = AudioEngine::new(AudioFormat::stereo(48_000));
        let data = SampleData::from_interleaved(vec![0.0; 8], AudioFormat::mono(48_000));
        let a = engine.play(data.clone(), PlayParams::default()).unwrap();
        let b = engine.play(data, PlayParams::default()).unwrap();
        assert_ne!(a, b);
        assert!(!a.is_none() && !b.is_none());
    }
}
