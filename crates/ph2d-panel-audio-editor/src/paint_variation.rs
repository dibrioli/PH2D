//! The Variation-container section of the Audio Editor panel (W6 asset-prep).
//!
//! A set of clips the game runtime plays **one** of per trigger, chosen by a strategy
//! (Random / Sequence / Shuffle) with per-play pitch/gain jitter and per-entry weights
//! — the Wwise Random/Sequence Container. Kills the robotic repetition of footsteps /
//! gunshots. Authored here, auditioned via **Play**, and saved to a manifest file.
//!
//! UI-only: the panel paints the list + arms intents; the shell owns the
//! [`ph2d_audio_edit::VariationSet`], the decoded clips, the picker and the audition.
//! The row labels + strategy name come from `variation_state` (the shell publishes
//! labels; the panel owns the selected row and the jitter slider positions).

use crate::paint::{ClippedHits, button};
use crate::{
    AEDIT_VAR_ADD, AEDIT_VAR_ADD_FOLDER, AEDIT_VAR_GAIN, AEDIT_VAR_LOAD, AEDIT_VAR_PITCH,
    AEDIT_VAR_PLAY, AEDIT_VAR_REMOVE, AEDIT_VAR_ROWS, AEDIT_VAR_SAVE, AEDIT_VAR_STRATEGY_NEXT,
    AEDIT_VAR_STRATEGY_PREV, AEDIT_VAR_WEIGHT_DOWN, AEDIT_VAR_WEIGHT_UP, MAX_VARIATIONS,
    variation_state,
};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, paint_text_centered, resolve};
use ph2d_editor_core::widget::{Slider, SliderOrientation, paint_slider};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Width of the `◀` / `▶` strategy selector arrows (matches the effects selector).
const ARROW_W: f32 = 26.0; // LITERAL-PX-OK: selector arrow button width (chrome)
/// Height of one variation list row.
const VAR_ROW_H: f32 = 22.0; // LITERAL-PX-OK: variation list row height (chrome)

/// Paint the Variations section starting at `y`; returns the `y` below it. `row_h` is
/// the shared button row height. Play/Remove/Weight need a variation to exist.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_variation_section(
    mut y: f32,
    x: f32,
    w: f32,
    row_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let gap = Spacing::Sm.px();
    let label_h = TypeToken::Xs.px();
    let count = variation_state::count();
    let has_any = count > 0;

    // Header: "Variations" left, the count right.
    paint_text(
        text_system,
        scene,
        "Variations",
        x,
        y,
        label_h,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let readout = match count {
        0 => "No clips".to_string(),
        1 => "1 clip".to_string(),
        n => format!("{n} clips"),
    };
    paint_text_centered(
        text_system,
        scene,
        &readout,
        Rect::new(x, y, w, label_h),
        label_h,
        resolve(
            if has_any {
                ColorToken::Text1
            } else {
                ColorToken::Text2
            },
            theme,
        ),
    );
    y += label_h + Spacing::Sm.px();

    // Strategy selector: ◀ | name | ▶.
    button(
        Rect::new(x, y, ARROW_W, row_h),
        "\u{25c0}",
        true,
        AEDIT_VAR_STRATEGY_PREV,
        scene,
        text_system,
        theme,
        hit_index,
    );
    paint_text_centered(
        text_system,
        scene,
        &variation_state::strategy_name(),
        Rect::new(
            x + ARROW_W + gap,
            y,
            (w - (ARROW_W + gap) * 2.0).max(1.0),
            row_h,
        ),
        TypeToken::Sm.px(),
        resolve(ColorToken::Text1, theme),
    );
    button(
        Rect::new(x + w - ARROW_W, y, ARROW_W, row_h),
        "\u{25b6}",
        true,
        AEDIT_VAR_STRATEGY_NEXT,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + gap;

    // The clip list (selectable rows).
    y = paint_var_list(y, x, w, scene, text_system, theme, hit_index);

    // Add file | Add folder (import by convention: a folder of `name_01..NN`).
    let half = ((w - gap) * 0.5).max(1.0);
    button(
        Rect::new(x, y, half, row_h),
        "Add\u{2026}",
        true,
        AEDIT_VAR_ADD,
        scene,
        text_system,
        theme,
        hit_index,
    );
    button(
        Rect::new(x + half + gap, y, half, row_h),
        "Add Folder\u{2026}",
        true,
        AEDIT_VAR_ADD_FOLDER,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + gap;

    // Remove the selected variation.
    button(
        Rect::new(x, y, w, row_h),
        "Remove",
        has_any,
        AEDIT_VAR_REMOVE,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + gap;

    // Play (audition the next variation).
    button(
        Rect::new(x, y, w, row_h),
        "Play Variation",
        has_any,
        AEDIT_VAR_PLAY,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + gap;

    // Weight of the selected entry: ÷2 | ×2 (the row label shows the result).
    button(
        Rect::new(x, y, half, row_h),
        "Weight \u{00f7}2",
        has_any,
        AEDIT_VAR_WEIGHT_DOWN,
        scene,
        text_system,
        theme,
        hit_index,
    );
    button(
        Rect::new(x + half + gap, y, half, row_h),
        "Weight \u{00d7}2",
        has_any,
        AEDIT_VAR_WEIGHT_UP,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + Spacing::Sm.px();

    // Per-play jitter (container-level). Always adjustable — they are set properties.
    y = paint_jitter_slider(
        y,
        x,
        w,
        "Pitch jitter",
        AEDIT_VAR_PITCH,
        variation_state::pitch_jitter_norm(),
        scene,
        text_system,
        theme,
        hit_index,
    );
    y = paint_jitter_slider(
        y,
        x,
        w,
        "Gain jitter",
        AEDIT_VAR_GAIN,
        variation_state::gain_jitter_norm(),
        scene,
        text_system,
        theme,
        hit_index,
    );

    // Save | Load the set (manifest files).
    button(
        Rect::new(x, y, half, row_h),
        "Save\u{2026}",
        has_any,
        AEDIT_VAR_SAVE,
        scene,
        text_system,
        theme,
        hit_index,
    );
    button(
        Rect::new(x + half + gap, y, half, row_h),
        "Load\u{2026}",
        true,
        AEDIT_VAR_LOAD,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + row_h + Spacing::Md.px()
}

/// The variation list: one selectable row per clip (the selected row is tinted). Rows
/// beyond [`MAX_VARIATIONS`] are never published, so the fixed id array always covers
/// the list. Returns the `y` below the list.
#[allow(clippy::too_many_arguments)]
fn paint_var_list(
    mut y: f32,
    x: f32,
    w: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let names = variation_state::names();
    if names.is_empty() {
        paint_text_centered(
            text_system,
            scene,
            "Add clips to build a set",
            Rect::new(x, y, w, VAR_ROW_H),
            TypeToken::Xs.px(),
            resolve(ColorToken::Text2, theme),
        );
        return y + VAR_ROW_H + Spacing::Sm.px();
    }
    let sel = variation_state::variation_sel();
    for (i, name) in names.iter().enumerate().take(MAX_VARIATIONS) {
        let rect = Rect::new(x, y, w, VAR_ROW_H);
        let bg = if i == sel {
            ColorToken::Accent
        } else {
            ColorToken::Bg3
        };
        let fg = if i == sel {
            ColorToken::AccentFg
        } else {
            ColorToken::Text1
        };
        fill_rounded_rect(scene, rect, Radius::Sm.px(), resolve(bg, theme));
        paint_text(
            text_system,
            scene,
            name,
            x + Spacing::Sm.px(),
            y + (VAR_ROW_H - TypeToken::Xs.px()) * 0.5,
            TypeToken::Xs.px(),
            (w - Spacing::Sm.px() * 2.0).max(1.0),
            resolve(fg, theme),
        );
        hit_index.register(AEDIT_VAR_ROWS[i], rect);
        y += VAR_ROW_H + Spacing::Xs.px();
    }
    y + Spacing::Xs.px()
}

/// A labelled, always-adjustable jitter slider (`0..1`; the shell maps it to a `±`
/// range). No numeric readout — it is a feel control, like the loop crossfade.
#[allow(clippy::too_many_arguments)]
fn paint_jitter_slider(
    mut y: f32,
    x: f32,
    w: f32,
    label: &str,
    id: ph2d_a11y::NodeId,
    value: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let label_h = TypeToken::Xs.px();
    paint_text_centered(
        text_system,
        scene,
        label,
        Rect::new(x, y, w, label_h),
        label_h,
        resolve(ColorToken::Text2, theme),
    );
    y += label_h + Spacing::Xs.px();
    let track = Rect::new(x, y, w, Spacing::Md.px());
    let mut slider = Slider::new(id, label).orientation(SliderOrientation::Horizontal);
    slider.set_value(value);
    paint_slider(&slider, track, scene, theme);
    hit_index.register(id, track);
    y + Spacing::Md.px() + Spacing::Sm.px()
}
