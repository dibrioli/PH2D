//! Palette tabs + swatch grid painter.

use super::state::{BlenderColorPicker, ColorPalette};
use crate::paint::{paint_text, resolve};
use crate::widget::{ColorSwatch, TabItem, Tabs, TabsVariant, paint_color_swatch, paint_tabs};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

pub fn paint_palettes(
    cp: &BlenderColorPicker,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let tabs_h = 28.0_f32;
    let tabs_rect = Rect::new(rect.x, rect.y, rect.w, tabs_h);
    let tab_items: Vec<TabItem> = cp
        .palettes
        .iter()
        .enumerate()
        .map(|(i, p)| TabItem::new(NodeId(i as u64), p.name.clone()))
        .collect();
    if tab_items.is_empty() {
        return;
    }
    let tabs = Tabs::new(NodeId(0), "Palettes", tab_items)
        .selected(cp.active_palette)
        .variant(TabsVariant::Segmented);
    paint_tabs(&tabs, tabs_rect, scene, text_system, theme);

    let body_y = rect.y + tabs_h + Spacing::Md.px();
    let body_rect = Rect::new(rect.x, body_y, rect.w, rect.y + rect.h - body_y);
    if let Some(palette) = cp.palettes.get(cp.active_palette) {
        paint_palette_grid(palette, body_rect, scene, text_system, theme);
    }
}

fn paint_palette_grid(
    palette: &ColorPalette,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let swatch_size = 24.0_f32;
    let gap = Spacing::Xs.px();
    let cols = ((rect.w + gap) / (swatch_size + gap)).max(1.0) as usize;
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
