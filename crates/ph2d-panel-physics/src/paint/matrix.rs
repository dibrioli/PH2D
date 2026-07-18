//! The collision-layer matrix — a triangular grid of toggles.
//!
//! One cell is one **fact about a pair**, not two facts about two directions:
//! rapier ANDs both ways, so a half-set pair means "no collision", and
//! `LayerMatrix::set` writes both halves. That is why only the lower triangle
//! is drawn — the mirror cell would be a second control for the same checkbox.
//!
//! ⚠️ The cells are registered in a LOOP, which
//! `architecture_panel_wiring_parity` cannot see. The seam test that clicks
//! every cell is therefore not redundant with the arch gates — it is the only
//! thing standing between this widget and being painted but dead.

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_physics_ecs::{LayerMatrix, MAX_LAYERS};
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, TypeToken};

/// Cell edge. Small enough that 8 columns plus the row label fit the dock, big
/// enough to hit — the grid is 8 × this wide, and the dock is ~300 px.
const CELL_PX: f32 = 22.0; // LITERAL-PX-OK: panel grid metric (matrix cell edge)

/// Paint the triangular matrix. Returns the y it ended at.
pub(super) fn paint(ctx: &mut PaintCtx, matrix: LayerMatrix, x: f32, y_in: f32) -> f32 {
    let theme = ctx.host.theme();
    let gap = Spacing::Xs.px();
    let step = CELL_PX + gap;
    let label_w = CELL_PX; // the row's own layer number, same width as a cell
    let font = TypeToken::Sm.px();

    let mut y = y_in;

    // Column headers, offset by the label gutter. Only as many as the widest
    // row needs — the last row is the only one with all eight.
    for j in 0..MAX_LAYERS {
        let cx = x + label_w + gap + j as f32 * step;
        paint_text(
            ctx.text_system,
            ctx.scene,
            &j.to_string(),
            cx + (CELL_PX - font) * 0.5,
            y,
            font,
            CELL_PX,
            resolve(ColorToken::Text2, theme),
        );
    }
    y += font + gap;

    for i in 0..MAX_LAYERS {
        // Row label.
        paint_text(
            ctx.text_system,
            ctx.scene,
            &i.to_string(),
            x + (label_w - font) * 0.5,
            y + (CELL_PX - font) * 0.5,
            font,
            label_w,
            resolve(ColorToken::Text2, theme),
        );
        for j in 0..=i {
            let cell = Rect::new(x + label_w + gap + j as f32 * step, y, CELL_PX, CELL_PX);
            let on = matrix.collides(i, j);
            // On = accent fill, off = the panel's recessed surface. A checkmark
            // glyph at this size would be mud; the fill IS the state.
            let token = if on {
                ColorToken::Accent
            } else {
                ColorToken::Bg3
            };
            fill_rounded_rect(ctx.scene, cell, Radius::Sm.px(), resolve(token, theme));
            // The diagonal is a layer against ITSELF, and turning it off is a
            // real and useful thing (a layer whose members ignore each other),
            // so it is a cell like any other — just outlined, so the eye can
            // find the axis of symmetry.
            if i == j {
                ph2d_editor_core::paint::stroke_rounded_rect(
                    ctx.scene,
                    cell,
                    Radius::Sm.px(),
                    StrokeToken::Default.px(),
                    resolve(ColorToken::Text2, theme),
                );
            }
            ctx.host.hit_index_mut().register(
                ids::PHYSICS_LAYER_CELL[ids::physics_layer_cell_index(i, j)],
                cell,
            );
        }
        y += step;
    }
    y
}
