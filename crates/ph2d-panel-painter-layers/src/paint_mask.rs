//! The collapsible **Mask** section — pinned at the TOP of the Brush panel while the Mask tool is
//! active. Three titled CARDS (each a width-adaptive grid that reflows when the panel narrows), named so
//! the repeated Blur/Smear labels can't be confused between the paint brushes and the whole-canvas ops:
//!
//! - **Brushes** — the mask sub-brush (Paint / Erase / Blur / Smear): a toggle group (one selected).
//! - **Modifiers** — the whole-canvas ops (Expand / Contract / Blur / Sharpen / Invert / Clear):
//!   one-click buttons with the shared hover/press surface.
//! - **Overlay Color** — the on-canvas quick-mask tint (neutral gray + 4 fluorescent-marker hues).
//!
//! Every widget is a FIXED-id registered in [`crate::populate`]; clicks forward over the frozen
//! `PanelEvent` Click channel (whitelisted in `event.rs`) to the tool's `route_mask_event`.

use crate::paint::register_button;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    ButtonState, ColorSwatch, SectionHeader, SwatchSize, SwatchState, flat_button_surface,
    paint_color_swatch, paint_section_header,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::BrushSettings;

/// Square edge (px) of the overlay-colour swatches — the small palette size, laid out in a wrap grid.
const SWATCH_PX: f32 = 26.0; // LITERAL-PX-OK: overlay-colour swatch square
/// Minimum op/brush-button width (px) before the flow grid drops to fewer columns (adaptive reflow floor).
const BTN_MIN_W: f32 = 64.0; // LITERAL-PX-OK: mask button min column width

/// The 5 overlay-tint colours (straight RGBA8): DARK gray (default) + fluorescent yellow / pink / green
/// / orange — must mirror `ph2d-tool-painter`'s `mask_overlay_rgb`. Data colours, not chrome tokens.
const OVERLAY_COLORS: [[u8; 4]; 5] = [
    [51, 51, 51, 255],   // LITERAL-COLOR-OK: dark gray (default)
    [220, 255, 0, 255],  // LITERAL-COLOR-OK: fluorescent yellow
    [255, 42, 160, 255], // LITERAL-COLOR-OK: fluorescent pink
    [80, 255, 60, 255],  // LITERAL-COLOR-OK: fluorescent green
    [255, 120, 0, 255],  // LITERAL-COLOR-OK: fluorescent orange
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

    // ── Card 1: Brushes — the mask sub-brush toggle group (one selected). Paint sits SOLO on the top
    //    row (the primary brush), Erase/Blur/Smear reflow below (Enio). ──
    let brush_labels = ["Paint", "Erase", "Blur", "Smear"];
    y = button_card(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Brushes",
        &brush_labels,
        &core_ids::PAINTER_MASK_BRUSH,
        Some(brush.mask_brush as usize),
        true, // Paint solo on the top row
    );

    // ── Card 2: Modifiers — whole-canvas ops (one-click). ──
    let op_labels = ["Expand", "Contract", "Blur", "Sharpen", "Invert", "Clear"];
    y = button_card(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Modifiers",
        &op_labels,
        &core_ids::PAINTER_MASK_OP,
        None,
        false,
    );

    // ── Card 3: Overlay Color — the quick-mask tint swatches. ──
    y = colors_card(
        ctx,
        theme,
        x,
        content_w,
        y,
        brush.mask_overlay_color as usize,
    );
    y
}

/// Draw a titled card box (Bg1 fill + Border + an ALL-CAPS `title`) and return `(inner_x, inner_w,
/// body_top, next_y)`. `body_h` is the pre-measured height of the card's content grid.
fn card_frame(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    title: &str,
    body_h: f32,
) -> (f32, f32, f32, f32) {
    let pad = Spacing::Sm.px();
    let title_h = TypeToken::Sm.px() + Spacing::Xs.px();
    let card_h = pad + title_h + body_h + pad;
    let card = Rect::new(x, y, content_w, card_h);
    let radius = Radius::Md.px();
    fill_rounded_rect(ctx.scene, card, radius, resolve(ColorToken::Bg1, theme));
    stroke_rounded_rect(
        ctx.scene,
        card,
        radius,
        StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );
    paint_text(
        ctx.text_system,
        ctx.scene,
        title,
        x + pad,
        y + pad,
        TypeToken::Sm.px(),
        content_w - 2.0 * pad,
        resolve(ColorToken::Text2, theme),
    );
    let inner_x = x + pad;
    let inner_w = (content_w - 2.0 * pad).max(0.0);
    let body_top = y + pad + title_h;
    (inner_x, inner_w, body_top, y + card_h + Spacing::Sm.px())
}

/// A card of labelled buttons in a reflowing grid. `selected` = `Some(i)` renders a toggle group (the
/// selected button is Accent-filled); `None` renders one-click action buttons. `first_solo` puts
/// `labels[0]` alone on a full-width top row (the primary brush), the rest reflowing below. Returns `y`.
#[allow(clippy::too_many_arguments)]
fn button_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    title: &str,
    labels: &[&str],
    ids: &[ph2d_a11y::NodeId],
    selected: Option<usize>,
    first_solo: bool,
) -> f32 {
    let gap = Spacing::Xs.px();
    let pad = Spacing::Sm.px();
    let inner_w = (content_w - 2.0 * pad).max(0.0);
    let solo = first_solo && labels.len() > 1;
    // Body height: solo top row + the reflow of the rest, or a single reflow of everything.
    let body_h = if solo {
        let (_, rest_h) = flow_fill(
            0.0,
            0.0,
            inner_w,
            labels.len() - 1,
            BTN_MIN_W,
            ROW_H_PX,
            gap,
        );
        ROW_H_PX + gap + rest_h
    } else {
        flow_fill(0.0, 0.0, inner_w, labels.len(), BTN_MIN_W, ROW_H_PX, gap).1
    };
    let (inner_x, inner_w, body_top, next_y) =
        card_frame(ctx, theme, x, content_w, y, title, body_h);
    if solo {
        // `labels[0]` full-width on top; the remainder reflows on the row(s) below.
        let top = Rect::new(inner_x, body_top, inner_w, ROW_H_PX);
        paint_button_cell(ctx, theme, top, labels[0], ids[0], selected == Some(0));
        let rest_top = body_top + ROW_H_PX + gap;
        let (rects, _) = flow_fill(
            inner_x,
            rest_top,
            inner_w,
            labels.len() - 1,
            BTN_MIN_W,
            ROW_H_PX,
            gap,
        );
        for (j, label) in labels[1..].iter().enumerate() {
            paint_button_cell(
                ctx,
                theme,
                rects[j],
                label,
                ids[j + 1],
                selected == Some(j + 1),
            );
        }
    } else {
        let (rects, _) = flow_fill(
            inner_x,
            body_top,
            inner_w,
            labels.len(),
            BTN_MIN_W,
            ROW_H_PX,
            gap,
        );
        for (i, label) in labels.iter().enumerate() {
            paint_button_cell(ctx, theme, rects[i], label, ids[i], selected == Some(i));
        }
    }
    next_y
}

/// A card of overlay-colour swatches (fixed-size wrap grid). The selected colour gets an accent ring.
fn colors_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    selected: usize,
) -> f32 {
    let gap = Spacing::Xs.px();
    let pad = Spacing::Sm.px();
    let inner_w = (content_w - 2.0 * pad).max(0.0);
    let n = OVERLAY_COLORS.len();
    let (_, body_h) = flow_fixed(0.0, 0.0, inner_w, n, SWATCH_PX, SWATCH_PX, gap);
    let (inner_x, inner_w, body_top, next_y) =
        card_frame(ctx, theme, x, content_w, y, "Overlay Color", body_h);
    let (rects, _) = flow_fixed(inner_x, body_top, inner_w, n, SWATCH_PX, SWATCH_PX, gap);
    for (i, rgba) in OVERLAY_COLORS.iter().enumerate() {
        let id = core_ids::PAINTER_MASK_COLOR[i];
        let state = swatch_state(ctx, id);
        let swatch = ColorSwatch::new(id, "", *rgba)
            .size(SwatchSize::Sm)
            .state(state);
        paint_color_swatch(&swatch, rects[i], ctx.scene, theme);
        if selected == i {
            stroke_rounded_rect(
                ctx.scene,
                rects[i],
                Radius::Sm.px(),
                StrokeToken::Thick.px(),
                resolve(ColorToken::Accent, theme),
            );
        }
        register_button(ctx.host.store_mut(), id);
        ctx.host.hit_index_mut().register(id, rects[i]);
    }
    next_y
}

/// One labelled mask button: `selected` → Accent-filled toggle; otherwise the shared flat surface with
/// hover/press feedback. Registers its Button slot + hit rect.
fn paint_button_cell(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    rect: Rect,
    label: &str,
    id: ph2d_a11y::NodeId,
    selected: bool,
) {
    let (bg, fg) = if selected {
        (ColorToken::Accent, ColorToken::Bg0)
    } else {
        let state = ctx
            .host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal);
        (flat_button_surface(state), ColorToken::Text1)
    };
    fill_rounded_rect(ctx.scene, rect, Radius::Sm.px(), resolve(bg, theme));
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
        resolve(fg, theme),
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
