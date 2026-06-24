//! Palette section: a dropdown of named palettes (select / New / Rename / Delete), an inline
//! rename field, the swatch grid, Import/Export, and the deferred dropdown popover.

use super::state::{BlenderColorPicker, ColorPalette};
use super::sub_ids::BlenderSubIds;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{paint_text, resolve};
use crate::widget::{
    Button, ButtonKind, ColorSwatch, Dropdown, DropdownOption, paint_button, paint_color_swatch,
    paint_dropdown_chip, paint_dropdown_popover_in_viewport,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Swatch-grid-only entry point (no palette CRUD chrome) — used by the demo/non-store picker.
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
        &mut HitIndex::new(),
        scene,
        text_system,
        theme,
    );
}

/// Paint the full palette section: the dropdown header (palette select + New / Rename / Delete),
/// an optional inline rename field, the swatch grid + Import/Export, and — last, on top — the open
/// dropdown popover. `picker_rect` is the whole picker (the popover's clamp viewport). Reads the
/// dropdown/rename open flags + the rename buffer from `store`; the swatch + dropdown-option hit
/// rects are registered here so dispatch can route clicks back into `apply_blender_hit`.
#[allow(clippy::too_many_arguments)]
pub fn paint_palette_section(
    cp: &BlenderColorPicker,
    rect: Rect,
    picker_rect: Rect,
    ids: &BlenderSubIds,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    if cp.palettes.is_empty() {
        return;
    }
    let parent = ids.parent;
    let gap = Spacing::Xs.px();
    let strip_h = 22.0_f32; // LITERAL-PX-OK: palette header strip height (chrome-specific)
    let active = cp.active_palette.min(cp.palettes.len() - 1);

    // Header strip: [ dropdown ▼ ] [+] [R] [×]. The dropdown lists every named palette (capped to the
    // 8 reusable `palette_tabs` ids); selecting a row switches the active palette via `PaletteTab`.
    let n_opts = cp.palettes.len().min(ids.palette_tabs.len());
    let options: Vec<DropdownOption<usize>> = cp
        .palettes
        .iter()
        .take(n_opts)
        .enumerate()
        .map(|(i, p)| DropdownOption::new(ids.palette_tabs[i], i, p.name.clone()))
        .collect();
    let open = store.palette_dropdown_open(parent);
    // `placeholder` doubles as the chip label for an active index past the 8-row cap (selected_label
    // returns None there) — either way the chip shows the active palette's name.
    let active_name = cp
        .palettes
        .get(active)
        .map(|p| p.name.clone())
        .unwrap_or_default();
    let mut dd = Dropdown::new(ids.palette_dropdown, "Palette", options)
        .placeholder(active_name)
        .open(open);
    dd.select(active);

    let btn_w = strip_h; // square New / Rename / Delete tiles
    let dd_w = (rect.w - 3.0 * (btn_w + gap)).max(48.0);
    let chip_rect = Rect::new(rect.x, rect.y, dd_w, strip_h);
    paint_dropdown_chip(&dd, chip_rect, scene, text_system, theme);
    if ids.palette_dropdown.0 != 0 {
        hit_index.register(ids.palette_dropdown, chip_rect);
    }
    let bx = rect.x + dd_w + gap;
    for (i, (id, label, kind)) in [
        (ids.new_palette, "+", ButtonKind::Default),
        (ids.rename_palette, "R", ButtonKind::Default),
        (ids.delete_palette, "x", ButtonKind::Danger),
    ]
    .into_iter()
    .enumerate()
    {
        if id.0 == 0 {
            continue;
        }
        let r = Rect::new(bx + (btn_w + gap) * i as f32, rect.y, btn_w, strip_h);
        paint_button(
            &Button::new(id, label).kind(kind),
            r,
            scene,
            text_system,
            theme,
        );
        hit_index.register(id, r);
    }
    let body_y = rect.y + strip_h + gap;

    // Swatch grid + Import/Export in the remaining region. (Rename is a centered modal opened by the
    // "R" button — see `ContextMenuKind::RenamePaletteDialog`; no inline field here.)
    let body_h = (rect.y + rect.h - body_y).max(0.0);
    if body_h > 0.0 {
        let body_rect = Rect::new(rect.x, body_y, rect.w, body_h);
        paint_palettes_with_hits(
            cp,
            body_rect,
            &ids.swatches,
            ids.add_swatch,
            ids.import_palette,
            ids.export_palette,
            hit_index,
            scene,
            text_system,
            theme,
        );
    }

    // Deferred dropdown popover — painted LAST so it occludes the grid, and its option hit rects
    // register LAST so they win the back-to-front HitIndex walk over the swatches beneath.
    if open && !dd.options.is_empty() {
        let panel = dd.popover_rect_clamped(chip_rect, picker_rect);
        paint_dropdown_popover_in_viewport(
            &dd,
            chip_rect,
            Some(picker_rect),
            scene,
            text_system,
            theme,
        );
        for (i, opt) in dd.options.iter().enumerate() {
            hit_index.register(opt.id, dd.option_rect_in(chip_rect, panel, i));
        }
    }
}

/// Paint the swatch grid + Import/Export strip and register each swatch's hit rect. `swatch_ids` is
/// a fixed-size array of up to 27 NodeIds; entries with id == 0 are skipped. `add_swatch_id` renders
/// a trailing "+" tile (id == 0 skips it). Import/Export buttons sit at the bottom when non-zero.
#[allow(clippy::too_many_arguments)]
pub fn paint_palettes_with_hits(
    cp: &BlenderColorPicker,
    rect: Rect,
    swatch_ids: &[NodeId; 27],
    add_swatch_id: NodeId,
    import_id: NodeId,
    export_id: NodeId,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    if cp.palettes.is_empty() {
        return;
    }
    let gap = Spacing::Xs.px();
    let has_io = import_id.0 != 0 || export_id.0 != 0;
    let btn_h = if has_io { 22.0 } else { 0.0 };
    let bottom = btn_h + if has_io { gap } else { 0.0 };
    let body_rect = Rect::new(rect.x, rect.y, rect.w, (rect.h - bottom).max(0.0));
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
            let btn = Button::new(id, label).kind(ButtonKind::Default);
            paint_button(&btn, br, scene, text_system, theme);
            hit_index.register(id, br);
        }
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
            let plus_btn = Button::new(add_swatch_id, "+").kind(ButtonKind::Default);
            paint_button(&plus_btn, plus_rect, scene, text_system, theme);
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
