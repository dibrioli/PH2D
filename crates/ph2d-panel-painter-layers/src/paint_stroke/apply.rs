//! The Stroke-Method chrome shown for the open-shape methods (Curve / Free Hand / Ellipse / Polygon): the
//! **Simplify** button, the **Apply / Apply & Keep / Edit / Delete** row, and the **Offset** card (the
//! offset slider + a **Trim** self-intersection checkbox). Split from `paint_stroke` for the workspace
//! LOC cap. Each control forwards a `Click` / `SetValue` the tool routes in `route_brush_dab_event`;
//! registering the Button slot is what lets the dispatch emit the `Click`.

use crate::paint::register_button;
use crate::paint_brush_top::{paint_checkbox_row, paint_slider_chip_row};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{ButtonState, flat_button_surface};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::BrushSettings;

/// Paint the Apply / Apply & Keep + trailing square-icon cluster (optional **E**dit, then **✕** Delete)
/// row, returning the next `y`. `with_edit` adds the E button (Ellipse/Polygon → editable curve), left of
/// ✕. The two text buttons share the row; when the panel is too narrow for two readable text buttons,
/// "Apply & Keep" wraps below and "Apply" keeps the icon cluster beside it so Edit/Delete stay reachable.
pub(super) fn paint_apply_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    with_edit: bool,
) -> f32 {
    let gap = Spacing::Xs.px();
    let sq = ROW_H_PX; // square icon-button side
    // Convertible shapes (Ellipse / Polygon) get a full-width **Convert to Curve** button on its own row
    // (renamed from the cramped square "E" — Enio 2026-07-03), then the Apply / Apply & Keep / ✕ row.
    let mut y = y;
    if with_edit {
        button(
            ctx,
            theme,
            Rect::new(x, y, content_w, ROW_H_PX),
            Some("Convert to Curve"),
            core_ids::PAINTER_BRUSH_STROKE_EDIT,
        );
        y += ROW_H_PX + gap;
    }
    // Trailing cluster is now just ✕ Delete.
    let icons = sq;
    let min_text_btn_w = 84.0; // LITERAL-PX-OK: layout breakpoint — min readable text-button width before wrap
    let one_row = content_w >= min_text_btn_w * 2.0 + icons + gap * 2.0;
    let apply = core_ids::PAINTER_BRUSH_STROKE_APPLY;
    let keep = core_ids::PAINTER_BRUSH_STROKE_APPLY_KEEP;
    if one_row {
        // Apply | Apply & Keep | E ✕ — three abutting clusters that exactly fill `content_w`. The icon
        // cluster sits AFTER Apply & Keep (a past bug placed both at the same x, so the Keep fill covered
        // E/✕ and stole their clicks).
        let w = (content_w - icons - gap * 2.0) * 0.5;
        button(
            ctx,
            theme,
            Rect::new(x, y, w, ROW_H_PX),
            Some("Apply"),
            apply,
        );
        let keep_x = x + w + gap;
        button(
            ctx,
            theme,
            Rect::new(keep_x, y, w, ROW_H_PX),
            Some("Apply & Keep"),
            keep,
        );
        paint_icon_cluster(ctx, theme, keep_x + w + gap, y, sq);
        y + ROW_H_PX + Spacing::Xs.px()
    } else {
        // Narrow: Apply (+ icons) on row 1; Apply & Keep full-width on row 2.
        let text_w = content_w - icons - gap;
        button(
            ctx,
            theme,
            Rect::new(x, y, text_w, ROW_H_PX),
            Some("Apply"),
            apply,
        );
        paint_icon_cluster(ctx, theme, x + text_w + gap, y, sq);
        let keep_y = y + ROW_H_PX + gap;
        button(
            ctx,
            theme,
            Rect::new(x, keep_y, content_w, ROW_H_PX),
            Some("Apply & Keep"),
            keep,
        );
        keep_y + ROW_H_PX + Spacing::Xs.px()
    }
}

/// Paint the **Simplify** button row (full width) directly under the Stroke-Method dropdown — re-fits the
/// editable curve to a clean minimal control polygon (the tool's `curve_simplify`, same Free Hand fit).
/// Shown for the four open-shape methods; a no-op when no curve editor is open. Returns the next `y`.
pub(super) fn paint_simplify_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
) -> f32 {
    button(
        ctx,
        theme,
        Rect::new(x, y, content_w, ROW_H_PX),
        Some("Simplify"),
        core_ids::PAINTER_BRUSH_STROKE_SIMPLIFY,
    );
    y + ROW_H_PX + Spacing::Xs.px()
}

/// Paint the **Offset** card (modeled on the Jitter card): the perpendicular-offset slider, then a **Trim**
/// checkbox (cut the offset's self-intersections — insert a point at the crossing, drop the looped excess).
/// Shown for the open-shape methods. Returns the next `y`.
pub(super) fn paint_offset_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let (pad, xs, font, title_h) = (
        Spacing::Sm.px(),
        Spacing::Xs.px(),
        TypeToken::Sm.px(),
        ROW_H_PX,
    );
    let rows_h = (ROW_H_PX + xs) * 2.0; // slider + Trim checkbox
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
        "Offset",
        inner_x,
        y + pad + (title_h - font) * 0.5,
        font,
        inner_w,
        resolve(ColorToken::Text2, theme),
    );
    let mut iy = y + pad + title_h;
    iy = paint_slider_chip_row(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Distance",
        core_ids::PAINTER_BRUSH_OFFSET,
        core_ids::PAINTER_BRUSH_OFFSET_CHIP,
        brush.offset,
    );
    paint_checkbox_row(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        core_ids::PAINTER_BRUSH_OFFSET_TRIM,
        "Trim",
        brush.offset_trim,
    );
    y + card_h + Spacing::Sm.px()
}

/// Paint the trailing **✕** Delete square-icon at `ix` (the Convert-to-Curve button moved to its own
/// full-width row above — Enio 2026-07-03).
fn paint_icon_cluster(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, ix: f32, iy: f32, sq: f32) {
    let delete = core_ids::PAINTER_BRUSH_STROKE_DELETE;
    button(ctx, theme, Rect::new(ix, iy, sq, ROW_H_PX), None, delete);
}

/// One stroke shape-editor TEXT button (Apply / Apply & Keep / E). Shares the flat-button chrome (fill /
/// border / height) so the row reads as peers. Registers the Button slot so the dispatch can `Click`.
fn button(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    r: Rect,
    label: Option<&str>,
    id: ph2d_a11y::NodeId,
) {
    // The icon-only ✕ Delete delegates to the shared icon-button chrome so it stays size-matched with the
    // Save-As-Object button beside the Method dropdown.
    let Some(text) = label else {
        paint_icon_button(ctx, theme, r, ph2d_editor_core::IconId::Close, id);
        return;
    };
    // Fill follows the button's ButtonState (idle / hover / press) via the central `flat_button_surface`,
    // so the click is visible — same source every flat button uses.
    let state = match ctx.host.store().get(id) {
        Some(InteractiveState::Button { state }) => *state,
        _ => ButtonState::Normal,
    };
    fill_rounded_rect(
        ctx.scene,
        r,
        Radius::Sm.px(),
        resolve(flat_button_surface(state), theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        r,
        Radius::Sm.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    paint_text_centered(
        ctx.text_system,
        ctx.scene,
        text,
        r,
        TypeToken::Base.px(),
        resolve(ColorToken::Text1, theme),
    );
    register_button(ctx.host.store_mut(), id);
    ctx.host.hit_index_mut().register(id, r);
}

/// Paint a square ICON button (fill / border / centered `icon`) + register its Button slot + hit rect — the
/// shared chrome for the ✕ Delete and the Save-As-Object button, so they read as size-matched peers.
pub(super) fn paint_icon_button(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    r: Rect,
    icon: ph2d_editor_core::IconId,
    id: ph2d_a11y::NodeId,
) {
    let state = match ctx.host.store().get(id) {
        Some(InteractiveState::Button { state }) => *state,
        _ => ButtonState::Normal,
    };
    fill_rounded_rect(
        ctx.scene,
        r,
        Radius::Sm.px(),
        resolve(flat_button_surface(state), theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        r,
        Radius::Sm.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    let pad = Spacing::Sm.px(); // inset so the glyph reads at the button's full height
    let icon_r = Rect::new(r.x + pad, r.y + pad, r.w - pad * 2.0, r.h - pad * 2.0);
    paint_icon(
        ctx.scene,
        icon,
        icon_r,
        resolve(ColorToken::Text1, theme),
        StrokeToken::Default.px(),
    );
    register_button(ctx.host.store_mut(), id);
    ctx.host.hit_index_mut().register(id, r);
}
