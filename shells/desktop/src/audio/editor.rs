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
    /// The effects rack's **live audition** (W3 block 3a): the effect rendered over
    /// `clip` but NOT committed. While present it is what sounds and what the
    /// waveform shows; `clip` stays pristine so Cancel is free. Apply commits this
    /// exact buffer, so what the user heard is what lands in the undo timeline.
    fx_audition: Option<ph2d_audio_edit::EditClip>,
    /// What produced `fx_audition`. Change-gate so a slider that doesn't actually
    /// move can't re-render the whole clip every frame.
    fx_sig: Option<FxSig>,
}

/// Everything the audition depends on: the effect, its parameters, **and the
/// target range** — moving the selection re-targets the effect, so a stale
/// audition rendered against the old range must be thrown away.
type FxSig = (
    usize,
    [i32; ph2d_panel_audio_editor::MAX_FX_PARAMS],
    Option<(usize, usize)>,
);

/// Quantize the slider normals so float jitter doesn't defeat the change-gate.
fn fx_sig(
    kind: usize,
    norms: &[f32; ph2d_panel_audio_editor::MAX_FX_PARAMS],
    selection: Option<(usize, usize)>,
) -> FxSig {
    let mut q = [0i32; ph2d_panel_audio_editor::MAX_FX_PARAMS];
    for (slot, n) in q.iter_mut().zip(norms) {
        *slot = (n * 1_000.0) as i32;
    }
    (kind, q, selection)
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
        self.editor_fx_discard();
    }

    /// The clip the editor currently **sounds and shows**: the live audition when
    /// the effects rack is previewing, else the committed clip.
    fn editor_sounding(&self) -> Option<&ph2d_audio_edit::EditClip> {
        self.editor
            .fx_audition
            .as_ref()
            .or(self.editor.clip.as_ref())
    }

    /// Push whatever should be sounding into the running preview voice, keeping
    /// its cursor (no restart). No-op when the preview is not playing.
    fn editor_hot_swap(&self) {
        if self.engine.preview_playing()
            && let Some(clip) = self.editor_sounding()
        {
            let _ = self.engine.set_preview_data(clip.data().clone());
        }
    }

    /// Drop the audition without touching the clip (used by load / by edits that
    /// would invalidate it). Does not restore the preview — callers that need the
    /// old sound back call [`AudioSystem::editor_fx_cancel`].
    fn editor_fx_discard(&mut self) {
        self.editor.fx_audition = None;
        self.editor.fx_sig = None;
        ph2d_panel_audio_editor::clear_fx_dirty();
    }

    /// Re-render the effects rack's audition when the user has touched it and the
    /// parameters actually changed, then hot-swap it into the sounding preview so
    /// it is heard **live**. The clip stays pristine — this is non-destructive.
    ///
    /// Cost is one full effect render over the target range per *parameter change*
    /// (change-gated, so at most once per frame while dragging). Selecting a range
    /// scopes the render, which is what keeps long clips responsive.
    pub(crate) fn editor_fx_update(
        &mut self,
        kind: usize,
        norms: &[f32; ph2d_panel_audio_editor::MAX_FX_PARAMS],
    ) {
        let Some(clip) = self.editor.clip.as_ref() else {
            return;
        };
        let selection = clip.selection().map(|r| (r.start, r.end));
        let sig = fx_sig(kind, norms, selection);
        if self.editor.fx_sig == Some(sig) {
            return;
        }
        let rendered = match super::fx_params::build(kind, norms) {
            Some(super::fx_params::FxCommand::Plain(fx)) => clip.render_effect(fx),
            Some(super::fx_params::FxCommand::Tail(fx)) => clip.render_tail_effect(fx),
            None => return,
        };
        self.editor.fx_audition = Some(ph2d_audio_edit::EditClip::new(rendered));
        self.editor.fx_sig = Some(sig);
        self.editor_hot_swap();
    }

    /// Commit the audition as one undo step — **the exact buffer that was heard**.
    /// Falls back to rendering now if Apply is pressed without any audition.
    pub(crate) fn editor_fx_apply(
        &mut self,
        kind: usize,
        norms: &[f32; ph2d_panel_audio_editor::MAX_FX_PARAMS],
    ) {
        if self.editor.fx_audition.is_none() {
            self.editor_fx_update(kind, norms);
        }
        if let Some(audition) = self.editor.fx_audition.take()
            && let Some(clip) = self.editor.clip.as_mut()
        {
            clip.commit_rendered(audition.data().clone());
        }
        self.editor_fx_discard();
        self.editor_hot_swap();
    }

    /// Throw the audition away and put the committed clip back on the wire.
    pub(crate) fn editor_fx_cancel(&mut self) {
        self.editor_fx_discard();
        self.editor_hot_swap();
    }

    /// Whether an audition is currently sounding (enables the panel's Cancel).
    pub(crate) fn editor_fx_auditioning(&self) -> bool {
        self.editor.fx_audition.is_some()
    }

    /// Cycle the transport: Stopped → play from 0, Playing → pause, Paused →
    /// resume. `looping` is read at play-from-start time.
    pub(crate) fn editor_toggle_play(&mut self, looping: bool) {
        match self.editor.state {
            EditorTransport::Stopped => {
                // Play what is currently SHOWING — the audition when the rack is
                // previewing, so pressing Play mid-audition hears the effect.
                if let Some(clip) = self.editor_sounding() {
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

    /// Duration of what is sounding/showing — the audition when previewing (a
    /// reverb tail makes it longer than the committed clip), else the clip.
    pub(crate) fn editor_duration_secs(&self) -> f64 {
        self.editor_sounding()
            .map(|c| c.duration_secs())
            .unwrap_or(0.0)
    }

    /// The loaded clip's display name.
    pub(crate) fn editor_name(&self) -> &str {
        &self.editor.name
    }

    /// The clip the overlay draws: the live audition when the rack is previewing
    /// (so the waveform shows the effect, tail and all), else the committed clip.
    /// The **selection** still comes from the committed clip via
    /// [`AudioSystem::editor_selection`].
    pub(crate) fn editor_clip(&self) -> Option<&ph2d_audio_edit::EditClip> {
        self.editor_sounding()
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
        use ph2d_panel_audio_editor::AudioEditCmd as Cmd;
        // The effects rack auditions over the CURRENT clip, so settle it first:
        // Apply/Cancel are its own verbs, and any OTHER edit would move the clip
        // out from under the audition (leaving a preview of a buffer that no
        // longer exists) — cancel it and edit the committed clip.
        match cmd {
            Cmd::ApplyFx => {
                let kind = ph2d_panel_audio_editor::fx_kind();
                let norms = ph2d_panel_audio_editor::fx_norms();
                self.editor_fx_apply(kind, &norms);
                return;
            }
            Cmd::CancelFx => {
                self.editor_fx_cancel();
                return;
            }
            _ if self.editor.fx_audition.is_some() => self.editor_fx_cancel(),
            _ => {}
        }
        {
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
                // Handled above (they act on the audition, not on `clip`).
                Cmd::ApplyFx | Cmd::CancelFx => {}
            }
        }
        // Hot-swap the edited buffer into the sounding preview (no stop). No-op if
        // stopped — the next Play uses the edited clip.
        self.editor_hot_swap();
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
