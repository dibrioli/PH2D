//! The Grain section's **tiling** rows — Offset X/Y + Size X/Y (each pair on ONE line) — and **Depth**,
//! how strongly the Grain bites.
//!
//! ⚠️ Cut from `paint_texture.rs` by responsibility (2026-09-13): the string-table migration (the
//! `tr(…)` calls the `rustfmt` breaks over several lines) pushed `paint_texture_section` to 205 lines
//! against the 200 cap of a panel function. The body is the one from there, byte for byte.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_tool_painter::{
    BrushSettings, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN,
};

/// ⭐⭐ **A COLUNA DESTA SECÇÃO — uma só, medida sobre os nomes que ela pinta.**
///
/// ⛔ Report do dono, 2026-09-15: *«a caixa recua quando na verdade o nome deveria criar as
/// colunas»*. Ver [`ph2d_editor_core::property_row::Seccao`].
pub(crate) fn seccao_do_grao(ctx: &mut PaintCtx) -> ph2d_editor_core::property_row::Seccao {
    ph2d_editor_core::property_row::Seccao::medida(
        ctx.text_system,
        crate::number_field::SECTION_FIELDS,
        &[
            tr("panel.painter_layers.grain.angle"),
            tr("panel.painter_layers.grain.offset"),
            tr("panel.painter_layers.grain.size"),
            tr("panel.painter_layers.grain.depth"),
        ],
    )
}

/// Paint the tiling rows (and, on the brush, Depth) at `y`, returning the next `y`.
pub(crate) fn paint_texture_tiling(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    mut y: f32,
    brush: BrushSettings,
    compact: bool,
) -> f32 {
    // ── Offset X/Y + Size X/Y — the TEXTURE tiling (each pair on ONE line). Always shown; under
    //    Stencil they tile the pattern INSIDE the rect (the rect placement is the Stencil card). ──
    let sec = seccao_do_grao(ctx);
    y = crate::number_field::paint_num_xy(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.grain.offset"),
        ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
        brush.texture_offset[0],
        ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y,
        brush.texture_offset[1],
        TEX_OFFSET_MIN,
        TEX_OFFSET_MAX,
        crate::number_field::FINE_STEP,
        2,
        sec,
    );
    y = crate::number_field::paint_num_xy(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.grain.size"),
        ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        brush.texture_size[0],
        ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_SIZE_Y,
        brush.texture_size[1],
        TEX_SIZE_MIN,
        TEX_SIZE_MAX,
        crate::number_field::SIZE_STEP,
        2,
        sec,
    );

    // ── Depth — how strongly the Grain bites (brush only; a Texture-LAYER is full-cover). ──
    if !compact {
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            tr("panel.painter_layers.grain.depth"),
            ph2d_tool_painter::ids::PAINTER_BRUSH_GRAIN_DEPTH,
            brush.grain_depth.clamp(0.0, 1.0),
            0.0,
            1.0,
            crate::number_field::FINE_STEP,
            2,
            sec,
        );
    }
    y
}
