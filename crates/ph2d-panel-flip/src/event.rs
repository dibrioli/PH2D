//! Flip Style panel event router.
//!
//! Classifies a raw `WidgetEvent` into a tool-agnostic `PanelEvent` pushed onto
//! the action bus (`ToolPanelEvent`), mirror of the Vector panel + the Painter
//! layers panel (for the dynamic per-row ids). Split:
//! - Brush sliders (Size/Hardness/Opacity/Smoothing) `ValueChanged` → `SetValue`
//!   (the tool projects the track); the `_NUM` chip edit is swallowed (the
//!   slider already mirrored + fired).
//! - Mode + Erase sub-mode buttons → `Click` (the tool sets the mode).
//! - Layers toolbar (Add/Delete) + per-row widgets (eye/lock/row-select/↑↓) →
//!   `Click`; per-row opacity slider → `SetValue`; blend option → close the
//!   dropdown + `SelectOption(blend_chip_id, mode)`. These are DOCUMENT edits —
//!   the shell drain applies them (the tool ignores them).
//! - Close (X) → `CancelActiveTool`.

use crate::ids;
use crate::state::{FlipPanelState, current_layers};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::FlipLayerWidget;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal, seam_reset_button};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_effects::MAX_BLEND_MODES;

/// Decode a runtime per-row widget id → `(layer_u64, kind)` via the published
/// layer list (brute-force over rows × kinds — a handful of layers).
fn decode_layer_widget(id: ph2d_a11y::NodeId) -> Option<(u64, FlipLayerWidget)> {
    for row in current_layers().rows {
        for kind in FlipLayerWidget::ALL {
            if ids::flip_layer_widget_id(row.id, kind) == id {
                return Some((row.id, kind));
            }
        }
    }
    None
}

/// Decode a runtime blend-option id → `(layer_u64, mode)`.
fn decode_blend_option(id: ph2d_a11y::NodeId) -> Option<(u64, u8)> {
    for row in current_layers().rows {
        for mode in 0..MAX_BLEND_MODES {
            if ids::flip_layer_blend_option_id(row.id, mode) == id {
                return Some((row.id, mode));
            }
        }
    }
    None
}

pub(crate) fn apply_event(
    _state: &mut FlipPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let consumed = match ev {
        // ── Brush sliders (track 0..1 → the tool projects the value). ──
        WidgetEvent::ValueChanged(id)
            if id == ids::FLIP_SIZE
                || id == ids::FLIP_HARDNESS
                || id == ids::FLIP_OPACITY
                || id == ids::FLIP_SMOOTHING
                || id == ids::FLIP_GAP
                || id == ids::FLIP_GROW
                || id == ids::FLIP_PRECISION =>
        {
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    f64::from(track),
                )));
            true
        }
        // Chip edits already mirrored to their slider (which fired its own
        // ValueChanged, handled above): swallow to avoid a double notify.
        WidgetEvent::ValueChanged(id)
            if id == ids::FLIP_SIZE_NUM
                || id == ids::FLIP_HARDNESS_NUM
                || id == ids::FLIP_OPACITY_NUM
                || id == ids::FLIP_SMOOTHING_NUM
                || id == ids::FLIP_GAP_NUM
                || id == ids::FLIP_GROW_NUM
                || id == ids::FLIP_PRECISION_NUM =>
        {
            true
        }
        // ── Mode + Erase sub-mode buttons → the tool sets the mode. ──
        WidgetEvent::Click(id)
            if id == ids::FLIP_MODE_SELECT
                || id == ids::FLIP_MODE_DRAW
                || id == ids::FLIP_MODE_ERASE
                || id == ids::FLIP_MODE_FILL
                || id == ids::FLIP_MODE_RESHAPE
                || id == ids::FLIP_MODE_EDIT
                || id == ids::FLIP_EDIT_DOM_STROKE
                || id == ids::FLIP_EDIT_DOM_POINT
                || id == ids::FLIP_SHAPE_LINE
                || id == ids::FLIP_SHAPE_FILLED
                || id == ids::FLIP_ERASE_SOFT
                || id == ids::FLIP_ERASE_HARD
                || id == ids::FLIP_ERASE_STROKE
                || id == ids::FLIP_FILL_PAINT
                || id == ids::FLIP_FILL_BEHIND
                || id == ids::FLIP_FILL_UNPAINT
                // Os oito pincéis de escultura (W5) — a tabela é a ordem do painel.
                || ids::FLIP_RESHAPE_KIND_IDS.contains(&id) =>
        {
            seam_reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // ── Edit section (W6): Select All / Deselect / Delete. São edições de
        //    DOCUMENTO (mexem nos traços, não no estilo da tool) → mesmo caminho das ops
        //    de camada: o bus leva, o drain do shell aplica.
        WidgetEvent::Click(id)
            if id == ids::FLIP_EDIT_SELECT_ALL
                || id == ids::FLIP_EDIT_DESELECT
                || id == ids::FLIP_EDIT_DELETE =>
        {
            seam_reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // ── Layers toolbar (Add/Delete) → the shell drain applies to the doc. ──
        WidgetEvent::Click(id) if id == ids::FLIP_LAYER_ADD || id == ids::FLIP_LAYER_DELETE => {
            seam_reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // ── Close (X) → deactivate the tool. ──
        WidgetEvent::Click(id) if id == ids::FLIP_CLOSE => {
            seam_reset_button(host, id);
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        // ── Dynamic per-row layer widgets. ──
        WidgetEvent::Click(id) => {
            // Blend option picked → close the dropdown + apply.
            if let Some((layer, mode)) = decode_blend_option(id) {
                let blend_id = ids::flip_layer_widget_id(layer, FlipLayerWidget::Blend);
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
                return EventOutcome::from_bool(true);
            }
            // Row-select / visibility / lock / reorder ↑↓ → forward as Click (the
            // shell drain applies to the doc). The Blend chip's click is the
            // generic Dropdown open/close, NOT forwarded.
            match decode_layer_widget(id) {
                Some((_, FlipLayerWidget::Blend)) => true, // open/close only
                Some(_) => {
                    seam_reset_button(host, id);
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
                    true
                }
                None => false,
            }
        }
        // Per-row opacity slider → forward the track (the shell drain sets the
        // layer's opacity).
        WidgetEvent::ValueChanged(id) => match decode_layer_widget(id) {
            Some((_, FlipLayerWidget::Opacity)) => {
                let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(1.0);
                host.bus_mut()
                    .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                        id,
                        f64::from(track),
                    )));
                true
            }
            _ => false,
        },
        _ => false,
    };
    EventOutcome::from_bool(consumed)
}
