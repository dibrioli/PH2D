//! Palette tabs + swatch grid painter.

use super::state::{BlenderColorPicker, ColorPalette};
use crate::interaction::HitIndex;
use crate::paint::{
    fill_rounded_rect, paint_text, paint_text_centered, resolve, stroke_rounded_rect,
};
use crate::widget::{ColorSwatch, paint_color_swatch};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

pub fn paint_palettes(
    cp: &BlenderColorPicker,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_palettes_with_hits(
        cp,
        rect,
        &[NodeId(0); 27],
        NodeId(0),
        &mut HitIndex::new(),
        scene,
        text_system,
        theme,
    );
}

/// Paint the palette section and register each swatch's hit rect.
/// `swatch_ids` is a fixed-size array of up to 12 NodeIds; entries
/// with id == 0 are skipped (no hit registration). `add_swatch_id`
/// renders a trailing "+" button (id == 0 skips it).
#[allow(clippy::too_many_arguments)]
pub fn paint_palettes_with_hits(
    cp: &BlenderColorPicker,
    rect: Rect,
    swatch_ids: &[NodeId; 27],
    add_swatch_id: NodeId,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // Single-palette picker: skip the palette-name tabs entirely
    // (only one palette exists, so the "Default" pill was visual
    // noise). Render the swatch grid directly in the full rect.
    if cp.palettes.is_empty() {
        return;
    }
    let body_rect = rect;
    if let Some(palette) = cp.palettes.get(cp.active_palette) {
        paint_palette_grid(
            palette,
            body_rect,
            swatch_ids,
            add_swatch_id,
            hit_index,
            scene,
            text_system,
            theme,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_palette_grid(
    palette: &ColorPalette,
    rect: Rect,
    swatch_ids: &[NodeId; 27],
    add_swatch_id: NodeId,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let swatch_size = 24.0_f32;
    let gap = Spacing::Xs.px();
    let cols = ((rect.w + gap) / (swatch_size + gap)).max(1.0) as usize;
    let last_idx = palette.swatches.len();
    for (i, value) in palette.swatches.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let x = rect.x + (swatch_size + gap) * col as f32;
        let y = rect.y + (swatch_size + gap) * row as f32;
        if y + swatch_size > rect.y + rect.h {
            break;
        }
        let swatch_rect = Rect::new(x, y, swatch_size, swatch_size);
        let mut sw = ColorSwatch::new(NodeId(i as u64), &palette.name, value.rgba);
        sw.size = crate::widget::SwatchSize::Sm;
        paint_color_swatch(&sw, swatch_rect, scene, theme);
        // Register hit rect for this swatch if an id was provided.
        if let Some(&id) = swatch_ids.get(i)
            && id.0 != 0
        {
            hit_index.register(id, swatch_rect);
        }
    }
    // "+ swatch" tile at the position right after the last swatch.
    // Hidden once the palette hits the static cap (27); deleting any
    // swatch via right-click brings it back.
    const PALETTE_CAP: usize = 27;
    if add_swatch_id.0 != 0 && palette.swatches.len() < PALETTE_CAP {
        let col = last_idx % cols;
        let row = last_idx / cols;
        let x = rect.x + (swatch_size + gap) * col as f32;
        let y = rect.y + (swatch_size + gap) * row as f32;
        if y + swatch_size <= rect.y + rect.h {
            let plus_rect = Rect::new(x, y, swatch_size, swatch_size);
            fill_rounded_rect(
                scene,
                plus_rect,
                Radius::Xs.px(),
                resolve(ColorToken::Bg2, theme),
            );
            stroke_rounded_rect(
                scene,
                plus_rect,
                Radius::Xs.px(),
                1.0,
                resolve(ColorToken::Border, theme),
            );
            paint_text_centered(
                text_system,
                scene,
                "+",
                plus_rect,
                TypeToken::Md.px(),
                resolve(ColorToken::Text2, theme),
            );
            hit_index.register(add_swatch_id, plus_rect);
        }
    }
    if !palette.editable {
        let hint_y = rect.y + rect.h - TypeToken::Xs.px();
        if hint_y > rect.y {
            paint_text(
                text_system,
                scene,
                "Read-only",
                rect.x,
                hint_y,
                TypeToken::Xs.px() - 2.0,
                rect.w,
                resolve(ColorToken::Text3, theme),
            );
        }
    }
}
