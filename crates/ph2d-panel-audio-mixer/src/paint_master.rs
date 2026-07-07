//! Audio Mixer master-section footer painter — Play Test · loudness (LUFS) ·
//! Limiter · EQ · Reverb · Delay · Ducking. Split out of `paint.rs` to keep it
//! under the panel LOC cap; a leaf over the shared widget-row helpers.

use crate::paint::MUTE_H;
use crate::paint_widgets::{paint_labeled_slider, paint_toggle};
use crate::{
    AMIX_DELAY, AMIX_DELAY_FEEDBACK, AMIX_DELAY_MIX, AMIX_DELAY_TIME, AMIX_DUCK, AMIX_DUCK_DEPTH,
    AMIX_DUCK_KEY, AMIX_EQ_HIGH, AMIX_EQ_LOW, AMIX_EQ_MID, AMIX_LIMITER, AMIX_PLAY, AMIX_REVERB,
    AMIX_REVERB_MIX, AMIX_REVERB_SIZE, SUB_BUS_COUNT, SUB_BUS_LABELS, SUB_DELAY_SEND, SUB_SEND,
    snapshot,
};
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{paint_text_centered, resolve};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const LUFS_SILENCE_DISPLAY: f32 = -70.0; // LITERAL-PX-OK: below this the loudness reads "-inf" (audio domain)

/// The master-section footer below the strips, top-down: Play Test · Limiter ·
/// EQ (Low/Mid/High) · Reverb (toggle + Size/Return + per-bus sends) · Ducking.
/// Split out of `paint` to stay under the fn LOC cap.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_master_section(
    mut y: f32,
    content_x: f32,
    content_w: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    // Play Test first — the primary "make sound" control stays reachable even if
    // the effect controls below overflow a short Inspector.
    let playing = snapshot::play_test();
    paint_toggle(
        Rect::new(content_x, y, content_w, MUTE_H),
        if playing { "Stop" } else { "Play Test" },
        playing,
        ColorToken::Accent,
        AMIX_PLAY,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += MUTE_H + Spacing::Sm.px();

    // Master momentary loudness readout (LUFS, BS.1770) — a monitoring number
    // below Play Test; "-inf" when the master is effectively silent.
    let lufs = snapshot::loudness();
    let loudness_text = if lufs <= LUFS_SILENCE_DISPLAY {
        "-inf LUFS".to_string()
    } else {
        format!("{lufs:.1} LUFS")
    };
    paint_text_centered(
        text_system,
        scene,
        &loudness_text,
        Rect::new(content_x, y, content_w, TypeToken::Xs.px()),
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Xs.px() + Spacing::Md.px();

    // Master output limiter (Accent when engaged) — tames peaks below the clip
    // ceiling instead of hard-clipping.
    paint_toggle(
        Rect::new(content_x, y, content_w, MUTE_H),
        "Limiter",
        snapshot::limiter(),
        ColorToken::Accent,
        AMIX_LIMITER,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += MUTE_H + Spacing::Md.px();

    // Master 3-band EQ — Low shelf / Mid peak / High shelf gain sliders (0.5 =
    // flat). An "EQ" group header so the bands read as their own section, not the
    // Limiter toggle's children.
    paint_text_centered(
        text_system,
        scene,
        "EQ",
        Rect::new(content_x, y, content_w, TypeToken::Xs.px()),
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Xs.px() + Spacing::Sm.px();
    let eq = snapshot::eq();
    for (label, id, value) in [
        ("Low", AMIX_EQ_LOW, eq[0]),
        ("Mid", AMIX_EQ_MID, eq[1]),
        ("High", AMIX_EQ_HIGH, eq[2]),
    ] {
        y = paint_labeled_slider(
            y,
            label,
            id,
            value,
            content_x,
            content_w,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    y += Spacing::Sm.px();

    // Master reverb: enable toggle + Size (decay) + Mix (wet/dry) thin sliders.
    paint_toggle(
        Rect::new(content_x, y, content_w, MUTE_H),
        "Reverb",
        snapshot::reverb_on(),
        ColorToken::Accent,
        AMIX_REVERB,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += MUTE_H + Spacing::Sm.px();
    for (label, id, value) in [
        ("Size", AMIX_REVERB_SIZE, snapshot::reverb_size()),
        ("Return", AMIX_REVERB_MIX, snapshot::reverb_mix()),
    ] {
        y = paint_labeled_slider(
            y,
            label,
            id,
            value,
            content_x,
            content_w,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    // Per-sub-bus reverb aux sends — how much of each bus feeds the return.
    let sends = snapshot::sub_send();
    for i in 0..SUB_BUS_COUNT {
        y = paint_labeled_slider(
            y,
            SUB_BUS_LABELS[i],
            SUB_SEND[i],
            sends[i],
            content_x,
            content_w,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    y += Spacing::Sm.px();

    // Master delay/echo: enable toggle + Time / Feedback / Return + per-bus sends.
    paint_toggle(
        Rect::new(content_x, y, content_w, MUTE_H),
        "Delay",
        snapshot::delay_on(),
        ColorToken::Accent,
        AMIX_DELAY,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += MUTE_H + Spacing::Sm.px();
    for (label, id, value) in [
        ("Time", AMIX_DELAY_TIME, snapshot::delay_time()),
        ("Fbk", AMIX_DELAY_FEEDBACK, snapshot::delay_feedback()),
        ("Return", AMIX_DELAY_MIX, snapshot::delay_mix()),
    ] {
        y = paint_labeled_slider(
            y,
            label,
            id,
            value,
            content_x,
            content_w,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    let delay_sends = snapshot::sub_delay_send();
    for i in 0..SUB_BUS_COUNT {
        y = paint_labeled_slider(
            y,
            SUB_BUS_LABELS[i],
            SUB_DELAY_SEND[i],
            delay_sends[i],
            content_x,
            content_w,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    y += Spacing::Sm.px();

    // Ducking (sidechain): enable toggle + Key selector + Depth. Every bus ducks
    // under the selected Key bus so it cuts through; click Key to cycle the bus.
    paint_toggle(
        Rect::new(content_x, y, content_w, MUTE_H),
        "Ducking",
        snapshot::ducking(),
        ColorToken::Accent,
        AMIX_DUCK,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += MUTE_H + Spacing::Sm.px();
    let key_label = format!(
        "Key: {}",
        SUB_BUS_LABELS[snapshot::ducking_key() % SUB_BUS_COUNT]
    );
    paint_toggle(
        Rect::new(content_x, y, content_w, MUTE_H),
        &key_label,
        false,
        ColorToken::Accent,
        AMIX_DUCK_KEY,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += MUTE_H + Spacing::Sm.px();
    paint_labeled_slider(
        y,
        "Depth",
        AMIX_DUCK_DEPTH,
        snapshot::duck_depth(),
        content_x,
        content_w,
        scene,
        text_system,
        theme,
        hit_index,
    );
}
