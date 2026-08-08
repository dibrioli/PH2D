//! The **taper widget's** drag decode (Procreate *Touch Taper*; Enio 2026-08-08). Both handles are
//! `CurvePoint`s parented on [`core_ids::PAINTER_TAPER_GIZMO`], distinguished by channel: `0` = the
//! START length (measured in from the left edge), `1` = the END length (measured in from the RIGHT).
//! The twin of [`super::dab_gizmo`], and deliberately built from the same parts.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_tool_painter::MAX_TAPER_DIAMETERS;

/// Route a `ValueChanged(PAINTER_TAPER_GIZMO)`: drain the `CurvePoint` drag and forward the decoded
/// length. The dispatch normalises the pointer to `[0,1]` across the widget, so `x` **is** the fraction
/// of the widget the handle sits at — and the widget's full width represents a stroke of
/// [`MAX_TAPER_DIAMETERS`], which is what makes the position read directly as a length.
///
/// ⚠️ The END handle measures from the RIGHT edge (`1 - x`). Dragging it left must LENGTHEN the end
/// taper — it is reaching further back into the stroke — and reading the raw `x` would invert it, a
/// control that moves the opposite way from the hand.
pub(super) fn on_taper_gizmo_value_changed(host: &mut dyn PanelHostInternal) {
    let Some((_p, channel, _idx, x, _y)) = host
        .store_mut()
        .take_curve_point_drag_if(|p| p == core_ids::PAINTER_TAPER_GIZMO)
    else {
        return;
    };
    let frac = if channel == 1 { 1.0 - x } else { x };
    let value = f64::from(frac.clamp(0.0, 1.0) * MAX_TAPER_DIAMETERS);
    let id = if channel == 1 {
        core_ids::PAINTER_TAPER_END
    } else {
        core_ids::PAINTER_TAPER_START
    };
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
            id, value,
        )));
}
