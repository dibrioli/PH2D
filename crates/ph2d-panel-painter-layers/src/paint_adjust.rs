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
    painter_curve_remove_id, painter_curve_tab_id, painter_gradient_add_id,
    painter_gradient_editor_id, painter_gradient_remove_id, painter_gradient_stop_id,
    painter_layer_widget_id, painter_mixer_tab_id, painter_selcolor_bucket_id,
};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text, paint_text_centered, resolve, stroke_polyline,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Slider, SliderOrientation, SliderState, Toggle, paint_slider, paint_toggle,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, TypeToken};
use ph2d_tool_painter::{AdjustmentParams, CurvesParams};

mod curve;
mod gradient;
use curve::paint_curve_editor;
use gradient::paint_gradient_map;

const ADJ_LABEL_W: f32 = 44.0; // LITERAL-PX-OK: slider-param label column ("Contrast")
const ADJ_TOGGLE_W: f32 = 34.0; // LITERAL-PX-OK: toggle-rack switch pill width
const ADJ_TOGGLE_H: f32 = 18.0; // LITERAL-PX-OK: toggle-rack switch pill height (inset in ROW_H)
const CURVE_CANVAS_H: f32 = 132.0; // LITERAL-PX-OK: bespoke curve-editor canvas height
const CURVE_HANDLE_R: f32 = 4.0; // LITERAL-PX-OK: control-point handle radius
const CURVE_GRAB_R: f32 = 9.0; // LITERAL-PX-OK: half-size of a handle's pointer grab box
const CURVE_STROKE_W: f32 = 1.5; // LITERAL-PX-OK: plotted-curve stroke width
const CURVE_GRID_W: f32 = 1.0; // LITERAL-PX-OK: grid / identity-diagonal stroke width
const CURVE_BTN_W: f32 = 22.0; // LITERAL-PX-OK: +/− point-button width
const MAX_CURVE_POINTS: usize = 8; // contract cap (≤8 control points per channel)
const GRAD_BAR_H: f32 = 20.0; // LITERAL-PX-OK: Gradient Map preview-bar height
const GRAD_HANDLE_R: f32 = 5.0; // LITERAL-PX-OK: gradient stop-handle radius
const MAX_GRADIENT_STOPS: usize = 16; // editor cap (matches the brush add_gradient_stop)

/// Channel tab labels (index = channel: 0 = master/RGB, 1 = R, 2 = G, 3 = B).
const CURVE_TAB_LABELS: [&str; 4] = ["RGB", "R", "G", "B"];

/// The generic per-slot slider widget kind (≤8 slider params per adjustment).
fn slot_kind(slot: usize) -> Option<PainterLayerWidget> {
    Some(match slot {
        0 => PainterLayerWidget::AdjParam0,
        1 => PainterLayerWidget::AdjParam1,
        2 => PainterLayerWidget::AdjParam2,
        3 => PainterLayerWidget::AdjParam3,
        4 => PainterLayerWidget::AdjParam4,
        5 => PainterLayerWidget::AdjParam5,
        6 => PainterLayerWidget::AdjParam6,
        7 => PainterLayerWidget::AdjParam7,
        _ => return None,
    })
}

/// The generic per-slot toggle widget kind (≤2 toggle params per adjustment).
fn toggle_slot_kind(slot: usize) -> Option<PainterLayerWidget> {
    Some(match slot {
        0 => PainterLayerWidget::AdjToggle0,
        1 => PainterLayerWidget::AdjToggle1,
        _ => return None,
    })
}

/// The per-option widget kind of the single segmented param (≤3 options).
fn segment_slot_kind(option: usize) -> Option<PainterLayerWidget> {
    Some(match option {
        0 => PainterLayerWidget::AdjSegment0,
        1 => PainterLayerWidget::AdjSegment1,
        2 => PainterLayerWidget::AdjSegment2,
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
    // Bespoke: Channel Mixer gets output-channel tabs + 4 weight sliders + mono.
    if matches!(params, AdjustmentParams::ChannelMixer(_)) {
        return paint_channel_mixer(ctx, theme, layer_id, params, x, w, y);
    }
    // Bespoke: Black & White wants the Tint toggle BETWEEN the 6 hue sliders and
    // the (conditional) tint sliders, so it owns its ordering.
    if matches!(params, AdjustmentParams::BlackAndWhite(_)) {
        return paint_black_and_white(ctx, theme, layer_id, params, x, w, y);
    }
    // Bespoke: Selective Color gets a 9-bucket tab row + 4 CMYK sliders + method.
    if matches!(params, AdjustmentParams::SelectiveColor(_)) {
        return paint_selective_color(ctx, theme, layer_id, params, x, w, y);
    }
    // Bespoke: Gradient Map gets a preview bar + draggable stops + add/remove +
    // the selected stop's RGB sliders + the interpolation segment.
    if matches!(params, AdjustmentParams::GradientMap(_)) {
        return paint_gradient_map(ctx, theme, layer_id, params, x, w, y);
    }
    let gap = Spacing::Xs.px();
    for (slot, (label, val01)) in ph2d_tool_painter::adjustment_slider_params(params)
        .into_iter()
        .enumerate()
    {
        let Some(kind) = slot_kind(slot) else { break };
        let id = painter_layer_widget_id(layer_id, kind);
        paint_labeled_slider(ctx, theme, id, label, val01, Rect::new(x, y, w, ROW_H_PX));
        y += ROW_H_PX + gap;
    }
    // Segment rack (the adjustment's single 1-of-N param, e.g. Color Balance's
    // tonal range / Gradient Map's interpolation), between the sliders + toggles.
    y = paint_segment_rack(ctx, theme, layer_id, params, x, w, y);
    // Toggle rack (W4 BATCH-1): a label on the left + a right-aligned switch per
    // boolean param (Photo Filter's Preserve Luminosity, …). The switch is a
    // button (click) painted with the live param value — the params are the
    // single source of truth, the tool flips on click (mirror of the mask-invert
    // affordance). Adding a toggle-bearing kind needs ZERO panel change.
    for (slot, (label, on)) in ph2d_tool_painter::adjustment_toggle_params(params)
        .into_iter()
        .enumerate()
    {
        let Some(kind) = toggle_slot_kind(slot) else {
            break;
        };
        let id = painter_layer_widget_id(layer_id, kind);
        paint_toggle_row(ctx, theme, id, label, on, Rect::new(x, y, w, ROW_H_PX));
        y += ROW_H_PX + gap;
    }
    y
}

/// Render one labeled `0..1` slider row (label column + horizontal slider): the
/// shared body of the generic slider rack AND the bespoke Channel-Mixer weight
/// sliders. The slider STORES `0..1`; the value is derived from the live params
/// each frame, so the caller passes `val01`. Registers the hit rect; the caller
/// advances `y`.
fn paint_labeled_slider(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    label: &str,
    val01: f32,
    row: Rect,
) {
    let (x, w, y) = (row.x, row.w, row.y);
    let font = TypeToken::Base.px();
    let gap = Spacing::Xs.px();
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
    let mut slider = Slider::new(id, "")
        .accent(true)
        .visual(ctx.host.store().slider_visual(id));
    slider.value = val01.clamp(0.0, 1.0);
    let rect = Rect::new(slider_x, y, slider_w, ROW_H_PX);
    paint_slider(&slider, rect, ctx.scene, theme);
    ctx.host.hit_index_mut().register(id, rect);
}

/// Render one toggle row (left label + right-aligned switch painted with the live
/// param value): the shared body of the generic toggle rack AND the Channel-Mixer
/// monochrome switch. Registers the switch as a button (click → the tool flips the
/// param); the caller advances `y`.
fn paint_toggle_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    label: &str,
    on: bool,
    row: Rect,
) {
    let (x, w, y) = (row.x, row.w, row.y);
    let font = TypeToken::Base.px();
    let gap = Spacing::Xs.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        label,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        (w - ADJ_TOGGLE_W - gap).max(0.0),
        resolve(ColorToken::Text2, theme),
    );
    let toggle_rect = Rect::new(
        x + w - ADJ_TOGGLE_W,
        y + (ROW_H_PX - ADJ_TOGGLE_H) * 0.5,
        ADJ_TOGGLE_W,
        ADJ_TOGGLE_H,
    );
    let toggle = Toggle::new(id, "")
        .visual(ctx.host.store().toggle_visual(id))
        .on(on);
    paint_toggle(&toggle, toggle_rect, ctx.scene, theme);
    register_button(ctx.host.store_mut(), id);
    ctx.host.hit_index_mut().register(id, toggle_rect);
}

/// Bespoke Channel Mixer editor: a row of output-channel tabs (Red/Green/Blue, or
/// a single Gray tab when monochrome) + the 4 weight sliders (R/G/B source +
/// Constant) of the ACTIVE output row + the Monochrome switch. The active tab is
/// panel-local VIEW state ([`state::active_mixer_channel`]); a weight-slider drag
/// forwards `SelectOption(PAINTER_MIXER_EDIT, "layer:output:slot:value")` from
/// `event.rs` (the active output carries the channel the generic slider rack can
/// not). Returns the next `y`.
fn paint_channel_mixer(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    layer_id: u64,
    params: &AdjustmentParams,
    x: f32,
    w: f32,
    mut y: f32,
) -> f32 {
    let AdjustmentParams::ChannelMixer(m) = params else {
        return y;
    };
    let gap = Spacing::Xs.px();
    let font = TypeToken::Base.px();
    // Monochrome collapses the three output rows to a single Gray tab.
    let mono = ph2d_tool_painter::adjustment_toggle_params(params)
        .first()
        .map(|(_, on)| *on)
        .unwrap_or(false);
    let tabs: &[&str] = if mono {
        &["Gray"]
    } else {
        &["Red", "Green", "Blue"]
    };
    let active = if mono {
        0u8
    } else {
        state::active_mixer_channel(layer_id).min(2)
    };
    // ── Output-channel tab row ──
    let tab_w = w / tabs.len() as f32;
    for (ch, label) in tabs.iter().enumerate() {
        let trect = Rect::new(
            x + ch as f32 * tab_w,
            y,
            (tab_w - 2.0).max(0.0), // LITERAL-PX-OK: 2px inter-tab gutter
            ROW_H_PX,
        );
        let (bg, fg) = if ch as u8 == active {
            (ColorToken::AccentSoft, ColorToken::Text1)
        } else {
            (ColorToken::Bg2, ColorToken::Text2)
        };
        fill_rounded_rect(ctx.scene, trect, Radius::Sm.px(), resolve(bg, theme));
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            label,
            trect,
            font,
            resolve(fg, theme),
        );
        let id = painter_mixer_tab_id(layer_id, ch as u8);
        register_button(ctx.host.store_mut(), id);
        ctx.host.hit_index_mut().register(id, trect);
    }
    y += ROW_H_PX + gap;
    // ── The active output row's 4 weight sliders (R/G/B source + Constant) ──
    for (slot, (label, val01)) in ph2d_tool_painter::channel_mixer_slider_params(m, active as usize)
        .into_iter()
        .enumerate()
    {
        let Some(kind) = slot_kind(slot) else { break };
        let id = painter_layer_widget_id(layer_id, kind);
        paint_labeled_slider(ctx, theme, id, label, val01, Rect::new(x, y, w, ROW_H_PX));
        y += ROW_H_PX + gap;
    }
    // ── Monochrome switch (the generic toggle rack, rendered inline) ──
    for (slot, (label, on)) in ph2d_tool_painter::adjustment_toggle_params(params)
        .into_iter()
        .enumerate()
    {
        let Some(kind) = toggle_slot_kind(slot) else {
            break;
        };
        let id = painter_layer_widget_id(layer_id, kind);
        paint_toggle_row(ctx, theme, id, label, on, Rect::new(x, y, w, ROW_H_PX));
        y += ROW_H_PX + gap;
    }
    y
}

/// Bespoke Black & White editor: the 6 per-hue weight sliders, then the Tint
/// switch, then (only while a tint is set) the Tint Hue + amount sliders — the
/// switch sits BETWEEN its hue sliders and the tint sliders it gates (the generic
/// rack would render it last). All slots come from the generic
/// `adjustment_slider_params` (6, or 8 with a tint); this fn only controls the
/// ORDER (toggle inserted after slot 5). Returns the next `y`.
fn paint_black_and_white(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    layer_id: u64,
    params: &AdjustmentParams,
    x: f32,
    w: f32,
    mut y: f32,
) -> f32 {
    let gap = Spacing::Xs.px();
    let sliders = ph2d_tool_painter::adjustment_slider_params(params);
    // The 6 per-hue weight sliders (always present).
    for (slot, &(label, val01)) in sliders.iter().enumerate().take(6) {
        let Some(kind) = slot_kind(slot) else { break };
        let id = painter_layer_widget_id(layer_id, kind);
        paint_labeled_slider(ctx, theme, id, label, val01, Rect::new(x, y, w, ROW_H_PX));
        y += ROW_H_PX + gap;
    }
    // The Tint switch.
    for (slot, (label, on)) in ph2d_tool_painter::adjustment_toggle_params(params)
        .into_iter()
        .enumerate()
    {
        let Some(kind) = toggle_slot_kind(slot) else {
            break;
        };
        let id = painter_layer_widget_id(layer_id, kind);
        paint_toggle_row(ctx, theme, id, label, on, Rect::new(x, y, w, ROW_H_PX));
        y += ROW_H_PX + gap;
    }
    // The tint Hue + amount sliders (slots 6+), present only while a tint is set.
    for (slot, &(label, val01)) in sliders.iter().enumerate().skip(6) {
        let Some(kind) = slot_kind(slot) else { break };
        let id = painter_layer_widget_id(layer_id, kind);
        paint_labeled_slider(ctx, theme, id, label, val01, Rect::new(x, y, w, ROW_H_PX));
        y += ROW_H_PX + gap;
    }
    y
}

/// Render the adjustment's single segmented param (1-of-N, N ≤ 3) as a row of
/// segment buttons spanning `w` — the shared body of the generic rack AND the
/// Selective-Color method row. No-op (returns `y` unchanged) for a kind with no
/// segmented param; otherwise advances one row.
fn paint_segment_rack(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    layer_id: u64,
    params: &AdjustmentParams,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let Some((options, selected)) = ph2d_tool_painter::adjustment_segment_params(params) else {
        return y;
    };
    let font = TypeToken::Base.px();
    let gap = Spacing::Xs.px();
    let n = options.len().clamp(1, 3);
    let seg_w = w / n as f32;
    for (option, label) in options.iter().enumerate().take(3) {
        let Some(kind) = segment_slot_kind(option) else {
            break;
        };
        let id = painter_layer_widget_id(layer_id, kind);
        let srect = Rect::new(
            x + option as f32 * seg_w,
            y,
            (seg_w - 2.0).max(0.0), // LITERAL-PX-OK: 2px inter-segment gutter
            ROW_H_PX,
        );
        let (bg, fg) = if option == selected {
            (ColorToken::AccentSoft, ColorToken::Text1)
        } else {
            (ColorToken::Bg2, ColorToken::Text2)
        };
        fill_rounded_rect(ctx.scene, srect, Radius::Sm.px(), resolve(bg, theme));
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            label,
            srect,
            font,
            resolve(fg, theme),
        );
        register_button(ctx.host.store_mut(), id);
        ctx.host.hit_index_mut().register(id, srect);
    }
    y + ROW_H_PX + gap
}

/// Bespoke Selective Color editor: a row of 9 color-group tabs
/// (R/Y/G/C/B/M/W/N/K), then the 4 CMYK sliders of the ACTIVE group, then the
/// method segment (Relative/Absolute). The active tab is panel-local VIEW state
/// ([`state::active_selective_bucket`]); a CMYK-slider drag forwards
/// `SelectOption(PAINTER_SELCOLOR_EDIT, "layer:bucket:slot:value")` from
/// `event.rs` (the active bucket carries the group). Returns the next `y`.
fn paint_selective_color(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    layer_id: u64,
    params: &AdjustmentParams,
    x: f32,
    w: f32,
    mut y: f32,
) -> f32 {
    let AdjustmentParams::SelectiveColor(s) = params else {
        return y;
    };
    let gap = Spacing::Xs.px();
    let font = TypeToken::Base.px();
    let active = state::active_selective_bucket(layer_id).min(8);
    // ── 9 color-group tabs (single-char labels span the width) ──
    const TABS: [&str; 9] = ["R", "Y", "G", "C", "B", "M", "W", "N", "K"];
    let tab_w = w / 9.0; // LITERAL-PX-OK: 9 color-group tab count
    for (bucket, label) in TABS.iter().enumerate() {
        let trect = Rect::new(
            x + bucket as f32 * tab_w,
            y,
            (tab_w - 1.0).max(0.0), // LITERAL-PX-OK: 1px inter-tab gutter
            ROW_H_PX,
        );
        let (bg, fg) = if bucket as u8 == active {
            (ColorToken::AccentSoft, ColorToken::Text1)
        } else {
            (ColorToken::Bg2, ColorToken::Text2)
        };
        fill_rounded_rect(ctx.scene, trect, Radius::Sm.px(), resolve(bg, theme));
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            label,
            trect,
            font,
            resolve(fg, theme),
        );
        let id = painter_selcolor_bucket_id(layer_id, bucket as u8);
        register_button(ctx.host.store_mut(), id);
        ctx.host.hit_index_mut().register(id, trect);
    }
    y += ROW_H_PX + gap;
    // ── 4 CMYK sliders for the active group ──
    for (slot, (label, val01)) in
        ph2d_tool_painter::selective_color_slider_params(s, active as usize)
            .into_iter()
            .enumerate()
    {
        let Some(kind) = slot_kind(slot) else { break };
        let id = painter_layer_widget_id(layer_id, kind);
        paint_labeled_slider(ctx, theme, id, label, val01, Rect::new(x, y, w, ROW_H_PX));
        y += ROW_H_PX + gap;
    }
    // ── method segment (Relative / Absolute) ──
    paint_segment_rack(ctx, theme, layer_id, params, x, w, y)
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
