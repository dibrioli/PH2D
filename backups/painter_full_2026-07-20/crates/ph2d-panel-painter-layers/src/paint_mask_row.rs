//! The layer-panel MASK sub-row (§2.7): an indented, selectable "Mask" row with the grayscale-view eye,
//! the Invert toggle, and the Apply (destructive bake) button. Split from [`crate::paint_rows`] for the
//! panel file-LOC cap. Selecting the row routes through the normal Row path → the tool makes the mask the
//! edit target (paint it with any tool / colour). The eye toggles the canvas between the masked EFFECT
//! (closed, default) and the mask's GRAYSCALE channel (open).

use crate::paint::register_button;
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
use ph2d_editor_core::paint::{paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Button, ButtonKind, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::LayerId;

const MASK_INV_W: f32 = 34.0; // LITERAL-PX-OK: mask-row "Inv" toggle button width
const MASK_APPLY_W: f32 = 48.0; // LITERAL-PX-OK: mask-row "Apply" button width
const MASK_EYE_W: f32 = 22.0; // LITERAL-PX-OK: mask-row grayscale-view eye button width

/// Paint the mask sub-row; returns the next `y`. See the module header.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_mask_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    mask_id: LayerId,
    inverted: bool,
    is_active: bool,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let font = TypeToken::Base.px();
    let cell_gap = Spacing::Xs.px();
    if is_active {
        stroke_rounded_rect(
            ctx.scene,
            Rect::new(x, y, w, ROW_H_PX),
            Radius::Sm.px(),
            StrokeToken::Default.px(),
            resolve(ColorToken::Accent, theme),
        );
    }

    // Right-aligned affordances: [ eye ] [ Inv ] [ Apply ]. The label runs up to them.
    let apply_rect = Rect::new(x + w - MASK_APPLY_W, y, MASK_APPLY_W, ROW_H_PX);
    let inv_rect = Rect::new(
        apply_rect.x - cell_gap - MASK_INV_W,
        y,
        MASK_INV_W,
        ROW_H_PX,
    );
    let eye_rect = Rect::new(inv_rect.x - cell_gap - MASK_EYE_W, y, MASK_EYE_W, ROW_H_PX);
    let view_open = crate::state::current_mask_grayscale_view() == Some(mask_id.0);

    let label = if inverted { "Mask · Inverted" } else { "Mask" };
    let label_x = x + Spacing::Sm.px();
    let label_y = y + (ROW_H_PX - font) * 0.5;
    let label_w = (eye_rect.x - cell_gap - label_x).max(0.0);
    paint_text(
        ctx.text_system,
        ctx.scene,
        label,
        label_x,
        label_y,
        font,
        label_w,
        resolve(ColorToken::Text2, theme),
    );

    // Row-select hit rect FIRST (full row); the buttons register AFTER and win their cells (last-wins).
    let row_id = painter_layer_widget_id(mask_id.0, PainterLayerWidget::Row);
    register_button(ctx.host.store_mut(), row_id);
    ctx.host
        .hit_index_mut()
        .register(row_id, Rect::new(x, y, w, ROW_H_PX));

    // Grayscale-view eye — Eye (open) = show the mask's grayscale, EyeClosed (default) = show the effect.
    let eye_id = painter_layer_widget_id(mask_id.0, PainterLayerWidget::MaskView);
    register_button(ctx.host.store_mut(), eye_id);
    let eye_st = ctx
        .host
        .store()
        .button_state(eye_id)
        .unwrap_or(ButtonState::Normal);
    let eye_icon = if view_open {
        IconId::Eye
    } else {
        IconId::EyeClosed
    };
    let eye_btn = Button::new(eye_id, "Mask view")
        .icon_only(eye_icon)
        .state(eye_st);
    paint_button(&eye_btn, eye_rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(eye_id, eye_rect);

    // Invert toggle — accent-filled when on (mirror of the modifier toolbar).
    let inv_id = painter_layer_widget_id(mask_id.0, PainterLayerWidget::MaskInvert);
    register_button(ctx.host.store_mut(), inv_id);
    let inv_st = ctx
        .host
        .store()
        .button_state(inv_id)
        .unwrap_or(ButtonState::Normal);
    let mut inv_btn = Button::new(inv_id, "Inv").state(inv_st);
    if inverted {
        inv_btn.kind = ButtonKind::Accent;
    }
    paint_button(&inv_btn, inv_rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(inv_id, inv_rect);

    // Apply — destructive bake into the parent alpha, then remove the mask.
    let apply_id = painter_layer_widget_id(mask_id.0, PainterLayerWidget::MaskApply);
    register_button(ctx.host.store_mut(), apply_id);
    let apply_st = ctx
        .host
        .store()
        .button_state(apply_id)
        .unwrap_or(ButtonState::Normal);
    let apply_btn = Button::new(apply_id, "Apply").state(apply_st);
    paint_button(&apply_btn, apply_rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(apply_id, apply_rect);

    y + ROW_H_PX + Spacing::Xs.px()
}
