//! The **Apply / Apply & Keep / Delete** button row under the Stroke-Method dropdown — shown for the
//! methods with a persistent on-canvas shape editor (Curve / Free Hand / Circle / Polygon). Split from
//! `paint_stroke` for the workspace LOC cap. Apply bakes + drops the editor, Apply & Keep bakes + keeps
//! it, the trash Deletes without baking; each forwards a `Click` (the tool routes it in
//! `route_brush_dab_event`). Registering the Button slot is what lets the dispatch emit the `Click`.

use crate::paint::register_button;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, flat_button_surface, paint_icon, paint_text_centered, resolve,
    stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};

/// Paint the Apply / Apply & Keep / Delete row, returning the next `y`. The two text buttons plus the
/// square trash share one row; when the panel is too narrow for two readable text buttons, "Apply & Keep"
/// wraps to its own row below — "Apply" keeps the trash beside it so cancel is always reachable.
pub(super) fn paint_apply_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
) -> f32 {
    let gap = Spacing::Xs.px();
    let del = ROW_H_PX; // the trash button is a square icon
    let apply = core_ids::PAINTER_BRUSH_STROKE_APPLY;
    let keep = core_ids::PAINTER_BRUSH_STROKE_APPLY_KEEP;
    let delete = core_ids::PAINTER_BRUSH_STROKE_DELETE;
    // One row when two readable text buttons + the square trash fit; else "Apply & Keep" wraps below and
    // the trash stays beside "Apply" so cancel is always reachable. `84` = min readable text-button width.
    if content_w >= 84.0 * 2.0 + del + gap * 2.0 {
        let w = (content_w - del - gap * 2.0) * 0.5;
        button(
            ctx,
            theme,
            Rect::new(x, y, w, ROW_H_PX),
            Some("Apply"),
            apply,
        );
        let kr = Rect::new(x + w + gap, y, w, ROW_H_PX);
        button(ctx, theme, kr, Some("Apply & Keep"), keep);
        button(
            ctx,
            theme,
            Rect::new(x + (w + gap) * 2.0, y, del, ROW_H_PX),
            None,
            delete,
        );
        y + ROW_H_PX + Spacing::Xs.px()
    } else {
        let aw = content_w - del - gap;
        button(
            ctx,
            theme,
            Rect::new(x, y, aw, ROW_H_PX),
            Some("Apply"),
            apply,
        );
        button(
            ctx,
            theme,
            Rect::new(x + aw + gap, y, del, ROW_H_PX),
            None,
            delete,
        );
        let y2 = y + ROW_H_PX + gap;
        button(
            ctx,
            theme,
            Rect::new(x, y2, content_w, ROW_H_PX),
            Some("Apply & Keep"),
            keep,
        );
        y2 + ROW_H_PX + Spacing::Xs.px()
    }
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
    fill_rounded_rect(ctx.scene, r, Radius::Sm.px(), resolve(flat_button_surface(state), theme));
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
