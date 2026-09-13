//! Color Equalization panel `apply_event` — the `WidgetEvent →
//! ToolPanelEvent` router. THE single chokepoint that turns dispatch
//! outputs into the tool's typed [`PanelEvent`] variants (forwarded
//! via [`EditorAction::ToolPanelEvent`]).
//!
//! Add a new control here whenever you wire one in [`crate::populate`]
//! alongside [`crate::paint`]. Without an arm here the dispatcher
//! updates the `WidgetStore` (slider knob moves, chip number changes)
//! but the tool's `handle_panel_event` never sees it and the pipeline
//! never re-runs.

use crate::ColorEqualizationPanelState;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ButtonState;

/// Pair a row's widget id (slider OR chip) → its slider id, so the host
/// always receives `PanelEvent::SetValue(slider_id, track)` with the
/// canonical slider NodeId, no matter whether the user dragged the
/// slider knob or scrubbed/typed the linked chip. Tool's
/// `apply_ui_edit` then maps `slider_id → typed UI edit` in one place.
///
/// MUST cover every slider/chip pair registered in [`crate::populate`].
/// An id not in this table silently never updates params on edit.
fn slider_for_widget(id: NodeId) -> Option<NodeId> {
    // Pairs aligned with `populate::rows` so an add-a-slider PR only
    // touches populate + paint + this table (and the tool's
    // `handle_panel_event` + `apply_ui_edit`).
    let pairs: &[(NodeId, NodeId)] = &[
        (
            ph2d_tool_color_equalization::ids::CEQ_CLIP_LIMIT,
            ph2d_tool_color_equalization::ids::CEQ_CLIP_LIMIT_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_TILE_GRID,
            ph2d_tool_color_equalization::ids::CEQ_TILE_GRID_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_EXPOSURE,
            ph2d_tool_color_equalization::ids::CEQ_EXPOSURE_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_TEMPERATURE,
            ph2d_tool_color_equalization::ids::CEQ_TEMPERATURE_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_TINT,
            ph2d_tool_color_equalization::ids::CEQ_TINT_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_BRIGHTNESS,
            ph2d_tool_color_equalization::ids::CEQ_BRIGHTNESS_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_CONTRAST,
            ph2d_tool_color_equalization::ids::CEQ_CONTRAST_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_VIBRANCE,
            ph2d_tool_color_equalization::ids::CEQ_VIBRANCE_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_SATURATION,
            ph2d_tool_color_equalization::ids::CEQ_SATURATION_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_SHARPEN_AMOUNT,
            ph2d_tool_color_equalization::ids::CEQ_SHARPEN_AMOUNT_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_SHARPEN_RADIUS,
            ph2d_tool_color_equalization::ids::CEQ_SHARPEN_RADIUS_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_LUT_INTENSITY,
            ph2d_tool_color_equalization::ids::CEQ_LUT_INTENSITY_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_LUT_MIX,
            ph2d_tool_color_equalization::ids::CEQ_LUT_MIX_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_POSTERIZE_DITHER_STRENGTH,
            ph2d_tool_color_equalization::ids::CEQ_POSTERIZE_DITHER_STRENGTH_NUM,
        ),
        (
            ph2d_tool_color_equalization::ids::CEQ_POSTERIZE_DITHER_GRAIN,
            ph2d_tool_color_equalization::ids::CEQ_POSTERIZE_DITHER_GRAIN_NUM,
        ),
    ];
    pairs
        .iter()
        .find_map(|&(slider, chip)| (id == slider || id == chip).then_some(slider))
}

/// `true` iff `id` belongs to ANY of the panel's dropdown option lists
/// (LUT 1/2 presets, Posterize levels, Quantize colours). Used by the
/// click arm below to forward the option click as
/// `PanelEvent::Click(option_id)` — the tool resolves the option NodeId
/// → typed edit via the parallel `CEQ_*_OPTS` arrays in
/// `handle_panel_event`.
fn is_dropdown_option(id: NodeId) -> bool {
    ph2d_tool_color_equalization::ids::CEQ_LUT_1_OPTS.contains(&id)
        || ph2d_tool_color_equalization::ids::CEQ_LUT_2_OPTS.contains(&id)
        || ph2d_tool_color_equalization::ids::CEQ_POSTERIZE_OPTS.contains(&id)
        || ph2d_tool_color_equalization::ids::CEQ_QUANTIZE_OPTS.contains(&id)
}

/// All buttons + toggles that need to forward `Click` (and `Toggle`,
/// for the toggleable buttons) to the tool. Cancel is handled
/// separately because it raises `CancelActiveTool` instead of a tool
/// panel event. Option clicks are forwarded by `is_dropdown_option`
/// above.
const FORWARD_CLICK_IDS: &[NodeId] = &[
    ph2d_tool_color_equalization::ids::CEQ_APPLY,
    ph2d_tool_color_equalization::ids::CEQ_RESET,
    ph2d_tool_color_equalization::ids::CEQ_AUTO_LEVELS,
    ph2d_tool_color_equalization::ids::CEQ_AUTO_CONTRAST,
    ph2d_tool_color_equalization::ids::CEQ_AUTO_COLORS,
    ph2d_tool_color_equalization::ids::CEQ_AUTO_WB,
    ph2d_tool_color_equalization::ids::CEQ_POSTERIZE_DITHERING,
];

pub(crate) fn apply_event(
    _state: &mut ColorEqualizationPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Slider value changed (drag, OR mirror from chip commit /
        // stepper / continuous-hold tick / drag-scrub via
        // `link_slider_number_mapped`). Forward as the canonical
        // `slider_id`-keyed `SetValue` so the tool's `apply_ui_edit`
        // does the 0..1 → natural unit projection in ONE place.
        WidgetEvent::ValueChanged(id) if slider_for_widget(id) == Some(id) => {
            let track = host.store().store_or_default_slider_value(id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    track as f64,
                )));
            true
        }
        // Chip ValueChanged — every mapped-link dispatch site
        // (`commit_number_buffer`, `apply_number_stepper_if_hit`,
        // `dispatch_tick`, drag-scrub) ALSO emits the slider's
        // ValueChanged after writing the mirrored slider value (see
        // dispatch/pointer.rs:295+918, mod.rs:265+, tick.rs:79+).
        // Swallow the chip event here so the tool isn't double-
        // notified for the same edit (audit finding #1, lens A —
        // pre-fix CEQ tool received SetValue twice per chip edit).
        WidgetEvent::ValueChanged(id) if slider_for_widget(id).is_some() => true,
        // Dropdown OPTION click. The chip itself toggles
        // `InteractiveState::Dropdown.open` via the shared dispatch
        // handler in `focus::apply_click` (no event emitted) — we
        // only need to route OPTION clicks here. The tool's
        // `handle_panel_event` resolves the option NodeId → typed
        // SetLutPreset / SetPosterizeLevels / SetQuantizeColors via
        // the parallel `CEQ_*_OPTS` arrays and stages a one-shot
        // close (`pending_close_lut_dropdown`) the shell bridge
        // drains into the matching dropdown's `open = false`.
        WidgetEvent::Click(id) if is_dropdown_option(id) => {
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Forward every button+toggle in `FORWARD_CLICK_IDS`. Reset
        // the ButtonState to Normal so the next paint sees the
        // canonical "not pressed" look (otherwise the press visual
        // sticks across the consumption boundary on some affordances).
        WidgetEvent::Click(id) if FORWARD_CLICK_IDS.contains(&id) => {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // `Toggled` parallels `Click` for buttons whose
        // `InteractiveState::Toggle` flip is observed instead of
        // emitting a Click (only fires for ids registered as
        // `Toggle`, which is currently no CEQ control — kept here as
        // future-proofing for when auto-* migrate from Button to
        // Toggle InteractiveState).
        WidgetEvent::Toggled(id) if FORWARD_CLICK_IDS.contains(&id) => {
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Toggle(id, false)));
            true
        }
        WidgetEvent::Click(id) if id == ph2d_tool_color_equalization::ids::CEQ_CANCEL => {
            reset_button(host, id);
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        _ => false,
    }
}

fn reset_button(host: &mut dyn PanelHostInternal, id: NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}

/// Helper trait extension to read a slider's `value` (or `0.0` when the
/// slider is missing from the store — keeps the call site one-liner-y
/// without an `Option` dance). Inlined here so the only `pub` surface
/// from this module remains `apply_event`.
trait StoreSliderValueExt {
    fn store_or_default_slider_value(&self, slider_id: NodeId) -> f32;
}

impl StoreSliderValueExt for ph2d_editor_core::interaction::WidgetStore {
    fn store_or_default_slider_value(&self, slider_id: NodeId) -> f32 {
        self.slider(slider_id).map(|(_, v)| v).unwrap_or(0.0)
    }
}
