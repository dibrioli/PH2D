//! Background-Removal panel `apply_event` — routes the panel's widget
//! events out to the shell over `EditorAction::BgremovalUiEdit`.
//!
//! The authoritative `BgRemovalTool` lives in the shell's `ToolRegistry`,
//! so the panel can't mutate params directly — every edit becomes a bus
//! action the shell drains into `BgRemovalTool::apply_ui_edit`. Mirrors
//! the Inspector's `host.bus_mut().push(...)` pattern.

use crate::ids;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tools::bgremoval::{BgRemovalUiEdit, BrushFalloff};
use ph2d_editor_core::widget::ButtonState;

use crate::state::BgRemovalPanelState;

pub(crate) fn apply_event(
    _state: &mut BgRemovalPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Slider drag — read the freshly-dispatched value and forward it
        // normalized. The tool maps 0..1 back to full scale.
        WidgetEvent::ValueChanged(id)
            if id == ids::BGR_TOLERANCE
                || id == ids::BGR_FEATHER
                || id == ids::BGR_REFINE
                || id == ids::BGR_GROW =>
        {
            let value = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
            let edit = if id == ids::BGR_TOLERANCE {
                BgRemovalUiEdit::Tolerance(value)
            } else if id == ids::BGR_FEATHER {
                BgRemovalUiEdit::Feather(value)
            } else if id == ids::BGR_REFINE {
                BgRemovalUiEdit::Refine(value)
            } else {
                BgRemovalUiEdit::Grow(value)
            };
            host.bus_mut().push(EditorAction::BgremovalUiEdit(edit));
            true
        }
        // Numeric chip edited (keyboard commit or drag-scrub) — read the
        // committed NumberInput value (normalized 0..1) and route the
        // same edit as the slider. Single source of truth: the chip uses
        // the canonical NumberInput dispatch, identical to Inspector /
        // color-picker chips.
        WidgetEvent::ValueChanged(id)
            if id == ids::BGR_TOLERANCE_NUM
                || id == ids::BGR_FEATHER_NUM
                || id == ids::BGR_REFINE_NUM
                || id == ids::BGR_GROW_NUM =>
        {
            let value = host.store().number_value(id).unwrap_or(0.0) as f32;
            let edit = if id == ids::BGR_TOLERANCE_NUM {
                BgRemovalUiEdit::Tolerance(value)
            } else if id == ids::BGR_FEATHER_NUM {
                BgRemovalUiEdit::Feather(value)
            } else if id == ids::BGR_REFINE_NUM {
                BgRemovalUiEdit::Refine(value)
            } else {
                BgRemovalUiEdit::Grow(value)
            };
            host.bus_mut().push(EditorAction::BgremovalUiEdit(edit));
            true
        }
        // Brush-size slider / chip → BrushSize edit (normalized 0..1).
        WidgetEvent::ValueChanged(id) if id == ids::BGR_BRUSH_SIZE => {
            let value = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::BgremovalUiEdit(BgRemovalUiEdit::BrushSize(
                    value,
                )));
            true
        }
        WidgetEvent::ValueChanged(id) if id == ids::BGR_BRUSH_SIZE_NUM => {
            let value = host.store().number_value(id).unwrap_or(0.0) as f32;
            host.bus_mut()
                .push(EditorAction::BgremovalUiEdit(BgRemovalUiEdit::BrushSize(
                    value,
                )));
            true
        }
        // Falloff 4-way segmented → SetFalloff. Active highlight is the
        // per-frame snapshot; reset the clicked button's pressed state.
        WidgetEvent::Click(id)
            if id == ids::BGR_FALLOFF_SMOOTH
                || id == ids::BGR_FALLOFF_SPHERE
                || id == ids::BGR_FALLOFF_SHARP
                || id == ids::BGR_FALLOFF_CONSTANT =>
        {
            let falloff = if id == ids::BGR_FALLOFF_SPHERE {
                BrushFalloff::Sphere
            } else if id == ids::BGR_FALLOFF_SHARP {
                BrushFalloff::Sharp
            } else if id == ids::BGR_FALLOFF_CONSTANT {
                BrushFalloff::Constant
            } else {
                BrushFalloff::Smooth
            };
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            host.bus_mut()
                .push(EditorAction::BgremovalUiEdit(BgRemovalUiEdit::SetFalloff(
                    falloff,
                )));
            true
        }
        // Show-mask toggle — flips the on-canvas tint overlay.
        WidgetEvent::Click(id) if id == ids::BGR_SHOW_MASK => {
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            host.bus_mut().push(EditorAction::BgremovalUiEdit(
                BgRemovalUiEdit::ToggleShowMask,
            ));
            true
        }
        // Apply button — commit at full resolution. The shell drains
        // `Apply` into `apply_ui_edit` (arms pending_apply) and then
        // pushes `EditorAction::Bgremoval` for the active selection.
        WidgetEvent::Click(id) if id == ids::BGR_APPLY => {
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            host.bus_mut()
                .push(EditorAction::BgremovalUiEdit(BgRemovalUiEdit::Apply));
            true
        }
        // Eyedropper toggle — flip the armed state. While armed, the
        // shell samples canvas click-drags into extra colours. Reset
        // the button's pressed state like the other buttons do; the
        // armed look is driven by the per-frame snapshot in `paint`.
        WidgetEvent::Click(id) if id == ids::BGR_EYEDROPPER => {
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            host.bus_mut().push(EditorAction::BgremovalUiEdit(
                BgRemovalUiEdit::ToggleEyedropper,
            ));
            true
        }
        // Protection-brush toggle — flip the armed state. While armed,
        // the shell paints canvas click-drags into the protection mask.
        // Reset the pressed state; the armed look is the per-frame
        // snapshot in `paint`.
        WidgetEvent::Click(id) if id == ids::BGR_PROTECT => {
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            host.bus_mut().push(EditorAction::BgremovalUiEdit(
                BgRemovalUiEdit::ToggleProtectBrush,
            ));
            true
        }
        // Clear protection — wipe the painted protection mask. The tool
        // reruns the preview without any forced-keep region.
        WidgetEvent::Click(id) if id == ids::BGR_PROTECT_CLEAR => {
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            host.bus_mut().push(EditorAction::BgremovalUiEdit(
                BgRemovalUiEdit::ClearProtectMask,
            ));
            true
        }
        // Cancel — abandon the preview and deactivate the tool (shell
        // switches back to the default tool, hiding this panel and
        // restoring the Inspector).
        WidgetEvent::Click(id) if id == ids::BGR_CANCEL => {
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            host.bus_mut().push(EditorAction::BgremovalCancel);
            true
        }
        _ => false,
    }
}
