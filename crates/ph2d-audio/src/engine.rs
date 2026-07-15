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
use crate::bus::{BusId, SUB_BUS_COUNT};
use crate::command::{AudioCommand, AudioReturn, Consumer, PlayParams, Producer, ring};
use crate::dsp::{BiquadCoeffs, LoudnessMeter, lufs_from_mean_square};
use crate::format::{AudioFormat, Sample};
use crate::meter::AudioMeter;
use crate::mixer::Mixer;
use crate::output::write_out;
use crate::stream::StreamHandle;
use crate::voice::{Source, Voice, VoiceId};
use crate::{CMD_CAPACITY, MAX_VOICES, RETURN_CAPACITY};

/// Internal marker id for the renderer's dedicated preview voice. It lives
/// outside the game voice pool (never minted by [`AudioEngine::play`], which
/// starts at 1 and increments), so a non-zero, out-of-band value keeps it
/// distinct and `is_free()` correct.
const PREVIEW_VOICE_ID: VoiceId = VoiceId(u64::MAX);

/// A low-pass cutoff at/above this reads as "fully open" (identity bypass): it is
/// the top of the mixer's Tone slider (20 kHz, the audible ceiling), so a filter
/// here is inaudible. Bypassing it exactly avoids coloring the top octave with a
/// near-Nyquist low-pass at 48 kHz+, and keeps the open default sample-rate-robust.
const OPEN_CUTOFF_HZ: f32 = 20_000.0; // LITERAL-PX-OK: audible ceiling / Tone "open" top (audio domain, not a UI metric)

/// A high-pass (low-cut) cutoff at/below this reads as "off" (identity bypass):
/// it is the bottom of the mixer's Low Cut slider (20 Hz, the audible floor), so
/// a high-pass here removes nothing audible. Symmetric to [`OPEN_CUTOFF_HZ`].
const FLOOR_CUTOFF_HZ: f32 = 20.0; // LITERAL-PX-OK: audible floor / Low Cut "off" bottom (audio domain, not a UI metric)

/// Master 3-band EQ band centers + shared Q. Fixed — only the per-band gain is
/// user-set; the classic low-shelf / mid-peak / high-shelf console tone stack.
const EQ_LOW_HZ: f32 = 120.0; // LITERAL-PX-OK: low-shelf corner (audio domain)
const EQ_MID_HZ: f32 = 1_000.0; // LITERAL-PX-OK: mid-peak center (audio domain)
const EQ_HIGH_HZ: f32 = 8_000.0; // LITERAL-PX-OK: high-shelf corner (audio domain)
const EQ_Q: f32 = std::f32::consts::FRAC_1_SQRT_2; // Butterworth-ish shelf/peak width

/// Per-sample one-pole coefficient for an envelope `time_secs`:
/// `1 - e^(-1/(t·sr))` (`<= 0` s → instantaneous, coeff `1`). Control-thread only
/// (uses `exp`) — the compressor's RT `process` takes the coefficient directly.
fn env_coeff(time_secs: f32, sample_rate: f32) -> f32 {
    if time_secs <= 0.0 {
        1.0
    } else {
        1.0 - (-1.0 / (time_secs * sample_rate)).exp()
    }
}

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

    /// The most recent post-fader peak per sub-bus, in [`BusId::SUB_BUSES`]
    /// order — one `[L, R]` pair per mixer strip.
    pub fn bus_levels(&self) -> [[f32; 2]; SUB_BUS_COUNT] {
        self.meter.bus_peaks()
    }

    /// The most recent master RMS `[left, right]` (≈ perceived loudness).
    pub fn rms(&self) -> [f32; 2] {
        self.meter.rms()
    }

    /// The master's momentary loudness in **LUFS** (BS.1770 K-weighting over a
    /// 400 ms window). Reads the RT-published mean-square and does the `log10`
    /// here on the control thread. Silence reads [`crate::dsp::SILENCE_LUFS`].
    pub fn momentary_lufs(&self) -> f32 {
        lufs_from_mean_square(self.meter.loudness_mean_square())
    }

    /// The most recent post-fader RMS per sub-bus, in [`BusId::SUB_BUSES`] order.
    pub fn bus_rms(&self) -> [[f32; 2]; SUB_BUS_COUNT] {
        self.meter.bus_rms()
    }

    /// The output format the renderer was built for.
    pub fn format(&self) -> AudioFormat {
        self.format
    }

    /// Start (or replace) the editor **preview** — audition `data` under
    /// `params` on a dedicated voice outside the game pool, so it never steals
    /// game voices. Its playback position is then readable via
    /// [`AudioEngine::preview_frame`] for the transport playhead. `Err` only if
    /// the command ring is full.
    pub fn play_preview(&self, data: SampleData, params: PlayParams) -> Result<(), AudioError> {
        self.send(AudioCommand::PlayPreview { data, params })
    }

    /// Jump the preview's playback cursor to `frame` (scrub / seek).
    pub fn seek_preview(&self, frame: u64) -> Result<(), AudioError> {
        self.send(AudioCommand::SeekPreview { frame })
    }

    /// Hot-swap the preview's sample WITHOUT stopping it — apply an edit live,
    /// keeping the play position + loop. No-op if no preview is sounding.
    pub fn set_preview_data(&self, data: SampleData) -> Result<(), AudioError> {
        self.send(AudioCommand::SetPreviewData { data })
    }

    /// Set (or clear) the preview's loop **region** live — the editor moving the loop while it
    /// sounds. Takes effect on the next lap rather than re-triggering the clip, which is what makes
    /// dragging a loop point audible instead of stuttery.
    pub fn set_preview_loop_region(
        &self,
        region: Option<crate::LoopRegion>,
    ) -> Result<(), AudioError> {
        self.send(AudioCommand::SetPreviewLoopRegion { region })
    }

    /// Enable/disable preview looping live (the editor's Loop toggle mid-play).
    pub fn set_preview_looping(&self, looping: bool) -> Result<(), AudioError> {
        self.send(AudioCommand::SetPreviewLooping { looping })
    }

    /// Pause (`true`) or resume (`false`) the preview, holding its position.
    pub fn pause_preview(&self, paused: bool) -> Result<(), AudioError> {
        self.send(AudioCommand::PausePreview { paused })
    }

    /// Stop the preview and free its sample.
    pub fn stop_preview(&self) -> Result<(), AudioError> {
        self.send(AudioCommand::StopPreview)
    }

    /// The preview's current playback position in source frames (transport
    /// playhead). `0` when idle.
    pub fn preview_frame(&self) -> u64 {
        self.meter.preview_frame()
    }

    /// Whether an editor preview is currently sounding (for showing/hiding the
    /// playhead and detecting end-of-clip).
    pub fn preview_playing(&self) -> bool {
        self.meter.preview_active()
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

    /// Play from a **stream**: the audio is decoded ahead, in chunks, by a producer thread, and
    /// never all resident at once (ADR-0118).
    ///
    /// This is what makes the codec mean something to memory. A three-minute song costs 65.9 MB
    /// resident — 2.2x HR-13's entire iPad audio budget — and the same song streamed costs the
    /// ring: a few tens of kilobytes, whatever its length.
    ///
    /// Pair with [`crate::stream`], and feed the [`StreamFeeder`](crate::StreamFeeder) half from a
    /// worker thread.
    pub fn play_stream(
        &mut self,
        handle: StreamHandle,
        params: PlayParams,
    ) -> Result<VoiceId, AudioError> {
        self.next_id += 1;
        let id = VoiceId(self.next_id);
        self.send(AudioCommand::PlayStream {
            voice: id,
            handle: Box::new(handle),
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

    /// Low-pass coefficients for `cutoff_hz`, or identity (true bypass) when the
    /// filter would be inaudible: at/above the audible ceiling (the Tone slider's
    /// fully-open top, [`OPEN_CUTOFF_HZ`]) or at/near Nyquist. Without the ceiling
    /// guard the "open" default (20 kHz) applies a real low-pass at 48 kHz+
    /// (`sr*0.5*0.9 = 21.6 kHz > 20 kHz`), coloring the top octave. Computed on
    /// the control thread — no transcendentals on the RT thread.
    fn lowpass_coeffs(&self, cutoff_hz: f32) -> BiquadCoeffs {
        let sr = self.format.sample_rate as f32;
        if cutoff_hz >= OPEN_CUTOFF_HZ || cutoff_hz >= sr * 0.5 * 0.9 {
            BiquadCoeffs::identity()
        } else {
            BiquadCoeffs::lowpass(sr, cutoff_hz.max(20.0), std::f32::consts::FRAC_1_SQRT_2)
        }
    }

    /// Set the master low-pass filter cutoff in Hz. At/above the audible ceiling
    /// ([`OPEN_CUTOFF_HZ`]) or near Nyquist the filter is a true bypass.
    pub fn set_master_cutoff(&self, cutoff_hz: f32) -> Result<(), AudioError> {
        let coeffs = self.lowpass_coeffs(cutoff_hz);
        self.send(AudioCommand::SetMasterFilter { coeffs })
    }

    /// Set a sub-bus's low-pass filter cutoff in Hz (true bypass at/above the
    /// audible ceiling or near Nyquist).
    pub fn set_bus_cutoff(&self, bus: BusId, cutoff_hz: f32) -> Result<(), AudioError> {
        let coeffs = self.lowpass_coeffs(cutoff_hz);
        self.send(AudioCommand::SetBusFilter { bus, coeffs })
    }

    /// High-pass (low-cut) coefficients for `cutoff_hz`, or identity (off) when
    /// the cut is at/below the audible floor ([`FLOOR_CUTOFF_HZ`] — the Low Cut
    /// slider's fully-off bottom), so "off" removes nothing. The RBJ builder
    /// clamps the frequency below Nyquist. Control-thread only (no RT
    /// transcendentals). Mirror of [`AudioEngine::lowpass_coeffs`].
    fn highpass_coeffs(&self, cutoff_hz: f32) -> BiquadCoeffs {
        let sr = self.format.sample_rate as f32;
        if cutoff_hz <= FLOOR_CUTOFF_HZ {
            BiquadCoeffs::identity()
        } else {
            BiquadCoeffs::highpass(sr, cutoff_hz, std::f32::consts::FRAC_1_SQRT_2)
        }
    }

    /// Set the master high-pass (low-cut) cutoff in Hz. At/below the audible
    /// floor ([`FLOOR_CUTOFF_HZ`]) the filter is off (true bypass).
    pub fn set_master_highpass(&self, cutoff_hz: f32) -> Result<(), AudioError> {
        let coeffs = self.highpass_coeffs(cutoff_hz);
        self.send(AudioCommand::SetMasterHighpass { coeffs })
    }

    /// Set a sub-bus's high-pass (low-cut) cutoff in Hz (off at/below the audible
    /// floor).
    pub fn set_bus_highpass(&self, bus: BusId, cutoff_hz: f32) -> Result<(), AudioError> {
        let coeffs = self.highpass_coeffs(cutoff_hz);
        self.send(AudioCommand::SetBusHighpass { bus, coeffs })
    }

    /// Set a sub-bus fader gain (smoothed). To mute a bus, send `0.0` (the
    /// control-side fold, mirroring the master strip). A [`BusId::Master`]
    /// target maps to the master gain.
    pub fn set_bus_gain(&self, bus: BusId, gain: f32) -> Result<(), AudioError> {
        self.send(AudioCommand::SetBusGain { bus, gain })
    }

    /// Set a bus's stereo balance, `-1.0` (left) … `1.0` (right); `0.0` = center.
    /// A [`BusId::Master`] target sets the master balance.
    pub fn set_bus_pan(&self, bus: BusId, pan: f32) -> Result<(), AudioError> {
        self.send(AudioCommand::SetBusPan { bus, pan })
    }

    /// Engage/disengage the master soft-clip limiter.
    pub fn set_master_limiter(&self, on: bool) -> Result<(), AudioError> {
        self.send(AudioCommand::SetMasterLimiter { on })
    }

    /// Set the master reverb: `on`, return `mix` level (0..1), `room_size` (0..1).
    /// The reverb is a parallel return fed by [`AudioEngine::set_bus_send`], not a
    /// master insert.
    pub fn set_reverb(&self, on: bool, mix: f32, room_size: f32) -> Result<(), AudioError> {
        self.send(AudioCommand::SetReverb { on, mix, room_size })
    }

    /// Set a sub-bus's reverb aux-send `amount` (0..1) — how much of that bus's
    /// post-fader signal feeds the reverb return.
    pub fn set_bus_send(&self, bus: BusId, amount: f32) -> Result<(), AudioError> {
        self.send(AudioCommand::SetBusSend { bus, amount })
    }

    /// Set the master delay/echo return: `on`, `time` (s), `feedback` (0..1),
    /// return `mix` level (0..1). Fed by [`AudioEngine::set_bus_delay_send`].
    pub fn set_delay(
        &self,
        on: bool,
        time: f32,
        feedback: f32,
        mix: f32,
    ) -> Result<(), AudioError> {
        self.send(AudioCommand::SetDelay {
            on,
            time,
            feedback,
            mix,
        })
    }

    /// Set a sub-bus's delay aux-send `amount` (0..1) — how much of that bus's
    /// post-fader signal feeds the delay return.
    pub fn set_bus_delay_send(&self, bus: BusId, amount: f32) -> Result<(), AudioError> {
        self.send(AudioCommand::SetBusDelaySend { bus, amount })
    }

    /// Set a sub-bus's compressor: `on`, `threshold` (linear 0..1), `ratio`
    /// (>=1), and `attack_secs` / `release_secs`. The attack/release times are
    /// converted to per-sample coefficients here (control thread) so no `exp`
    /// runs on the audio thread.
    pub fn set_bus_compressor(
        &self,
        bus: BusId,
        on: bool,
        threshold: f32,
        ratio: f32,
        attack_secs: f32,
        release_secs: f32,
    ) -> Result<(), AudioError> {
        let sr = self.format.sample_rate as f32;
        self.send(AudioCommand::SetBusCompressor {
            bus,
            on,
            threshold,
            ratio,
            attack: env_coeff(attack_secs, sr),
            release: env_coeff(release_secs, sr),
        })
    }

    /// Set the master 3-band EQ gains in dB — low shelf ([`EQ_LOW_HZ`]), mid peak
    /// ([`EQ_MID_HZ`]), high shelf ([`EQ_HIGH_HZ`]). `0 dB` per band = flat
    /// (transparent). Coeffs computed control-side (no RT transcendentals).
    pub fn set_master_eq(&self, low_db: f32, mid_db: f32, high_db: f32) -> Result<(), AudioError> {
        let sr = self.format.sample_rate as f32;
        let low = BiquadCoeffs::lowshelf(sr, EQ_LOW_HZ, EQ_Q, low_db);
        let mid = BiquadCoeffs::peak(sr, EQ_MID_HZ, EQ_Q, mid_db);
        let high = BiquadCoeffs::highshelf(sr, EQ_HIGH_HZ, EQ_Q, high_db);
        self.send(AudioCommand::SetMasterEq { low, mid, high })
    }

    /// Drain and drop the sources finished voices handed back. Call once per game frame so their
    /// memory frees on the control thread, not the RT one.
    ///
    /// A resident source frees an `Arc<[f32]>`; a **streamed** one frees its chunks. Both are a
    /// `free()`, and a `free()` on the audio thread is an allocation running backwards (HR-3) —
    /// which is why the return ring carries the whole source, not just the sample (ADR-0118).
    pub fn collect_returns(&self) {
        while let Some(AudioReturn::FinishedSource(_source)) = self.returns.pop() {
            // Dropping `_source` here frees it on the control thread (HR-3).
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
    /// Momentary-loudness (LUFS) meter over the final master output.
    loudness: LoudnessMeter,
    /// The editor's dedicated **preview** voice, separate from the game voice
    /// pool so auditioning a clip in the editor never steals game voices and its
    /// position is published independently. Reuses [`Voice`]'s resample/interp.
    preview: Voice,
    /// Whether [`Self::preview`] is currently sounding (a clip is loaded + not
    /// stopped/finished).
    preview_active: bool,
    /// Whether the preview is paused (holds position, renders silence).
    preview_paused: bool,
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
            loudness: LoudnessMeter::new(format.sample_rate),
            preview: Voice::silent(),
            preview_active: false,
            preview_paused: false,
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
            loudness,
            preview,
            preview_active,
            preview_paused,
            format,
        } = self;

        let mut on_finished = |source: Source| {
            // Return ring full only if the control thread stalled badly; dropping
            // here (a free on the RT thread) is the rare last resort.
            let _ = returns.push(AudioReturn::FinishedSource(source));
        };

        // 1. Apply queued control commands. Preview commands drive the dedicated
        //    preview voice here (outside the mixer); everything else is a mixer op.
        while let Some(cmd) = commands.pop() {
            match cmd {
                AudioCommand::PlayPreview { data, params } => {
                    if let Some(old) = preview.free() {
                        on_finished(old);
                    }
                    preview.start(PREVIEW_VOICE_ID, data, &params, format.sample_rate);
                    *preview_active = true;
                    *preview_paused = false;
                }
                AudioCommand::SeekPreview { frame } => preview.set_cursor(frame),
                AudioCommand::SetPreviewData { data } => {
                    if *preview_active && !preview.is_free() {
                        if let Some(old) = preview.replace_data(data) {
                            on_finished(old);
                        }
                    } else {
                        // Not sounding — ship it back to drop on the control thread.
                        on_finished(Source::Resident(data));
                    }
                }
                AudioCommand::SetPreviewLooping { looping } => preview.set_looping(looping),
                AudioCommand::SetPreviewLoopRegion { region } => preview.set_loop_region(region),
                AudioCommand::PausePreview { paused } => *preview_paused = paused,
                AudioCommand::StopPreview => {
                    if let Some(old) = preview.free() {
                        on_finished(old);
                    }
                    *preview_active = false;
                }
                other => mixer.apply(other, &mut on_finished),
            }
        }

        // 2. Zero the stereo scratch for this block (reuses capacity when warm).
        scratch.reset(frames * 2);
        let (master, bus_scratch, send, delay_send) = scratch.split_mut();

        // 3. Mix active voices through their sub-buses + master gain, capturing
        //    each sub-bus's post-fader peak + RMS for the strip meters. `send` /
        //    `delay_send` accumulate the per-bus reverb / delay aux-sends.
        let mut bus_peaks = [[0.0f32; 2]; SUB_BUS_COUNT];
        let mut bus_rms = [[0.0f32; 2]; SUB_BUS_COUNT];
        mixer.render(
            master,
            bus_scratch,
            send,
            delay_send,
            &mut bus_peaks,
            &mut bus_rms,
            frames,
            &mut on_finished,
        );

        // 3b. Mix the editor preview voice into the master (so it is heard and
        //     metered). Paused → hold position, render nothing. Publish the
        //     playback frame + active flag for the transport playhead.
        if *preview_active
            && !*preview_paused
            && !preview.is_free()
            && let Some(done) = preview.render_add(master, frames)
        {
            on_finished(done);
            *preview_active = false;
        }
        meter.store_preview(preview.cursor_frame(), *preview_active);

        // 4. Publish this block's master peak + RMS (pre-clamp, so clipping reads
        //    > 1). Peak catches transients; RMS ≈ perceived loudness. Each sample
        //    also feeds the momentary-loudness (LUFS) window.
        let mut peak_l = 0.0f32;
        let mut peak_r = 0.0f32;
        let mut sq_l = 0.0f32;
        let mut sq_r = 0.0f32;
        for f in 0..frames {
            let l = master[2 * f];
            let r = master[2 * f + 1];
            peak_l = peak_l.max(l.abs());
            peak_r = peak_r.max(r.abs());
            sq_l += l * l;
            sq_r += r * r;
            loudness.process(l, r);
        }
        meter.store_loudness(loudness.mean_square());
        let inv_frames = 1.0 / frames.max(1) as f32;
        meter.store(
            [peak_l, peak_r],
            [(sq_l * inv_frames).sqrt(), (sq_r * inv_frames).sqrt()],
        );
        for i in 0..SUB_BUS_COUNT {
            meter.store_bus(i, bus_peaks[i], bus_rms[i]);
        }

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

    /// Per-bus scratch capacity — the other half of the HR-3 no-alloc gate.
    pub fn bus_scratch_capacity(&self) -> usize {
        self.scratch.bus_capacity()
    }

    /// Reverb-send bus capacity — the third buffer the HR-3 no-alloc gate checks.
    pub fn send_scratch_capacity(&self) -> usize {
        self.scratch.send_capacity()
    }

    /// Delay-send bus capacity — the fourth buffer the HR-3 no-alloc gate checks.
    pub fn delay_send_scratch_capacity(&self) -> usize {
        self.scratch.delay_send_capacity()
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

    /// **The Mobile export, played through the WHOLE preview path.** `editor_toggle_play` calls
    /// `play_preview`; the renderer drains that command onto its dedicated preview voice and mixes
    /// it into `out`. A 24 kHz mono clip (the Mobile target) against a 48 kHz device must come out
    /// of `render` audible — the report is that it does not. Every prior gate stops at a return
    /// value or a stream-vs-resident diff; this drives the real door end to end.
    #[test]
    fn a_24k_mono_preview_is_audible_through_the_full_renderer() {
        let tau = std::f32::consts::TAU;
        let (engine, mut r) = AudioEngine::new(AudioFormat::stereo(48_000));
        let src: Vec<f32> = (0..12_000)
            .map(|i| 0.5 * (tau * 220.0 * (i as f32 / 24_000.0)).sin())
            .collect();
        let clip = SampleData::from_interleaved(src, AudioFormat::mono(24_000));
        engine.play_preview(clip, PlayParams::default()).unwrap();

        let mut peak = 0.0f32;
        let mut out = [0.0f32; 256];
        for _ in 0..250 {
            out.fill(0.0);
            r.render(&mut out, 128);
            for &s in &out {
                peak = peak.max(s.abs());
            }
        }
        println!("full renderer, 24k mono preview @ 48k: peak {peak:.4}");
        assert!(
            peak > 0.1,
            "the 24 kHz mono Mobile clip came out of the FULL renderer silent (peak {peak:.4}) -- \
             this is what the Enio hears when he loads .mobile.ogg back and presses Play"
        );
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

    #[test]
    fn open_cutoff_is_true_bypass_across_sample_rates() {
        // The Tone slider's fully-open top is OPEN_CUTOFF_HZ (20 kHz). At 48 kHz
        // the Nyquist guard (sr*0.5*0.9 = 21.6 kHz) alone would NOT bypass it, so
        // the ceiling guard must — at 48 kHz and 96 kHz alike. (Regression: the
        // "open" default used to apply a real 20 kHz low-pass at 48 kHz+.)
        for &sr in &[44_100, 48_000, 96_000] {
            let (engine, _r) = AudioEngine::new(AudioFormat::stereo(sr));
            assert_eq!(
                engine.lowpass_coeffs(OPEN_CUTOFF_HZ),
                BiquadCoeffs::identity(),
                "fully-open Tone must be a true bypass at {sr} Hz"
            );
        }
        // A cutoff below the ceiling is still a real low-pass (not bypassed).
        let (engine, _r) = AudioEngine::new(AudioFormat::stereo(48_000));
        assert_ne!(
            engine.lowpass_coeffs(1_000.0),
            BiquadCoeffs::identity(),
            "a 1 kHz cutoff must filter, not bypass"
        );
    }

    #[test]
    fn lowcut_off_is_true_bypass_across_sample_rates() {
        // The Low Cut slider's fully-off bottom is FLOOR_CUTOFF_HZ (20 Hz); a
        // high-pass there must be an exact bypass at every sample rate (symmetric
        // to the low-pass "open" ceiling). Above the floor it filters for real.
        for &sr in &[44_100, 48_000, 96_000] {
            let (engine, _r) = AudioEngine::new(AudioFormat::stereo(sr));
            assert_eq!(
                engine.highpass_coeffs(FLOOR_CUTOFF_HZ),
                BiquadCoeffs::identity(),
                "fully-off Low Cut must be a true bypass at {sr} Hz"
            );
        }
        let (engine, _r) = AudioEngine::new(AudioFormat::stereo(48_000));
        assert_ne!(
            engine.highpass_coeffs(500.0),
            BiquadCoeffs::identity(),
            "a 500 Hz low-cut must filter, not bypass"
        );
    }
}
