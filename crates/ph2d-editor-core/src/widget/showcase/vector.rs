//! Showcase `paint_vector_section` painter.
//!
//! Pre-2026-05-24: rendered via `Vector3Editor` (3 axes X/Y/Z). User
//! feedback (2026-05-24): "retire z do widget gallery e coloque as
//! caixas numéricas no padrão do inspector que é a referência" —
//! Inspector is a 2D editor, so VECTOR is now X+Y only and the chips
//! match Inspector's Transform-row style exactly (paint_number_input_with_buffer
//! directly, with Danger/Success tags + canonical chip width).

use super::*;
use crate::widget::panel_chrome::{SECTION_BOTTOM_PAD_PX, SECTION_LABEL_TO_CONTROL_PX};

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_vector_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (mut y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_VECTOR,
        ids::INSP_SECTION_VECTOR_COLOR,
        "Vector",
        2,
    );
    if !open {
        return y;
    }

    // Match Inspector Transform row layout: X tag + chip + Y tag + chip.
    // axis_col_w bumped to 14 px (was 12 = Spacing::Lg.px()) to guarantee
    // the X/Y glyph at TypeToken::Base font fits without parley clipping
    // it to the column width. Original Vector3Editor used (host.h*0.7).max(14),
    // so 14 is the documented floor for axis labels.
    let col_gap = Spacing::Md.px();
    let tag_box_gap = Spacing::Xxs.px();
    let axis_col_w = 14.0_f32; // LITERAL-PX-OK: floor from Vector3Editor canon
    let axis_label_font = TypeToken::Base.px();

    let two_chip_w = ((w - 2.0 * (axis_col_w + tag_box_gap) - col_gap) / 2.0).max(0.0);

    y += SECTION_LABEL_TO_CONTROL_PX;

    let (sx, vx, bx, cx, ax) = read_number_input(store, ids::INSP_SAMPLE_V3_X);
    let (sy, vy, by, cy, ay) = read_number_input(store, ids::INSP_SAMPLE_V3_Y);

    // X tag.
    paint_text(
        text_system,
        scene,
        "X",
        x,
        y + (FIELD_H - axis_label_font) * 0.5,
        axis_label_font,
        axis_col_w + tag_box_gap,
        resolve(ColorToken::Danger, theme),
    );
    let x_box_x = x + axis_col_w + tag_box_gap;
    let x_rect = Rect::new(x_box_x, y, two_chip_w, FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_V3_X, x_rect);
    let nx = NumberInput::new(ids::INSP_SAMPLE_V3_X, "", vx).state(sx);
    crate::widget::paint_number_input_with_buffer(
        &nx,
        Some(bx),
        cx,
        ax,
        x_rect,
        scene,
        text_system,
        theme,
    );

    // Y tag.
    let y_tag_x = x_box_x + two_chip_w + col_gap;
    paint_text(
        text_system,
        scene,
        "Y",
        y_tag_x,
        y + (FIELD_H - axis_label_font) * 0.5,
        axis_label_font,
        axis_col_w + tag_box_gap,
        resolve(ColorToken::Success, theme),
    );
    let y_box_x = y_tag_x + axis_col_w + tag_box_gap;
    let y_rect = Rect::new(y_box_x, y, two_chip_w, FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_V3_Y, y_rect);
    let ny = NumberInput::new(ids::INSP_SAMPLE_V3_Y, "", vy).state(sy);
    crate::widget::paint_number_input_with_buffer(
        &ny,
        Some(by),
        cy,
        ay,
        y_rect,
        scene,
        text_system,
        theme,
    );

    y + FIELD_H + SECTION_BOTTOM_PAD_PX
}
