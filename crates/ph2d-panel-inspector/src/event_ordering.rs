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
use ph2d_editor_core::screens::hero::{
    BlendFieldEdit, OrderingFieldEdit, SamplingFieldEdit, VisibilityFieldEdit,
};
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
            // The field IS the override: 0 detaches `ZIndexOverride`
            // (default / pure DFS), any other value attaches it.
            OrderingFieldEdit::ZIndex(if v == 0 { None } else { Some(v) })
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
    // §9 Sampling — Texture Filter / Repeat segmented selected.
    if let WidgetEvent::Click(id) = ev
        && let Some(info) = state::current_inspector_sampling()
    {
        let edit = ids::INSP_SAMPLE_FILTER
            .iter()
            .position(|&o| o == id)
            .map(|i| SamplingFieldEdit::Filter(i as u8))
            .or_else(|| {
                ids::INSP_SAMPLE_REPEAT
                    .iter()
                    .position(|&o| o == id)
                    .map(|i| SamplingFieldEdit::Repeat(i as u8))
            });
        if let Some(edit) = edit {
            host.bus_mut().push(EditorAction::InspectorSamplingEdit {
                entity_bits: info.entity_bits,
                edit,
            });
            return true;
        }
    }
    // §10 Material & Blend — Blend Mode segmented selected (tag 0 = Mix
    // detaches the optional component).
    if let WidgetEvent::Click(id) = ev
        && let Some(info) = state::current_inspector_blend()
        && let Some(i) = ids::INSP_SAMPLE_BLEND.iter().position(|&o| o == id)
    {
        host.bus_mut().push(EditorAction::InspectorBlendEdit {
            entity_bits: info.entity_bits,
            edit: BlendFieldEdit::Blend(i as u8),
        });
        return true;
    }
    // §9 Sampling — UV Scale / Offset NumberInput commits.
    if let WidgetEvent::ValueChanged(id) = ev
        && let Some(info) = state::current_inspector_sampling()
    {
        let v = host.store().number_value(id).unwrap_or(0.0) as f32;
        let edit = match id {
            ids::INSP_SAMPLE_UV_SCALE_X => Some(SamplingFieldEdit::UvScaleX(v)),
            ids::INSP_SAMPLE_UV_SCALE_Y => Some(SamplingFieldEdit::UvScaleY(v)),
            ids::INSP_SAMPLE_UV_OFFSET_X => Some(SamplingFieldEdit::UvOffsetX(v)),
            ids::INSP_SAMPLE_UV_OFFSET_Y => Some(SamplingFieldEdit::UvOffsetY(v)),
            _ => None,
        };
        if let Some(edit) = edit {
            host.bus_mut().push(EditorAction::InspectorSamplingEdit {
                entity_bits: info.entity_bits,
                edit,
            });
            return true;
        }
    }
    // §8 Visibility — split out to keep this fn under the 200-LOC cap.
    apply_visibility_section_event(host, ev)
}

/// Handle a §8 visibility-section widget event (segmented Clip / Mask,
/// the 32-bit layer bitmask, the On-Screen toggle, and the cutoff / rect
/// NumberInputs). Returns `true` if consumed.
fn apply_visibility_section_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    // Segmented (Clip / Mask) + bitmask + on-screen clicks.
    if let WidgetEvent::Click(id) = ev
        && let Some(info) = state::current_inspector_visibility_section()
    {
        let edit = ids::INSP_VIS_CLIP
            .iter()
            .position(|&o| o == id)
            .map(|i| VisibilityFieldEdit::ClipMode(i as u8))
            .or_else(|| {
                ids::INSP_VIS_MASK
                    .iter()
                    .position(|&o| o == id)
                    .map(|i| VisibilityFieldEdit::MaskMode(i as u8))
            })
            .or_else(|| {
                ids::INSP_VIS_LAYER_BIT
                    .iter()
                    .position(|&o| o == id)
                    .map(|n| {
                        // Toggle this bit relative to the snapshot value.
                        let on = (info.layer_mask >> n as u32) & 1 == 0;
                        VisibilityFieldEdit::LayerBit(n as u8, on)
                    })
            })
            .or_else(|| {
                (id == ids::INSP_VIS_MASK_SOURCE)
                    .then_some(VisibilityFieldEdit::MaskSource(!info.mask_source))
            })
            .or_else(|| {
                (id == ids::INSP_VIS_ON_SCREEN)
                    .then_some(VisibilityFieldEdit::OnScreen(!info.on_screen))
            });
        if let Some(edit) = edit {
            host.bus_mut()
                .push(EditorAction::InspectorVisibilitySectionEdit {
                    entity_bits: info.entity_bits,
                    edit,
                });
            return true;
        }
    }
    // Mask Alpha Cutoff + Enabler Rect NumberInput commits.
    if let WidgetEvent::ValueChanged(id) = ev
        && let Some(info) = state::current_inspector_visibility_section()
    {
        let v = host.store().number_value(id).unwrap_or(0.0) as f32;
        let edit = match id {
            ids::INSP_VIS_ALPHA_CUTOFF => Some(VisibilityFieldEdit::AlphaCutoff(v)),
            ids::INSP_VIS_RECT_X => Some(VisibilityFieldEdit::RectX(v)),
            ids::INSP_VIS_RECT_Y => Some(VisibilityFieldEdit::RectY(v)),
            ids::INSP_VIS_RECT_W => Some(VisibilityFieldEdit::RectW(v)),
            ids::INSP_VIS_RECT_H => Some(VisibilityFieldEdit::RectH(v)),
            _ => None,
        };
        if let Some(edit) = edit {
            host.bus_mut()
                .push(EditorAction::InspectorVisibilitySectionEdit {
                    entity_bits: info.entity_bits,
                    edit,
                });
            return true;
        }
    }
    false
}
