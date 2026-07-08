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
    AudioEngine, AudioFormat, AudioRenderer, BusId, PlayParams, SUB_BUS_COUNT, SampleData, VoiceId,
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
    /// Change-gate for the master high-pass (low-cut) cutoff.
    last_highpass: std::cell::Cell<f32>,
    /// Change-gate for the master balance.
    last_master_pan: std::cell::Cell<f32>,
    /// Change-gate for the master limiter toggle.
    last_limiter: std::cell::Cell<bool>,
    /// Change-gate for the master reverb (on, size, mix) — send only on change.
    last_reverb: std::cell::Cell<(bool, f32, f32)>,
    /// Change-gate for the master 3-band EQ gains (low, mid, high dB).
    last_eq: std::cell::Cell<(f32, f32, f32)>,
    /// Change-gate for the master delay (on, time, feedback, mix).
    last_delay: std::cell::Cell<(bool, f32, f32, f32)>,
    /// Same change-gate, per sub-bus fader (index-aligned with `BusId::SUB_BUSES`).
    last_bus_gain: [std::cell::Cell<f32>; SUB_BUS_COUNT],
    /// Same change-gate, per sub-bus balance.
    last_bus_pan: [std::cell::Cell<f32>; SUB_BUS_COUNT],
    /// Same change-gate, per sub-bus tone (low-pass cutoff Hz).
    last_bus_cutoff: [std::cell::Cell<f32>; SUB_BUS_COUNT],
    /// Same change-gate, per sub-bus low-cut (high-pass cutoff Hz).
    last_bus_highpass: [std::cell::Cell<f32>; SUB_BUS_COUNT],
    /// Same change-gate, per sub-bus reverb aux-send amount.
    last_bus_send: [std::cell::Cell<f32>; SUB_BUS_COUNT],
    /// Same change-gate, per sub-bus delay aux-send amount.
    last_bus_delay_send: [std::cell::Cell<f32>; SUB_BUS_COUNT],
    /// Same change-gate, per sub-bus compressor amount (0 = off .. 1 = heavy).
    last_bus_comp: [std::cell::Cell<f32>; SUB_BUS_COUNT],
    /// Voices of the built-in test signal while the panel's Play Test is on
    /// (empty = stopped). Looping, so they sound until explicitly stopped.
    test_voices: Vec<VoiceId>,
    /// Ducking gain-reduction envelope (`1.0` = no duck), smoothed per frame.
    duck_env: std::cell::Cell<f32>,
    /// Audio Editor runtime (docs/Audio/, W1): the loaded clip + preview
    /// transport state, driven by the `panel-audio-editor` bridge.
    #[cfg(feature = "panel-audio-editor")]
    editor: AudioEditorRuntime,
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
            last_highpass: std::cell::Cell::new(20.0),
            last_master_pan: std::cell::Cell::new(0.0),
            last_limiter: std::cell::Cell::new(false),
            last_reverb: std::cell::Cell::new((false, 0.5, 0.3)),
            last_eq: std::cell::Cell::new((0.0, 0.0, 0.0)),
            last_delay: std::cell::Cell::new((false, 0.25, 0.3, 0.3)),
            last_bus_gain: std::array::from_fn(|_| std::cell::Cell::new(1.0)),
            last_bus_pan: std::array::from_fn(|_| std::cell::Cell::new(0.0)),
            last_bus_cutoff: std::array::from_fn(|_| std::cell::Cell::new(20_000.0)),
            last_bus_highpass: std::array::from_fn(|_| std::cell::Cell::new(20.0)),
            last_bus_send: std::array::from_fn(|_| std::cell::Cell::new(0.0)),
            last_bus_delay_send: std::array::from_fn(|_| std::cell::Cell::new(0.0)),
            last_bus_comp: std::array::from_fn(|_| std::cell::Cell::new(0.0)),
            test_voices: Vec::new(),
            duck_env: std::cell::Cell::new(1.0),
            #[cfg(feature = "panel-audio-editor")]
            editor: AudioEditorRuntime::default(),
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

    /// Current master RMS `[L, R]` (≈ perceived loudness) for the meter fill.
    pub(crate) fn rms(&self) -> [f32; 2] {
        self.engine.rms()
    }

    /// Current master momentary loudness in LUFS (BS.1770, 400 ms window).
    pub(crate) fn momentary_lufs(&self) -> f32 {
        self.engine.momentary_lufs()
    }

    /// Current post-fader RMS per sub-bus, for the strip meter fills.
    pub(crate) fn bus_rms(&self) -> [[f32; 2]; SUB_BUS_COUNT] {
        self.engine.bus_rms()
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

    /// Set sub-bus `i`'s low-pass cutoff (Hz), change-gated per bus.
    pub(crate) fn set_bus_cutoff(&self, i: usize, hz: f32) {
        if let Some(cell) = self.last_bus_cutoff.get(i)
            && (hz - cell.get()).abs() > f32::EPSILON
        {
            let _ = self.engine.set_bus_cutoff(BusId::SUB_BUSES[i], hz);
            cell.set(hz);
        }
    }

    /// Set sub-bus `i`'s high-pass (low-cut) cutoff (Hz), change-gated per bus.
    pub(crate) fn set_bus_highpass(&self, i: usize, hz: f32) {
        if let Some(cell) = self.last_bus_highpass.get(i)
            && (hz - cell.get()).abs() > f32::EPSILON
        {
            let _ = self.engine.set_bus_highpass(BusId::SUB_BUSES[i], hz);
            cell.set(hz);
        }
    }

    /// Set sub-bus `i`'s reverb aux-send amount (0..1), change-gated per bus.
    pub(crate) fn set_bus_send(&self, i: usize, amount: f32) {
        if let Some(cell) = self.last_bus_send.get(i)
            && (amount - cell.get()).abs() > f32::EPSILON
        {
            let _ = self.engine.set_bus_send(BusId::SUB_BUSES[i], amount);
            cell.set(amount);
        }
    }

    /// Set the master delay (on, time, feedback, mix), change-gated on the tuple.
    pub(crate) fn set_delay(&self, on: bool, time: f32, feedback: f32, mix: f32) {
        let (lon, lt, lf, lm) = self.last_delay.get();
        if on != lon
            || (time - lt).abs() > f32::EPSILON
            || (feedback - lf).abs() > f32::EPSILON
            || (mix - lm).abs() > f32::EPSILON
        {
            let _ = self.engine.set_delay(on, time, feedback, mix);
            self.last_delay.set((on, time, feedback, mix));
        }
    }

    /// Set sub-bus `i`'s delay aux-send amount (0..1), change-gated per bus.
    pub(crate) fn set_bus_delay_send(&self, i: usize, amount: f32) {
        if let Some(cell) = self.last_bus_delay_send.get(i)
            && (amount - cell.get()).abs() > f32::EPSILON
        {
            let _ = self.engine.set_bus_delay_send(BusId::SUB_BUSES[i], amount);
            cell.set(amount);
        }
    }

    /// Set sub-bus `i`'s compressor from a single 0..1 `amount` knob, change-gated
    /// per bus. The amount drives the threshold down (1.0 = off .. 0.1 = heavy)
    /// with a fixed musical 4:1 ratio + 10 ms attack / 100 ms release.
    pub(crate) fn set_bus_compressor(&self, i: usize, amount: f32) {
        if let Some(cell) = self.last_bus_comp.get(i)
            && (amount - cell.get()).abs() > f32::EPSILON
        {
            let a = amount.clamp(0.0, 1.0);
            let on = a > 0.001;
            let threshold = 1.0 - a * 0.9; // 1.0 (off) .. 0.1 (heavy)
            let _ =
                self.engine
                    .set_bus_compressor(BusId::SUB_BUSES[i], on, threshold, 4.0, 0.01, 0.1);
            cell.set(amount);
        }
    }

    /// Advance the ducking envelope from sub-bus `key_idx`'s level (the sidechain
    /// key) and return the current duck multiplier (`1.0` = none) for every other
    /// bus. Fast attack (dip quickly when the key sounds), slow release. Pure
    /// control-side — the engine's per-bus smoothing turns the per-frame target
    /// into clean audio. `key_idx` indexes [`BusId::SUB_BUSES`].
    pub(crate) fn update_ducking(&self, on: bool, depth: f32, key_idx: usize) -> f32 {
        const KEY_THRESHOLD: f32 = 0.03; // key RMS above which ducking engages
        const ATTACK: f32 = 0.35; // per-frame toward a deeper duck
        const RELEASE: f32 = 0.06; // per-frame back toward unity
        // The selected key bus's post-fader RMS drives the duck.
        let key_bus = self
            .engine
            .bus_rms()
            .get(key_idx)
            .copied()
            .unwrap_or([0.0, 0.0]);
        let key = key_bus[0].max(key_bus[1]);
        let target = if on && key > KEY_THRESHOLD {
            1.0 - depth.clamp(0.0, 1.0)
        } else {
            1.0
        };
        let env = self.duck_env.get();
        let coeff = if target < env { ATTACK } else { RELEASE };
        let next = env + (target - env) * coeff;
        self.duck_env.set(next);
        next
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

    /// Set the master high-pass (low-cut) cutoff (Hz), change-gated.
    pub(crate) fn set_master_highpass(&self, hz: f32) {
        if (hz - self.last_highpass.get()).abs() > f32::EPSILON {
            let _ = self.engine.set_master_highpass(hz);
            self.last_highpass.set(hz);
        }
    }

    /// Engage/disengage the master limiter, change-gated (send only on change).
    pub(crate) fn set_master_limiter(&self, on: bool) {
        if on != self.last_limiter.get() {
            let _ = self.engine.set_master_limiter(on);
            self.last_limiter.set(on);
        }
    }

    /// Set the master reverb (on, size, mix), change-gated on the whole triple.
    pub(crate) fn set_reverb(&self, on: bool, size: f32, mix: f32) {
        let next = (on, size, mix);
        let (lon, lsize, lmix) = self.last_reverb.get();
        if on != lon || (size - lsize).abs() > f32::EPSILON || (mix - lmix).abs() > f32::EPSILON {
            let _ = self.engine.set_reverb(on, mix, size);
            self.last_reverb.set(next);
        }
    }

    /// Set the master 3-band EQ gains in dB (low, mid, high), change-gated on the
    /// whole triple (send only when a band moves).
    pub(crate) fn set_master_eq(&self, low: f32, mid: f32, high: f32) {
        let next = (low, mid, high);
        let (ll, lm, lh) = self.last_eq.get();
        if (low - ll).abs() > f32::EPSILON
            || (mid - lm).abs() > f32::EPSILON
            || (high - lh).abs() > f32::EPSILON
        {
            let _ = self.engine.set_master_eq(low, mid, high);
            self.last_eq.set(next);
        }
    }

    /// Start or stop the built-in test signal to match the panel's Play Test
    /// toggle. Idempotent: only acts on the transition (empty ↔ playing). Plays a
    /// repeating pluck on the Music bus (transients → the peak-hold marker jumps)
    /// and a slow swell on the SFX bus, so every strip control is exercisable.
    pub(crate) fn set_test_playing(&mut self, on: bool) {
        let active = !self.test_voices.is_empty();
        if on && !active {
            let pluck = pluck_loop(self.format, 330.0, 0.6, 0.7);
            let swell = swell_loop(self.format, 220.0, 0.9, 0.4);
            // A periodic "voice" blip on the Voice bus so Ducking is demoable.
            let blip = blip_loop(self.format, 200.0, 0.3, 1.2, 0.6);
            for (data, bus) in [
                (pluck, BusId::Music),
                (swell, BusId::Sfx),
                (blip, BusId::Voice),
            ] {
                let params = PlayParams {
                    looping: true,
                    bus,
                    ..PlayParams::default()
                };
                match self.engine.play(data, params) {
                    Ok(id) => self.test_voices.push(id),
                    Err(e) => eprintln!("audio: test signal dropped ({e})"),
                }
            }
            println!("audio: test signal ON (pluck->Music, swell->SFX, blip->Voice)");
        } else if !on && active {
            for id in self.test_voices.drain(..) {
                let _ = self.engine.stop(id);
            }
            println!("audio: test signal OFF");
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

/// Audio Editor transport state (docs/Audio/, W1). The panel's single Play/Pause
/// button cycles Stopped → Playing → Paused → Playing; Stop returns to Stopped.
#[cfg(feature = "panel-audio-editor")]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum EditorTransport {
    #[default]
    Stopped,
    Playing,
    Paused,
}

/// The Audio Editor's shell-side runtime: the loaded clip + preview transport.
#[cfg(feature = "panel-audio-editor")]
#[derive(Default)]
struct AudioEditorRuntime {
    clip: Option<ph2d_audio_edit::EditClip>,
    state: EditorTransport,
    /// Whether the renderer has *confirmed* the current play (its preview-active
    /// flag went true at least once). Guards the Playing→Stopped transition
    /// against the 1-frame lag between enqueuing `PlayPreview` and the audio
    /// callback processing it — without it, `editor_poll` sees `preview_playing()`
    /// still false right after Play and wrongly snaps back to Stopped.
    started: bool,
    /// Display name of the loaded clip (file stem).
    name: String,
}

#[cfg(feature = "panel-audio-editor")]
impl AudioSystem {
    /// Load + decode a file into the editor's [`EditClip`] (rebuilding the peak
    /// cache). Stops any running preview; playback starts on the next Play.
    pub(crate) fn editor_load(&mut self, path: &std::path::Path) {
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
        let _ = self.engine.stop_preview();
        self.editor.name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("clip")
            .to_string();
        self.editor.clip = Some(ph2d_audio_edit::EditClip::new(data));
        self.editor.state = EditorTransport::Stopped;
    }

    /// Cycle the transport: Stopped → play from 0, Playing → pause, Paused →
    /// resume. `looping` is read at play-from-start time.
    pub(crate) fn editor_toggle_play(&mut self, looping: bool) {
        match self.editor.state {
            EditorTransport::Stopped => {
                if let Some(clip) = &self.editor.clip {
                    let params = PlayParams {
                        looping,
                        ..PlayParams::default()
                    };
                    if self.engine.play_preview(clip.data().clone(), params).is_ok() {
                        self.editor.state = EditorTransport::Playing;
                        self.editor.started = false;
                    }
                }
            }
            EditorTransport::Playing => {
                let _ = self.engine.pause_preview(true);
                self.editor.state = EditorTransport::Paused;
            }
            EditorTransport::Paused => {
                let _ = self.engine.pause_preview(false);
                self.editor.state = EditorTransport::Playing;
            }
        }
    }

    /// Stop the preview and rewind to the clip start.
    pub(crate) fn editor_stop(&mut self) {
        let _ = self.engine.stop_preview();
        self.editor.state = EditorTransport::Stopped;
    }

    /// Write the loaded clip out to `path` as a 16-bit PCM WAV.
    pub(crate) fn editor_export(&self, path: &std::path::Path) {
        let Some(clip) = &self.editor.clip else {
            return;
        };
        match ph2d_audio_encode::write_wav(path, clip.data(), ph2d_audio_encode::BitDepth::Pcm16) {
            Ok(()) => println!("audio: exported {} (WAV PCM16)", path.display()),
            Err(e) => eprintln!("audio: export failed for {}: {e}", path.display()),
        }
    }

    /// Advance transport state on a natural end-of-clip (a non-looping preview
    /// that finished). Call once per frame from the bridge.
    pub(crate) fn editor_poll(&mut self) {
        if self.editor.state == EditorTransport::Playing {
            if self.engine.preview_playing() {
                self.editor.started = true;
            } else if self.editor.started {
                // Confirmed-playing then went silent → the clip reached its end.
                self.editor.state = EditorTransport::Stopped;
                self.editor.started = false;
            }
        }
    }

    /// Whether the editor preview is actively playing (not paused/stopped).
    pub(crate) fn editor_playing(&self) -> bool {
        self.editor.state == EditorTransport::Playing
    }

    /// Whether a clip is loaded.
    pub(crate) fn editor_loaded(&self) -> bool {
        self.editor.clip.is_some()
    }

    /// The preview's current playback position in seconds (clip time base).
    pub(crate) fn editor_position_secs(&self) -> f64 {
        match &self.editor.clip {
            Some(clip) => clip
                .data()
                .format()
                .frames_to_secs(self.engine.preview_frame()),
            None => 0.0,
        }
    }

    /// The loaded clip's duration in seconds (`0` when none).
    pub(crate) fn editor_duration_secs(&self) -> f64 {
        self.editor.clip.as_ref().map(|c| c.duration_secs()).unwrap_or(0.0)
    }

    /// The loaded clip's display name.
    pub(crate) fn editor_name(&self) -> &str {
        &self.editor.name
    }

    /// The loaded editor clip (for the overlay waveform), if any.
    pub(crate) fn editor_clip(&self) -> Option<&ph2d_audio_edit::EditClip> {
        self.editor.clip.as_ref()
    }

    /// The preview's current playback frame (for the overlay playhead).
    pub(crate) fn editor_preview_frame(&self) -> u64 {
        self.engine.preview_frame()
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

/// A looping plucked note: fast attack then decay to silence, so it loops as a
/// repeating pluck (silence at the seam = no click). The decay gives transients
/// that make the meter's peak-hold marker jump then fall.
fn pluck_loop(format: AudioFormat, freq_hz: f32, secs: f32, gain: f32) -> SampleData {
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
fn blip_loop(
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
fn swell_loop(format: AudioFormat, freq_hz: f32, secs: f32, gain: f32) -> SampleData {
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
