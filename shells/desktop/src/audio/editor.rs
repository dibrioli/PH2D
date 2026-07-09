//! Audio Editor shell-side runtime (docs/Audio/, W1) — split out of `audio.rs`
//! to keep that file under the HR-18 600-LOC shell cap. This is a **descendant
//! submodule** of `audio`, so its `impl AudioSystem` still reaches the private
//! fields of `AudioSystem` (Rust: private items are visible to the defining
//! module and its descendants). The transport + clip logic is unchanged.

use super::AudioSystem;
use ph2d_audio::PlayParams;

/// Audio Editor transport state (docs/Audio/, W1). The panel's single Play/Pause
/// button cycles Stopped → Playing → Paused → Playing; Stop returns to Stopped.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum EditorTransport {
    #[default]
    Stopped,
    Playing,
    Paused,
}

/// The Audio Editor's shell-side runtime: the loaded clip + preview transport.
#[derive(Default)]
pub(super) struct AudioEditorRuntime {
    clip: Option<ph2d_audio_edit::EditClip>,
    state: EditorTransport,
    /// Whether the renderer has *confirmed* the current play (its preview-active
    /// flag went true at least once). Guards the Playing→Stopped transition
    /// against the 1-frame lag between enqueuing `PlayPreview` and the audio
    /// callback processing it — without it, `editor_poll` sees `preview_playing()`
    /// still false right after Play and wrongly snaps back to Stopped.
    started: bool,
    /// Change-gate for the live Loop toggle pushed to the preview.
    last_loop: std::cell::Cell<bool>,
    /// Display name of the loaded clip (file stem).
    name: String,
}

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
                    if self
                        .engine
                        .play_preview(clip.data().clone(), params)
                        .is_ok()
                    {
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
        self.editor
            .clip
            .as_ref()
            .map(|c| c.duration_secs())
            .unwrap_or(0.0)
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

    /// Apply a one-shot edit command from the panel to the loaded clip (each
    /// commits an undo step; undo/redo step the timeline). Keeps the preview
    /// PLAYING (and looping): the edited buffer is hot-swapped into the sounding
    /// preview voice at its current position, so the change is heard live.
    pub(crate) fn editor_apply(&mut self, cmd: ph2d_panel_audio_editor::AudioEditCmd) {
        {
            use ph2d_panel_audio_editor::AudioEditCmd as Cmd;
            let Some(clip) = self.editor.clip.as_mut() else {
                return;
            };
            // ±3 dB per click (10^(±3/20)).
            const GAIN_UP: f32 = 1.412_537_5;
            const GAIN_DOWN: f32 = 0.707_945_8;
            match cmd {
                Cmd::Undo => {
                    clip.undo();
                }
                Cmd::Redo => {
                    clip.redo();
                }
                Cmd::NormalizePeak => clip.apply_normalize_peak(1.0),
                Cmd::NormalizeLufs => clip.apply_normalize_lufs(-16.0), // LITERAL-PX-OK: -16 LUFS target
                Cmd::Reverse => clip.apply_reverse(),
                Cmd::RemoveDc => clip.apply_remove_dc_offset(),
                Cmd::Invert => clip.apply_invert(),
                Cmd::GainDown => clip.apply_gain(GAIN_DOWN),
                Cmd::GainUp => clip.apply_gain(GAIN_UP),
                // Range ops (act on the selection).
                Cmd::Trim => clip.apply_trim(),
                Cmd::Cut => clip.apply_delete(),
                Cmd::Silence => clip.apply_silence(),
                Cmd::FadeIn => clip.apply_fade(
                    ph2d_audio_edit::FadeShape::SCurve,
                    ph2d_audio_edit::FadeDir::In,
                ),
                Cmd::FadeOut => clip.apply_fade(
                    ph2d_audio_edit::FadeShape::SCurve,
                    ph2d_audio_edit::FadeDir::Out,
                ),
                // Effects rack (W3 block 1) — curated fixed presets (parametric
                // control + chain + presets are a later block).
                Cmd::LowPass => clip.apply_effect(ph2d_audio_edit::Effect::LowPass {
                    cutoff: 3_000.0,
                    q: 0.707,
                }),
                Cmd::HighPass => clip.apply_effect(ph2d_audio_edit::Effect::HighPass {
                    cutoff: 150.0,
                    q: 0.707,
                }),
                Cmd::Compress => clip.apply_effect(ph2d_audio_edit::Effect::Compress {
                    threshold: 0.3,
                    ratio: 4.0,
                    attack_secs: 0.005,
                    release_secs: 0.1,
                    makeup: 1.6,
                }),
                Cmd::Saturate => {
                    clip.apply_effect(ph2d_audio_edit::Effect::Saturate { drive: 3.0 })
                }
                Cmd::Bitcrush => clip.apply_effect(ph2d_audio_edit::Effect::Bitcrush {
                    bits: 6,
                    downsample: 4,
                }),
                Cmd::StereoWiden => {
                    clip.apply_effect(ph2d_audio_edit::Effect::StereoWidth { width: 1.6 })
                }
                // Tail-extending (W3 block 2): the ring-out bleeds past the target
                // range, growing the clip when the range reaches its end. `tail_secs`
                // must clear the effect's own latency — Freeverb's shortest comb is
                // ~25 ms, so a short tail would render pure silence.
                Cmd::Reverb => clip.apply_tail_effect(ph2d_audio_edit::TailEffect::Reverb {
                    room_size: 0.7,
                    damp: 0.5,
                    mix: 0.35,
                    tail_secs: 2.5,
                }),
                Cmd::Echo => clip.apply_tail_effect(ph2d_audio_edit::TailEffect::Delay {
                    time_secs: 0.25,
                    feedback: 0.4,
                    mix: 0.35,
                    tail_secs: 2.0,
                }),
            }
        }
        // Hot-swap the edited buffer into the sounding preview (no stop). No-op if
        // stopped — the next Play uses the edited clip.
        if self.engine.preview_playing()
            && let Some(clip) = self.editor.clip.as_ref()
        {
            let _ = self.engine.set_preview_data(clip.data().clone());
        }
    }

    /// Push the Loop toggle to the sounding preview live (so toggling Loop during
    /// playback takes effect immediately). Change-gated so it doesn't flood the ring.
    pub(crate) fn editor_set_looping(&self, looping: bool) {
        if self.editor.last_loop.get() != looping {
            self.editor.last_loop.set(looping);
            let _ = self.engine.set_preview_looping(looping);
        }
    }

    /// Set the clip selection to `a..b` (frames, order-independent). Empty range
    /// clears it. Drives the range edits (Trim/Cut/Silence/Fade).
    pub(crate) fn editor_set_selection(&mut self, a: u64, b: u64) {
        if let Some(clip) = self.editor.clip.as_mut() {
            let (lo, hi) = (a.min(b) as usize, a.max(b) as usize);
            clip.set_selection(Some(lo..hi));
        }
    }

    /// Clear the clip selection.
    pub(crate) fn editor_clear_selection(&mut self) {
        if let Some(clip) = self.editor.clip.as_mut() {
            clip.set_selection(None);
        }
    }

    /// The current selection as `(start, end)` frames, if any (for the overlay).
    pub(crate) fn editor_selection(&self) -> Option<(u64, u64)> {
        self.editor
            .clip
            .as_ref()
            .and_then(|c| c.selection().map(|r| (r.start as u64, r.end as u64)))
    }

    /// Whether the loaded clip can undo / redo (dims the panel buttons).
    pub(crate) fn editor_can_undo(&self) -> bool {
        self.editor.clip.as_ref().is_some_and(|c| c.can_undo())
    }

    pub(crate) fn editor_can_redo(&self) -> bool {
        self.editor.clip.as_ref().is_some_and(|c| c.can_redo())
    }
}
