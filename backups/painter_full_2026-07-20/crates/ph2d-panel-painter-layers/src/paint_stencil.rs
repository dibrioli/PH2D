//! The **Stencil card** — the on-canvas stencil gizmo's own placement (Size X/Y, Offset X/Y,
//! Rotation), shown in the Grain section ONLY under the Stencil mapping. A decorative rounded-rect
//! card (mirroring the Jitter card) whose number boxes are the SAME drag-scrub fields as the texture
//! params, but bound to the brush's dedicated `stencil_*` state — INDEPENDENT of the texture tiling.
//! The on-canvas handle drag and these boxes write the same fields via `route_brush_stencil_event`.

use crate::number_field::{ANGLE_STEP, FINE_STEP, SIZE_STEP, paint_num_row, paint_num_xy};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{
    BrushSettings, TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN,
};

/// Paint the Stencil placement card (title + Size X/Y, Offset X/Y, Rotation number boxes). The card
/// background is drawn first (height pre-computed from the row count), then the rows on top. Returns
/// the next `y`.
pub(crate) fn paint_stencil_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let pad = Spacing::Sm.px();
    let xs = Spacing::Xs.px();
    let font = TypeToken::Sm.px();
    let title_h = ROW_H_PX;
    // Title + three number-box rows (Size X/Y, Offset X/Y, Rotation), each ROW_H + Xs trailing.
    let rows_h = (ROW_H_PX + xs) + (ROW_H_PX + xs) + (ROW_H_PX + xs);
    let card_h = pad + title_h + rows_h + xs;
    let card = Rect::new(x, y, content_w, card_h);
    fill_rounded_rect(
        ctx.scene,
        card,
        Radius::Md.px(),
        resolve(ColorToken::Bg1, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        card,
        Radius::Md.px(),
        StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );
    let inner_x = x + pad;
    let inner_w = (content_w - pad * 2.0).max(0.0);
    paint_text(
        ctx.text_system,
        ctx.scene,
        "Stencil",
        inner_x,
        y + pad + (title_h - font) * 0.5,
        font,
        inner_w,
        resolve(ColorToken::Text2, theme),
    );
    let mut iy = y + pad + title_h;
    // Size X/Y — the rect half-extent as a sprite fraction (default 0.5 = 50 % of the sprite).
    iy = paint_num_xy(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Size",
        core_ids::PAINTER_BRUSH_STENCIL_SIZE_X,
        brush.stencil_size[0],
        core_ids::PAINTER_BRUSH_STENCIL_SIZE_Y,
        brush.stencil_size[1],
        TEX_SIZE_MIN,
        TEX_SIZE_MAX,
        SIZE_STEP,
        2,
    );
    // Offset X/Y — the rect centre in `−1..1` of the sprite.
    iy = paint_num_xy(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Offset",
        core_ids::PAINTER_BRUSH_STENCIL_OFFSET_X,
        brush.stencil_offset[0],
        core_ids::PAINTER_BRUSH_STENCIL_OFFSET_Y,
        brush.stencil_offset[1],
        TEX_OFFSET_MIN,
        TEX_OFFSET_MAX,
        FINE_STEP,
        2,
    );
    // Rotation — whole degrees.
    iy = paint_num_row(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Rotation",
        core_ids::PAINTER_BRUSH_STENCIL_ANGLE,
        f32::from(brush.stencil_angle_deg),
        0.0,
        f32::from(TEX_ANGLE_MAX_DEG),
        ANGLE_STEP,
        0,
    );
    let _ = iy;
    y + card_h + Spacing::Sm.px()
}
