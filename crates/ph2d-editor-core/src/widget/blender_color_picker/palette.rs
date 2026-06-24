//! Palette tabs + swatch grid painter.

use super::state::{BlenderColorPicker, ColorPalette};
use crate::interaction::HitIndex;
use crate::paint::{paint_text, resolve};
use crate::widget::{ColorSwatch, paint_color_swatch};
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
    paint_palettes_with_hits(
        cp,
        rect,
        &[NodeId(0); 27],
        NodeId(0),
        NodeId(0),
        NodeId(0),
        &[],
        NodeId(0),
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
    import_id: NodeId,
    export_id: NodeId,
    tab_ids: &[NodeId],
    new_id: NodeId,
    delete_id: NodeId,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    if cp.palettes.is_empty() {
        return;
    }
    let gap = Spacing::Xs.px();
    // Top: the named-palette tab strip (select / New / Delete). Bottom: the Import / Export strip.
    // The swatch grid takes the middle.
    let has_tabs = new_id.0 != 0 && tab_ids.iter().any(|t| t.0 != 0);
    let tab_h = if has_tabs { 22.0 } else { 0.0 };
    if has_tabs {
        let tab_rect = Rect::new(rect.x, rect.y, rect.w, tab_h);
        paint_palette_tabs(
            cp,
            tab_rect,
            tab_ids,
            new_id,
            delete_id,
            hit_index,
            scene,
            text_system,
            theme,
        );
    }
    let has_io = import_id.0 != 0 || export_id.0 != 0;
    let btn_h = if has_io { 22.0 } else { 0.0 };
    let top = tab_h + if has_tabs { gap } else { 0.0 };
    let bottom = btn_h + if has_io { gap } else { 0.0 };
    let body_rect = Rect::new(
        rect.x,
        rect.y + top,
        rect.w,
        (rect.h - top - bottom).max(0.0),
    );
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
    if has_io {
        let by = rect.y + rect.h - btn_h;
        let bw = (rect.w - gap) / 2.0;
        for (id, label, bx) in [
            (import_id, "Import", rect.x),
            (export_id, "Export", rect.x + bw + gap),
        ] {
            if id.0 == 0 {
                continue;
            }
            let br = Rect::new(bx, by, bw, btn_h);
            let btn =
                crate::widget::Button::new(id, label).kind(crate::widget::ButtonKind::Default);
            crate::widget::paint_button(&btn, br, scene, text_system, theme);
            hit_index.register(id, br);
        }
    }
}

/// The named-palette tab strip: one button per palette (the active one filled, the rest ghost) that
/// selects it, then a "+" (New palette) and "×" (Delete palette) at the right. Tabs share the width
/// left of the two square buttons; only the first `tab_ids.len()` palettes get a clickable tab.
#[allow(clippy::too_many_arguments)]
fn paint_palette_tabs(
    cp: &BlenderColorPicker,
    rect: Rect,
    tab_ids: &[NodeId],
    new_id: NodeId,
    delete_id: NodeId,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    use crate::widget::{Button, ButtonKind, paint_button};
    let gap = Spacing::Xs.px();
    let btn_w = rect.h; // square New / Delete tiles at the right
    let tabs_w = (rect.w - 2.0 * (btn_w + gap)).max(0.0);
    let n = cp.palettes.len().min(tab_ids.len());
    if n > 0 {
        let tab_w = (tabs_w - gap * (n as f32 - 1.0)) / n as f32;
        for (i, palette) in cp.palettes.iter().take(n).enumerate() {
            let id = tab_ids[i];
            if id.0 == 0 {
                continue;
            }
            let r = Rect::new(rect.x + (tab_w + gap) * i as f32, rect.y, tab_w, rect.h);
            let kind = if i == cp.active_palette {
                ButtonKind::Accent
            } else {
                ButtonKind::Default
            };
            paint_button(
                &Button::new(id, &palette.name).kind(kind),
                r,
                scene,
                text_system,
                theme,
            );
            hit_index.register(id, r);
        }
    }
    let nx = rect.x + rect.w - 2.0 * btn_w - gap;
    for (id, label, x, kind) in [
        (new_id, "+", nx, ButtonKind::Default),
        (delete_id, "x", nx + btn_w + gap, ButtonKind::Danger),
    ] {
        if id.0 == 0 {
            continue;
        }
        let r = Rect::new(x, rect.y, btn_w, rect.h);
        paint_button(
            &Button::new(id, label).kind(kind),
            r,
            scene,
            text_system,
            theme,
        );
        hit_index.register(id, r);
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
            // Canonical button (single source of truth) — bordered ghost
            // "+" tile, consistent with every other secondary button.
            let plus_btn = crate::widget::Button::new(add_swatch_id, "+")
                .kind(crate::widget::ButtonKind::Default);
            crate::widget::paint_button(&plus_btn, plus_rect, scene, text_system, theme);
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
