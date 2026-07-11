//! Vector Style panel event router (thin forwarder).
//!
//! Same forwarder shape as the `panel_seam!`-generated code (Padding /
//! BgRemoval / …), hand-written here so the Width slider can be **registered at
//! a non-neutral initial value** in `populate.rs` (the macro hardcodes `0.5`,
//! which would render the knob at ~10 px instead of the tool's 3 px default).
//!
//! - Width slider drag (or mirror from the chip) → `ToolPanelEvent::SetValue`
//!   carrying the live track `0..1`; the tool projects it to px.
//! - Width chip `ValueChanged` → swallowed (the dispatch mirror already fired
//!   the slider's `ValueChanged`, handled above — avoids a double notify).
//! - Fill "None" `Click` → `ToolPanelEvent::Click` (tool clears the fill).
//! - Close (X) `Click` → `CancelActiveTool` (deactivates the tool, mirror of
//!   the Padding panel's Cancel).
//!
//! The two colour swatches never reach here — their Down opens the shared OKLCH
//! picker via the generic `is_picker_swatch` dispatch (short-circuits in
//! pointer.rs); the shell's `vector_bridge` reads the pick back into the tool.

use crate::ids;
use crate::state::VectorPanelState;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal, seam_reset_button};
use ph2d_editor_core::tool::PanelEvent;

pub(crate) fn apply_event(
    _state: &mut VectorPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let consumed = match ev {
        WidgetEvent::ValueChanged(id) if id == ids::VECTOR_WIDTH => {
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    f64::from(track),
                )));
            true
        }
        // Rotation field (R) — a RELATIVE scrub: the panel owns the per-gesture
        // accumulator, so forward the DELTA since the last report (degrees). The
        // shell rotates the selected path incrementally about its bbox center.
        WidgetEvent::ValueChanged(id) if id == ids::VECTOR_TRANSFORM_R => {
            let cur = host.store().number_value(id).unwrap_or(0.0);
            let delta = cur - crate::state::rot_last();
            crate::state::set_rot_last(cur);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id, delta,
                )));
            true
        }
        // Transform number fields (X/Y/W/H) — standalone NumberInputs (NOT slider-
        // linked): forward the committed VALUE as a document command; the shell
        // drain translates (X/Y) / scales (W/H) the selected path.
        WidgetEvent::ValueChanged(id)
            if id == ids::VECTOR_TRANSFORM_X
                || id == ids::VECTOR_TRANSFORM_Y
                || id == ids::VECTOR_TRANSFORM_W
                || id == ids::VECTOR_TRANSFORM_H =>
        {
            let val = host.store().number_value(id).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, val)));
            true
        }
        // Shape-parameter sliders — same shape as Width (track 0..1 → the tool
        // projects to a side/point count, inner ratio, or radius).
        WidgetEvent::ValueChanged(id)
            if id == ids::VECTOR_SIDES
                || id == ids::VECTOR_STAR_POINTS
                || id == ids::VECTOR_STAR_INNER
                || id == ids::VECTOR_RRECT_RADIUS
                || id == ids::VECTOR_SPIRAL_TURNS
                || id == ids::VECTOR_ARC_DEGREES
                || id == ids::VECTOR_STROKE_OPACITY
                || id == ids::VECTOR_FILL_OPACITY
                || id == ids::VECTOR_DASH
                || id == ids::VECTOR_GAP
                || id == ids::VECTOR_GRAD_ANGLE
                || id == ids::VECTOR_GRAD_INFLUENCE
                || id == ids::VECTOR_GRAD_JITTER =>
        {
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    f64::from(track),
                )));
            true
        }
        // Chip edits already mirrored to their slider (which fires its own
        // ValueChanged, handled above): swallow to avoid a double notify.
        WidgetEvent::ValueChanged(id)
            if id == ids::VECTOR_WIDTH_NUM
                || id == ids::VECTOR_SIDES_NUM
                || id == ids::VECTOR_STAR_POINTS_NUM
                || id == ids::VECTOR_STAR_INNER_NUM
                || id == ids::VECTOR_RRECT_RADIUS_NUM
                || id == ids::VECTOR_SPIRAL_TURNS_NUM
                || id == ids::VECTOR_STROKE_OPACITY_NUM
                || id == ids::VECTOR_FILL_OPACITY_NUM
                || id == ids::VECTOR_DASH_NUM
                || id == ids::VECTOR_GAP_NUM
                || id == ids::VECTOR_GRAD_ANGLE_NUM
                || id == ids::VECTOR_GRAD_INFLUENCE_NUM
                || id == ids::VECTOR_GRAD_JITTER_NUM =>
        {
            true
        }
        // Draw-mode buttons + Boolean buttons: forward the Click over the generic
        // tool channel. Mode clicks land on `VectorTool::handle_panel_event`
        // (sets the mode); Boolean clicks are picked up by the shell drain, which
        // applies the op to the document (they are not Style edits, so the tool
        // ignores them). Same forwarder shape either way.
        WidgetEvent::Click(id)
            if id == ids::VECTOR_MODE_SELECT
                || id == ids::VECTOR_MODE_NODE
                || id == ids::VECTOR_MODE_PEN
                || id == ids::VECTOR_MODE_RECT
                || id == ids::VECTOR_MODE_ELLIPSE
                || id == ids::VECTOR_MODE_POLYGON
                || id == ids::VECTOR_MODE_STAR
                || id == ids::VECTOR_MODE_RRECT
                || id == ids::VECTOR_MODE_SPIRAL
                || id == ids::VECTOR_MODE_LINE
                || id == ids::VECTOR_MODE_ARC
                || id == ids::VECTOR_CAP_BUTT
                || id == ids::VECTOR_CAP_ROUND
                || id == ids::VECTOR_CAP_SQUARE
                || id == ids::VECTOR_JOIN_MITER
                || id == ids::VECTOR_JOIN_ROUND
                || id == ids::VECTOR_JOIN_BEVEL
                || id == ids::VECTOR_VERT_CORNER
                || id == ids::VECTOR_VERT_SMOOTH
                || id == ids::VECTOR_VERT_SYMMETRIC
                || id == ids::VECTOR_VERT_DELETE
                || id == ids::VECTOR_BOOL_UNION
                || id == ids::VECTOR_BOOL_SUBTRACT
                || id == ids::VECTOR_BOOL_INTERSECT
                || id == ids::VECTOR_BOOL_EXCLUDE
                || id == ids::VECTOR_COMPOUND_MAKE
                || id == ids::VECTOR_COMPOUND_RELEASE
                || id == ids::VECTOR_FILL_RULE_NONZERO
                || id == ids::VECTOR_FILL_RULE_EVENODD
                || id == ids::VECTOR_SNAP_OFF
                || id == ids::VECTOR_SNAP_ON
                || id == ids::VECTOR_ARRANGE_DUPLICATE
                || id == ids::VECTOR_ARRANGE_TO_BACK
                || id == ids::VECTOR_ARRANGE_BACKWARD
                || id == ids::VECTOR_ARRANGE_FORWARD
                || id == ids::VECTOR_ARRANGE_TO_FRONT
                || id == ids::VECTOR_ARRANGE_FLIP_H
                || id == ids::VECTOR_ARRANGE_FLIP_V
                || id == ids::VECTOR_ARRANGE_ROTATE_CW
                || id == ids::VECTOR_ARRANGE_ROTATE_CCW
                || id == ids::VECTOR_PATH_SMOOTH
                || id == ids::VECTOR_PATH_SHARPEN
                || id == ids::VECTOR_PATH_SIMPLIFY
                || id == ids::VECTOR_PATH_SUBDIVIDE
                || id == ids::VECTOR_PATH_CLOSE
                || id == ids::VECTOR_FILL_KIND_SOLID
                || id == ids::VECTOR_FILL_KIND_LINEAR
                || id == ids::VECTOR_FILL_KIND_RADIAL
                || id == ids::VECTOR_FILL_KIND_MULTI
                || id == ids::VECTOR_GRAD_ADD_POINT
                || id == ids::VECTOR_GRAD_REMOVE_POINT
                || id == ids::VECTOR_GRAD_ADD_STOP
                || id == ids::VECTOR_GRAD_REMOVE_STOP
                || id == ids::VECTOR_ALIGN_LEFT
                || id == ids::VECTOR_ALIGN_HCENTER
                || id == ids::VECTOR_ALIGN_RIGHT
                || id == ids::VECTOR_ALIGN_TOP
                || id == ids::VECTOR_ALIGN_VCENTER
                || id == ids::VECTOR_ALIGN_BOTTOM
                || id == ids::VECTOR_DISTRIBUTE_H
                || id == ids::VECTOR_DISTRIBUTE_V
                || id == ids::VECTOR_PIVOT_EDIT =>
        {
            seam_reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        WidgetEvent::Click(id) if id == ids::VECTOR_CLOSE => {
            seam_reset_button(host, id);
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        _ => false,
    };
    EventOutcome::from_bool(consumed)
}
