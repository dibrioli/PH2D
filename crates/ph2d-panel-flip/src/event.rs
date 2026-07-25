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
use crate::state::{FlipPanelState, LayerRename, current_layers};
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
    state: &mut FlipPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let consumed = match ev {
        // ── Inline layer rename (§4.C). Double-clicking a layer NAME opens the field;
        //    Enter (Submit) / click-away (Blur) commit; Esc (Cancel) abandons. The
        //    commit forwards the new name on the layer's Row id via `SelectOption`
        //    (the shell drain decodes the Row id → layer and renames — a DOCUMENT
        //    edit, like the blend chip). Mirror of the timeline marker rename. ──
        WidgetEvent::DoubleClick(id) => {
            if let Some((layer, FlipLayerWidget::Row)) = decode_layer_widget(id) {
                state.layer_rename = Some(LayerRename {
                    layer,
                    opened: false,
                });
                true
            } else {
                false
            }
        }
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if id == ids::FLIP_LAYER_RENAME_INPUT => {
            commit_layer_rename(state, host);
            true
        }
        WidgetEvent::Cancel(id) if id == ids::FLIP_LAYER_RENAME_INPUT => {
            cancel_layer_rename(state, host);
            true
        }
        // ── Brush sliders (track 0..1 → the tool projects the value). ──
        WidgetEvent::ValueChanged(id)
            if id == ids::FLIP_SIZE
                || id == ids::FLIP_HARDNESS
                || id == ids::FLIP_OPACITY
                || id == ids::FLIP_SMOOTHING
                || id == ids::FLIP_GAP
                || id == ids::FLIP_GROW
                || id == ids::FLIP_TRAP
                || id == ids::FLIP_COLORIZE_BLEED
                || id == ids::FLIP_PRECISION
                // O Spacing do *tip* pontilhado (03 §8) — sem este arm o arrasto do slider
                // era dropado em silêncio (o slider mexia na tela e nunca chegava ao tool).
                || id == ids::FLIP_DOT_SPACING
                // Os sliders PRÓPRIOS da borracha (§4.C) — só existem na tela com o
                // link desligado, mas o arm vale sempre (o painel não gateia evento).
                || id == ids::FLIP_ERASE_SIZE
                || id == ids::FLIP_ERASE_STRENGTH =>
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
                || id == ids::FLIP_TRAP_NUM
                || id == ids::FLIP_COLORIZE_BLEED_NUM
                || id == ids::FLIP_PRECISION_NUM
                || id == ids::FLIP_DOT_SPACING_NUM
                || id == ids::FLIP_ERASE_SIZE_NUM
                || id == ids::FLIP_ERASE_STRENGTH_NUM =>
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
                || id == ids::FLIP_MODE_COLORIZE
                || id == ids::FLIP_MODE_TRACE
                || id == ids::FLIP_EDIT_DOM_STROKE
                || id == ids::FLIP_EDIT_DOM_POINT
                || id == ids::FLIP_EDIT_DOM_SEGMENT
                || id == ids::FLIP_SHAPE_LINE
                || id == ids::FLIP_SHAPE_FILLED
                // Tip (Draw, 03 §8): linha cheia / contas / quadrados.
                || id == ids::FLIP_TIP_LINE
                || id == ids::FLIP_TIP_DOTS
                || id == ids::FLIP_TIP_SQUARES
                || id == ids::FLIP_ERASE_SOFT
                || id == ids::FLIP_ERASE_HARD
                || id == ids::FLIP_ERASE_STROKE
                // Os toggles de LINK da borracha (§4.C) — a tool inverte o flag.
                || id == ids::FLIP_LINK_SIZE
                || id == ids::FLIP_LINK_STRENGTH
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
                || id == ids::FLIP_EDIT_DELETE
                // Colorize (C2): Apply roda o corte, Clear descarta os rabiscos — os dois
                // mexem no buffer transiente do shell + no documento (não na tool).
                || id == ids::FLIP_COLORIZE_APPLY
                || id == ids::FLIP_COLORIZE_CLEAR
                // Trace: o Reset limpa os deslocamentos, que são SESSÃO do shell.
                || id == ids::FLIP_TRACE_RESET =>
        {
            seam_reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // ── Layers toolbar (Add/Duplicate/Delete) → the shell drain applies to the doc. ──
        WidgetEvent::Click(id)
            if id == ids::FLIP_LAYER_ADD
                || id == ids::FLIP_LAYER_DUPLICATE
                || id == ids::FLIP_LAYER_DELETE =>
        {
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
        // Per-row Opacity/Depth sliders (canonical label+track+chip rows).
        WidgetEvent::ValueChanged(id) => match decode_layer_widget(id) {
            // Opacity and multiplane Depth are both `0..1` sliders forwarded as
            // `SetValue(id, v)` — the shell decodes the id to know which one it is.
            Some((_, FlipLayerWidget::Opacity | FlipLayerWidget::Depth)) => {
                let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(1.0);
                host.bus_mut()
                    .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                        id,
                        f64::from(track),
                    )));
                true
            }
            // The value CHIPS echo their edit onto the linked slider (handled above) —
            // swallow to avoid a double notify. Mirror of the brush `*_NUM` arm.
            Some((_, FlipLayerWidget::OpacityNum | FlipLayerWidget::DepthNum)) => true,
            _ => false,
        },
        _ => false,
    };
    EventOutcome::from_bool(consumed)
}

/// Commit the open inline rename (§4.C): read the field's trimmed text and forward it
/// as `SelectOption(row_id, name)` — the shell drain renames the layer. An empty name
/// is ignored (the layer keeps its old name). The `take` makes the Enter→Submit+Blur
/// pair idempotent. Clears focus IF it is still on the field, so the shell's Flip
/// shortcuts (which auto-suppress while a text field is focused) come back to life.
fn commit_layer_rename(state: &mut FlipPanelState, host: &mut dyn PanelHostInternal) {
    let Some(lr) = state.layer_rename.take() else {
        return;
    };
    let name = match host.store().get(ids::FLIP_LAYER_RENAME_INPUT) {
        Some(InteractiveState::TextInput { text, .. }) => text.trim().to_string(),
        _ => String::new(),
    };
    release_rename_focus(host);
    if name.is_empty() {
        return; // empty name ignored — the layer keeps its old name
    }
    let row_id = ids::flip_layer_widget_id(lr.layer, FlipLayerWidget::Row);
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            row_id, name,
        )));
}

/// Abandon the open rename (Esc) without committing.
fn cancel_layer_rename(state: &mut FlipPanelState, host: &mut dyn PanelHostInternal) {
    state.layer_rename = None;
    release_rename_focus(host);
}

/// Drop focus IFF it still rests on the rename field — never steal focus the user
/// moved elsewhere (a Blur fires because focus already went to the next widget).
fn release_rename_focus(host: &mut dyn PanelHostInternal) {
    if host.store().focus_id() == Some(ids::FLIP_LAYER_RENAME_INPUT) {
        host.store_mut().set_focus(None);
    }
}
