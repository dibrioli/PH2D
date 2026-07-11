//! Audio Editor loop-points runtime (docs/Audio/, W6 — asset-prep). A **descendant
//! submodule** of `audio::editor`, so `impl AudioSystem` here still reaches the
//! private `AudioSystem`/`AudioEditorRuntime` fields (Rust descendant visibility) —
//! same trick as `fx_rack`.
//!
//! The loop region lives on the `EditClip` (metadata, not an undo edit). It is played
//! **click-free** by the transport: when **Loop** is on and a region is set, **Play**
//! loops the crossfaded region buffer (`EditClip::loop_audition_buffer`) — reusing the
//! whole-buffer preview loop, no RT-thread change. There is no separate Audition
//! control; Loop + Play *is* the audition (see `editor_toggle_play`).

use super::{AudioSystem, EditorTransport};

/// Longest loop crossfade the slider reaches, in milliseconds (mapped from the
/// panel's normalized `0..1`). 50 ms is a generous seam blend; the DSP clamps it to
/// the lead-in actually available before the loop start.
const LOOP_XFADE_MAX_MS: f32 = 50.0;
/// Half-window (fraction of the sample rate) the zero-crossing snap searches — ~5 ms,
/// close enough to not move the loop point audibly, wide enough to find a crossing.
const SNAP_WINDOW_DIV: usize = 200;

impl AudioSystem {
    /// Adopt the current selection as the loop region, **snapping both ends to zero
    /// crossings** (no-op with no selection). Snap is folded into Set so a loop always
    /// starts on clean crossings without a separate button — it moves the endpoints by
    /// under a millisecond, so there is nothing to see, only the click it prevents.
    pub(crate) fn editor_set_loop_from_selection(&mut self) {
        if let Some(clip) = self.editor.clip.as_mut() {
            clip.set_loop_from_selection();
            let window = (clip.data().format().sample_rate as usize / SNAP_WINDOW_DIV).max(1);
            clip.snap_loop_to_zero_crossing(window);
        }
    }

    /// Clear the loop region. Stops region playback if it was running (nothing left to
    /// loop).
    pub(crate) fn editor_clear_loop(&mut self) {
        if let Some(clip) = self.editor.clip.as_mut() {
            clip.clear_loop();
        }
        if self.editor.playing_loop_region {
            let _ = self.engine.stop_preview();
            self.editor.playing_loop_region = false;
            self.editor.loop_sig = None;
            self.editor.state = EditorTransport::Stopped;
            self.editor.scrub_frame = None;
        }
    }

    /// The loop region as `(start_secs, end_secs)`, if any (for the panel readout).
    pub(crate) fn editor_loop_span(&self) -> Option<(f64, f64)> {
        let clip = self.editor.clip.as_ref()?;
        let lp = clip.loop_region()?;
        let fmt = clip.data().format();
        Some((
            fmt.frames_to_secs(lp.start as u64),
            fmt.frames_to_secs(lp.end as u64),
        ))
    }

    /// The loop region as `(start, end)` frames, if any (for the overlay brackets).
    pub(crate) fn editor_loop_frames(&self) -> Option<(u64, u64)> {
        let lp = self.editor.clip.as_ref()?.loop_region()?;
        Some((lp.start as u64, lp.end as u64))
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

    /// Cache the crossfade the panel asks for, so the transport reads one source.
    pub(crate) fn editor_set_pending_xfade(&mut self, frames: usize) {
        self.editor.pending_xfade = frames;
    }

    /// While a loop region is playing, hot-swap a freshly crossfaded buffer when the
    /// region or crossfade changed — so tuning the Crossfade slider (or moving the
    /// loop) updates the sounding loop live. **Never starts playback**, so it cannot
    /// fight Stop (that was the bug when a persistent Audition toggle drove this).
    pub(crate) fn editor_loop_live_update(&mut self) {
        if !self.editor.playing_loop_region {
            return;
        }
        let xfade = self.editor.pending_xfade;
        let Some(sig) = self
            .editor
            .clip
            .as_ref()
            .and_then(|c| c.loop_region())
            .map(|lp| (lp.start, lp.end, xfade))
        else {
            return;
        };
        if self.editor.loop_sig == Some(sig) {
            return;
        }
        if let Some(buf) = self
            .editor
            .clip
            .as_ref()
            .and_then(|c| c.loop_audition_buffer(xfade))
        {
            let _ = self.engine.set_preview_data(buf);
            self.editor.loop_sig = Some(sig);
        }
    }

    /// Seek the preview by grabbing the playhead: map a **full-clip** frame to the
    /// voice's current buffer — the region (offset by its start) while looping a
    /// region, else the whole clip — AND record it as the manual playhead override so
    /// the drawn bar follows the mouse even while stopped/paused (when the engine does
    /// not republish `preview_frame`).
    pub(crate) fn editor_scrub_to_frame(&mut self, full_frame: u64) {
        let visual = if self.editor.playing_loop_region {
            match self.editor.clip.as_ref().and_then(|c| c.loop_region()) {
                Some(lp) => {
                    let (s, e) = (lp.start as u64, lp.end as u64);
                    let clamped = full_frame.clamp(s, e);
                    let _ = self.engine.seek_preview(clamped - s);
                    clamped
                }
                None => full_frame,
            }
        } else {
            let _ = self.engine.seek_preview(full_frame);
            full_frame
        };
        self.editor.scrub_frame = Some(visual);
    }

    /// End a ruler scrub. If playback is advancing, hand the playhead back to it;
    /// otherwise leave the manual position frozen where the user dropped it (stopped /
    /// paused → the bar stays put).
    pub(crate) fn editor_end_scrub(&mut self) {
        if self.editor.state == EditorTransport::Playing {
            self.editor.scrub_frame = None;
        }
    }

    /// The frame to DRAW the playhead at (full-clip timebase). A live scrub override
    /// wins (the bar follows the mouse); otherwise it's the preview position — offset
    /// by the loop start while looping a region (the region plays as its own buffer
    /// whose frames start at 0, so without the offset the line sweeps at the far left,
    /// OUTSIDE the loop brackets).
    pub(crate) fn editor_playhead_frame(&self) -> u64 {
        if let Some(f) = self.editor.scrub_frame {
            return f;
        }
        let raw = self.engine.preview_frame();
        if self.editor.playing_loop_region {
            let start = self
                .editor
                .clip
                .as_ref()
                .and_then(|c| c.loop_region())
                .map(|lp| lp.start as u64)
                .unwrap_or(0);
            start + raw
        } else {
            raw
        }
    }

    /// Dev smoke (`PH2D_AUDIO_LOOP_SMOKE=1`): stage a loop with no file picking. Loads
    /// a 2 s 220 Hz tone and sets a loop over its middle third **UN-snapped** on
    /// purpose (endpoints mid-phase → a raw loop would click), so turning **Loop** on
    /// and pressing **Play** with the Crossfade at 0 vs. the default is the click →
    /// click-free A/B. Open the Audio Editor pill to see the green loop brackets;
    /// Export writes the `smpl` chunk.
    pub(crate) fn editor_loop_smoke(&mut self) {
        use ph2d_audio::AudioFormat;
        let data = super::super::signals::sine_tone(AudioFormat::mono(48_000), 220.0, 2.0, 0.4);
        let _ = self.engine.stop_preview();
        self.editor.name = "loop-smoke".to_string();
        self.editor.clip = Some(ph2d_audio_edit::EditClip::new(data));
        self.editor.state = EditorTransport::Stopped;
        self.editor.playing_loop_region = false;
        self.editor.loop_sig = None;
        if let Some(clip) = self.editor.clip.as_mut() {
            let frames = clip.frame_count();
            // Direct, un-snapped region (bypasses the auto-snap in Set) so the
            // crossfade has a real click to remove.
            clip.set_loop_region(Some(frames * 35 / 100..frames * 65 / 100));
        }
    }
}
