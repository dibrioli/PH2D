//! Os três braços de **CAIXA** do `paint_rows` — ângulo, semente e texto livre.
//!
//! A família que se distingue dos outros nove: eles não são um slider com rótulo nem um editor
//! que se dimensiona sozinho — são **uma caixa de altura de ROW** montada num `Rect` explícito,
//! e por isso repetem a mesma armação. Extraídos juntos porque é essa armação, e não o corpo do
//! `match`, que levava o `paint_rows` ao teto de 200 LOC de fn de painel.

use super::{
    ParamRow, paint_angle_row, paint_seed_row, paint_text_row, param_number_id, param_reroll_id,
    param_text_id,
};
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Theme};
use ph2d_vector::VectorScene;

/// Desenha a row se ela for uma caixa, devolvendo a ALTURA usada; `None` para as demais.
#[expect(
    clippy::too_many_arguments,
    reason = "espelha a porta de paint das rows deste painel"
)]
pub(crate) fn paint_box_row(
    row: &ParamRow,
    i: usize,
    inner_x: f32,
    inner_w: f32,
    y: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> Option<f32> {
    let used = match row {
        ParamRow::Angle(row) => {
            // A `deg` number box — never a raw turns/radians slider.
            paint_angle_row(
                Rect::new(inner_x, y, inner_w, ROW_H_PX),
                &row.label,
                param_number_id(i),
                row.step_deg,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            )
        }
        ParamRow::Seed(row) => {
            // A whole-number box + a re-roll button (never a slider).
            paint_seed_row(
                Rect::new(inner_x, y, inner_w, ROW_H_PX),
                &row.label,
                param_number_id(i),
                param_reroll_id(i),
                store,
                hit_index,
                scene,
                text_system,
                theme,
            )
        }
        ParamRow::Text(row) => {
            // A full-width free-text field (a formula) — the shared TextInput.
            paint_text_row(
                Rect::new(inner_x, y, inner_w, ROW_H_PX),
                &row.label,
                "e.g. sin(t)",
                param_text_id(i),
                store,
                hit_index,
                scene,
                text_system,
                theme,
            )
        }
        _ => return None,
    };
    Some(used)
}
