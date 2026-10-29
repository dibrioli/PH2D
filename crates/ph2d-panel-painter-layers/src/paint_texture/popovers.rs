//! Os popovers da secção **Texture** (Kind + Mapping) e as opções deles — pintados no fim do corpo do Brush,
//! por cima de toda linha. Saíram de `paint_texture.rs` pelo tecto de LOC (só mover, 29/10).

use super::texture_layer_editor_active;
use crate::state;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_tool_painter::ids::{
    painter_brush_texture_kind_option_id, painter_brush_texture_mapping_option_id,
};
use ph2d_tool_painter::{TextureKind, TextureMapping};

/// Deferred paint of the Texture section's open dropdown popovers (Kind + Mapping), drained at the
/// very end of the Brush body so they sit above every row.
pub(crate) fn paint_texture_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip_rect, cur)) = state::take_pending_brush_texture_kind_dd() {
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_KIND,
            texture_kind_options(),
            chip_rect,
            cur,
        );
    }
    if let Some((chip_rect, cur)) = state::take_pending_brush_texture_mapping_dd() {
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_MAPPING,
            texture_mapping_options(),
            chip_rect,
            cur,
        );
    }
    crate::paint_texture_ramp::paint_texture_ramp_popovers(ctx, theme);
}

/// The texture kinds as dropdown options. The brush includes `None` (clear) + `Image` (importable);
/// a **Texture layer** omits both — a layer can never be given image pixels (renders flat) and `None`
/// would make it an opaque slab, so neither is a useful choice there.
fn texture_kind_options() -> Vec<DropdownOption<u8>> {
    let layer = texture_layer_editor_active();
    (0..TextureKind::COUNT)
        .filter(|&k| {
            !(layer
                && matches!(
                    TextureKind::from_u8(k),
                    TextureKind::None | TextureKind::Image
                ))
        })
        .map(|k| {
            DropdownOption::new(
                painter_brush_texture_kind_option_id(k),
                k,
                ph2d_i18n::tr(TextureKind::from_u8(k).name_key()),
            )
        })
        .collect()
}

/// The texture mappings as dropdown options (View Plane / Tiled / Random).
fn texture_mapping_options() -> Vec<DropdownOption<u8>> {
    (0..TextureMapping::COUNT)
        .map(|m| {
            DropdownOption::new(
                painter_brush_texture_mapping_option_id(m),
                m,
                ph2d_i18n::tr(TextureMapping::from_u8(m).name_key()),
            )
        })
        .collect()
}
