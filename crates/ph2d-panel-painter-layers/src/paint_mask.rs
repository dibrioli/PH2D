//! The collapsible **Mask** section — pinned at the TOP of the Brush panel while the Mask tool is
//! active. Three width-adaptive button groups (they reflow when the panel narrows):
//!
//! 1. the mask **sub-brush** (Paint / Erase / Blur / Smear) — a segmented toggle group;
//! 2. the whole-canvas **ops** (Expand / Contract / Blur / Sharpen / Invert / Clear) — one-click
//!    buttons with the shared hover/press surface;
//! 3. the on-canvas **overlay-tint colour** (neutral gray + 4 fluorescent-marker hues) — square swatches.
//!
//! Every widget is a FIXED-id registered in [`crate::populate`]; clicks forward over the frozen
//! `PanelEvent` Click channel (whitelisted in `event.rs`) to the tool's `route_mask_event`.

use crate::paint::register_button;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::panel_chrome::paint_segmented_group_adaptive;
use ph2d_editor_core::widget::{
    ButtonState, ColorSwatch, SectionHeader, SwatchSize, SwatchState, flat_button_surface,
    paint_color_swatch, paint_section_header,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::BrushSettings;

/// Square edge (px) of the overlay-colour swatches — the small palette size, laid out in a wrap grid.
const SWATCH_PX: f32 = 26.0; // LITERAL-PX-OK: overlay-colour swatch square
/// Minimum op-button width (px) before the flow grid drops to fewer columns (adaptive reflow floor).
const OP_MIN_W: f32 = 74.0; // LITERAL-PX-OK: op-button min column width

/// The 5 overlay-tint colours (straight RGBA8): neutral gray + fluorescent yellow / pink / green /
/// orange — must mirror `ph2d-tool-painter`'s `mask_overlay_rgb`. Data colours, not chrome tokens.
const OVERLAY_COLORS: [[u8; 4]; 5] = [
    [128, 128, 128, 255], // LITERAL-COLOR-OK: neutral gray (default)
    [220, 255, 0, 255],   // LITERAL-COLOR-OK: fluorescent yellow
    [255, 42, 160, 255],  // LITERAL-COLOR-OK: fluorescent pink
    [80, 255, 60, 255],   // LITERAL-COLOR-OK: fluorescent green
    [255, 120, 0, 255],   // LITERAL-COLOR-OK: fluorescent orange
];

/// Paint the Mask section from `y`; returns the next `y`. Collapsed → just the header. Only called by
/// [`crate::paint_brush::paint_brush_body`] when the active tool is Mask.
pub(crate) fn paint_mask_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let gap = Spacing::Xs.px();

    // ── Collapsible header ──
    let header_h = TypeToken::Md.px() + Spacing::Md.px();
    let collapsed = ctx
        .host
        .store()
        .is_collapsed(core_ids::PAINTER_MASK_SECTION);
    let header = SectionHeader::new(core_ids::PAINTER_MASK_SECTION, "Mask").collapsible(!collapsed);
    let header_rect = Rect::new(x, y, content_w, header_h);
    {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        paint_section_header(&header, header_rect, scene, text_system, theme);
    }
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_MASK_SECTION, header_rect);
    let mut y = y + header_h + Spacing::Xs.px();
    if collapsed {
        return y;
    }

    // ── 1. Mask sub-brush toggle group (adaptive; reflows when narrow) ──
    let brush_labels = ["Paint", "Erase", "Blur", "Smear"];
    let segs: Vec<(&str, bool, ph2d_a11y::NodeId)> = brush_labels
        .iter()
        .enumerate()
        .map(|(i, l)| {
            (
                *l,
                brush.mask_brush == i as u8,
                core_ids::PAINTER_MASK_BRUSH[i],
            )
        })
        .collect();
    let used = {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let hit = ctx.host.hit_index_mut();
        paint_segmented_group_adaptive(
            Rect::new(x, y, content_w, ROW_H_PX),
            &segs,
            scene,
            text_system,
            theme,
            hit,
        )
    };
    // The segmented segments dispatch a Click only if registered as Buttons — done in `populate`.
    y += used + gap;

    // ── 2. Whole-canvas op buttons (flow grid; reflow to fewer columns when narrow) ──
    let op_labels = ["Expand", "Contract", "Blur", "Sharpen", "Invert", "Clear"];
    let (op_rects, op_h) = flow_fill(x, y, content_w, op_labels.len(), OP_MIN_W, ROW_H_PX, gap);
    for (i, label) in op_labels.iter().enumerate() {
        paint_action_button(ctx, theme, op_rects[i], label, core_ids::PAINTER_MASK_OP[i]);
    }
    y += op_h + gap;

    // ── 3. Overlay-tint colour swatches (fixed-size wrap grid) ──
    let (col_rects, col_h) = flow_fixed(
        x,
        y,
        content_w,
        OVERLAY_COLORS.len(),
        SWATCH_PX,
        SWATCH_PX,
        gap,
    );
    for (i, rgba) in OVERLAY_COLORS.iter().enumerate() {
        let id = core_ids::PAINTER_MASK_COLOR[i];
        let state = swatch_state(ctx, id);
        let swatch = ColorSwatch::new(id, "", *rgba)
            .size(SwatchSize::Sm)
            .state(state);
        paint_color_swatch(&swatch, col_rects[i], ctx.scene, theme);
        // Selected colour → an accent ring over the swatch's own border.
        if brush.mask_overlay_color == i as u8 {
            stroke_rounded_rect(
                ctx.scene,
                col_rects[i],
                Radius::Sm.px(),
                StrokeToken::Thick.px(),
                resolve(ColorToken::Accent, theme),
            );
        }
        register_button(ctx.host.store_mut(), id);
        ctx.host.hit_index_mut().register(id, col_rects[i]);
    }
    y += col_h + Spacing::Sm.px();
    y
}

/// A one-click op button with the shared resting surface + hover/press feedback (`flat_button_surface`),
/// a border, and a centred label. Registers its Button slot + hit rect. Mirror of the Stroke card's icon
/// button, so every mask op reads as a peer of the rest of the panel's flat buttons.
fn paint_action_button(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    rect: Rect,
    label: &str,
    id: ph2d_a11y::NodeId,
) {
    let state = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    fill_rounded_rect(
        ctx.scene,
        rect,
        Radius::Sm.px(),
        resolve(flat_button_surface(state), theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        rect,
        Radius::Sm.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    paint_text_centered(
        ctx.text_system,
        ctx.scene,
        label,
        rect,
        TypeToken::Sm.px(),
        resolve(ColorToken::Text1, theme),
    );
    register_button(ctx.host.store_mut(), id);
    ctx.host.hit_index_mut().register(id, rect);
}

/// Map the widget's `ButtonState` (set by the dispatcher on hover/press) to the swatch's visual state.
fn swatch_state(ctx: &PaintCtx, id: ph2d_a11y::NodeId) -> SwatchState {
    match ctx.host.store().button_state(id) {
        Some(ButtonState::Hovered) => SwatchState::Hovered,
        Some(ButtonState::Pressed) => SwatchState::Pressed,
        _ => SwatchState::Normal,
    }
}

/// Wrap `n` equal-width cells into rows: choose the column count from `min_cell_w`, then stretch the
/// cells to fill `avail_w` (a tidy filled grid). Returns each cell rect + the total height used.
fn flow_fill(
    x: f32,
    y: f32,
    avail_w: f32,
    n: usize,
    min_cell_w: f32,
    cell_h: f32,
    gap: f32,
) -> (Vec<Rect>, f32) {
    if n == 0 {
        return (Vec::new(), 0.0);
    }
    // Column count in `[1, n]` as two steps (not `.clamp(1, n)`): n ≥ 1 here (early return above) but the
    // max is dynamic, which the clamp-safety gate rejects — split also dodges clippy's manual_clamp.
    let cols = ((avail_w + gap) / (min_cell_w + gap)).floor() as usize;
    let cols = cols.max(1);
    let cols = cols.min(n);
    let rows = n.div_ceil(cols);
    let cell_w = ((avail_w - gap * (cols as f32 - 1.0)) / cols as f32).max(0.0);
    let mut rects = Vec::with_capacity(n);
    for i in 0..n {
        let cx = x + (cell_w + gap) * (i % cols) as f32;
        let cy = y + (cell_h + gap) * (i / cols) as f32;
        rects.push(Rect::new(cx, cy, cell_w, cell_h));
    }
    (
        rects,
        rows as f32 * cell_h + (rows.saturating_sub(1)) as f32 * gap,
    )
}

/// Wrap `n` fixed-size cells left-to-right into rows that fit `avail_w` (the square colour swatches keep
/// their size — a stretched square would read wrong). Returns each cell rect + the total height used.
fn flow_fixed(
    x: f32,
    y: f32,
    avail_w: f32,
    n: usize,
    cell_w: f32,
    cell_h: f32,
    gap: f32,
) -> (Vec<Rect>, f32) {
    if n == 0 {
        return (Vec::new(), 0.0);
    }
    // Column count in `[1, n]` as two steps (see `flow_fill`): the clamp-safety gate rejects a dynamic max.
    let cols = ((avail_w + gap) / (cell_w + gap)).floor() as usize;
    let cols = cols.max(1);
    let cols = cols.min(n);
    let rows = n.div_ceil(cols);
    let mut rects = Vec::with_capacity(n);
    for i in 0..n {
        let cx = x + (cell_w + gap) * (i % cols) as f32;
        let cy = y + (cell_h + gap) * (i / cols) as f32;
        rects.push(Rect::new(cx, cy, cell_w, cell_h));
    }
    (
        rects,
        rows as f32 * cell_h + (rows.saturating_sub(1)) as f32 * gap,
    )
}
