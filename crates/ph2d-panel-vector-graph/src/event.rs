//! Geometry-Graph panel `apply_event` — slider → param-bus writer.
//!
//! On `WidgetEvent::ValueChanged(id)` where `id` is one of the 8 param
//! sliders, read the slider's live `0..1` track from the store, map it to
//! the param's real range via the shared
//! [`crate::state::ParamSpec::track_to_value`] projection, and write the
//! result into the thread-local [`crate::state`] param bus (which the
//! shell render bridge reads each frame). All other events are not
//! handled.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};

use crate::state::{self, PARAMS, VectorGraphPanelState};

pub(crate) fn apply_event(
    _state: &mut VectorGraphPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(match ev {
        WidgetEvent::ValueChanged(id) => {
            // Find the matching param spec; ignore unrelated sliders.
            match PARAMS.iter().find(|s| s.id == id) {
                Some(spec) => {
                    let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
                    state::set_param(id, spec.track_to_value(track));
                    true
                }
                None => false,
            }
        }
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids;

    #[test]
    fn track_to_value_maps_rotation_full_range() {
        let rot = PARAMS
            .iter()
            .find(|s| s.id == ids::VGRAPH_ROTATION)
            .unwrap();
        assert_eq!(rot.track_to_value(0.0), 0.0);
        assert!((rot.track_to_value(1.0) - std::f32::consts::TAU).abs() < 1e-4);
    }
}
