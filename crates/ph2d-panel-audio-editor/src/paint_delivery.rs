//! The Delivery section of the Audio Editor panel (W6 — asset-prep).
//!
//! Two numbers decide whether an asset ships: what the player **downloads** and what
//! the engine **holds**. This section prices both before the export, not after.
//!
//! The trade it exists to make visible: **the codec moves Disk and never RAM.** There
//! is no streaming path in the mixer, so a Vorbis clip is decoded to the same `f32`
//! buffer a WAV would be — compressing an asset shrinks the download and buys back
//! exactly zero memory. People assume the opposite, and a number on screen is the only
//! way to un-assume it.
//!
//! UI-only: the panel owns the codec choice and the quality slider; the shell owns the
//! encoders, sizes the file for real (no bitrate guesses) and publishes the readout as
//! finished strings via `delivery_state`.

use crate::paint::{ClippedHits, button};
use crate::{AEDIT_CODEC_NEXT, AEDIT_CODEC_PREV, AEDIT_OGG_QUALITY, delivery_state};
use ph2d_editor_core::paint::{paint_text, paint_text_centered, resolve};
use ph2d_editor_core::widget::{Slider, SliderOrientation, paint_slider, paint_slider_track};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Width of the `◀` / `▶` codec selector arrows (matches the other selectors).
const ARROW_W: f32 = 26.0; // LITERAL-PX-OK: selector arrow button width (chrome)

/// A clip eating more than this share of the audio subsystem's RAM budget (HR-13,
/// 30 MB) is worth a second look — one sound should not be most of the envelope.
const BUDGET_WARN_FRAC: f32 = 0.25; // LITERAL-PX-OK: share of a RAM budget, not a design value

/// Paint the Delivery section starting at `y`; returns the `y` below it. Everything
/// dims when there is no clip: there is nothing to price.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_delivery_section(
    mut y: f32,
    x: f32,
    w: f32,
    loaded: bool,
    row_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let gap = Spacing::Sm.px();
    let label_h = TypeToken::Xs.px();

    // Header: "Delivery" left, the download size right — the number people came for.
    paint_text(
        text_system,
        scene,
        "Delivery",
        x,
        y,
        label_h,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let disk = delivery_state::disk();
    paint_text_centered(
        text_system,
        scene,
        if loaded && !disk.is_empty() {
            &disk
        } else {
            "\u{2014}"
        },
        Rect::new(x, y, w, label_h),
        label_h,
        resolve(
            if loaded {
                ColorToken::Text1
            } else {
                ColorToken::Text2
            },
            theme,
        ),
    );
    y += label_h + gap;

    // Codec selector: ◀ | name | ▶. It drives both the readout and the Export button,
    // so there is exactly one place the codec is decided.
    button(
        Rect::new(x, y, ARROW_W, row_h),
        "\u{25c0}",
        loaded,
        AEDIT_CODEC_PREV,
        scene,
        text_system,
        theme,
        hit_index,
    );
    paint_text_centered(
        text_system,
        scene,
        &delivery_state::codec_name(),
        Rect::new(
            x + ARROW_W + gap,
            y,
            (w - (ARROW_W + gap) * 2.0).max(1.0),
            row_h,
        ),
        TypeToken::Sm.px(),
        resolve(
            if loaded {
                ColorToken::Text1
            } else {
                ColorToken::Text2
            },
            theme,
        ),
    );
    button(
        Rect::new(x + w - ARROW_W, y, ARROW_W, row_h),
        "\u{25b6}",
        loaded,
        AEDIT_CODEC_NEXT,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + gap;

    // Quality — Vorbis only. On a lossless codec it is inert, and it says so by dimming
    // rather than by pretending to do something.
    let lossy = delivery_state::is_lossy();
    let live = loaded && lossy;
    paint_text(
        text_system,
        scene,
        "Quality",
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
    if live {
        let mut slider =
            Slider::new(AEDIT_OGG_QUALITY, "Quality").orientation(SliderOrientation::Horizontal);
        slider.set_value(delivery_state::quality_norm());
        paint_slider(&slider, track, scene, theme);
        hit_index.register(AEDIT_OGG_QUALITY, track);
    } else {
        // Inert track (no thumb, not hit-registered): a lossless codec has no quality
        // to trade, and a live-looking slider that did nothing would be a lie.
        paint_slider_track(
            track,
            delivery_state::quality_norm(),
            SliderOrientation::Horizontal,
            scene,
            theme,
        );
    }
    y += Spacing::Md.px() + gap;

    // What the ENGINE pays — the half of the trade the codec has no say in.
    let frac = delivery_state::budget_frac();
    let over = frac > BUDGET_WARN_FRAC;
    let ram = delivery_state::ram();
    paint_text(
        text_system,
        scene,
        if loaded && !ram.is_empty() {
            &ram
        } else {
            "RAM \u{2014}"
        },
        x,
        y,
        label_h,
        w,
        resolve(
            match (loaded, over) {
                (false, _) => ColorToken::Text3,
                (true, true) => ColorToken::Warn,
                (true, false) => ColorToken::Text2,
            },
            theme,
        ),
    );
    y += label_h + gap;

    // Loop points and cue markers live in WAV chunks. Exporting to Vorbis silently
    // loses them, so the panel says it out loud while the choice can still be changed.
    if loaded && delivery_state::drops_meta() {
        paint_text(
            text_system,
            scene,
            "This codec drops loop points and markers",
            x,
            y,
            label_h,
            w,
            resolve(ColorToken::Warn, theme),
        );
        y += label_h + gap;
    }
    y
}
