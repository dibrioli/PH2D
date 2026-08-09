//! O card do **Grid Stamp**: o tamanho da célula e o deslocamento da grade, um slider-com-chip por
//! eixo, mais o checkbox `Show Grid`.
//!
//! Módulo filho pelo mesmo motivo dos irmãos `apply` e `op_card` — é um card, não um punhado de rows
//! soltas no meio do orquestrador de seção.

use super::section_header;
use crate::paint_brush_top::{paint_checkbox_row, paint_slider_chip_row};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_tool_painter::BrushSettings;

/// Paint the **Grid Stamp** rows, returning the next `y`: the cell size and the lattice offset, one
/// slider-with-chip per axis, plus the Show Grid checkbox.
///
/// Painted ONLY while the Grid Stamp method is selected — the lattice is that method's, and a row that
/// governs nothing is the dead control this panel's per-method gating exists to prevent. The chips are
/// the canonical linked pair (`populate_brush_chips`), so typing a number and dragging the slider are
/// two ways to ask for the same thing rather than two paths that can disagree.
pub(super) fn paint_grid_stamp_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let mut y = section_header(ctx, theme, x, content_w, y, "Grid");
    for (axis, label) in [(0usize, "Cell X"), (1, "Cell Y")] {
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            label,
            core_ids::PAINTER_BRUSH_GRID_CELL[axis],
            core_ids::PAINTER_BRUSH_GRID_CELL_CHIPS[axis],
            brush.grid_cell[axis],
        );
    }
    for (axis, label) in [(0usize, "Offset X"), (1, "Offset Y")] {
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            label,
            core_ids::PAINTER_BRUSH_GRID_OFFSET[axis],
            core_ids::PAINTER_BRUSH_GRID_OFFSET_CHIPS[axis],
            brush.grid_offset[axis],
        );
    }
    paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_BRUSH_GRID_SHOW,
        "Show Grid",
        brush.grid_show,
    )
}
