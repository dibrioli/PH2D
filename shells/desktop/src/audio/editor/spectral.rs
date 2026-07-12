//! The Spectral bridge (W5, ADR-0115): the spectrogram the overlay draws, and the three
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

/// Identifies a clip buffer. `SampleData` is an immutable `Arc<[f32]>`, so a new pointer
/// is a new buffer and any edit hands us a different one.
#[derive(PartialEq, Eq, Clone, Copy)]
struct BufKey {
    ptr: usize,
    len: usize,
}

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
        let buf = BufKey {
            ptr: data.samples().as_ptr() as usize,
            len: data.samples().len(),
        };
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
}

/// A resolved theme colour as plain RGB — the spectral crate takes bytes, so it never has
/// to learn what a token is.
fn rgb(token: ColorToken, theme: Theme) -> [u8; 3] {
    let c = token.resolve(theme);
    [c.r, c.g, c.b]
}
