//! The special `ValueChanged` forwards split out of `event.rs` (file-LOC
//! cap): the two colour-ramp editors, the flatten/rotate dab gizmo and the
//! Wet Paint TILT pad. `route` returns whether it consumed the id.

use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;

use super::{dab_gizmo, ramp_picker, shape_ramp_picker};

pub(super) fn route(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) -> bool {
    // Grain Color Ramp: bar-stop drag + the editable index / position chips.
    if core_ids::PAINTER_BRUSH_TEXTURE_RAMP_VALUE_IDS.contains(&id) {
        ramp_picker::on_ramp_value_changed(host, id);
        return true;
    }
    // Shape Color ramp: same editors, the Shape slot.
    if core_ids::PAINTER_SHAPE_RAMP_VALUE_IDS.contains(&id) {
        shape_ramp_picker::on_shape_ramp_value_changed(host, id);
        return true;
    }
    // Flatten/rotate gizmo: a handle `CurvePoint` drag → decode.
    if id == core_ids::PAINTER_BRUSH_DAB_GIZMO {
        dab_gizmo::on_dab_gizmo_value_changed(host);
        return true;
    }
    // Wet Paint TILT pad (doc 22): drain + snap + forward ring/spoke (the
    // body lives beside the dial's paint — one conversion, one house).
    if id == core_ids::PAINTER_WETPAINT_TILT_PAD {
        crate::paint_wetpaint_tilt::forward_tilt_pad_drag(host);
        return true;
    }
    false
}
