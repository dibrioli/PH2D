//! Audio Editor loop-points runtime (docs/Audio/, W6 — asset-prep). A **descendant
//! submodule** of `audio::editor`, so `impl AudioSystem` here still reaches the
//! private `AudioSystem`/`AudioEditorRuntime` fields (Rust descendant visibility) —
//! same trick as `fx_rack`.
//!
//! The loop region lives on the `EditClip` (metadata, not an undo edit). This file
//! drives it from the panel intents: adopt it from the selection, snap its ends to
//! zero crossings, and audition it **click-free** by playing the crossfaded loop
//! buffer (`EditClip::loop_audition_buffer`) on repeat through the preview voice —
//! reusing the whole-buffer preview loop, no RT-thread change.

use super::{AudioSystem, EditorTransport};
use ph2d_audio::PlayParams;

/// Longest loop crossfade the slider reaches, in milliseconds (mapped from the
/// panel's normalized `0..1`). 50 ms is a generous seam blend; the DSP clamps it to
/// the lead-in actually available before the loop start.
const LOOP_XFADE_MAX_MS: f32 = 50.0;
/// Half-window (fraction of the sample rate) the zero-crossing snap searches — ~5 ms,
/// close enough to not move the loop point audibly, wide enough to find a crossing.
const SNAP_WINDOW_DIV: usize = 200;

impl AudioSystem {
    /// Adopt the current selection as the loop region (no-op with no selection).
    pub(crate) fn editor_set_loop_from_selection(&mut self) {
        if let Some(clip) = self.editor.clip.as_mut() {
            clip.set_loop_from_selection();
        }
    }

    /// Snap both loop endpoints to the nearest zero crossings.
    pub(crate) fn editor_snap_loop(&mut self) {
        if let Some(clip) = self.editor.clip.as_mut() {
            let window = (clip.data().format().sample_rate as usize / SNAP_WINDOW_DIV).max(1);
            clip.snap_loop_to_zero_crossing(window);
        }
    }

    /// Clear the loop region. Stops the audition if it was running (it now loops over
    /// nothing).
    pub(crate) fn editor_clear_loop(&mut self) {
        if let Some(clip) = self.editor.clip.as_mut() {
            clip.clear_loop();
        }
        if self.editor.loop_auditioning {
            let _ = self.engine.stop_preview();
            self.editor.loop_auditioning = false;
            self.editor.loop_sig = None;
            self.editor.state = EditorTransport::Stopped;
        }
    }

    /// The loop region as `(start_secs, end_secs)`, if any (for the panel readout +
    /// the overlay).
    pub(crate) fn editor_loop_span(&self) -> Option<(f64, f64)> {
        let clip = self.editor.clip.as_ref()?;
        let lp = clip.loop_region()?;
        let fmt = clip.data().format();
        Some((
            fmt.frames_to_secs(lp.start as u64),
            fmt.frames_to_secs(lp.end as u64),
        ))
    }

    /// Dev smoke (`PH2D_AUDIO_LOOP_SMOKE=1`): stage a ready-to-audition loop with no
    /// file picking. Loads a 2 s 220 Hz tone into the editor and sets a loop over its
    /// middle third — left UN-snapped on purpose, so its endpoints sit mid-phase and a
    /// raw loop WOULD click: toggling Audition with the crossfade at 0 vs. the default
    /// is the click → click-free A/B. Open the Audio Editor pill to see the green loop
    /// brackets; Export writes the `smpl` chunk.
    pub(crate) fn editor_loop_smoke(&mut self) {
        use ph2d_audio::AudioFormat;
        let data = super::super::signals::sine_tone(AudioFormat::mono(48_000), 220.0, 2.0, 0.4);
        let _ = self.engine.stop_preview();
        self.editor.name = "loop-smoke".to_string();
        self.editor.clip = Some(ph2d_audio_edit::EditClip::new(data));
        self.editor.state = EditorTransport::Stopped;
        self.editor.loop_auditioning = false;
        self.editor.loop_sig = None;
        if let Some(clip) = self.editor.clip.as_mut() {
            let frames = clip.frame_count();
            clip.set_selection(Some(frames * 35 / 100..frames * 65 / 100));
            clip.set_loop_from_selection();
        }
    }

    /// The loop region as `(start, end)` frames, if any (for the overlay brackets).
    pub(crate) fn editor_loop_frames(&self) -> Option<(u64, u64)> {
        let lp = self.editor.clip.as_ref()?.loop_region()?;
        Some((lp.start as u64, lp.end as u64))
    }

    /// Whether the loop is currently auditioning (published back so the panel toggle
    /// tracks the real runtime state — e.g. it goes dark when the loop is cleared).
    pub(crate) fn editor_loop_auditioning(&self) -> bool {
        self.editor.loop_auditioning
    }

    /// Map the panel's normalized crossfade position (`0..1`) to a frame count at the
    /// loaded clip's sample rate.
    pub(crate) fn editor_xfade_frames(&self, norm: f32) -> usize {
        let sr = self
            .editor
            .clip
            .as_ref()
            .map(|c| c.data().format().sample_rate)
            .unwrap_or(48_000);
        let ms = norm.clamp(0.0, 1.0) * LOOP_XFADE_MAX_MS;
        (ms * 0.001 * sr as f32).round() as usize
    }

    /// Reconcile the loop audition with what the panel asks for, each frame. Edge-
    /// triggered + change-gated: starts the click-free loop on the preview voice when
    /// `want` turns on, stops it when it turns off, and hot-swaps a fresh crossfaded
    /// buffer (no restart) when the region or crossfade changes while it plays.
    pub(crate) fn editor_update_loop_audition(&mut self, want: bool, xfade_frames: usize) {
        let sig = self
            .editor
            .clip
            .as_ref()
            .and_then(|c| c.loop_region())
            .map(|lp| (lp.start, lp.end, xfade_frames));
        let want = want && sig.is_some();

        if !want {
            if self.editor.loop_auditioning {
                let _ = self.engine.stop_preview();
                self.editor.loop_auditioning = false;
                self.editor.loop_sig = None;
                self.editor.state = EditorTransport::Stopped;
            }
            return;
        }
        // Already looping this exact region + crossfade → nothing to do.
        if self.editor.loop_auditioning && self.editor.loop_sig == sig {
            return;
        }
        let Some(buf) = self
            .editor
            .clip
            .as_ref()
            .and_then(|c| c.loop_audition_buffer(xfade_frames))
        else {
            return;
        };
        if self.editor.loop_auditioning {
            // Same audition, moved region/crossfade → hot-swap, keeping the loop going.
            let _ = self.engine.set_preview_data(buf);
        } else {
            let params = PlayParams {
                looping: true,
                ..PlayParams::default()
            };
            if self.engine.play_preview(buf, params).is_ok() {
                self.editor.loop_auditioning = true;
                self.editor.state = EditorTransport::Playing;
                self.editor.started = false;
            }
        }
        self.editor.loop_sig = sig;
    }
}
