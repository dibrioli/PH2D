//! The Spectral section (W5, ADR-0122): see the sound, and repair what you can see.
//!
//! Two tools, and they need different things selected — which is the one thing this
//! section has to teach, and it teaches it by **dimming**:
//!
//! - **Repair** needs a region of time *and frequency*: a box drawn in the spectrogram. A
//!   plain waveform selection says only *when*, and repair also has to know *what* — the
//!   beep and the voice under it occupy the very same samples. So Repair stays dim until a
//!   box exists, and the status line says what is missing.
//! - **Denoise** needs a **noise profile**: a stretch of the clip that is noise ALONE (the
//!   room tone before the take), which the user hands over with Learn. Until then Denoise
//!   is dim too — denoising against a profile it was never taught would either do nothing
//!   or gut the clip, and clicking is a bad way to find out which.
//!
//! Everything the section knows is published by the shell (`spectral_state`); the panel
//! owns only the view toggle and the Amount slider.

use crate::paint::{ClippedHits, button, toggle};
use crate::{
    AEDIT_SPEC_AMOUNT, AEDIT_SPEC_DENOISE, AEDIT_SPEC_DENOISE_ML, AEDIT_SPEC_LEARN,
    AEDIT_SPEC_REPAIR, AEDIT_SPEC_VIEW, spectral_state,
};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::{Slider, SliderOrientation, paint_slider};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Paint the Spectral section starting at `y`; returns the `y` below it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_spectral_section(
    mut y: f32,
    x: f32,
    w: f32,
    loaded: bool,
    has_sel: bool,
    row_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let gap = Spacing::Sm.px();
    let label_h = TypeToken::Xs.px();

    // The view toggle. It is the first thing here because it is the precondition for
    // everything else: the box Repair needs can only be drawn in the spectrogram.
    toggle(
        Rect::new(x, y, w, row_h),
        "Spectrogram",
        spectral_state::view(),
        loaded,
        AEDIT_SPEC_VIEW,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + gap;

    // Repair — lit only when there is a time-frequency box to act on.
    let can_repair = loaded && spectral_state::has_band();
    button(
        Rect::new(x, y, w, row_h),
        "Repair Selection",
        can_repair,
        AEDIT_SPEC_REPAIR,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + gap;

    // Denoise: Learn (from a time selection of pure noise) → Amount → apply.
    let half = (w - gap) * 0.5;
    button(
        Rect::new(x, y, half, row_h),
        "Learn Noise",
        loaded && has_sel,
        AEDIT_SPEC_LEARN,
        scene,
        text_system,
        theme,
        hit_index,
    );
    button(
        Rect::new(x + half + gap, y, half, row_h),
        "Denoise",
        loaded && spectral_state::has_profile(),
        AEDIT_SPEC_DENOISE,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + gap;

    // AI Denoise (W7, ADR-0123) — DeepFilterNet, which learns the noise itself, so it needs no
    // profile and no Learn: it is lit whenever a clip is loaded. Painted only when the shell was
    // built with the `audio-ml` feature (it publishes availability); otherwise nothing is shown.
    let ml = spectral_state::ml_available();
    if ml {
        button(
            Rect::new(x, y, w, row_h),
            "AI Denoise",
            loaded,
            AEDIT_SPEC_DENOISE_ML,
            scene,
            text_system,
            theme,
            hit_index,
        );
        y += row_h + gap;
    }

    // The Amount slider drives whichever denoise is used: the W5 one (needs a profile) or the
    // AI one (always available in this build). Live when either can run.
    let live = loaded && (spectral_state::has_profile() || ml);
    paint_text(
        text_system,
        scene,
        "Amount",
        x,
        y,
        label_h,
        w,
        resolve(
            if live {
                ColorToken::Text2
            } else {
                ColorToken::Text3
            },
            theme,
        ),
    );
    y += label_h + Spacing::Xs.px();
    let track = Rect::new(x, y, w, Spacing::Md.px());
    let mut slider =
        Slider::new(AEDIT_SPEC_AMOUNT, "Amount").orientation(SliderOrientation::Horizontal);
    slider.set_value(spectral_state::amount());
    paint_slider(&slider, track, scene, theme);
    hit_index.register(AEDIT_SPEC_AMOUNT, track);
    y += Spacing::Md.px() + gap;

    // The status line is the section's teacher: it says what is selected and what is
    // missing, so a dimmed button is never a mystery.
    let status = spectral_state::status();
    if loaded && !status.is_empty() {
        paint_text(
            text_system,
            scene,
            &status,
            x,
            y,
            label_h,
            w,
            resolve(ColorToken::Text3, theme),
        );
        y += text_system
            .layout(&status, label_h, w)
            .height()
            .max(label_h)
            + gap;
    }
    y
}

/// The section header's readout: which view is showing, at a glance, without unfolding.
pub(crate) fn spectral_readout() -> &'static str {
    if spectral_state::view() {
        "Spectrogram"
    } else {
        "Waveform"
    }
}
