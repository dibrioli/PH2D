//! Audio Editor shell-side runtime (docs/Audio/, W1) — split out of `audio.rs`
//! to keep that file under the HR-18 600-LOC shell cap. This is a **descendant
//! submodule** of `audio`, so its `impl AudioSystem` still reaches the private
//! fields of `AudioSystem` (Rust: private items are visible to the defining
//! module and its descendants). The transport + clip logic is unchanged.
//!
//! The effects **chain** runtime lives in the `fx_rack` submodule (same visibility
//! trick, one level deeper) — this file is the transport, the clip and the edits.

mod fx_rack;
mod loops;

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
    /// The effects rack's **live audition** (W3 blocks 3a/3b): the whole chain
    /// rendered over `clip` but NOT committed. While present it is what sounds and
    /// what the waveform shows; `clip` stays pristine so Cancel is free. Apply
    /// commits this exact buffer, so what the user heard lands in the undo timeline.
    fx_audition: Option<ph2d_audio_edit::EditClip>,
    /// What produced `fx_audition`. Change-gate so a slider that doesn't actually
    /// move can't re-render the whole clip every frame.
    fx_sig: Option<fx_rack::ChainSig>,
    /// `clip` with the chain stages BEFORE the one being edited already rendered
    /// into it — cached, so dragging a slider re-renders only that stage and the
    /// ones below it (usually exactly one).
    fx_head: Option<ph2d_audio_edit::EditClip>,
    /// What produced `fx_head`.
    fx_head_sig: Option<fx_rack::HeadSig>,
    /// Global A/B: the dry clip sounds/shows/exports while the chain is kept intact.
    fx_bypass: bool,
    /// Loop-region playback (W6): `true` iff the preview voice currently holds the
    /// crossfaded loop buffer rather than the full clip — i.e. Play was pressed with
    /// Loop on and a region set. Any action that reloads the preview (Play a full
    /// clip, Stop, Load, Clear loop) clears it; `editor_toggle_play` is where it goes
    /// true.
    playing_loop_region: bool,
    /// Change-gate for the loop buffer — `(start, end, xfade_frames)`. While a region
    /// is playing, `editor_loop_live_update` hot-swaps a fresh buffer only when this
    /// moves (so tuning the Crossfade slider updates the sounding loop).
    loop_sig: Option<(usize, usize, usize)>,
    /// The crossfade length (frames) the panel's slider currently asks for, refreshed
    /// each frame so `editor_toggle_play` / `editor_loop_live_update` read one source.
    pending_xfade: usize,
    /// Manual playhead override (full-clip frame) while scrubbing the ruler. The
    /// engine only republishes `preview_frame` while a voice is actively rendering, so
    /// a seek while stopped/paused would not move the drawn playhead — this makes the
    /// bar follow the mouse. Cleared when playback takes authority back (Play / resume
    /// / Stop / Load, or a scrub release while playing).
    scrub_frame: Option<u64>,
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
        self.editor.playing_loop_region = false;
        self.editor.loop_sig = None;
        self.editor.scrub_frame = Some(0);
        self.editor_fx_discard();
    }

    /// The clip the editor currently **sounds and shows**: the live audition when
    /// the effects rack is previewing, else the committed clip. The global Bypass
    /// forces the dry clip — that is the whole point of the A/B, and routing Play,
    /// the waveform, Export and Apply through here is what keeps them agreeing.
    fn editor_sounding(&self) -> Option<&ph2d_audio_edit::EditClip> {
        if self.editor.fx_bypass {
            return self.editor.clip.as_ref();
        }
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

    /// Cycle the transport: Stopped → play from the playhead, Playing → pause,
    /// Paused → resume. `looping` is read at play-from-start time.
    ///
    /// **Loop unifies with the region (W6):** when Loop is on AND a loop region is set,
    /// Play plays the **click-free crossfaded region** on repeat (the green brackets);
    /// otherwise it plays the whole sounding clip (looping per the toggle). There is no
    /// separate Audition control — Loop + Play *is* the audition, so Stop/Pause behave
    /// normally with nothing to re-trigger them.
    pub(crate) fn editor_toggle_play(&mut self, looping: bool) {
        match self.editor.state {
            EditorTransport::Stopped => {
                let xfade = self.editor.pending_xfade;
                // Region buffer only when Loop is on and a region exists.
                let region = looping
                    .then(|| {
                        self.editor
                            .clip
                            .as_ref()
                            .and_then(|c| c.loop_audition_buffer(xfade))
                    })
                    .flatten();
                let plays_region = region.is_some();
                let sig = plays_region
                    .then(|| {
                        self.editor
                            .clip
                            .as_ref()
                            .and_then(|c| c.loop_region())
                            .map(|lp| (lp.start, lp.end, xfade))
                    })
                    .flatten();
                // Region buffer, or the sounding clip (audition when the rack previews).
                let buf = match region {
                    Some(b) => Some(b),
                    None => self.editor_sounding().map(|c| c.data().clone()),
                };
                let Some(buf) = buf else {
                    return;
                };
                let params = PlayParams {
                    looping,
                    ..PlayParams::default()
                };
                if self.engine.play_preview(buf, params).is_ok() {
                    // Begin at the parked playhead (the vertical line) — map it into the
                    // buffer and seek right after starting, so Play ALWAYS starts from
                    // the line, never sometimes at 0. Commands drain in order before the
                    // first block renders, so there is no blip.
                    if let Some(sf) = self.editor.scrub_frame {
                        let start = if plays_region {
                            self.editor
                                .clip
                                .as_ref()
                                .and_then(|c| c.loop_region())
                                .map(|lp| {
                                    sf.clamp(lp.start as u64, lp.end as u64) - lp.start as u64
                                })
                                .unwrap_or(0)
                        } else {
                            sf
                        };
                        let _ = self.engine.seek_preview(start);
                    }
                    self.editor.state = EditorTransport::Playing;
                    self.editor.started = false;
                    self.editor.playing_loop_region = plays_region;
                    self.editor.loop_sig = sig;
                    self.editor.scrub_frame = None; // playback drives the playhead now
                }
            }
            EditorTransport::Playing => {
                let _ = self.engine.pause_preview(true);
                self.editor.state = EditorTransport::Paused;
            }
            EditorTransport::Paused => {
                let _ = self.engine.pause_preview(false);
                self.editor.state = EditorTransport::Playing;
                self.editor.scrub_frame = None; // resume from the (possibly scrubbed) cursor
            }
        }
    }

    /// Stop the preview and rewind to the clip start.
    pub(crate) fn editor_stop(&mut self) {
        let _ = self.engine.stop_preview();
        self.editor.state = EditorTransport::Stopped;
        self.editor.playing_loop_region = false;
        self.editor.loop_sig = None;
        // Park the playhead at 0 deterministically — the engine's reported frame can
        // linger after a free, so pin the line so the next Play starts from it.
        self.editor.scrub_frame = Some(0);
    }

    /// Write the loaded clip out to `path` as a 16-bit PCM WAV.
    pub(crate) fn editor_export(&self, path: &std::path::Path) {
        // Export what is SOUNDING and SHOWN — the live audition when the rack is
        // previewing, else the committed clip. `editor_clip` / `editor_duration_secs`
        // already report the audition, so exporting `self.editor.clip` here silently
        // wrote a dry, shorter file than the waveform on screen (2026-07-09 audit).
        let Some(clip) = self.editor_sounding() else {
            return;
        };
        // Carry the loop region into the WAV's `smpl` chunk so the loop is sample-exact
        // on re-import / in a game runtime. The loop is metadata on the COMMITTED clip;
        // clamp it to the exported buffer (a length-preserving audition keeps the frame
        // indices, a reverb tail only extends past the loop end).
        let frames = clip.frame_count() as u32;
        let loops: Vec<_> = self
            .editor
            .clip
            .as_ref()
            .and_then(|c| c.loop_region())
            .and_then(|lp| {
                let start = (lp.start as u32).min(frames);
                let end = (lp.end as u32).min(frames);
                (start < end).then_some(ph2d_audio_encode::LoopRegion { start, end })
            })
            .into_iter()
            .collect();
        let meta = ph2d_audio_encode::WavMeta { loops };
        match ph2d_audio_encode::write_wav_with_meta(
            path,
            clip.data(),
            ph2d_audio_encode::BitDepth::Pcm16,
            &meta,
        ) {
            Ok(()) => println!(
                "audio: exported {} (WAV PCM16{})",
                path.display(),
                if meta.loops.is_empty() {
                    ""
                } else {
                    ", smpl loop"
                }
            ),
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
                self.editor.scrub_frame = Some(0); // rewind the line to the start
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
                let chain = ph2d_panel_audio_editor::fx_chain();
                let sel = ph2d_panel_audio_editor::fx_sel();
                self.editor_fx_apply(&chain, sel);
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
        // The loop audition forces looping on the preview; the main Loop toggle must
        // not reach in and turn it off underneath it.
        if self.editor.playing_loop_region {
            return;
        }
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
