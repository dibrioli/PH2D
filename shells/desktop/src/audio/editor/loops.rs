//! Audio Editor loop-points runtime (docs/Audio/, W6 — asset-prep; **ADR-0119**). A **descendant
//! submodule** of `audio::editor`, so `impl AudioSystem` here still reaches the private
//! `AudioSystem`/`AudioEditorRuntime` fields (Rust descendant visibility) — same trick as `fx_rack`.
//!
//! The loop region is part of the clip's structure. It is played by handing it to the **mixer**
//! (`PlayParams::loop_region`): Loop + Play plays the whole clip with a real loop region, so the
//! preview is what a game would hear, sample for sample.
//!
//! ## What this used to be
//!
//! It used to build a **separate buffer** containing only the region, crossfaded, and play *that* on
//! a whole-buffer loop — then carry an offset through the playhead and the scrub to hide the fact.
//! The mixer had no loop region, so the whole Loop section was authoring for a tool that did not
//! exist. The offsets, the `playing_loop_region` flag, the hot-swap and the fabricated buffer are
//! all gone; the region is simply the region (ADR-0119 A5).
//!
//! ## The crossfade is a bake now
//!
//! A runtime loop **jumps**; it does not crossfade (that needs a second read head, and on a stream
//! it needs audio the producer has already thrown away). So the Crossfade slider stopped being a
//! preview trick and became a destructive edit — one undo step — that writes the seam **into the
//! audio**, using the intro as pre-roll. What gets exported already loops cleanly, and the editor
//! and the game hear the same thing because there is only one thing to hear.

use super::{AudioSystem, EditorTransport};

/// Longest loop crossfade the slider reaches, in milliseconds (mapped from the panel's normalized
/// `0..1`). 50 ms is a generous seam blend; the bake clamps it to the lead-in actually available
/// before the loop start.
const LOOP_XFADE_MAX_MS: f32 = 50.0;
/// Half-window (fraction of the sample rate) the zero-crossing snap searches — ~5 ms, close enough
/// to not move the loop point audibly, wide enough to find a crossing.
const SNAP_WINDOW_DIV: usize = 200;

impl AudioSystem {
    /// Adopt the current selection as the loop region, **snapping both ends to zero crossings**
    /// (no-op with no selection). Snap is folded into Set so a loop always starts on clean crossings
    /// without a separate button — it moves the endpoints by under a millisecond, so there is nothing
    /// to see, only the click it prevents.
    pub(crate) fn editor_set_loop_from_selection(&mut self) {
        if let Some(clip) = self.editor.clip.as_mut() {
            clip.set_loop_from_selection();
            let window = (clip.data().format().sample_rate as usize / SNAP_WINDOW_DIV).max(1);
            clip.snap_loop_to_zero_crossing(window);
        }
        self.editor_refresh_preview_loop();
    }

    /// Clear the loop region. The preview keeps playing — it is playing the whole clip, and always
    /// was; only the turn-around goes away.
    pub(crate) fn editor_clear_loop(&mut self) {
        if let Some(clip) = self.editor.clip.as_mut() {
            clip.clear_loop();
        }
        self.editor_refresh_preview_loop();
    }

    /// The loop region as [`ph2d_audio::LoopRegion`] — what the mixer is handed.
    pub(crate) fn editor_loop_region(&self) -> Option<ph2d_audio::LoopRegion> {
        let lp = self.editor.clip.as_ref()?.loop_region()?;
        Some(ph2d_audio::LoopRegion {
            start: lp.start as u64,
            end: lp.end as u64,
        })
    }

    /// Push the current region to a **sounding** preview, so moving the loop while it plays takes
    /// effect on the next lap. No-op when nothing is playing.
    ///
    /// (Re-issuing `PlayPreview` would restart the clip; the region rides on the voice, so the
    /// engine sets it in place.)
    pub(crate) fn editor_refresh_preview_loop(&mut self) {
        if self.editor.state == EditorTransport::Stopped {
            return;
        }
        let _ = self
            .engine
            .set_preview_loop_region(self.editor_loop_region());
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

    /// Map the panel's normalized crossfade position (`0..1`) to a frame count at the loaded clip's
    /// sample rate.
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

    /// Cache the crossfade the panel asks for, so the bake reads one source.
    pub(crate) fn editor_set_pending_xfade(&mut self, frames: usize) {
        self.editor.pending_xfade = frames;
    }

    /// **Bake the loop crossfade into the audio** (ADR-0119 A6) — one undo step.
    ///
    /// Writes the seam into the clip so that jumping `end → start` is continuous, which is the only
    /// kind of clean loop a runtime that jumps can play. Needs the intro as pre-roll, so it is a
    /// no-op with a loop that starts at 0 (there is nothing before the loop to fade from — the tool
    /// there is the zero-crossing snap).
    pub(crate) fn editor_bake_loop_crossfade(&mut self) {
        let xfade = self.editor.pending_xfade;
        if let Some(clip) = self.editor.clip.as_mut() {
            clip.apply_loop_crossfade(xfade);
        }
        self.editor_hot_swap();
    }

    /// Whether a crossfade bake would do anything — what lights the button. There must be a loop,
    /// a non-zero crossfade, and **audio before the loop start** to fade from.
    pub(crate) fn editor_can_bake_crossfade(&self) -> bool {
        let Some(clip) = self.editor.clip.as_ref() else {
            return false;
        };
        clip.loop_region().is_some_and(|lp| lp.start > 0) && self.editor.pending_xfade > 0
    }

    /// Seek the preview by grabbing the playhead — and record it as the manual playhead override so
    /// the drawn bar follows the mouse even while stopped/paused (when the engine does not republish
    /// `preview_frame`).
    ///
    /// One timebase now: the preview plays the whole clip, so a clip frame IS a preview frame. The
    /// clamp-and-offset dance that mapped a full-clip frame into a fabricated region buffer is gone
    /// with the buffer.
    pub(crate) fn editor_scrub_to_frame(&mut self, full_frame: u64) {
        let _ = self.engine.seek_preview(full_frame);
        self.editor.scrub_frame = Some(full_frame);
    }

    /// End a ruler scrub. If playback is advancing, hand the playhead back to it; otherwise leave the
    /// manual position frozen where the user dropped it (stopped / paused → the bar stays put).
    pub(crate) fn editor_end_scrub(&mut self) {
        if self.editor.state == EditorTransport::Playing {
            self.editor.scrub_frame = None;
        }
    }

    /// The frame to DRAW the playhead at. A live scrub override wins (the bar follows the mouse);
    /// otherwise it is the preview position, straight — no offset, because the preview is the clip.
    pub(crate) fn editor_playhead_frame(&self) -> u64 {
        if let Some(f) = self.editor.scrub_frame {
            return f;
        }
        self.engine.preview_frame()
    }

    /// Dev smoke (`PH2D_AUDIO_LOOP_SMOKE=1`): stage a demo clip with no file picking — a 2 s
    /// **stereo** tone (L 220 Hz, R 223 Hz, so it's genuinely two channels) with a loop over its
    /// middle third **UN-snapped** on purpose (endpoints mid-phase → a raw loop would click). Open
    /// the Audio Editor pill to see the two-lane waveform + green loop brackets.
    ///
    /// Demos: **Loop** + **Play** now plays the intro once and then the region for ever, in the
    /// mixer, exactly as a game would; **Crossfade Loop** bakes the seam so the click goes away for
    /// good (and Undo brings it back); **Force Mono** collapses the two lanes; **Export** writes the
    /// `smpl` chunk, and **Load**ing that file back brings the loop and the markers with it.
    pub(crate) fn editor_loop_smoke(&mut self) {
        use ph2d_audio::{AudioFormat, SampleData};
        let sr = 48_000u32;
        let frames = sr as usize * 2; // 2 s
        let l_step = std::f32::consts::TAU * 220.0 / sr as f32;
        let r_step = std::f32::consts::TAU * 223.0 / sr as f32; // detuned → real stereo + downmix
        let mut v = Vec::with_capacity(frames * 2);
        for i in 0..frames {
            v.push((i as f32 * l_step).sin() * 0.4);
            v.push((i as f32 * r_step).sin() * 0.4);
        }
        let data = SampleData::from_interleaved(v, AudioFormat::stereo(sr));
        let _ = self.engine.stop_preview();
        self.editor.name = "loop-smoke".to_string();
        self.editor.clip = Some(ph2d_audio_edit::EditClip::new(data));
        self.editor.state = EditorTransport::Stopped;
        self.editor.scrub_frame = Some(0);
        if let Some(clip) = self.editor.clip.as_mut() {
            let frames = clip.frame_count();
            // Direct, un-snapped region (bypasses the auto-snap in Set) so the crossfade bake has a
            // real click to remove.
            clip.set_loop_region(Some(frames * 35 / 100..frames * 65 / 100));
            // A couple of cue markers so the purple flags are visible on load.
            clip.add_marker(frames / 4, "M1");
            clip.add_marker(frames * 3 / 4, "M2");
        }
        // Seed a variation container too (four pitched blips) so the Variations section is live on
        // launch — Play Variation cycles them.
        self.editor_variation_smoke();
    }
}
