//! The per-row paint loop (split out of `lib.rs::paint` for the HR-18 200-LOC fn cap +
//! the 600-LOC panel-file cap). One `match` arm per [`ParamRow`] kind, each delegating to
//! the SHARED source-of-truth painters (slider/chip, swatch, checkbox, segmented button,
//! number box, text field). `super` is the crate root, so the pooled-id helpers and the
//! `normalized_track`/`row_value` mappers are in scope.

use super::curve_row::{self, CurveWidgets};
use super::gradient_row::{self, GradientWidgets};
use super::{
    CHANNELS_EXTRA_BASE, MAX_ENUM_OPTIONS, MAX_PARAM_ROWS, ParamRow, normalized_track,
    paint_angle_row, paint_seed_row, paint_text_row, param_checkbox_id, param_chip_id,
    param_enum_id, param_number_id, param_reroll_id, param_slider_id, param_swatch_id,
    param_text_id, row_value,
};
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::widget::{
    ButtonState, Checkbox, CheckboxState, CheckboxValue, ColorSwatch, DEFAULT_LABEL_W, SwatchSize,
    paint_checkbox, paint_color_swatch, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[path = "rows_paint_kinds.rs"]
mod kinds;
use kinds::{
    paint_channels_row, paint_color_row, paint_driven_row, paint_enum_row, paint_scalar_row,
    paint_source_row, paint_toggle_row,
};

/// Paint each param row from `body_top` down, registering hit rects as it goes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_rows(
    rows: &[ParamRow],
    inner_x: f32,
    inner_w: f32,
    chip_w: f32,
    row_gap: f32,
    body_top: f32,
    label_font: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> (CurveWidgets, GradientWidgets) {
    let mut y = body_top;
    let mut curve_widgets = CurveWidgets::new();
    let mut gradient_widgets = GradientWidgets::new();
    for (i, row) in rows.iter().enumerate().take(MAX_PARAM_ROWS) {
        match row {
            // A DRIVEN param (doc 58): the wire decides the number, so there is no widget —
            // just the label and the live value. Nothing is registered, so nothing can be
            // dragged; the artist unplugs the wire to take the knob back.
            ParamRow::Scalar(row) if row.driven => {
                paint_driven_row(
                    row,
                    inner_x,
                    inner_w,
                    y,
                    label_font,
                    scene,
                    text_system,
                    theme,
                );
                y += ROW_H_PX + row_gap;
            }
            ParamRow::Scalar(row) => {
                y = paint_scalar_row(
                    row,
                    i,
                    inner_x,
                    inner_w,
                    chip_w,
                    row_gap,
                    y,
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
            }
            ParamRow::Color(row) => {
                y = paint_color_row(
                    row,
                    inner_x,
                    inner_w,
                    row_gap,
                    y,
                    label_font,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
            }
            ParamRow::Toggle(row) => {
                y = paint_toggle_row(
                    row,
                    i,
                    inner_x,
                    inner_w,
                    row_gap,
                    y,
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
            }
            ParamRow::Enum(row) => {
                y = paint_enum_row(
                    row,
                    i,
                    inner_x,
                    inner_w,
                    row_gap,
                    y,
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
            }
            ParamRow::Angle(row) => {
                // A `deg` number box — never a raw turns/radians slider.
                let used = paint_angle_row(
                    Rect::new(inner_x, y, inner_w, ROW_H_PX),
                    &row.label,
                    param_number_id(i),
                    row.step_deg,
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
                y += used + row_gap;
            }
            ParamRow::Seed(row) => {
                // A whole-number box + a re-roll button (never a slider).
                let used = paint_seed_row(
                    Rect::new(inner_x, y, inner_w, ROW_H_PX),
                    &row.label,
                    param_number_id(i),
                    param_reroll_id(i),
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
                y += used + row_gap;
            }
            ParamRow::Text(row) => {
                // A full-width free-text field (a formula) — the shared TextInput.
                let used = paint_text_row(
                    Rect::new(inner_x, y, inner_w, ROW_H_PX),
                    &row.label,
                    "e.g. sin(t)",
                    param_text_id(i),
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
                y += used + row_gap;
            }
            ParamRow::Channels(row) => {
                y = paint_channels_row(
                    row,
                    i,
                    inner_x,
                    inner_w,
                    row_gap,
                    y,
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
            }
            ParamRow::Source(row) => {
                y = paint_source_row(
                    row,
                    i,
                    inner_x,
                    inner_w,
                    row_gap,
                    y,
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
            }
            ParamRow::Curve(row) => {
                // The interactive Curve editor — a graph with draggable handles. Its
                // `CurvePoint`/`Button` store states ride back in `curve_widgets` (this
                // pass has only an immutable store); the caller registers them (Phase C).
                let used = curve_row::paint_curve_row(
                    row,
                    i,
                    inner_x,
                    inner_w,
                    y,
                    label_font,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                    &mut curve_widgets,
                );
                y += used + row_gap;
            }
            ParamRow::Gradient(row) => {
                // The interactive Gradient editor — a bar with draggable position markers +
                // per-stop swatches. Its `CurvePoint`/`Button`/picker-swatch store states ride
                // back in `gradient_widgets` (this pass has only an immutable store); the
                // caller registers them (Phase B/C), the mirror of the Curve editor.
                let used = gradient_row::paint_gradient_row(
                    row,
                    i,
                    inner_x,
                    inner_w,
                    y,
                    label_font,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                    &mut gradient_widgets,
                );
                y += used + row_gap;
            }
        }
    }
    (curve_widgets, gradient_widgets)
}
