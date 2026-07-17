//! The Spectral bridge (W5, ADR-0122): the spectrogram the overlay draws, and the three
//! frequency-domain edits.
//!
//! The panel cannot do any of this — it has no clip and no FFT — so it owns the view toggle
//! and the Amount slider, and everything else lives here. `ph2d-audio-spectral` is a
//! dependency of the SHELL, not of `ph2d-audio-edit`: the FFT stays confined to the one
//! crate that needs it, and the edit crate never learns about it. Each result is committed
//! through `EditClip::commit_rendered`, so a repair or a denoise is one ordinary undo step
//! like any other edit.
//!
//! **The picture is cached, because building it is an FFT of the whole clip.** Twice, in
//! fact: once for the analysis (keyed on the buffer) and once for the RGBA (keyed on the
//! buffer, the size on screen, and the theme). Without both, the overlay would re-analyse
//! the clip every frame it is painted.

use ph2d_audio_spectral::{Band, NoiseProfile, Spectrogram};
use ph2d_tokens::{ColorToken, Theme};
use std::sync::Arc;

/// Everything spectral the shell holds for the loaded clip.
///
/// **Two of these fields are LEARNED, and learned state has to be told when the clip it was
/// learned from is gone.** The two caches below are keyed on the buffer, so a new clip is a
/// cache miss and they are safe by construction. The profile and the band are not keyed on
/// anything — they *survive edits on purpose* (learn once, denoise, undo, denoise again at a
/// different amount) — so nothing distinguished "survive an edit" from "survive a different
/// file". Load a room tone, Learn, load a dialogue: Denoise stayed lit and subtracted the
/// shape of a noise floor from a recording that never had it. Silent, and destructive
/// (audit 2026-07-12). `editor_load` now clears this whole struct.
#[derive(Default)]
pub(crate) struct SpectralState {
    /// The analysed picture, and the buffer it was analysed from.
    sg: Option<Spectrogram>,
    sg_key: Option<BufKey>,
    /// The rendered picture, and what it was rendered for.
    img: Option<Arc<Vec<u8>>>,
    img_key: Option<ImgKey>,
    /// The learned noise floor. Survives edits on purpose: you learn it from the room
    /// tone once, then denoise, then maybe undo and denoise again at a different amount.
    profile: Option<NoiseProfile>,
    /// The frequency half of a time-frequency selection, as a fraction of Nyquist
    /// (`0.0` = DC, `1.0` = Nyquist). The TIME half is the clip's ordinary selection —
    /// there is only ever one selection, and in the spectrogram it simply gains an axis.
    band: Option<(f32, f32)>,
}

/// Identifies a clip buffer — see `ph2d_audio::SampleData::version`.
///
/// This used to be the buffer's ADDRESS, on the reasoning that a `SampleData` is immutable, so a
/// new buffer is a new pointer and any edit hands us a different one. ADR-0124 ended that: a range
/// edit now rewrites the samples where they lie, and the address of an edited clip is the address
/// of the clip before it. Asking the buffer is the same question with an answer that survives.
type BufKey = ph2d_audio::BufferVersion;

/// What a rendered image depends on: the buffer, the size it is drawn at, and the theme
/// (which supplies the colour ramp).
#[derive(PartialEq, Eq, Clone, Copy)]
struct ImgKey {
    buf: BufKey,
    w: u32,
    h: u32,
    theme: Theme,
}

impl super::super::AudioSystem {
    /// Publish what the Spectral section has to work with. Called once per frame from the
    /// bridge; a cache hit on all but the frame after something changed.
    pub(crate) fn editor_publish_spectral(&mut self) {
        use ph2d_panel_audio_editor::spectral_state as ss;

        // Tell the panel whether AI Denoise (DeepFilterNet, W7) has a home in this build. Compiled
        // to a constant `true`/`false` — the panel paints the button only when it is `true`, so a
        // build without `audio-ml` never shows it. Published before the no-clip early return so it
        // is always current.
        ss::set_ml_available(cfg!(feature = "audio-ml"));
        // Is one running right now? What dims this whole section (and what `event.rs` reads to
        // refuse a click that reaches it anyway). Always false without the feature — no job can
        // exist in a build with no model.
        #[cfg(feature = "audio-ml")]
        ss::set_ml_busy(self.editor_ml_busy());

        let Some(clip) = self.editor.clip.as_ref() else {
            ss::set_ready(false, false, "");
            self.spectral = SpectralState::default();
            return;
        };
        // A band only means anything in the spectrogram: the box is drawn there, and in
        // the waveform there is no frequency axis to have drawn it on.
        let has_band = ss::view() && self.spectral.band.is_some() && clip.selection().is_some();
        let has_profile = self.spectral.profile.is_some();

        // The status line teaches the section. Each dimmed button gets a reason.
        let status = if !ss::view() {
            "Switch to Spectrogram to select a frequency band".to_string()
        } else if has_band {
            let (lo, hi) = self.spectral.band.unwrap_or((0.0, 0.0));
            let ny = clip.data().format().sample_rate as f32 * 0.5;
            format!(
                "Band {:.0}-{:.0} Hz \u{b7} drag in the spectrogram to change",
                lo.min(hi) * ny,
                lo.max(hi) * ny
            )
        } else {
            "Drag a box in the spectrogram to select a region".to_string()
        };
        let status = if has_profile {
            format!("{status}\nNoise profile learned")
        } else {
            format!("{status}\nDenoise needs a noise profile: select silence, then Learn")
        };
        ss::set_ready(has_profile, has_band, &status);
    }

    /// The spectrogram as RGBA at `w` × `h`, for the overlay. `None` when there is no clip.
    ///
    /// The colour ramp is built from **theme tokens** — silence is the panel's own
    /// background, loud is the accent, and the peaks go to Text1. No colour is invented
    /// (HR-15), and the picture changes clothes with the theme like everything else.
    pub(crate) fn editor_spectrogram_rgba(
        &mut self,
        w: u32,
        h: u32,
        theme: Theme,
    ) -> Option<Arc<Vec<u8>>> {
        // **The picture must be of the clip the overlay is BUILT ON — the sounding one.**
        //
        // The ruler, the playhead, the hit-test and `WaveView.frames` all come from
        // `editor_clip()`, which is the live rack audition when one is running (and the
        // force-mono view when that is on). Drawing the *committed* clip here instead put a
        // picture of one buffer inside a rectangle whose time axis belongs to another: with a
        // reverb auditioning, the audition is LONGER, so a beep at pixel x maps to a
        // different frame — and the box drawn around it repairs the wrong instant. Nothing
        // crashes; the repair "works" and removes the wrong thing (audit 2026-07-12).
        //
        // The cost is real and accepted: an audition hands us a new buffer every frame the
        // user drags a rack slider, and each one is a fresh FFT (~24 ms for a 60 s clip). A
        // picture that disagrees with the ruler you are clicking on is not a picture, though.
        let clip = self.editor_clip()?;
        if w == 0 || h == 0 {
            return None;
        }
        let data = clip.data();
        let buf = data.version();
        if self.spectral.sg_key != Some(buf) {
            self.spectral.sg = Some(Spectrogram::build(data));
            self.spectral.sg_key = Some(buf);
            // A new picture invalidates the rendering of the old one.
            self.spectral.img = None;
            self.spectral.img_key = None;
        }
        let key = ImgKey { buf, w, h, theme };
        if self.spectral.img_key != Some(key) {
            let sg = self.spectral.sg.as_ref()?;
            let ramp = [
                rgb(ColorToken::BgElev, theme),
                rgb(ColorToken::Accent, theme),
                rgb(ColorToken::Text1, theme),
            ];
            self.spectral.img = Some(Arc::new(sg.rgba(w as usize, h as usize, &ramp)));
            self.spectral.img_key = Some(key);
        }
        self.spectral.img.clone()
    }

    /// The overlay: a drag in the spectrogram set the frequency half of the selection.
    /// `a` / `b` are fractions of Nyquist, in either order.
    pub(crate) fn editor_set_spectral_band(&mut self, a: f32, b: f32) {
        self.spectral.band = Some((a.clamp(0.0, 1.0), b.clamp(0.0, 1.0)));
    }

    /// The overlay: this selection was drawn in the WAVEFORM, which has no frequency axis —
    /// so it has no band, and Repair has nothing to act on.
    ///
    /// It has to be said out loud rather than left alone: a stale band from an earlier
    /// spectrogram drag would otherwise still be sitting there, and switching back to the
    /// spectrogram would light Repair up over a box the user never drew for this selection
    /// (audit 2026-07-12).
    pub(crate) fn editor_clear_spectral_band(&mut self) {
        self.spectral.band = None;
    }

    /// The frequency band, for the overlay's selection box.
    pub(crate) fn editor_spectral_band(&self) -> Option<(f32, f32)> {
        self.spectral.band
    }

    /// **Repair**: rebuild the selected time-frequency region from what surrounds it.
    pub(crate) fn editor_spectral_repair(&mut self) {
        let Some(clip) = self.editor.clip.as_mut() else {
            return;
        };
        let (Some(sel), Some((a, b))) = (clip.selection(), self.spectral.band) else {
            return;
        };
        let nyquist = clip.data().format().sample_rate as f32 * 0.5;
        let band = Band {
            frames: sel,
            hz: (a.min(b) * nyquist)..(a.max(b) * nyquist),
        };
        let out = ph2d_audio_spectral::repair(clip.data(), &band);
        // A no-op must not cost an undo step. `repair` returns the input unchanged when the
        // band collapses, and `commit_rendered` pushes onto the history unconditionally — so
        // the user got a lit Undo button for an edit that never happened (audit 2026-07-12).
        if out.samples() == clip.data().samples() {
            return;
        }
        clip.commit_rendered(out);
        self.editor_hot_swap();
    }

    /// **Learn**: take the selection to be noise ALONE, and remember its shape.
    ///
    /// This is the one control here that can do real damage by being pointed at the wrong
    /// thing — teach it that a voice is the noise and Denoise will dutifully remove the
    /// voice. It cannot check that for the user (only they know what the sound *is*), so
    /// the panel's job is to say what it wants, which the status line does.
    pub(crate) fn editor_learn_noise(&mut self) {
        let Some(clip) = self.editor.clip.as_ref() else {
            return;
        };
        let Some(sel) = clip.selection() else {
            return;
        };
        // Keep the old profile if the new selection is too short to learn from. Assigning the
        // `None` straight through would delete a good profile in silence and darken the
        // Denoise button, with nothing on screen to say why (audit 2026-07-12).
        if let Some(p) = NoiseProfile::learn(clip.data(), sel) {
            self.spectral.profile = Some(p);
        }
    }

    /// **Denoise**: suppress the learned noise across the whole clip.
    pub(crate) fn editor_denoise(&mut self) {
        let amount = ph2d_panel_audio_editor::spectral_state::amount();
        let Some(profile) = self.spectral.profile.as_ref() else {
            return;
        };
        let Some(clip) = self.editor.clip.as_mut() else {
            return;
        };
        let out = ph2d_audio_spectral::denoise(clip.data(), profile, amount);
        // Same guard as the repair: `denoise` returns the input untouched at amount 0 (and
        // when the profile's rate does not match the clip's), and an undo step for nothing is
        // a lie about what happened.
        if out.samples() == clip.data().samples() {
            return;
        }
        clip.commit_rendered(out);
        self.editor_hot_swap();
    }

    /// **AI Denoise** (W7, ADR-0123): suppress noise across the whole clip with DeepFilterNet.
    ///
    /// The sibling of [`Self::editor_denoise`] without the Learn step — DeepFilterNet learns the
    /// noise itself, so there is no profile. `amount` is the same slider: `0` is a byte-identical
    /// no-op, `1` is the full model output. Only compiled when the shell was built with `audio-ml`.
    ///
    /// ## Why this one is a job and the W5 denoise is not
    ///
    /// The model runs at 0.03× real-time, which is *instant* on the 4 s clips this feature was
    /// developed against (0.16 s) and **~5 s on a 3-minute take** — five seconds in which the
    /// window would not redraw, not resize, and not answer. Every other edit in the rack is
    /// milliseconds; this is the one that had to leave the UI thread, and it is the first thing
    /// in the shell that ever has.
    ///
    /// The bar this raises is app infrastructure (`ph2d_editor_core::progress`), not an audio
    /// widget: batch LUFS, export and upscale have the identical shape and are meant to copy
    /// this, not reinvent it.
    #[cfg(feature = "audio-ml")]
    pub(crate) fn editor_denoise_ml(&mut self) {
        // Refuse to stack jobs. The button is dimmed while one runs (`set_ml_busy`), but a
        // dimmed button that still dispatches is a lie — the refusal has to be here, where the
        // work is, not only in the paint.
        if self.ml_job.is_some() {
            return;
        }
        let amount = ph2d_panel_audio_editor::spectral_state::amount();
        let Some(clip) = self.editor.clip.as_ref() else {
            return;
        };
        // `amount == 0` is a byte-identical no-op. Answer it HERE rather than paying a thread,
        // a bar and a frame of latency to have `denoise_ml` hand the clip straight back — and
        // an undo step for an edit that never happened would be a lie either way.
        if amount <= 0.0 {
            return;
        }
        // Cheap to send: `SampleData` is an `Arc<[f32]>`, so this clones a refcount, not a
        // clip. (The engine may be sounding the very same buffer on the RT thread — it is
        // immutable, and that is exactly why this is safe to hand to a worker.)
        let data = clip.data().clone();
        let job = ph2d_editor::Job::spawn("AI Denoise (Voice)", move |p| {
            // The bridge from this crate's `Progress` to the DSP crate's `&dyn Fn(f32)`. It is
            // one line, and it is the whole reason `ph2d-audio-ml` does not depend on the
            // editor: a DSP crate that must link a UI to report a percentage is a DSP crate
            // that cannot be used without one.
            let out = ph2d_audio_ml::denoise_ml_with_progress(&data, amount, &|f| p.set(f));
            MlDenoise { source: data, out }
        });
        self.started_job = Some(job.progress().clone());
        self.ml_job = Some(job);
    }

    /// Hand the bar of a just-started job to the app-wide queue. Called once per frame by the
    /// bridge; `None` on every frame but the one after a click.
    #[cfg(feature = "audio-ml")]
    pub(crate) fn editor_take_started_job(&mut self) -> Option<ph2d_editor::Progress> {
        self.started_job.take()
    }

    /// Is an AI Denoise running? What dims the Spectral section.
    #[cfg(feature = "audio-ml")]
    pub(crate) fn editor_ml_busy(&self) -> bool {
        self.ml_job.is_some()
    }

    /// Install the AI Denoise result once the worker has it. Called once per frame.
    ///
    /// **Both branches drop the job**, including the one where `try_take` comes back empty: a
    /// worker that panicked (the model failing to initialise is an `expect`) is finished and has
    /// no result, and a caller that only cleared the slot on success would leave the section
    /// dimmed for the rest of the session, waiting on a thread that is gone.
    #[cfg(feature = "audio-ml")]
    pub(crate) fn editor_poll_ml(&mut self) {
        let Some(job) = self.ml_job.as_mut() else {
            return;
        };
        if !job.is_finished() {
            return;
        }
        let taken = job.try_take();
        self.ml_job = None;
        let Some(MlDenoise { source, out }) = taken else {
            return; // the worker panicked; it has already said so on stderr
        };
        let Some(clip) = self.editor.clip.as_mut() else {
            return; // the clip was closed while we worked — the result is about nothing now
        };
        // **The clip may have been EDITED while the model ran** — the UI stayed live, which was
        // the entire point of moving the work off it, and only the Spectral section is dimmed
        // (Cut, Paste and Normalize are all still there). Committing a result computed from a
        // buffer that is no longer on screen would silently throw away whatever the user did in
        // the meantime. Buffer identity answers it exactly, and `SampleData::version` is what
        // identity means since ADR-0124: an edited clip can keep its address, so the address alone
        // would say "still the same audio" about audio the user has since changed. This is the
        // `BufKey` idiom used at the top of this file, and it is sound here because `source` came
        // back from the worker still alive — an identity cannot be recycled by a buffer we hold.
        if !same_buffer(&source, clip.data()) {
            return;
        }
        if out.samples() == clip.data().samples() {
            return; // nothing changed → no undo step for an edit that never happened
        }
        clip.commit_rendered(out);
        self.editor_hot_swap();
    }
}

/// What the AI Denoise worker hands back: the result **and the buffer it was computed from**.
///
/// The source travels with the result for one reason — it is what lets the installer ask "is
/// this still about the clip on screen?" — and carrying it (rather than, say, an address noted
/// down before the spawn) is what makes that question answerable: the buffer is still alive, so
/// its identity cannot have been recycled by a different clip while we were not looking.
#[cfg(feature = "audio-ml")]
pub(crate) struct MlDenoise {
    source: ph2d_audio::SampleData,
    out: ph2d_audio::SampleData,
}

/// Are these two the same audio? Same question `BufKey` asks at the top of this file, and it is
/// asked the same way — an address stopped being an answer when ADR-0124 let an edit rewrite a
/// buffer in place.
#[cfg(feature = "audio-ml")]
fn same_buffer(a: &ph2d_audio::SampleData, b: &ph2d_audio::SampleData) -> bool {
    a.version() == b.version()
}

/// A resolved theme colour as plain RGB — the spectral crate takes bytes, so it never has
/// to learn what a token is.
fn rgb(token: ColorToken, theme: Theme) -> [u8; 3] {
    let c = token.resolve(theme);
    [c.r, c.g, c.b]
}
