//! The Shape **Per-Layer Color** UI (multi-layer Shape): a mode toggle below the Texture dropdown, plus
//! — when on — one "Layer N Color" checkbox + colour swatch per captured layer. The swatch appears only
//! when that layer's checkbox is on; its picked colour replaces the brush base colour for that layer.
//! Shown only for a multi-layer (`> 1`) Shape; while on, the caller hides the Shape Color ramp section.
//! Split from `paint_shape` for the workspace LOC cap.

use crate::paint::register_button;
use crate::paint_brush_top::paint_checkbox_row;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::{
    Checkbox, CheckboxValue, ColorSwatch, SwatchSize, SwatchState, paint_checkbox,
    paint_color_swatch,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_painter::BrushSettings;

const SWATCH_W: f32 = 44.0; // LITERAL-PX-OK: per-layer colour swatch width

/// sRGB 8-bit normalize for the picker round-trip.
fn enc(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8 // LITERAL-PX-OK: sRGB 8-bit normalize
}

/// Paint the **Per-Layer Color** toggle + (when on) the per-layer "Layer N Color" checkbox + swatch
/// rows. A no-op for a single-layer / no Shape. Returns the next `y`.
pub(crate) fn paint_shape_per_layer_color(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    mut y: f32,
    brush: BrushSettings,
) -> f32 {
    let count = brush.shape_layer_count as usize;
    if count <= 1 {
        return y; // single-layer image (or none) ⇒ no per-layer-colour UI
    }
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_SHAPE_PER_LAYER_COLOR,
        "Per-Layer Color",
        brush.shape_per_layer_color,
    );
    if !brush.shape_per_layer_color {
        return y;
    }
    let n = count.min(brush.shape_layer_color_on.len());
    for i in 0..n {
        let iu = i as u8;
        let check_id = core_ids::painter_shape_layer_color_check_id(iu);
        let on = brush.shape_layer_color_on[i];
        // Checkbox on the left; the colour swatch sits in front of it (right) only when checked.
        let cb_w = if on {
            (content_w - SWATCH_W - Spacing::Sm.px()).max(0.0)
        } else {
            content_w
        };
        let cb = Checkbox::new(check_id, format!("Layer {} Color", i + 1)).value(if on {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        });
        let cb_rect = Rect::new(x, y, cb_w, ROW_H_PX);
        paint_checkbox(&cb, cb_rect, ctx.scene, ctx.text_system, theme);
        register_button(ctx.host.store_mut(), check_id);
        ctx.host.hit_index_mut().register(check_id, cb_rect);
        if on {
            let sw_id = core_ids::painter_shape_layer_color_swatch_id(iu);
            let sw_rect = Rect::new(x + content_w - SWATCH_W, y, SWATCH_W, ROW_H_PX);
            let c = brush.shape_layer_color[i];
            let open = ctx.host.store().picker_target() == Some(sw_id);
            paint_color_swatch(
                &ColorSwatch {
                    id: sw_id,
                    label: String::new(),
                    rgba: [enc(c[0]), enc(c[1]), enc(c[2]), 255],
                    state: if open {
                        SwatchState::Focused
                    } else {
                        SwatchState::Normal
                    },
                    size: SwatchSize::Sm,
                },
                sw_rect,
                ctx.scene,
                theme,
            );
            register_button(ctx.host.store_mut(), sw_id);
            ctx.host.hit_index_mut().register(sw_id, sw_rect);
            layer_color_readback(ctx, sw_id, iu, c);
        }
        y += ROW_H_PX + Spacing::Sm.px();
    }
    y
}

/// When the picker targets layer `i`'s swatch, forward its live colour as `"i,r,g,b"` (u8) once it
/// differs — the tool decodes it to `set_brush_shape_layer_color(i, rgb)`.
fn layer_color_readback(ctx: &mut PaintCtx, swatch_id: NodeId, i: u8, cur: [f32; 3]) {
    if ctx.host.store().picker_target() != Some(swatch_id) {
        return;
    }
    let Some(picked) = ctx.host.store().widget_color(swatch_id) else {
        return;
    };
    if [enc(cur[0]), enc(cur[1]), enc(cur[2])] == [picked[0], picked[1], picked[2]] {
        return;
    }
    ctx.host
        .bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            swatch_id,
            format!("{i},{},{},{}", picked[0], picked[1], picked[2]),
        )));
}
