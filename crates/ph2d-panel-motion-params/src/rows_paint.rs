//! The per-row paint loop (split out of `lib.rs::paint` for the HR-18 200-LOC fn cap +
//! the 600-LOC panel-file cap). One `match` arm per [`ParamRow`] kind, each delegating to
//! the SHARED source-of-truth painters (slider/chip, swatch, checkbox, segmented button,
//! number box, text field). `super` is the crate root, so the pooled-id helpers and the
//! `normalized_track`/`row_value` mappers are in scope.

use super::curve_row::{self, CurveWidgets};
use super::{
    MAX_ENUM_OPTIONS, MAX_PARAM_ROWS, ParamRow, normalized_track, paint_angle_row, paint_seed_row,
    paint_text_row, param_checkbox_id, param_chip_id, param_enum_id, param_number_id,
    param_reroll_id, param_slider_id, param_swatch_id, param_text_id, row_value,
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

/// **A row DIRIGIDA (doc 58) — o único caso que não é um widget.** O fio decide o número,
/// então não há nada a registrar: sem hit rect, sem arrasto, sem id no store. É por isso que
/// ela sai do laço em vez de virar mais um braço parecido com os outros — os oito braços
/// restantes registram e despacham; este só *mostra*, e o acento diz que o valor vem de fora.
/// (Extraída para o `paint_rows` caber no teto de 200 LOC de fn de painel, HR-18.)
#[allow(clippy::too_many_arguments)]
fn paint_driven_row(
    row: &super::ScalarRow,
    inner_x: f32,
    inner_w: f32,
    y: f32,
    label_font: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let mid = y + (ROW_H_PX - label_font) * 0.5;
    paint_text(
        text_system,
        scene,
        &row.label,
        inner_x,
        mid,
        label_font,
        DEFAULT_LABEL_W,
        resolve(ColorToken::Text2, theme),
    );
    let display = row_value(
        normalized_track(row.value, row.min, (row.max - row.min).max(f64::EPSILON)),
        row.min,
        row.max,
        row.integer,
    );
    paint_text(
        text_system,
        scene,
        // The SAME formatter the chip uses — a second one would show the same
        // number with two faces.
        &ph2d_editor_core::widget::format_number(display),
        inner_x + DEFAULT_LABEL_W,
        mid,
        label_font,
        inner_w - DEFAULT_LABEL_W,
        // The accent says it: this number is coming from somewhere else.
        resolve(ColorToken::Accent, theme),
    );
}

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
) -> CurveWidgets {
    let mut y = body_top;
    let mut curve_widgets = CurveWidgets::new();
    for (i, row) in rows.iter().enumerate().take(MAX_PARAM_ROWS) {
        match row {
            // A DRIVEN param (doc 58): the wire decides the number, so there is no widget —
            // just the label and the live value. Nothing is registered, so nothing can be
            // dragged; the artist unplugs the wire to take the knob back.
            ParamRow::Scalar(row) if row.driven => {
                paint_driven_row(
                    row, inner_x, inner_w, y, label_font, scene, text_system, theme,
                );
                y += ROW_H_PX + row_gap;
            }
            ParamRow::Scalar(row) => {
                let slider_id = param_slider_id(i);
                let chip_id = param_chip_id(i);
                let span = (row.max - row.min).max(f64::EPSILON);
                let track = store
                    .slider(slider_id)
                    .map(|(_, v)| v)
                    .unwrap_or(normalized_track(row.value, row.min, span));
                let display = row_value(track, row.min, row.max, row.integer);
                let used = paint_slider_with_chip_layout_adaptive(
                    Rect::new(inner_x, y, inner_w, ROW_H_PX),
                    &row.label,
                    track,
                    display,
                    None,
                    slider_id,
                    chip_id,
                    DEFAULT_LABEL_W,
                    chip_w,
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
                y += used + row_gap;
            }
            ParamRow::Color(row) => {
                let swatch_w = SwatchSize::Md.px();
                paint_text(
                    text_system,
                    scene,
                    &row.label,
                    inner_x,
                    y + (ROW_H_PX - label_font) * 0.5,
                    label_font,
                    DEFAULT_LABEL_W,
                    resolve(ColorToken::Text1, theme),
                );
                let swatch_id = param_swatch_id(row.channels[0]);
                let srect = Rect::new(inner_x + inner_w - swatch_w, y, swatch_w, ROW_H_PX);
                let sw = ColorSwatch::new(swatch_id, "Color", row.srgb).size(SwatchSize::Md);
                paint_color_swatch(&sw, srect, scene, theme);
                hit_index.register(swatch_id, srect);
                y += ROW_H_PX + row_gap;
            }
            ParamRow::Toggle(row) => {
                // A real checkbox — label + box on the left (the box owns the click; the
                // dispatch flips its value + fires Toggled).
                let cb_id = param_checkbox_id(i);
                let (cb_state, _) = store
                    .checkbox(cb_id)
                    .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
                let value = if row.on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                };
                let cb = Checkbox::new(cb_id, row.label.clone())
                    .state(cb_state)
                    .value(value);
                let crect = Rect::new(inner_x, y, inner_w, ROW_H_PX);
                paint_checkbox(&cb, crect, scene, text_system, theme);
                hit_index.register(cb_id, crect);
                y += ROW_H_PX + row_gap;
            }
            ParamRow::Enum(row) => {
                // Named segmented selector (label line + option buttons), mirror of the
                // Vector panel's Cap / Join / Draw rows.
                paint_text(
                    text_system,
                    scene,
                    &row.label,
                    inner_x,
                    y,
                    TypeToken::Sm.px(),
                    inner_w,
                    resolve(ColorToken::Text2, theme),
                );
                y += TypeToken::Sm.px() + Spacing::Xs.px();
                let k = row.labels.len().min(MAX_ENUM_OPTIONS);
                // Up to 4 buttons across, then wrap; a single option → 1.
                let cols = k.clamp(1, 4); // CLAMP-OK: segmented column count (option-count layout, not a UI metric)
                let gap = Spacing::Sm.px();
                let seg_w = ((inner_w - gap * (cols as f32 - 1.0)) / cols as f32).max(1.0);
                for (opt, caption) in row.labels.iter().enumerate().take(k) {
                    let bid = param_enum_id(i, opt);
                    let rx = inner_x + (opt % cols) as f32 * (seg_w + gap);
                    let ry = y + (opt / cols) as f32 * (ROW_H_PX + gap);
                    let brect = Rect::new(rx, ry, seg_w, ROW_H_PX);
                    let bstate = store.button_state(bid).unwrap_or(ButtonState::Normal);
                    paint_segmented_button(
                        brect,
                        caption,
                        opt == row.selected,
                        bstate,
                        scene,
                        text_system,
                        theme,
                    );
                    hit_index.register(bid, brect);
                }
                let seg_rows = k.div_ceil(cols) as f32;
                y += seg_rows * ROW_H_PX + (seg_rows - 1.0) * gap + row_gap;
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
        }
    }
    curve_widgets
}
