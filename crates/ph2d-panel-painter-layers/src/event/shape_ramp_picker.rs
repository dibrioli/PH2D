//! Shape **value-ramp** interaction handlers (the B&W twin of `ramp_picker`): the `ValueChanged`
//! reactions for the position-bar + value-bar drags (both `CurvePoint`s parented on the EDIT id,
//! distinguished by channel) and the editable index / position chips. The selection is a stable stop
//! id (so it follows a stop dragged past a neighbour); the index chip resolves a typed sorted-index
//! back to that id, the position bar moves it, the value bar sets its grayscale value.

use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;

/// Route a Shape-ramp `ValueChanged`: the bar-stop drag, or a commit on the editable index / position /
/// value chip. (Called by `event.rs` for the four Shape-ramp ids only.)
pub(super) fn on_shape_ramp_value_changed(host: &mut dyn PanelHostInternal, id: NodeId) {
    if id == core_ids::PAINTER_SHAPE_RAMP_EDIT {
        // Both bars report here: the `CurvePoint` stashed `(_, channel, stop_id, x, _)`. Channel `0` =
        // the stop-position bar (move the stop), channel `1` = the value bar (set its grayscale value).
        if let Some((_p, channel, stop_id, x, _y)) = host.store_mut().take_curve_point_drag() {
            state::set_selected_shape_ramp_stop(stop_id);
            let target = if channel == 1 {
                core_ids::PAINTER_SHAPE_RAMP_STOP_VALUE
            } else {
                core_ids::PAINTER_SHAPE_RAMP_EDIT
            };
            forward(host, target, format!("{stop_id}:{x}"));
        }
    } else if id == core_ids::PAINTER_SHAPE_RAMP_STOP_INDEX {
        on_index_commit(host);
    } else if id == core_ids::PAINTER_SHAPE_RAMP_STOP_POS
        && let Some(v) = host
            .store()
            .number_value(core_ids::PAINTER_SHAPE_RAMP_STOP_POS)
    {
        let sel = state::selected_shape_ramp_stop();
        forward(
            host,
            core_ids::PAINTER_SHAPE_RAMP_EDIT,
            format!("{sel}:{}", (v as f32).clamp(0.0, 1.0)),
        );
    }
}

/// The index selector committed: select the stop at that sorted index (clamped), tracked by stable id.
fn on_index_commit(host: &mut dyn PanelHostInternal) {
    let Some(v) = host
        .store()
        .number_value(core_ids::PAINTER_SHAPE_RAMP_STOP_INDEX)
    else {
        return;
    };
    let Some(b) = state::current_brush() else {
        return;
    };
    let count = (b.shape_ramp_stop_count as usize).min(b.shape_ramp_stops.len());
    if count == 0 {
        return;
    }
    let target = (v.round() as i64).clamp(0, count as i64 - 1) as usize; // CLAMP-OK: integer (i64) bounds, no NaN; count>=1 guarded above
    state::set_selected_shape_ramp_stop(b.shape_ramp_stops[target][2] as u8);
}

fn forward(host: &mut dyn PanelHostInternal, id: NodeId, value: String) {
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            id, value,
        )));
}
