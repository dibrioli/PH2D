//! Painter layers panel — adjustment-layer controls (W4 T4.3+).
//!
//! Split out of `paint_rows.rs` (panel-LOC cap): renders the per-kind edit
//! controls for a `LayerKind::Adjustment` row. The slider params come from
//! `ph2d_tool_painter::adjustment_slider_params` (the single per-kind source of
//! truth, next to the params) — HSB = Hue/Sat/Bright, Brightness/Contrast =
//! Bright/Contrast, etc. Adding a slider-based kind needs ZERO panel change.
//! Kinds with bespoke controls (Curves, Gradient Map, …) return no sliders and
//! get their own UI here later.

use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Slider, SliderOrientation, SliderState, paint_slider};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};
use ph2d_tool_painter::AdjustmentParams;

const ADJ_LABEL_W: f32 = 44.0; // LITERAL-PX-OK: slider-param label column ("Contrast")

/// The generic per-slot slider widget kind (≤6 slider params per adjustment).
fn slot_kind(slot: usize) -> Option<PainterLayerWidget> {
    Some(match slot {
        0 => PainterLayerWidget::AdjParam0,
        1 => PainterLayerWidget::AdjParam1,
        2 => PainterLayerWidget::AdjParam2,
        3 => PainterLayerWidget::AdjParam3,
        4 => PainterLayerWidget::AdjParam4,
        5 => PainterLayerWidget::AdjParam5,
        _ => return None,
    })
}

/// Render an adjustment layer's slider params (label + slider per slot),
/// indented below its main row. Each slider STORES `0..1`; the tool maps it back
/// per kind (`set_adjustment_param`), so the displayed position is derived from
/// the live params each frame. Returns the next `y`.
pub(crate) fn paint_adjustment_params(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    layer_id: u64,
    params: &AdjustmentParams,
    x: f32,
    w: f32,
    mut y: f32,
) -> f32 {
    let font = TypeToken::Base.px();
    let gap = Spacing::Xs.px();
    for (slot, (label, val01)) in ph2d_tool_painter::adjustment_slider_params(params)
        .into_iter()
        .enumerate()
    {
        let Some(kind) = slot_kind(slot) else { break };
        let id = painter_layer_widget_id(layer_id, kind);
        register_slider(ctx.host.store_mut(), id, val01);
        paint_text(
            ctx.text_system,
            ctx.scene,
            label,
            x,
            y + (ROW_H_PX - font) * 0.5,
            font,
            ADJ_LABEL_W,
            resolve(ColorToken::Text2, theme),
        );
        let slider_x = x + ADJ_LABEL_W + gap;
        let slider_w = (w - ADJ_LABEL_W - gap).max(0.0);
        let st = ctx
            .host
            .store()
            .slider(id)
            .map(|(s, _)| s)
            .unwrap_or(SliderState::Normal);
        let mut slider = Slider::new(id, "").accent(true).state(st);
        slider.value = val01.clamp(0.0, 1.0);
        let rect = Rect::new(slider_x, y, slider_w, ROW_H_PX);
        paint_slider(&slider, rect, ctx.scene, theme);
        ctx.host.hit_index_mut().register(id, rect);
        y += ROW_H_PX + gap;
    }
    y
}

/// `register_if_absent` a per-row adjustment slider (bare, `0..1` storage). The
/// dispatch maps a drag to a fresh value; `event.rs` forwards the resulting
/// `ValueChanged` as `SetValue` to the tool. Mirror of `paint_rows`'s opacity
/// slider registration (chip-less — no per-row Vello clip).
fn register_slider(store: &mut WidgetStore, id: ph2d_a11y::NodeId, value: f32) {
    store.register_if_absent(
        id,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value,
            orientation: SliderOrientation::Horizontal,
        },
    );
}
