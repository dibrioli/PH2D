//! Painter layers `apply_event` — thin forwarder (ADR-0040 TG-B), mirror
//! do `ph2d-panel-painter-sidebar` event.
//!
//! The panel keeps NO semantic mapping. Each `WidgetEvent` is classified into a
//! tool-agnostic [`PanelEvent`] and pushed via `EditorAction::ToolPanelEvent`;
//! the shell's action-bus drain calls `PainterTool::handle_panel_event` on the
//! active tool, which decodes the per-row id back to its `(layer, kind)` and
//! applies the edit.
//!
//! Per-row ids are decoded here only to pick the right `PanelEvent` shape:
//! row-select / visibility eye → `Click`, opacity slider → `SetValue`, blend
//! dropdown option → `SelectOption(blend_id, mode_u8)`. The blend chip itself
//! opens/closes its popover via the generic `Dropdown` dispatch (not routed
//! here). The decode uses the published `current_layers()` snapshot.

use crate::state::{self, PainterLayersPanelState};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{
    self as core_ids, PainterLayerWidget, painter_curve_add_id, painter_curve_editor_id,
    painter_curve_remove_id, painter_curve_tab_id, painter_layer_blend_option_id,
    painter_layer_widget_id,
};
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_tool_painter::{LayerId, LayerStack, MAX_BLEND_MODES};

pub(crate) fn apply_event(
    _state: &mut PainterLayersPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
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
            // W4 §3 — Curves editor chrome: channel tabs (panel-local view state,
            // never forwarded) + the +/− point buttons (forwarded to the tool on
            // the ACTIVE channel; remove targets the last-dragged point, else a
            // middle interior point).
            for layer in stack.all_ids() {
                let lid = layer.0;
                if (0u8..4).any(|ch| painter_curve_tab_id(lid, ch) == id) {
                    let ch = (0u8..4)
                        .find(|&ch| painter_curve_tab_id(lid, ch) == id)
                        .unwrap();
                    state::set_active_curve_channel(lid, ch);
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
                    // Multi-select: carry the live Cmd/Shift state to the tool's
                    // row-select. The frozen PanelEvent can not hold it and the
                    // tool's handle_panel_event gets no store, so stash it in the
                    // tool-crate thread-local right before forwarding the Click.
                    // Cmd/Ctrl = toggle additive, Shift = range, plain = single.
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
                    | PainterLayerWidget::MaskApply,
                )) => {
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
                    true
                }
                _ => false,
            }
        }
        // Per-row opacity slider drag — read the freshly-dispatched `0..1`
        // value and forward normalized (the linked chip edit propagates back
        // to the slider, so its ValueChanged arrives here too — single route).
        WidgetEvent::ValueChanged(id) => {
            // W4 §3 — a Curves control-point 2-D drag stashed its (parent, channel,
            // index, x, y) on the store (global slot, `Some` only when the active
            // widget is a `CurvePoint`, so this `ValueChanged` IS that drag). Drain
            // it, re-derive the layer from the editor `parent`, and forward to the
            // tool as `SelectOption(PAINTER_CURVE_EDIT, "layer:ch:idx:x:y")`.
            if let Some((parent, ch, idx, x, y)) = host.store_mut().take_curve_point_drag() {
                if let Some(stack) = state::current_layers()
                    && let Some(layer) = stack
                        .all_ids()
                        .find(|l| painter_curve_editor_id(l.0) == parent)
                {
                    // Remember the touched point so the "−" button knows what to drop.
                    state::set_selected_curve_point(Some((layer.0, ch, usize::from(idx))));
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                            core_ids::PAINTER_CURVE_EDIT,
                            format!("{}:{ch}:{idx}:{x}:{y}", layer.0),
                        )));
                }
                return true;
            }
            let Some(stack) = state::current_layers() else {
                return false;
            };
            // Per-row sliders: opacity + the adjustment param slots (0..1).
            if let Some((_, kind)) = decode(&stack, id)
                && matches!(
                    kind,
                    PainterLayerWidget::Opacity
                        | PainterLayerWidget::AdjParam0
                        | PainterLayerWidget::AdjParam1
                        | PainterLayerWidget::AdjParam2
                        | PainterLayerWidget::AdjParam3
                        | PainterLayerWidget::AdjParam4
                        | PainterLayerWidget::AdjParam5
                )
            {
                let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
                host.bus_mut()
                    .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                        id, v as f64,
                    )));
                return true;
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

/// Decode a "+ Adjustment" kind-picker option id → its index into
/// [`ph2d_tool_painter::AdjustmentKind::ALL`]. Fixed (not per-layer), so iterate
/// the 24 stable ids. `None` for any other widget.
fn decode_adjustment_kind_option(id: ph2d_a11y::NodeId) -> Option<usize> {
    (0..ph2d_tool_painter::AdjustmentKind::ALL.len() as u8)
        .find(|&i| core_ids::painter_adjustment_kind_option_id(i) == id)
        .map(usize::from)
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
