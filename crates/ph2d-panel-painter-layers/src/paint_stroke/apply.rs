//! The **Apply / Apply & Keep / Delete** button row under the Stroke-Method dropdown — shown for the
//! methods with a persistent on-canvas shape editor (Curve / Free Hand / Circle / Polygon). Split from
//! `paint_stroke` for the workspace LOC cap. Apply bakes + drops the editor, Apply & Keep bakes + keeps
//! it, the trash Deletes without baking; each forwards a `Click` (the tool routes it in
//! `route_brush_dab_event`). Registering the Button slot is what lets the dispatch emit the `Click`.

use crate::paint::register_button;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{ButtonState, flat_button_surface};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};

/// Paint the Apply / Apply & Keep + trailing square-icon cluster (optional **E**dit, then **✕** Delete)
/// row, returning the next `y`. `with_edit` adds the E button (Circle/Polygon → editable curve), left of
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
    // Width of the trailing cluster: ✕ alone, or E + ✕ (with a gap) when convertible.
    let icons = if with_edit { sq * 2.0 + gap } else { sq };
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
        paint_icon_cluster(ctx, theme, keep_x + w + gap, y, sq, gap, with_edit);
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
        paint_icon_cluster(ctx, theme, x + text_w + gap, y, sq, gap, with_edit);
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

/// Paint the trailing square-icon cluster at `ix`: the optional **E**dit button then the **✕** Delete.
fn paint_icon_cluster(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    ix: f32,
    iy: f32,
    sq: f32,
    gap: f32,
    with_edit: bool,
) {
    let mut cx = ix;
    if with_edit {
        let edit = core_ids::PAINTER_BRUSH_STROKE_EDIT;
        button(ctx, theme, Rect::new(cx, iy, sq, ROW_H_PX), Some("E"), edit);
        cx += sq + gap;
    }
    let delete = core_ids::PAINTER_BRUSH_STROKE_DELETE;
    button(ctx, theme, Rect::new(cx, iy, sq, ROW_H_PX), None, delete);
}

/// One stroke shape-editor button. `label = Some(text)` paints a text button (Apply / Apply & Keep);
/// `None` paints the square Close (✕) icon (Delete). All three share the same chrome (fill / border /
/// height) so Delete reads as a peer, not an alarm. Registers the Button slot so the dispatch can `Click`.
fn button(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    r: Rect,
    label: Option<&str>,
    id: ph2d_a11y::NodeId,
) {
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
    match label {
        Some(text) => paint_text_centered(
            ctx.text_system,
            ctx.scene,
            text,
            r,
            TypeToken::Base.px(),
            resolve(ColorToken::Text1, theme),
        ),
        None => {
            let pad = Spacing::Sm.px(); // inset so the ✕ reads at the button's full height
            let icon = Rect::new(r.x + pad, r.y + pad, r.w - pad * 2.0, r.h - pad * 2.0);
            paint_icon(
                ctx.scene,
                ph2d_editor_core::IconId::Close,
                icon,
                resolve(ColorToken::Text1, theme),
                StrokeToken::Default.px(),
            );
        }
    }
    register_button(ctx.host.store_mut(), id);
    ctx.host.hit_index_mut().register(id, r);
}
