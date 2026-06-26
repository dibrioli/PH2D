//! Painter layers `apply_event` — thin forwarder (ADR-0040 TG-B). The panel keeps NO semantic
//! mapping: each `WidgetEvent` is classified into a tool-agnostic [`PanelEvent`] and pushed via
//! `EditorAction::ToolPanelEvent`; the shell's action-bus drain calls `PainterTool::handle_panel_event`,
//! which decodes the id back to its `(layer, kind)` and applies the edit. Per-row ids are decoded here
//! only to pick the `PanelEvent` shape (Click / SetValue / SelectOption) off the `current_layers()`
//! snapshot; dropdown chips open/close via the generic `Dropdown` dispatch, not here.

use crate::state::{self, PainterLayersPanelState};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{
    self as core_ids, PainterLayerWidget, painter_curve_add_id, painter_curve_editor_id,
    painter_curve_remove_id, painter_curve_tab_id, painter_gradient_add_id,
    painter_gradient_editor_id, painter_gradient_remove_id, painter_layer_blend_option_id,
    painter_layer_widget_id, painter_mixer_tab_id, painter_selcolor_bucket_id,
};
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_tool_painter::{AdjustmentParams, LayerId, LayerKind, LayerStack, MAX_BLEND_MODES};

/// Dropdown option-id decoders + the dropdown-option routing table (split out for the LOC cap).
mod decode;
mod option_route;
mod ramp_picker;
mod shape_ramp_picker;

pub(crate) fn apply_event(
    _state: &mut PainterLayersPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    // Brush-section widgets are fixed-id + tool-global → route before the per-layer dispatch.
    if let Some(consumed) = try_apply_brush_event(host, ev) {
        return EventOutcome::from_bool(consumed);
    }
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Close (X) → CancelActiveTool (canon BgRemoval/Painter sidebar).
        WidgetEvent::Click(id) if id == core_ids::PAINTER_LAYERS_CLOSE => {
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        // Fixed chrome buttons: "+ Layer" + dock toggle + Apply CTA → forward
        // as Click (the tool maps each to its action).
        WidgetEvent::Click(id)
            if id == core_ids::PAINTER_LAYERS_ADD
                || id == core_ids::PAINTER_LAYERS_TOGGLE_DOCK
                || id == core_ids::PAINTER_APPLY
                || id == core_ids::PAINTER_LAYERS_DUPLICATE
                || id == core_ids::PAINTER_LAYERS_DELETE
                || id == core_ids::PAINTER_LAYERS_GROUP
                || id == core_ids::PAINTER_LAYERS_ADD_TEXTURE
                || id == core_ids::PAINTER_LAYERS_MASK
                || id == core_ids::PAINTER_LAYERS_CLIP
                || id == core_ids::PAINTER_LAYERS_ALPHA_LOCK
                || id == core_ids::PAINTER_LAYERS_REFERENCE =>
        {
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // "+ Adjustment" kind-picker option (W4 T4.15): close the dropdown +
        // forward the chosen kind index. The `+ Adj` chip itself is a Dropdown —
        // its open/close is the generic dispatch (no Click forwarded here). The
        // kind index is the position in `AdjustmentKind::ALL`; the tool maps it
        // back via `add_adjustment_layer`.
        WidgetEvent::Click(id) if decode_adjustment_kind_option(id).is_some() => {
            let idx = decode_adjustment_kind_option(id).unwrap();
            if let Some(InteractiveState::Dropdown { open, .. }) = host
                .store_mut()
                .get_mut(core_ids::PAINTER_LAYERS_ADD_ADJUSTMENT)
            {
                *open = false;
            }
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                    core_ids::PAINTER_LAYERS_ADD_ADJUSTMENT,
                    idx.to_string(),
                )));
            true
        }
        WidgetEvent::Click(id) => {
            let Some(stack) = state::current_layers() else {
                return false;
            };
            // W4 §3 — Curves editor chrome: channel tabs (panel-local) + the +/− point buttons
            // (forwarded on the ACTIVE channel; remove targets the last-dragged point).
            for layer in stack.all_ids() {
                let lid = layer.0;
                if (0u8..4).any(|ch| painter_curve_tab_id(lid, ch) == id) {
                    let ch = (0u8..4)
                        .find(|&ch| painter_curve_tab_id(lid, ch) == id)
                        .unwrap();
                    state::set_active_curve_channel(lid, ch);
                    return true;
                }
                // Channel Mixer output-channel tab (W4 BATCH-1): panel-local view
                // state, never forwarded (the weight edit carries the channel).
                if (0u8..3).any(|ch| painter_mixer_tab_id(lid, ch) == id) {
                    let ch = (0u8..3)
                        .find(|&ch| painter_mixer_tab_id(lid, ch) == id)
                        .unwrap();
                    state::set_active_mixer_channel(lid, ch);
                    return true;
                }
                // Selective Color group tab (W4 BATCH-2): panel-local view state,
                // never forwarded (the CMYK edit carries the bucket).
                if (0u8..9).any(|bk| painter_selcolor_bucket_id(lid, bk) == id) {
                    let bk = (0u8..9)
                        .find(|&bk| painter_selcolor_bucket_id(lid, bk) == id)
                        .unwrap();
                    state::set_active_selective_bucket(lid, bk);
                    return true;
                }
                if painter_curve_add_id(lid) == id {
                    let ch = state::active_curve_channel(lid);
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                            core_ids::PAINTER_CURVE_ADD,
                            format!("{lid}:{ch}"),
                        )));
                    return true;
                }
                if painter_curve_remove_id(lid) == id {
                    let (ch, idx) = match state::selected_curve_point() {
                        Some((sl, sch, sidx)) if sl == lid => (sch, sidx),
                        _ => (state::active_curve_channel(lid), 1),
                    };
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                            core_ids::PAINTER_CURVE_REMOVE,
                            format!("{lid}:{ch}:{idx}"),
                        )));
                    return true;
                }
                // Gradient Map +/− stop buttons (W4 BATCH-2): "−" drops the
                // selected stop (else the last one).
                if painter_gradient_add_id(lid) == id {
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                            core_ids::PAINTER_GRADIENT_ADD,
                            lid.to_string(),
                        )));
                    return true;
                }
                if painter_gradient_remove_id(lid) == id {
                    let idx = state::selected_gradient_stop(lid);
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                            core_ids::PAINTER_GRADIENT_REMOVE,
                            format!("{lid}:{idx}"),
                        )));
                    return true;
                }
            }
            // Blend dropdown option picked → close the dropdown + apply.
            if let Some((layer, mode)) = decode_blend_option(&stack, id) {
                let blend_id = painter_layer_widget_id(layer.0, PainterLayerWidget::Blend);
                if let Some(InteractiveState::Dropdown {
                    open,
                    selected_index,
                    ..
                }) = host.store_mut().get_mut(blend_id)
                {
                    *open = false;
                    *selected_index = Some(mode as usize);
                }
                host.bus_mut()
                    .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                        blend_id,
                        mode.to_string(),
                    )));
                return true;
            }
            // Per-row row-select / visibility eye / reorder ↑↓ → forward as
            // Click. (The blend chip click is the dropdown open/close — handled
            // by the generic Dropdown dispatch, not forwarded.)
            match decode(&stack, id) {
                Some((_, PainterLayerWidget::Row)) => {
                    // Multi-select: the frozen PanelEvent can't carry Cmd/Shift, so stash it in the
                    // tool-crate thread-local right before the Click (Cmd = additive, Shift = range).
                    ph2d_tool_painter::set_pending_select_mods(
                        id,
                        host.store().cmd_held(),
                        host.store().shift_held(),
                    );
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
                    true
                }
                Some((
                    _,
                    PainterLayerWidget::Visibility
                    | PainterLayerWidget::MoveUp
                    | PainterLayerWidget::MoveDown
                    | PainterLayerWidget::MaskInvert
                    | PainterLayerWidget::MaskApply
                    // Adjustment toggle rack (W4 BATCH-1) — bare click, the tool
                    // flips the boolean param slot (source of truth = params).
                    | PainterLayerWidget::AdjToggle0
                    | PainterLayerWidget::AdjToggle1
                    // Adjustment segment rack (W4 BATCH-1) — bare click, the tool
                    // selects that option of the segmented param.
                    | PainterLayerWidget::AdjSegment0
                    | PainterLayerWidget::AdjSegment1
                    | PainterLayerWidget::AdjSegment2,
                )) => {
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
                    true
                }
                _ => false,
            }
        }
        // Per-row opacity slider drag — forward the dispatched `0..1` (the linked chip edit
        // propagates back to the slider, so its ValueChanged arrives here too — single route).
        WidgetEvent::ValueChanged(id) => {
            // W4 §3 — a Curves control-point 2-D drag stashed `(parent, ch, idx, x, y)` → drain it.
            if let Some((parent, ch, idx, x, y)) = host.store_mut().take_curve_point_drag() {
                if let Some(stack) = state::current_layers() {
                    if let Some(layer) = stack
                        .all_ids()
                        .find(|l| painter_curve_editor_id(l.0) == parent)
                    {
                        // Remember the touched point so the "−" button knows what to drop.
                        state::set_selected_curve_point(Some((layer.0, ch, usize::from(idx))));
                        host.bus_mut().push(EditorAction::ToolPanelEvent(
                            PanelEvent::SelectOption(
                                core_ids::PAINTER_CURVE_EDIT,
                                format!("{}:{ch}:{idx}:{x}:{y}", layer.0),
                            ),
                        ));
                    } else if let Some(layer) = stack
                        .all_ids()
                        .find(|l| painter_gradient_editor_id(l.0) == parent)
                    {
                        // Gradient Map stop drag — `x` is the new offset; selecting the
                        // dragged stop drives its color sliders + the "−" button.
                        state::set_selected_gradient_stop(layer.0, usize::from(idx));
                        host.bus_mut().push(EditorAction::ToolPanelEvent(
                            PanelEvent::SelectOption(
                                core_ids::PAINTER_GRADIENT_EDIT,
                                format!("{}:{idx}:{x}", layer.0),
                            ),
                        ));
                    }
                }
                return true;
            }
            let Some(stack) = state::current_layers() else {
                return false;
            };
            // Per-row sliders: opacity + the adjustment param slots (0..1).
            if let Some((layer, kind)) = decode(&stack, id) {
                // Channel Mixer weight slider (AdjParam0..3 on a ChannelMixer layer): forward the
                // active output tab + slot via PAINTER_MIXER_EDIT (generic SetValue can't carry the row).
                if let Some(slot) = adj_param_slot(kind)
                    && slot <= 3
                    && layer_is_channel_mixer(&stack, layer)
                {
                    let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
                    let out = state::active_mixer_channel(layer.0).min(2);
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                            core_ids::PAINTER_MIXER_EDIT,
                            format!("{}:{out}:{slot}:{v}", layer.0),
                        )));
                    return true;
                }
                // Selective Color CMYK slider (AdjParam0..3): forward bucket + slot via PAINTER_SELCOLOR_EDIT.
                if let Some(slot) = adj_param_slot(kind)
                    && slot <= 3
                    && layer_is_selective_color(&stack, layer)
                {
                    let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
                    let bucket = state::active_selective_bucket(layer.0).min(8);
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                            core_ids::PAINTER_SELCOLOR_EDIT,
                            format!("{}:{bucket}:{slot}:{v}", layer.0),
                        )));
                    return true;
                }
                // Gradient Map RGB slider (AdjParam0..2 on a GradientMap layer):
                // forward the selected stop + slot via PAINTER_GRADIENT_COLOR.
                if let Some(slot) = adj_param_slot(kind)
                    && slot <= 2
                    && layer_is_gradient_map(&stack, layer)
                {
                    let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
                    let stop = state::selected_gradient_stop(layer.0);
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                            core_ids::PAINTER_GRADIENT_COLOR,
                            format!("{}:{stop}:{slot}:{v}", layer.0),
                        )));
                    return true;
                }
                if matches!(
                    kind,
                    PainterLayerWidget::Opacity
                        | PainterLayerWidget::AdjParam0
                        | PainterLayerWidget::AdjParam1
                        | PainterLayerWidget::AdjParam2
                        | PainterLayerWidget::AdjParam3
                        | PainterLayerWidget::AdjParam4
                        | PainterLayerWidget::AdjParam5
                        | PainterLayerWidget::AdjParam6
                        | PainterLayerWidget::AdjParam7
                ) {
                    let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                            id, v as f64,
                        )));
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// Decode a per-row widget id → `(layer, kind)` via the published snapshot.
fn decode(stack: &LayerStack, id: ph2d_a11y::NodeId) -> Option<(LayerId, PainterLayerWidget)> {
    for layer in stack.all_ids() {
        for kind in PainterLayerWidget::ALL {
            if painter_layer_widget_id(layer.0, kind) == id {
                return Some((layer, kind));
            }
        }
    }
    None
}

/// The slider slot index of an `AdjParamN` widget kind (`None` for non-slider
/// widgets). Used to route a Channel-Mixer weight slider with its slot.
fn adj_param_slot(kind: PainterLayerWidget) -> Option<usize> {
    Some(match kind {
        PainterLayerWidget::AdjParam0 => 0,
        PainterLayerWidget::AdjParam1 => 1,
        PainterLayerWidget::AdjParam2 => 2,
        PainterLayerWidget::AdjParam3 => 3,
        PainterLayerWidget::AdjParam4 => 4,
        PainterLayerWidget::AdjParam5 => 5,
        PainterLayerWidget::AdjParam6 => 6,
        PainterLayerWidget::AdjParam7 => 7,
        _ => return None,
    })
}

/// `true` when `layer` is a Channel-Mixer adjustment (its weight sliders route
/// through `PAINTER_MIXER_EDIT`, not the generic `SetValue`).
fn layer_is_channel_mixer(stack: &LayerStack, layer: LayerId) -> bool {
    matches!(
        stack.get(layer).map(|l| &l.kind),
        Some(LayerKind::Adjustment(adj)) if matches!(adj.params, AdjustmentParams::ChannelMixer(_))
    )
}

/// `true` when `layer` is a Selective-Color adjustment (its CMYK sliders route
/// through `PAINTER_SELCOLOR_EDIT`, not the generic `SetValue`).
fn layer_is_selective_color(stack: &LayerStack, layer: LayerId) -> bool {
    matches!(
        stack.get(layer).map(|l| &l.kind),
        Some(LayerKind::Adjustment(adj)) if matches!(adj.params, AdjustmentParams::SelectiveColor(_))
    )
}

/// `true` when `layer` is a Gradient-Map adjustment (its RGB sliders route through
/// `PAINTER_GRADIENT_COLOR`, not the generic `SetValue`).
fn layer_is_gradient_map(stack: &LayerStack, layer: LayerId) -> bool {
    matches!(
        stack.get(layer).map(|l| &l.kind),
        Some(LayerKind::Adjustment(adj)) if matches!(adj.params, AdjustmentParams::GradientMap(_))
    )
}

/// Decode a "+ Adjustment" kind-picker option id → its index into
/// [`ph2d_tool_painter::AdjustmentKind::ALL`]. Fixed (not per-layer), so iterate
/// the 24 stable ids. `None` for any other widget.
fn decode_adjustment_kind_option(id: ph2d_a11y::NodeId) -> Option<usize> {
    (0..ph2d_tool_painter::AdjustmentKind::ALL.len() as u8)
        .find(|&i| core_ids::painter_adjustment_kind_option_id(i) == id)
        .map(usize::from)
}

/// Handle a Brush-section widget event (fixed-id, tool-global). Returns
/// `Some(true)` when `ev` belonged to the Brush section (consumed), `None`
/// otherwise (the caller falls through to the per-layer dispatch). Split out of
/// `apply_event_impl` so that already-at-cap function stays put.
fn try_apply_brush_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> Option<bool> {
    match ev {
        // Colour swatch → toggle the shared Blender picker, seeded with the brush colour (the
        // read-back in `paint_brush::paint_brush_mode` forwards the picked colour back each frame).
        WidgetEvent::Click(id) if id == core_ids::PAINTER_COLOR_THUMB => {
            let store = host.store_mut();
            if store.picker_target() == Some(id) {
                store.set_picker_target(None);
            } else {
                let rgba = brush_seed_rgba8();
                store.set_blender_value(
                    core_ids::INSP_BLENDER_PICKER,
                    ph2d_tokens::ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
                );
                store.set_widget_color(id, rgba);
                store.set_picker_target(Some(id));
            }
            Some(true)
        }
        // Toggles + momentary buttons → forward as a Click (the tool flips the matching bool / acts).
        WidgetEvent::Click(id)
            if id == core_ids::PAINTER_BRUSH_ERASER
                || id == core_ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE
                || core_ids::PAINTER_BRUSH_TILING.contains(&id)
                || id == core_ids::PAINTER_BRUSH_REPEAT_IMAGE
                || id == core_ids::PAINTER_BRUSH_SPACE_ATTEN
                || id == core_ids::PAINTER_BRUSH_ACCUMULATE
                || id == core_ids::PAINTER_BRUSH_EDGE_TO_EDGE
                || id == core_ids::PAINTER_BRUSH_TEXTURE_RAKE
                || id == core_ids::PAINTER_BRUSH_TEXTURE_RANDOM
                || id == core_ids::PAINTER_SHAPE_RAKE
                || id == core_ids::PAINTER_SHAPE_RANDOM
                || id == core_ids::PAINTER_SHAPE_RESET
                || core_ids::PAINTER_BRUSH_TEXTURE_RAMP_BUTTONS.contains(&id)
                || core_ids::PAINTER_SHAPE_RAMP_BUTTONS.contains(&id)
                || id == core_ids::PAINTER_BRUSH_FALLOFF_ADD // Custom-falloff "+" point button
                || core_ids::PAINTER_BRUSH_SECTION_RESETS.contains(&id) =>
        {
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            Some(true)
        }
        // Custom-falloff "−" → drop the selected point (else the first non-endpoint by stable id).
        WidgetEvent::Click(id) if id == core_ids::PAINTER_BRUSH_FALLOFF_REMOVE => {
            let Some(target) = state::selected_falloff_point().or_else(default_falloff_remove_id)
            else {
                return Some(true); // nothing to remove (only the 2 endpoints)
            };
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                    core_ids::PAINTER_BRUSH_FALLOFF_REMOVE,
                    target.to_string(),
                )));
            Some(true)
        }
        // Color Ramp colour box → toggle the shared picker targeting the selected stop.
        WidgetEvent::Click(id) if id == core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH => {
            ramp_picker::on_swatch_click(host);
            Some(true)
        }
        // A dropdown popover option was picked → close the chip + apply (table-driven, see `option_route`).
        WidgetEvent::Click(id) => option_route::route_brush_dropdown_option(host, id),
        // Custom-falloff 2-D drag: `CurvePoint` stashed `(parent, ch, idx, x, y)` → forward `idx:x:y`.
        WidgetEvent::ValueChanged(id) if id == core_ids::PAINTER_BRUSH_FALLOFF_EDIT => {
            if let Some((_parent, _ch, idx, x, y)) = host.store_mut().take_curve_point_drag() {
                // `idx` is the point's STABLE id (panel-registered), valid across a drag-past re-sort.
                state::set_selected_falloff_point(Some(idx));
                host.bus_mut()
                    .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                        core_ids::PAINTER_BRUSH_FALLOFF_EDIT,
                        format!("{idx}:{x}:{y}"),
                    )));
            }
            Some(true)
        }
        // Grain Color Ramp: bar-stop drag + the editable index / position chips → ramp_picker.
        WidgetEvent::ValueChanged(id)
            if core_ids::PAINTER_BRUSH_TEXTURE_RAMP_VALUE_IDS.contains(&id) =>
        {
            ramp_picker::on_ramp_value_changed(host, id);
            Some(true)
        }
        // Shape Tone ramp: bar-stop drag + the editable index / position / value chips.
        WidgetEvent::ValueChanged(id) if core_ids::PAINTER_SHAPE_RAMP_VALUE_IDS.contains(&id) => {
            shape_ramp_picker::on_shape_ramp_value_changed(host, id);
            Some(true)
        }
        // Grain/Shape param number-fields: forward the committed/scrubbed REAL value (the tool's
        // real-value setters clamp it; params/Depth are already `0..1`). Enio 2026-06-25.
        WidgetEvent::ValueChanged(id) if crate::number_field::is_param_field(id) => {
            let v = host.store().number_value(id).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v)));
            Some(true)
        }
        // Brush + Stroke-section slider drag → forward the dispatched `0..1` track; the tool maps it.
        WidgetEvent::ValueChanged(id)
            if id == core_ids::PAINTER_BRUSH_SIZE_SLIDER
                || id == core_ids::PAINTER_BRUSH_STRENGTH_SLIDER
                || id == core_ids::PAINTER_BRUSH_SPACING
                || id == core_ids::PAINTER_BRUSH_JITTER
                || id == core_ids::PAINTER_BRUSH_DASH_RATIO
                || id == core_ids::PAINTER_BRUSH_DASH_LENGTH
                || id == core_ids::PAINTER_BRUSH_INPUT_SAMPLES
                || id == core_ids::PAINTER_BRUSH_STABILIZE
                || id == core_ids::PAINTER_BRUSH_RATE
                || core_ids::PAINTER_BRUSH_RANDOMIZE_SLIDERS.contains(&id) =>
        {
            let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id, v as f64,
                )));
            Some(true)
        }
        _ => None,
    }
}

/// The active brush colour as 8-bit RGBA (opaque), to seed the shared picker.
fn brush_seed_rgba8() -> [u8; 4] {
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    state::current_brush()
        .map(|b| [enc(b.color[0]), enc(b.color[1]), enc(b.color[2]), 255])
        .unwrap_or([0, 0, 0, 255])
}

/// Fallback target for the Falloff "−" button when no point is selected: the
/// stable id of the first interior (non-endpoint) control point, or `None` when
/// only the two endpoints remain (nothing removable).
fn default_falloff_remove_id() -> Option<u8> {
    let b = state::current_brush()?;
    let n = b.falloff_len as usize;
    (n > 2).then(|| b.falloff_points[1].id)
}

/// Decode a blend-mode popover option id → `(layer, mode_u8)`.
fn decode_blend_option(stack: &LayerStack, id: ph2d_a11y::NodeId) -> Option<(LayerId, u8)> {
    for layer in stack.all_ids() {
        for m in 0..MAX_BLEND_MODES {
            if painter_layer_blend_option_id(layer.0, m) == id {
                return Some((layer, m));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests;
