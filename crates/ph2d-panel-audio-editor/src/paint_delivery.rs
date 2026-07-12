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

    // No title row: the section header above carries the name AND the download size —
    // the number people came for, readable without unfolding the section.

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

    // The compression control. It is **Quality** for Vorbis and **Bitrate** for Opus — the same
    // slider, driving a different knob, and it says which rather than calling a bitrate
    // "quality" (ADR-0116). On a lossless codec it is inert, and it says so by dimming rather
    // than by pretending to do something.
    let lossy = delivery_state::is_lossy();
    let live = loaded && lossy;
    let label = delivery_state::quality_label();
    y = text_row(
        if label.is_empty() { "Quality" } else { &label },
        x,
        y,
        w,
        label_h,
        resolve(
            if live {
                ColorToken::Text2
            } else {
                ColorToken::Text3
            },
            theme,
        ),
        scene,
        text_system,
    ) + Spacing::Xs.px();
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
    y = text_row(
        if loaded && !ram.is_empty() {
            &ram
        } else {
            "RAM \u{2014}"
        },
        x,
        y,
        w,
        label_h,
        resolve(
            match (loaded, over) {
                (false, _) => ColorToken::Text3,
                (true, true) => ColorToken::Warn,
                (true, false) => ColorToken::Text2,
            },
            theme,
        ),
        scene,
        text_system,
    ) + gap;

    // Loop points and cue markers live in WAV chunks. Exporting to Vorbis silently
    // loses them, so the panel says it out loud while the choice can still be changed.
    if loaded && delivery_state::drops_meta() {
        y = text_row(
            "Drops loop points and markers",
            x,
            y,
            w,
            label_h,
            resolve(ColorToken::Warn, theme),
            scene,
            text_system,
        ) + gap;
    }

    // **Export Pieces** — one file per piece, in the codec priced above, named `<stem>_01..NN`:
    // exactly what the variation importer reads back as one group. Record eight footsteps in one
    // pass, split, export — and the eight assets come back as a ready-made variation container.
    //
    // It is dim until the clip is actually cut, because with one piece this is just Export.
    button(
        Rect::new(x, y, w, row_h),
        "Export Pieces",
        loaded && crate::tool_state::has_cuts(),
        crate::AEDIT_EXPORT_PIECES,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + row_h + Spacing::Md.px()
}

/// The download size, for the section header. Empty when there is nothing to price.
pub(crate) fn delivery_readout() -> String {
    let disk = delivery_state::disk();
    if disk.is_empty() {
        "\u{2014}".to_string()
    } else {
        disk
    }
}

/// Paint one line of body text and return the `y` **below what was actually laid out**.
///
/// `paint_text`'s `max_width` is a *wrap budget*, not a clip: a string wider than the
/// panel comes out as two rows. Advancing `y` by one line height then prints the next
/// row **on top of the wrapped one** — which is exactly what Enio's smoke caught
/// (2026-07-12), with the RAM readout and the codec warning stacked on each other.
///
/// So measure, do not assume. The strings are also kept short enough to fit on one line
/// at the panel's width, but a readout is built from numbers and a translation can be
/// longer than the English — the layout has to survive that on its own.
#[allow(clippy::too_many_arguments)]
fn text_row(
    text: &str,
    x: f32,
    y: f32,
    w: f32,
    size: f32,
    color: ph2d_vector::Color,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
) -> f32 {
    let h = text_system.layout(text, size, w).height().max(size);
    paint_text(text_system, scene, text, x, y, size, w, color);
    y + h
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::interaction::HitIndex;

    /// Paint the section with `ram` published and report how tall it came out.
    fn section_height(ram: &str) -> f32 {
        delivery_state::set_codec_info(4, "Ogg Vorbis", true, "Quality");
        delivery_state::set_cost("25 KB", ram, 0.07, false);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let mut hits = HitIndex::default();
        let clip = Rect::new(0.0, 0.0, 220.0, 4_000.0);
        let mut ch = ClippedHits::new(&mut hits, clip);
        paint_delivery_section(
            0.0,
            0.0,
            220.0, // the panel's real body width, near enough
            true,
            24.0,
            &mut scene,
            &mut text,
            Theme::default(),
            &mut ch,
        )
    }

    /// **The readout must make room for what it actually printed.**
    ///
    /// `paint_text`'s `max_width` is a wrap budget, not a clip: a string too wide for the
    /// panel comes back as TWO rows. The section used to advance `y` by one line height
    /// regardless, so the next line printed on top of the wrapped one — Enio's smoke
    /// showed the RAM readout and the codec warning stacked into an unreadable smear
    /// (2026-07-12).
    ///
    /// Red with a fixed advance: both strings would report the same height.
    #[test]
    fn a_readout_that_wraps_makes_room_for_its_second_line() {
        let short = section_height("RAM 2.2 MB");
        let wraps = section_height(
            "RAM 2.2 MB and then a great deal more text than could ever fit across one \
             single line of this panel at any sane width",
        );
        assert!(
            wraps > short,
            "a wrapped readout did not push the section down: {short} vs {wraps} \
             (the next line is printing on top of it)"
        );
    }
}
