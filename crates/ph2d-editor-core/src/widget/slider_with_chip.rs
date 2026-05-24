//! Canonical "label + horizontal slider track + numeric chip"
//! composite — the form the BlenderColorPicker channel rows use,
//! extracted so the Inspector and any other slider site can share
//! the exact same visual + interaction surface.
//!
//! **Use this anywhere a slider with a value chip is needed.**
//! Don't roll a one-off `Slider + paint_number_input_with_buffer`
//! pair like the Inspector did pre-M13 — that was a recurring
//! source of "the slider in panel X looks different from the one
//! in panel Y" bugs. See `docs/UI_Bugs/README.md` §6.1.
//!
//! Two pieces:
//! - [`paint_slider_with_chip`] — the full row (label + track + chip),
//!   reads the slider's state + the chip's NumberInput state straight
//!   from the [`crate::interaction::WidgetStore`] and registers both
//!   sub-rect hits in the [`crate::interaction::HitIndex`].
//! - [`paint_number_chip`] — the standalone chip (interactive
//!   NumberInput-style: focus border + caret + buffer + selection +
//!   centered text). Used by `paint_slider_with_chip` and directly
//!   by callers that just want the chip on its own (e.g. the
//!   color-picker hue/V channels).
//!
//! Default layout: label_w=70 left, track in the middle, chip_w=60
//! right with a `Spacing::Sm.px()` gap on each side. Override via
//! [`paint_slider_with_chip_layout`] if a particular row needs a
//! wider chip or label.

use crate::interaction::{HitIndex, InteractiveState, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text_centered, resolve, stroke_rounded_rect};
use crate::widget::TextInputState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

pub const DEFAULT_LABEL_W: f32 = 70.0; // LITERAL-PX-OK: slider-with-chip default label column width (chrome-specific)
pub const DEFAULT_CHIP_W: f32 = 60.0; // LITERAL-PX-OK: slider-with-chip default numeric chip width (chrome-specific)

/// Paint a label + slider track + numeric chip composite using the
/// canonical layout. Both `slider_id` and `chip_id` register in the
/// hit index so the dispatch can route drag (slider) and click /
/// type (chip) separately.
///
/// `value` is the slider value in `[0..1]`; the chip displays the
/// same value formatted via [`crate::interaction::format_number`].
/// For displays that diverge from the slider's normalised value
/// (e.g. Inspector fields that show "160" for a slider at 0.62),
/// use [`paint_slider_with_chip_layout`] and pass `chip_value` /
/// `display_override` separately.
#[allow(clippy::too_many_arguments)]
pub fn paint_slider_with_chip(
    rect: Rect,
    label: &str,
    value: f32,
    slider_id: NodeId,
    chip_id: NodeId,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_slider_with_chip_layout(
        rect,
        label,
        value,
        value as f64,
        None,
        slider_id,
        chip_id,
        DEFAULT_LABEL_W,
        DEFAULT_CHIP_W,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
}

/// Layout-flexible variant of [`paint_slider_with_chip`].
///
/// `chip_value` is the f64 the chip displays when not focused (and
/// what the painter formats via `format_number`). `display_override`
/// wins over `chip_value` when present — useful for Inspector fields
/// that display "160" instead of "0.62" for a slider at 62%.
#[allow(clippy::too_many_arguments)]
pub fn paint_slider_with_chip_layout(
    rect: Rect,
    label: &str,
    value: f32,
    chip_value: f64,
    display_override: Option<&str>,
    slider_id: NodeId,
    chip_id: NodeId,
    label_w: f32,
    chip_w: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let gap = Spacing::Sm.px();
    let track_x = rect.x + label_w + gap;
    let track_w = (rect.w - label_w - chip_w - gap * 2.0).max(1.0);
    let label_rect = Rect::new(rect.x, rect.y, label_w, rect.h);
    let track_rect = Rect::new(
        track_x,
        rect.y + Spacing::Sm.px(),
        track_w,
        rect.h - Spacing::Lg.px(),
    );
    let chip_rect = Rect::new(rect.x + rect.w - chip_w, rect.y, chip_w, rect.h);

    // Plain text label, no pill background — the previous AccentPress
    // fill made every label read as a "selected button"; channel rows
    // are not interactive at the label, so a chrome-less label keeps
    // the eye on the slider track.
    paint_text_centered(
        text_system,
        scene,
        label,
        label_rect,
        TypeToken::Xs.px() - 1.0,
        resolve(ColorToken::Text2, theme),
    );

    // Track background + filled portion — shared canonical painter so
    // this matches the bare `paint_slider` look exactly.
    crate::widget::paint_slider_track(
        track_rect,
        value,
        crate::widget::SliderOrientation::Horizontal,
        scene,
        theme,
    );
    if slider_id.0 != 0 {
        hit_index.register(slider_id, track_rect);
    }

    // Chip — read its NumberInput state straight from the store so
    // typing / caret / selection are live.
    let (chip_state, chip_buffer, chip_caret, chip_anchor) = match store.get(chip_id) {
        Some(InteractiveState::NumberInput {
            state,
            buffer,
            caret,
            selection_anchor,
            ..
        }) => (*state, Some(buffer.as_str()), *caret, *selection_anchor),
        _ => (TextInputState::Normal, None, 0, None),
    };
    paint_number_chip(
        chip_rect,
        chip_state,
        chip_value,
        display_override,
        chip_buffer,
        chip_caret,
        chip_anchor,
        scene,
        text_system,
        theme,
    );
    if chip_id.0 != 0 {
        hit_index.register(chip_id, chip_rect);
    }
}

/// Paint a compact numeric chip (interactive NumberInput-style):
/// background, optional focus border, centered text, caret, and
/// selection highlight. Used by [`paint_slider_with_chip`] and
/// callable directly when a chip needs to live somewhere a slider
/// row layout doesn't fit.
///
/// `display_override` wins over `value` when present (e.g. for chips
/// that display engineering units while the linked slider still
/// drives a 0..1 normalised value).
#[allow(clippy::too_many_arguments)]
pub fn paint_number_chip(
    rect: Rect,
    state: TextInputState,
    value: f64,
    display_override: Option<&str>,
    buffer: Option<&str>,
    caret: usize,
    selection_anchor: Option<usize>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let focused = state == TextInputState::Focused;
    let radius = Radius::Xs.px();
    let bg = if focused {
        ColorToken::Bg2
    } else {
        ColorToken::Bg3
    };
    fill_rounded_rect(scene, rect, radius, resolve(bg, theme));
    if focused {
        stroke_rounded_rect(scene, rect, radius, 2.0, resolve(ColorToken::Accent, theme));
    }
    let display_owned;
    let display = match (focused, buffer, display_override) {
        (true, Some(b), _) => b,
        (_, _, Some(s)) => s,
        _ => {
            display_owned = crate::interaction::format_number(value);
            display_owned.as_str()
        }
    };
    let font_size = TypeToken::Xs.px();
    let total_w = if display.is_empty() {
        0.0
    } else {
        text_system
            .layout(display, font_size, f32::INFINITY)
            .width()
    };
    let text_start = rect.x + (rect.w - total_w) * 0.5;
    if focused
        && let Some(anchor) = selection_anchor
        && anchor != caret
    {
        let (sel_start, sel_end) = if anchor < caret {
            (anchor, caret)
        } else {
            (caret, anchor)
        };
        let sel_start = sel_start.min(display.len());
        let sel_end = sel_end.min(display.len());
        let prefix_w = text_system.prefix_width(&display[..sel_start], font_size);
        let mid_w = if sel_start == sel_end {
            0.0
        } else {
            text_system.prefix_width(&display[sel_start..sel_end], font_size)
        };
        let sel_top = rect.y + Spacing::Xs.px();
        let sel_bot = rect.y + rect.h - Spacing::Xs.px();
        let sel_x = (text_start + prefix_w).clamp(rect.x + 2.0, rect.x + rect.w - 2.0); // CLAMP-OK: rect-bound text-selection clamp (bounds well-formed by construction)
        let sel_w = mid_w.min(rect.x + rect.w - 2.0 - sel_x);
        if sel_w > 0.0 {
            let sel_rect = Rect::new(sel_x, sel_top, sel_w, (sel_bot - sel_top).max(2.0));
            fill_rounded_rect(scene, sel_rect, 1.0, resolve(ColorToken::AccentSoft, theme));
        }
    }
    paint_text_centered(
        text_system,
        scene,
        display,
        rect,
        font_size,
        resolve(ColorToken::Text1, theme),
    );
    if focused {
        let caret_clamped = caret.min(display.len());
        let prefix = &display[..caret_clamped];
        let prefix_w = if prefix.is_empty() {
            0.0
        } else {
            text_system.prefix_width(prefix, font_size)
        };
        let caret_x = (text_start + prefix_w).clamp(rect.x + 2.0, rect.x + rect.w - 2.0); // CLAMP-OK: rect-bound caret clamp (bounds well-formed by construction)
        let caret_top = rect.y + Spacing::Xs.px();
        let caret_bot = rect.y + rect.h - Spacing::Xs.px();
        let caret_rect = Rect::new(
            caret_x,
            caret_top,
            StrokeToken::Default.px(),
            (caret_bot - caret_top).max(2.0),
        );
        fill_rounded_rect(scene, caret_rect, 0.75, resolve(ColorToken::Accent, theme)); // LITERAL-PX-OK: caret half-width radius
    }
}
