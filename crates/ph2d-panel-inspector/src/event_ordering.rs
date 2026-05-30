//! Inspector §7 (Ordering / Sorting) event handlers — Sprite Inspector
//! v2 W3. Split from `event.rs` for the panel HR-18 LOC cap. Each widget
//! interaction maps to an [`OrderingFieldEdit`] dispatched as
//! [`EditorAction::InspectorOrderingEdit`]; the shell's
//! `apply_ordering_edit` turns it into a SetComponent / RemoveComponent.

use crate::state;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::OrderingFieldEdit;
use ph2d_editor_core::widget::CheckboxValue;

/// Handle a §7 ordering widget event. Returns `true` if consumed.
pub(crate) fn apply_ordering_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    // Checkbox toggles → optional sorting components.
    if let WidgetEvent::Toggled(id) = ev
        && let Some(info) = state::current_inspector_ordering()
    {
        let checked = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        let edit = match id {
            ids::INSP_ORDER_Z_OVERRIDE => {
                // Attach with the current Z value (store / snapshot / 0);
                // detach when unchecked.
                if checked {
                    let v = host
                        .store()
                        .number_value(ids::INSP_ORDER_Z_INDEX)
                        .map(|f| f as i32)
                        .or(info.z_index)
                        .unwrap_or(0);
                    Some(OrderingFieldEdit::ZIndex(Some(v)))
                } else {
                    Some(OrderingFieldEdit::ZIndex(None))
                }
            }
            ids::INSP_ORDER_Z_RELATIVE => Some(OrderingFieldEdit::ZAsRelative(checked)),
            ids::INSP_ORDER_SHOW_BEHIND => Some(OrderingFieldEdit::ShowBehindParent(checked)),
            ids::INSP_ORDER_YSORT_ENABLED => Some(OrderingFieldEdit::YSortEnabled(checked)),
            ids::INSP_ORDER_SORTING_GROUP => Some(OrderingFieldEdit::SortingGroup(checked)),
            ids::INSP_ORDER_SORT_AT_ROOT => Some(OrderingFieldEdit::SortAtRoot(checked)),
            ids::INSP_ORDER_TOP_LEVEL => Some(OrderingFieldEdit::TopLevel(checked)),
            _ => None,
        };
        if let Some(edit) = edit {
            host.bus_mut().push(EditorAction::InspectorOrderingEdit {
                entity_bits: info.entity_bits,
                edit,
            });
            return true;
        }
    }
    // Integer NumberInput commits (Z Index, Order in Layer).
    if let WidgetEvent::ValueChanged(id) = ev
        && matches!(id, ids::INSP_ORDER_Z_INDEX | ids::INSP_ORDER_ORDER_IN_LAYER)
        && let Some(info) = state::current_inspector_ordering()
    {
        let v = host.store().number_value(id).unwrap_or(0.0).round() as i32;
        let edit = if id == ids::INSP_ORDER_Z_INDEX {
            OrderingFieldEdit::ZIndex(Some(v))
        } else {
            OrderingFieldEdit::OrderInLayer(v)
        };
        host.bus_mut().push(EditorAction::InspectorOrderingEdit {
            entity_bits: info.entity_bits,
            edit,
        });
        return true;
    }
    // Sorting Layer dropdown option selected. Reflect the pick in the
    // store (close + selected_index) so the chip updates immediately.
    if let WidgetEvent::Click(id) = ev
        && let Some(idx) = ids::INSP_ORDER_LAYER_OPT.iter().position(|&o| o == id)
        && let Some(info) = state::current_inspector_ordering()
    {
        if let Some(InteractiveState::Dropdown {
            open,
            selected_index,
            ..
        }) = host.store_mut().get_mut(ids::INSP_ORDER_SORTING_LAYER)
        {
            *open = false;
            *selected_index = Some(idx);
        }
        host.bus_mut().push(EditorAction::InspectorOrderingEdit {
            entity_bits: info.entity_bits,
            edit: OrderingFieldEdit::SortingLayer(idx as u8),
        });
        return true;
    }
    // Y-Sort Sort Point segmented selected.
    if let WidgetEvent::Click(id) = ev
        && let Some(info) = state::current_inspector_ordering()
        && let Some(tag) = match id {
            ids::INSP_ORDER_SP_CENTER => Some(0u8),
            ids::INSP_ORDER_SP_PIVOT => Some(1),
            ids::INSP_ORDER_SP_CUSTOM => Some(2),
            _ => None,
        }
    {
        host.bus_mut().push(EditorAction::InspectorOrderingEdit {
            entity_bits: info.entity_bits,
            edit: OrderingFieldEdit::YSortPoint(tag),
        });
        return true;
    }
    // Y-Sort Custom Axis committed (X or Y → whole vector).
    if let WidgetEvent::ValueChanged(id) = ev
        && matches!(id, ids::INSP_ORDER_AXIS_X | ids::INSP_ORDER_AXIS_Y)
        && let Some(info) = state::current_inspector_ordering()
    {
        let ax = host
            .store()
            .number_value(ids::INSP_ORDER_AXIS_X)
            .unwrap_or(info.y_sort_axis[0] as f64) as f32;
        let ay = host
            .store()
            .number_value(ids::INSP_ORDER_AXIS_Y)
            .unwrap_or(info.y_sort_axis[1] as f64) as f32;
        host.bus_mut().push(EditorAction::InspectorOrderingEdit {
            entity_bits: info.entity_bits,
            edit: OrderingFieldEdit::YSortAxis([ax, ay]),
        });
        return true;
    }
    false
}
