//! **The background grid** — the dotted-lattice backdrop of the graph canvas. Split from
//! `paint` for the panel LOC cap; `super` is `paint`. Drawn first (under the wires and cards).

use ph2d_editor_core::paint::{resolve, stroke_polyline};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

const GRID_STEP: f32 = 32.0; // LITERAL-PX-OK: background grid spacing

pub(super) fn draw_grid(ctx: &mut PaintCtx, rect: Rect, theme: Theme) {
    let grid = resolve(ColorToken::GraphGrid, theme);
    let step = GRID_STEP;
    let mut x = rect.x;
    while x < rect.x + rect.w {
        stroke_polyline(ctx.scene, &[(x, rect.y), (x, rect.y + rect.h)], 1.0, grid);
        x += step;
    }
    let mut y = rect.y;
    while y < rect.y + rect.h {
        stroke_polyline(ctx.scene, &[(rect.x, y), (rect.x + rect.w, y)], 1.0, grid);
        y += step;
    }
}
