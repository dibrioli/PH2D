//! Geometry-Graph panel `populate` — pre-registers the 8 param sliders in
//! the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Each param is a canonical normalized `0..1` horizontal [`SliderState`]
//! (`VGRAPH_*`). The track is seeded from the param's default via the
//! shared [`crate::state::ParamSpec::value_to_track`] projection
//! (`track = (default - lo) / (hi - lo)`), so the initial slider position
//! matches the bus seed in [`crate::state`]. On `ValueChanged`,
//! [`crate::event`] maps the live track back to the real range and writes
//! the param bus.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{SliderOrientation, SliderState};

use crate::state::PARAMS;

pub fn populate(store: &mut WidgetStore) {
    for spec in PARAMS {
        store.register(
            spec.id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: spec.value_to_track(spec.default),
                orientation: SliderOrientation::Horizontal,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_registers_8_sliders_seeded_from_defaults() {
        let mut store = WidgetStore::with_capacity(16);
        populate(&mut store);
        assert_eq!(PARAMS.len(), 8, "spec table must hold exactly 8 params");
        for spec in PARAMS {
            let (_, track) = store
                .slider(spec.id)
                .unwrap_or_else(|| panic!("slider {:?} missing", spec.id));
            let expected = spec.value_to_track(spec.default);
            assert!(
                (track - expected).abs() < f32::EPSILON,
                "{}: seed track {track} != {expected}",
                spec.label
            );
        }
    }
}
