//! Painter layers panel — adjustment-layer controls (W4 T4.3+).
//!
//! Split out of `paint_rows.rs` (panel-LOC cap): renders the per-kind edit
//! controls for a `LayerKind::Adjustment` row. Slider-driven kinds come from
//! `ph2d_tool_painter::adjustment_slider_params` (the single per-kind source of
//! truth, next to the params) — HSB = Hue/Sat/Bright, Brightness/Contrast =
//! Bright/Contrast, Levels = Black/Gamma/White/Out-Lo/Out-Hi, etc. Adding a
//! slider-based kind needs ZERO panel change.
//!
//! **Bespoke kinds (W4 §3):** Curves gets a dedicated editor
//! ([`paint_curve_editor`]) — RGB/R/G/B channel tabs + add/remove-point buttons
//! over a canvas with FREE 2-D draggable control points. Each handle registers an
//! `InteractiveState::CurvePoint` (carrying the plotting canvas) + a small grab
//! rect; the foundational dispatch (`interaction/dispatch/curve.rs`) normalizes
//! the pointer within the canvas to `(x, y)` and stashes it on the store, which
//! the panel drains on `ValueChanged(editor_id)` and forwards to
//! `PainterTool::set_curve_point` (see `event.rs`). Tabs are panel-local view
//! state ([`crate::state::active_curve_channel`]); +/− forward add/remove on the
//! active channel.

use crate::paint::register_button;
use crate::state;
use ph2d_editor_core::ids::{
    PainterLayerWidget, painter_curve_add_id, painter_curve_editor_id, painter_curve_point_id,
    painter_curve_remove_id, painter_curve_tab_id, painter_layer_widget_id,
};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text, paint_text_centered, resolve, stroke_polyline,
    stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Slider, SliderOrientation, SliderState, paint_slider};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, TypeToken};
use ph2d_tool_painter::{AdjustmentParams, CurvesParams};

const ADJ_LABEL_W: f32 = 44.0; // LITERAL-PX-OK: slider-param label column ("Contrast")
const CURVE_CANVAS_H: f32 = 132.0; // LITERAL-PX-OK: bespoke curve-editor canvas height
const CURVE_HANDLE_R: f32 = 4.0; // LITERAL-PX-OK: control-point handle radius
const CURVE_GRAB_R: f32 = 9.0; // LITERAL-PX-OK: half-size of a handle's pointer grab box
const CURVE_STROKE_W: f32 = 1.5; // LITERAL-PX-OK: plotted-curve stroke width
const CURVE_GRID_W: f32 = 1.0; // LITERAL-PX-OK: grid / identity-diagonal stroke width
const CURVE_BTN_W: f32 = 22.0; // LITERAL-PX-OK: +/− point-button width
const MAX_CURVE_POINTS: usize = 8; // contract cap (≤8 control points per channel)

/// Channel tab labels (index = channel: 0 = master/RGB, 1 = R, 2 = G, 3 = B).
const CURVE_TAB_LABELS: [&str; 4] = ["RGB", "R", "G", "B"];

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

/// Bespoke Curves editor: a row of channel tabs (RGB/R/G/B) + add/remove-point
/// buttons, over a canvas plotting the ACTIVE channel's tone curve with its FREE
/// 2-D draggable control points. Each handle registers an
/// `InteractiveState::CurvePoint` (carrying the `canvas` rect, so the dispatch
/// normalizes the drag against the full plotting area, not the small grab box) +
/// a grab rect in the `HitIndex`. The drag result is drained in `event.rs` and
/// forwarded to `PainterTool::set_curve_point`.
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
    let font = TypeToken::Base.px();
    let channel = state::active_curve_channel(layer_id);

    // ── Tab + button row: [RGB][R][G][B] … [+][−] ──
    let buttons_w = CURVE_BTN_W * 2.0 + gap;
    let tabs_w = (w - buttons_w - gap).max(0.0);
    let tab_w = tabs_w / 4.0;
    for ch in 0u8..4 {
        let trect = Rect::new(x + ch as f32 * tab_w, y, (tab_w - 2.0).max(0.0), ROW_H_PX);
        let active = ch == channel;
        let (bg, fg) = if active {
            (ColorToken::AccentSoft, ColorToken::Text1)
        } else {
            (ColorToken::Bg2, ColorToken::Text2)
        };
        fill_rounded_rect(ctx.scene, trect, Radius::Sm.px(), resolve(bg, theme));
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            CURVE_TAB_LABELS[ch as usize],
            trect,
            font,
            resolve(fg, theme),
        );
        let id = painter_curve_tab_id(layer_id, ch);
        register_button(ctx.host.store_mut(), id);
        ctx.host.hit_index_mut().register(id, trect);
    }
    let add_rect = Rect::new(x + tabs_w + gap, y, CURVE_BTN_W, ROW_H_PX);
    let rem_rect = Rect::new(add_rect.x + CURVE_BTN_W + gap, y, CURVE_BTN_W, ROW_H_PX);
    for (brect, label, id) in [
        (add_rect, "+", painter_curve_add_id(layer_id)),
        (rem_rect, "−", painter_curve_remove_id(layer_id)),
    ] {
        fill_rounded_rect(
            ctx.scene,
            brect,
            Radius::Sm.px(),
            resolve(ColorToken::Bg2, theme),
        );
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            label,
            brect,
            font,
            resolve(ColorToken::Text1, theme),
        );
        register_button(ctx.host.store_mut(), id);
        ctx.host.hit_index_mut().register(id, brect);
    }
    y += ROW_H_PX + gap;

    // ── Canvas: the active channel's curve + draggable points ──
    let canvas = Rect::new(x, y, w.max(1.0), CURVE_CANVAS_H);
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
    // Quarter grid + the identity diagonal (so a curve's deviation from no-op is
    // readable) — the canonical curve-editor backdrop, behind the curve.
    let grid = resolve(ColorToken::GridLine, theme);
    for i in 1..4 {
        let f = i as f32 / 4.0;
        let gx = canvas.x + f * canvas.w;
        let gy = canvas.y + f * canvas.h;
        stroke_polyline(
            ctx.scene,
            &[(gx, canvas.y), (gx, canvas.y + canvas.h)],
            CURVE_GRID_W,
            grid,
        );
        stroke_polyline(
            ctx.scene,
            &[(canvas.x, gy), (canvas.x + canvas.w, gy)],
            CURVE_GRID_W,
            grid,
        );
    }
    stroke_polyline(
        ctx.scene,
        &[
            (canvas.x, canvas.y + canvas.h), // bottom-left (in 0 → out 0)
            (canvas.x + canvas.w, canvas.y), // top-right (in 1 → out 1)
        ],
        CURVE_GRID_W,
        resolve(ColorToken::GridAxis, theme),
    );
    let pts = match channel {
        1 => &c.points_r,
        2 => &c.points_g,
        3 => &c.points_b,
        _ => &c.points_rgb,
    };
    // Tint the curve + handle rings by the active channel (master = accent), so
    // it's obvious which channel you're editing.
    let curve_color = resolve(
        match channel {
            1 => ColorToken::CurveR,
            2 => ColorToken::CurveG,
            3 => ColorToken::CurveB,
            _ => ColorToken::Accent,
        },
        theme,
    );
    // Plot the channel's own tone curve (the monotone spline the GPU also bakes)
    // as a smooth polyline through its sampled output.
    let samples = (canvas.w * 0.5).clamp(48.0, 256.0) as usize;
    let mut poly: Vec<(f32, f32)> = Vec::with_capacity(samples + 1);
    for k in 0..=samples {
        let t = k as f32 / samples as f32; // display x 0..1
        let yv = ph2d_tool_painter::curve_value_at(&pts.points, t);
        poly.push((canvas.x + t * canvas.w, canvas.y + (1.0 - yv) * canvas.h));
    }
    stroke_polyline(ctx.scene, &poly, CURVE_STROKE_W, curve_color);

    let parent = painter_curve_editor_id(layer_id);
    let ring = curve_color;
    let fill = resolve(ColorToken::Text1, theme);
    for (index, pt) in pts.points.iter().enumerate().take(MAX_CURVE_POINTS) {
        let id = painter_curve_point_id(layer_id, channel, index as u8);
        let cx = canvas.x + pt[0].clamp(0.0, 1.0) * canvas.w;
        let cy = canvas.y + (1.0 - pt[1].clamp(0.0, 1.0)) * canvas.h;
        // Overwrite each frame so the carried `canvas` (and active `channel`)
        // track panel resizes / tab switches (CurvePoint has no per-frame drag
        // state — the result lands in the store's `curve_point_drag` slot).
        ctx.host.store_mut().register(
            id,
            InteractiveState::CurvePoint {
                parent,
                channel,
                index: index as u8,
                canvas,
            },
        );
        let grab = Rect::new(
            cx - CURVE_GRAB_R,
            cy - CURVE_GRAB_R,
            CURVE_GRAB_R * 2.0,
            CURVE_GRAB_R * 2.0,
        );
        ctx.host.hit_index_mut().register(id, grab);
        // Ring + fill (no stroke-circle helper — a slightly larger ring circle
        // under the fill gives a 1.5px outline).
        fill_circle(ctx.scene, cx, cy, CURVE_HANDLE_R + 1.5, ring);
        fill_circle(ctx.scene, cx, cy, CURVE_HANDLE_R, fill);
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
