//! Painter layers panel — adjustment-layer controls (W4 T4.3+).
//!
//! Split out of `paint_rows.rs` (panel-LOC cap): renders the per-kind edit
//! controls for a `LayerKind::Adjustment` row. Slider-driven kinds come from
//! `ph2d_tool_painter::adjustment_slider_params` (the single per-kind source of
//! truth, next to the params) — HSB = Hue/Sat/Bright, Brightness/Contrast =
//! Bright/Contrast, Levels = Black/Gamma/White/Out-Lo/Out-Hi, etc. Adding a
//! slider-based kind needs ZERO panel change.
//!
//! **Bespoke kinds (W4):** Curves does not fit the generic ≤6-slider rack, so it
//! gets a dedicated curve canvas here ([`paint_curve_editor`]). To stay inside
//! the existing interaction machinery (no new `InteractiveState`/dispatch), the
//! editor REUSES the generic `AdjParam` slider widgets as its fixed-x master
//! handles — one *vertical* slider per control point, whose value is the point's
//! Y. A free-2D point drag (arbitrary X) is the foundational
//! `InteractiveState::CurvePoint` + dispatch upgrade owned by the Coordinator
//! (see `docs/HANDOFF_painter_w4_bespoke_kinds_coord.md`).

use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Slider, SliderOrientation, SliderState, paint_slider};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, TypeToken};
use ph2d_tool_painter::{AdjustmentParams, CurvesParams};

const ADJ_LABEL_W: f32 = 44.0; // LITERAL-PX-OK: slider-param label column ("Contrast")
const CURVE_CANVAS_H: f32 = 132.0; // LITERAL-PX-OK: bespoke curve-editor canvas height
const CURVE_HANDLE_R: f32 = 4.0; // LITERAL-PX-OK: control-point handle radius
const CURVE_DOT_R: f32 = 1.25; // LITERAL-PX-OK: curve sample-dot radius (the plotted spline)

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

/// Render an adjustment layer's edit controls, indented below its main row. For
/// slider kinds (the common case) this is one labeled slider per slot; for
/// bespoke kinds (Curves) it dispatches to the dedicated editor. Each slider
/// STORES `0..1`; the tool maps it back per kind (`set_adjustment_param`), so the
/// displayed position is derived from the live params each frame. Returns the
/// next `y`.
pub(crate) fn paint_adjustment_params(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    layer_id: u64,
    params: &AdjustmentParams,
    x: f32,
    w: f32,
    mut y: f32,
) -> f32 {
    // Bespoke: Curves gets a curve canvas instead of the generic slider rack.
    if let AdjustmentParams::Curves(c) = params {
        return paint_curve_editor(ctx, theme, layer_id, c, x, w, y);
    }
    let font = TypeToken::Base.px();
    let gap = Spacing::Xs.px();
    for (slot, (label, val01)) in ph2d_tool_painter::adjustment_slider_params(params)
        .into_iter()
        .enumerate()
    {
        let Some(kind) = slot_kind(slot) else { break };
        let id = painter_layer_widget_id(layer_id, kind);
        register_slider(
            ctx.host.store_mut(),
            id,
            val01,
            SliderOrientation::Horizontal,
        );
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

/// Bespoke Curves editor: a square canvas plotting the live master tone curve
/// plus its draggable control points. The control points reuse the generic
/// `AdjParam` slider widgets as *vertical* sliders (one per point) over a thin
/// drag strip at the point's fixed X — so a drag updates the point's Y through
/// the same `SetValue → set_adjustment_param` path as every other adjustment,
/// with no new interaction primitive. v1 edits the master (RGB) curve at fixed X;
/// per-channel R/G/B and free-X point drags are the Coordinator upgrade.
fn paint_curve_editor(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    layer_id: u64,
    c: &CurvesParams,
    x: f32,
    w: f32,
    mut y: f32,
) -> f32 {
    let gap = Spacing::Xs.px();
    let canvas = Rect::new(x, y, w.max(1.0), CURVE_CANVAS_H);
    // Backdrop + border.
    fill_rounded_rect(
        ctx.scene,
        canvas,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        canvas,
        Radius::Sm.px(),
        1.0, // LITERAL-PX-OK: 1px hairline border
        resolve(ColorToken::TextDisabled, theme),
    );
    // Plot the master tone curve by sampling its display-space LUT (the exact
    // table the GPU will bind once Curves goes real-time). Dense small dots read
    // as a continuous stroke without a polyline primitive in the panel toolkit.
    let luts = ph2d_tool_painter::curves_display_luts(c);
    let master = &luts[0];
    let curve_color = resolve(ColorToken::Accent, theme);
    let samples = (canvas.w * 0.5).clamp(48.0, 200.0) as usize;
    for k in 0..=samples {
        let t = k as f32 / samples as f32; // display x 0..1
        let idx = (t * (master.len() - 1) as f32).round() as usize;
        let yv = master[idx]; // display y 0..1
        let px = canvas.x + t * canvas.w;
        let py = canvas.y + (1.0 - yv) * canvas.h;
        fill_rounded_rect(
            ctx.scene,
            Rect::new(
                px - CURVE_DOT_R,
                py - CURVE_DOT_R,
                CURVE_DOT_R * 2.0,
                CURVE_DOT_R * 2.0,
            ),
            CURVE_DOT_R,
            curve_color,
        );
    }
    // Control-point handles: one vertical slider per master point. Strips are
    // narrower than the inter-point spacing so they never overlap.
    let n = c.points_rgb.points.len().max(1) as f32;
    let strip_w = (canvas.w / n * 0.6).clamp(10.0, 22.0);
    let handle_color = resolve(ColorToken::Text1, theme);
    let handle_ring = resolve(ColorToken::Accent, theme);
    for (slot, pt) in c.points_rgb.points.iter().enumerate() {
        let Some(kind) = slot_kind(slot) else { break };
        let id = painter_layer_widget_id(layer_id, kind);
        let px = canvas.x + pt[0].clamp(0.0, 1.0) * canvas.w;
        // Vertical drag strip spanning the full canvas height (the dispatch maps
        // pointer Y → `1 - (py - rect.y) / rect.h`, i.e. the point's Y in 0..1).
        let strip_x = (px - strip_w * 0.5).clamp(canvas.x, canvas.x + canvas.w - strip_w);
        let strip = Rect::new(strip_x, canvas.y, strip_w, canvas.h);
        register_slider(
            ctx.host.store_mut(),
            id,
            pt[1].clamp(0.0, 1.0),
            SliderOrientation::Vertical,
        );
        ctx.host.hit_index_mut().register(id, strip);
        // Handle dot at the point's (x, y).
        let py = canvas.y + (1.0 - pt[1].clamp(0.0, 1.0)) * canvas.h;
        let dot = Rect::new(
            px - CURVE_HANDLE_R,
            py - CURVE_HANDLE_R,
            CURVE_HANDLE_R * 2.0,
            CURVE_HANDLE_R * 2.0,
        );
        fill_rounded_rect(ctx.scene, dot, CURVE_HANDLE_R, handle_color);
        stroke_rounded_rect(ctx.scene, dot, CURVE_HANDLE_R, 1.5, handle_ring); // LITERAL-PX-OK: handle ring
    }
    y += CURVE_CANVAS_H + gap;
    y
}

/// `register_if_absent` a per-row adjustment slider (bare, `0..1` storage) with
/// the given orientation. The dispatch maps a drag to a fresh value; `event.rs`
/// forwards the resulting `ValueChanged` as `SetValue` to the tool. Mirror of
/// `paint_rows`'s opacity slider registration (chip-less — no per-row Vello clip).
fn register_slider(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    value: f32,
    orientation: SliderOrientation,
) {
    store.register_if_absent(
        id,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value,
            orientation,
        },
    );
}
