//! Flatten/rotate **gizmo** interaction (Procreate Shape panel; Enio 2026-06-26). Both handles are
//! `CurvePoint`s parented on [`core_ids::PAINTER_BRUSH_DAB_GIZMO`], distinguished by channel: `0` =
//! flatten (the radial distance from the centre → `1 - dist`, rim = round, centre = flat), `1` =
//! rotation (the screen angle from the centre). Decoded here and forwarded as `SetValue` to the tool's
//! `set_brush_dab_flatten` / `set_brush_dab_angle`.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_tool_painter::TEX_ANGLE_MAX_DEG;

/// Route a `ValueChanged(PAINTER_BRUSH_DAB_GIZMO)`: drain the `CurvePoint` drag and forward the decoded
/// flatten (channel 0) or rotation (channel 1). The dispatch normalises the pointer in `[0,1]` within
/// the (square) gizmo canvas, y-UP (canvas top = `1.0`), centre at `(0.5, 0.5)`.
pub(super) fn on_dab_gizmo_value_changed(host: &mut dyn PanelHostInternal) {
    let Some((_p, channel, _idx, x, y)) = host
        .store_mut()
        .take_curve_point_drag_if(|p| p == core_ids::PAINTER_BRUSH_DAB_GIZMO)
    else {
        return;
    };
    let (dx, dy) = (x - 0.5, y - 0.5);
    let (id, value) = if channel == 1 {
        // Rotation: screen angle (dispatch y is up-inverted, so screen-down = −dy), wrapped to [0,360).
        let full = f64::from(TEX_ANGLE_MAX_DEG);
        let deg = f64::from((-dy).atan2(dx)).to_degrees().rem_euclid(full);
        (core_ids::PAINTER_BRUSH_DAB_ANGLE, deg)
    } else {
        // Flatten: radial distance (0.5 = the rim) → 1 - dist, clamped.
        let dist = f64::from((dx * dx + dy * dy).sqrt()) / 0.5;
        (
            core_ids::PAINTER_BRUSH_DAB_FLATTEN,
            (1.0 - dist).clamp(0.0, 1.0),
        )
    };
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
            id, value,
        )));
}
