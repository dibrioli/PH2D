//! The **taper widget's** drag decode (Procreate *Touch Taper*; Enio 2026-08-08). The handle is a
//! `CurvePoint` parented on [`core_ids::PAINTER_TAPER_GIZMO`], channel `0` = the START length, measured
//! in from the left edge. The twin of [`super::dab_gizmo`], and deliberately built from the same parts.
//!
//! ⛔ Channel `1` was the END length, measured in from the RIGHT (`1 - x`, so dragging it left
//! LENGTHENED the taper). It went with the far end (Enio 2026-08-10); see `ph2d_painter_brush::taper`.
//! The channel is still read and still has to be `0`: a drag on a stale id must not be decoded as a head
//! length just because the head is the only length left.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_tool_painter::MAX_TAPER_DIAMETERS;

/// Route a `ValueChanged(PAINTER_TAPER_GIZMO)`: drain the `CurvePoint` drag and forward the decoded
/// length. The dispatch normalises the pointer to `[0,1]` across the widget, so `x` **is** the fraction
/// of the track the handle sits at — and the track represents a stroke of [`MAX_TAPER_DIAMETERS`], which
/// is what makes the position read directly as a length.
pub(super) fn on_taper_gizmo_value_changed(host: &mut dyn PanelHostInternal) {
    let Some((_p, channel, _idx, x, _y)) = host
        .store_mut()
        .take_curve_point_drag_if(|p| p == core_ids::PAINTER_TAPER_GIZMO)
    else {
        return;
    };
    if channel != 0 {
        return;
    }
    let value = f64::from(x.clamp(0.0, 1.0) * MAX_TAPER_DIAMETERS);
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
            core_ids::PAINTER_TAPER_START,
            value,
        )));
}
