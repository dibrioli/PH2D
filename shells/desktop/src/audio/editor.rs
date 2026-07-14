//! Audio Editor shell-side runtime (docs/Audio/, W1) — split out of `audio.rs`
//! to keep that file under the HR-18 600-LOC shell cap. This is a **descendant
//! submodule** of `audio`, so its `impl AudioSystem` still reaches the private
//! fields of `AudioSystem` (Rust: private items are visible to the defining
//! module and its descendants). The transport + clip logic is unchanged.
//!
//! The effects **chain** runtime lives in the `fx_rack` submodule (same visibility
//! trick, one level deeper) — this file is the transport, the clip and the edits.

mod batch;
mod clipboard;
pub(crate) mod delivery;
mod delivery_smoke;
mod export;
mod export_pieces;
mod fx_rack;
pub(crate) mod ir;
mod loops;
mod manifest_path;
mod markers;
mod meta;
mod multiband_smoke;
pub(crate) mod pieces;
pub(crate) mod platforms;
pub(crate) mod spectral;
mod variation;
mod voice_smoke;

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
    /// The clipboard (W2). It **outlives the clip** — copy from one take, `Load` another, paste —
    /// which is exactly why `apply_paste` conforms it to the destination's rate and layout first.
    clipboard: Option<ph2d_audio::SampleData>,
    /// A piece drag in flight (Move / Scale). Nothing is committed while this exists — the overlay
    /// draws the outline and the release does the edit. See `editor/pieces.rs`.
    piece_drag: Option<pieces::PieceDrag>,
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
    /// The crossfade length (frames) the panel's slider currently asks for, refreshed each frame.
    /// It drives the **bake** now (ADR-0119 A6) — a runtime loop jumps, so a crossfade that only
    /// existed in the preview was a promise the game could not keep.
    pending_xfade: usize,
    /// Manual playhead override (full-clip frame) while scrubbing the ruler. The
    /// engine only republishes `preview_frame` while a voice is actively rendering, so
    /// a seek while stopped/paused would not move the drawn playhead — this makes the
    /// bar follow the mouse. Cleared when playback takes authority back (Play / resume
    /// / Stop / Load, or a scrub release while playing).
    scrub_frame: Option<u64>,
    /// Force-to-mono (W6): a **non-destructive** output toggle, like the fx Bypass. The
    /// clip stays stereo (undo intact); while on, `editor_sounding` returns the
    /// downmixed [`mono_view`] so Play, the waveform, Export all hear/show/write mono.
    force_mono: bool,
    /// Cached mono downmix of the current output base while [`force_mono`] is on,
    /// rebuilt only when the base buffer changes (see [`mono_sig`]).
    mono_view: Option<ph2d_audio_edit::EditClip>,
    /// Signature `(ptr, len)` of the base buffer `mono_view` was built from — so the
    /// downmix is not recomputed every frame.
    mono_sig: Option<(usize, usize)>,
    /// Variation container (W6 asset-prep): the model (entries + strategy + jitter),
    /// its decoded clips (index-aligned with `variation_set.entries`; `None` = decode
    /// failed), and the picker's selection state. Independent of the loaded clip — it
    /// is its own set of files, auditioned through the preview voice.
    variation_set: ph2d_audio_edit::VariationSet,
    variation_clips: Vec<Option<ph2d_audio::SampleData>>,
    variation_picker: ph2d_audio_edit::VariationPicker,
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
        let data = match crate::audio::decode_any::decode(&bytes) {
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
        self.editor.scrub_frame = Some(0);
        // A fresh clip has no cuts, so a Move tool left armed from the last one is a pointer with
        // no legal gesture — and a drag that does nothing reads as a broken editor, not an empty
        // one. Back to Select, which always means something.
        self.editor.piece_drag = None;
        ph2d_panel_audio_editor::tool_state::set_tool(
            ph2d_panel_audio_editor::tool_state::EditTool::Select,
        );
        // **The loop points and the markers come back with the file** (ADR-0119 A4) — the call
        // that was missing for the whole life of the feature. `editor/meta.rs` owns both ends of
        // that round trip, and gates it.
        if let Some(clip) = self.editor.clip.as_mut() {
            meta::adopt_wav_meta(clip, &bytes);
        }
        // A fresh clip starts un-mono'd.
        self.editor.force_mono = false;
        self.editor.mono_view = None;
        self.editor.mono_sig = None;
        self.editor_fx_discard();
        // …and it starts with nothing LEARNED about it. The noise profile and the
        // time-frequency band are the only spectral state not keyed on the buffer (they
        // survive edits on purpose), so a new file has to clear them explicitly — otherwise
        // Denoise stays lit after a Load and subtracts the previous file's noise floor from
        // this one (audit 2026-07-12).
        self.spectral = Default::default();
    }

    /// The clip **before** the force-mono view: the live audition when the effects rack
    /// is previewing, else the committed clip. The global Bypass forces the dry clip.
    fn editor_base(&self) -> Option<&ph2d_audio_edit::EditClip> {
        if self.editor.fx_bypass {
            return self.editor.clip.as_ref();
        }
        self.editor
            .fx_audition
            .as_ref()
            .or(self.editor.clip.as_ref())
    }

    /// The clip the editor currently **sounds and shows**: [`editor_base`] with the
    /// **force-mono** view laid on top when that toggle is on. Routing Play, the
    /// waveform, Export and Apply through here keeps them all agreeing.
    fn editor_sounding(&self) -> Option<&ph2d_audio_edit::EditClip> {
        if self.editor.force_mono
            && let Some(m) = self.editor.mono_view.as_ref()
        {
            return Some(m);
        }
        self.editor_base()
    }

    /// Whether the non-destructive force-mono view is engaged (published to the panel
    /// so its toggle lights up).
    pub(crate) fn editor_force_mono(&self) -> bool {
        self.editor.force_mono
    }

    /// Flip the force-mono output toggle. Non-destructive — the clip stays as it is;
    /// only what sounds/shows/exports changes. Rebuilds the view + live-switches the
    /// preview so the change is heard immediately.
    pub(crate) fn editor_toggle_force_mono(&mut self) {
        self.editor.force_mono = !self.editor.force_mono;
        self.editor_refresh_mono_view();
        self.editor_hot_swap();
    }

    /// Rebuild the cached mono downmix when force-mono is on and the base buffer
    /// changed (an edit, an fx audition, a Bypass flip). Cheap no-op otherwise. Call
    /// once per frame from the bridge so the mono view tracks edits.
    pub(crate) fn editor_refresh_mono_view(&mut self) {
        if !self.editor.force_mono {
            self.editor.mono_view = None;
            self.editor.mono_sig = None;
            return;
        }
        let sig = self.editor_base().map(|c| {
            let s = c.data().samples();
            (s.as_ptr() as usize, s.len())
        });
        if sig.is_none() {
            self.editor.mono_view = None;
            self.editor.mono_sig = None;
            return;
        }
        if self.editor.mono_sig == sig && self.editor.mono_view.is_some() {
            return;
        }
        if let Some(data) = self
            .editor_base()
            .map(|c| ph2d_audio_edit::force_mono(c.data()))
        {
            self.editor.mono_view = Some(ph2d_audio_edit::EditClip::new(data));
            self.editor.mono_sig = sig;
        }
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
                // The WHOLE clip, with a loop REGION (ADR-0119) — not a fabricated region-only
                // buffer. The preview now plays exactly what a game would play, so "it loops
                // cleanly in the editor" finally means something about the asset.
                //
                // This used to build a separate crossfaded buffer containing only the region, play
                // *that* on a whole-buffer loop, and then carry an offset through the playhead and
                // the scrub to undo the lie. All of that is gone: the region is the region.
                let Some(buf) = self.editor_sounding().map(|c| c.data().clone()) else {
                    return;
                };
                // The region goes on the voice **unconditionally**; `looping` decides whether it
                // is used. Gating it here instead would mean that turning Loop ON mid-playback
                // (having pressed Play with it off) gave a whole-buffer loop — the voice would
                // never have been told where the region was.
                let params = PlayParams {
                    looping,
                    loop_region: self.editor_loop_region(),
                    ..PlayParams::default()
                };
                if self.engine.play_preview(buf, params).is_ok() {
                    // Begin at the parked playhead (the vertical line), so Play ALWAYS starts from
                    // the line, never sometimes at 0. Commands drain in order before the first block
                    // renders, so there is no blip.
                    if let Some(sf) = self.editor.scrub_frame {
                        let _ = self.engine.seek_preview(sf);
                    }
                    self.editor.state = EditorTransport::Playing;
                    self.editor.started = false;
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
        // Park the playhead at 0 deterministically — the engine's reported frame can
        // linger after a free, so pin the line so the next Play starts from it.
        self.editor.scrub_frame = Some(0);
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
                // Structure: the cuts that divide the clip into pieces.
                Cmd::SplitAtMarkers => {
                    clip.split_at_markers();
                }
                Cmd::ClearCuts => {
                    clip.clear_cuts();
                }
                // Handled below — they need the runtime (the playhead, the crossfade amount), not
                // just the clip.
                Cmd::SplitAtPlayhead | Cmd::BakeLoopCrossfade => {}
                // Range ops (act on the selection).
                Cmd::Trim => clip.apply_trim(),
                Cmd::Cut | Cmd::Copy | Cmd::Paste => {} // the clipboard needs `self`, below
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
                // Spectral (W5) — they need the FFT and the shell's spectral state, so
                // they are run below, outside this `clip` borrow.
                Cmd::SpectralRepair | Cmd::LearnNoise | Cmd::Denoise => {}
            }
        }
        // Split at the playhead. The frame comes from the transport (or the scrub), which the clip
        // knows nothing about — so it is resolved out here and handed in.
        if matches!(cmd, Cmd::SplitAtPlayhead) {
            let at = self.editor_playhead_frame() as usize;
            if let Some(clip) = self.editor.clip.as_mut() {
                clip.split_at(at);
            }
        }

        // Bake the loop crossfade into the audio (ADR-0119 A6) — the crossfade amount lives on the
        // runtime, not the clip.
        if matches!(cmd, Cmd::BakeLoopCrossfade) {
            self.editor_bake_loop_crossfade();
            return;
        }

        // The clipboard ops (W2) live in `editor/clipboard.rs`. `Copy` changes nothing, so it
        // returns before the hot-swap below; Cut and Paste fall through to it like any other edit.
        if self.editor_clipboard_cmd(cmd) {
            return;
        }

        match cmd {
            Cmd::SpectralRepair => {
                self.editor_spectral_repair();
                return;
            }
            Cmd::LearnNoise => {
                self.editor_learn_noise();
                return;
            }
            Cmd::Denoise => {
                self.editor_denoise();
                return;
            }
            _ => {}
        }
        // Hot-swap the edited buffer into the sounding preview (no stop). No-op if
        // stopped — the next Play uses the edited clip.
        self.editor_hot_swap();
    }

    /// Push the Loop toggle to the sounding preview live (so toggling Loop during playback takes
    /// effect immediately). Change-gated so it doesn't flood the ring.
    ///
    /// There used to be a guard here refusing to touch the preview while a loop region was
    /// "auditioning" — because the audition was a *different buffer* that had to stay looping, and
    /// the Loop toggle would have switched it off underneath itself. The preview plays the real clip
    /// with a real region now (ADR-0119), so the toggle means exactly what it says.
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
