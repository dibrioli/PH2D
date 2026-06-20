//! Behavioral SEAM test for the Geometry-Graph (vector-graph) panel.
//!
//! This panel has NO tool and pushes NO bus action: on
//! `WidgetEvent::ValueChanged(slider)` it maps the slider's `0..1` track
//! through `ParamSpec::track_to_value` and writes the real value into a
//! thread-local "param bus" via `crate::state::set_param(..)`. The shell
//! render bridge reads that bus each frame via `current_graph_params()` to
//! cook + render the `vector.source` node.
//!
//! Unit tests in `event.rs`/`state.rs` exercise `track_to_value` and
//! `set_param` in isolation — but NEITHER proves the wire between them is
//! intact. A forgotten `event.rs` arm, a missing `PARAMS` entry, or a wrong
//! projection all leave the slider painted, draggable and SILENTLY DEAD
//! while every unit test stays green.
//!
//! This test runs the full path the desktop shell runs, headless:
//!   populate → set slider value → apply_event → track_to_value → bus →
//!   current_graph_params() → assert the real param value changed.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_panel_vector_graph::state::VectorGraphPanelState;
use ph2d_panel_vector_graph::{VectorGraphPanel, current_graph_params, ids};
use ph2d_ui_testkit::MockPanelHost;

/// Drag the Rotation slider to its full-positive end and prove the projected
/// real value (≈ TAU) lands in the thread-local param bus — exercising every
/// site from `populate` through `apply_event`/`track_to_value`/`set_param` to
/// the public `current_graph_params()` reader the shell bridge calls.
#[test]
fn rotation_slider_drag_reaches_param_bus() {
    let mut host = MockPanelHost::with_panel::<VectorGraphPanel>();
    let mut panel_state = VectorGraphPanelState;

    // Precondition: the bus reader must NOT already equal the target, or the
    // post-assertion would be vacuous. Default rotation is 0.0 (≠ TAU).
    let initial = current_graph_params().rotation;
    assert!(
        (initial - std::f32::consts::TAU).abs() > 1e-4,
        "precondition: initial rotation {initial} already equals the target — assertion would be vacuous"
    );

    // A drag writes the slider's stored value, then the dispatch emits
    // ValueChanged. Simulate both. Track 1.0 maps to the param's `hi` = TAU.
    host.set_slider_value(ids::VGRAPH_ROTATION, 1.0);
    let outcome = host.apply_panel_event::<VectorGraphPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::VGRAPH_ROTATION),
    );

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored a real slider edit — `event.rs` PARAMS lookup for VGRAPH_ROTATION is missing"
    );

    // End-to-end proof: the public reader the shell render bridge polls each
    // frame now returns the projected real value. This is precisely what the
    // isolated unit tests cannot prove.
    let rotation = current_graph_params().rotation;
    assert!(
        (rotation - std::f32::consts::TAU).abs() < 1e-4,
        "slider→thread-local-param seam delivered the wrong rotation: got {rotation}, expected ≈ {}",
        std::f32::consts::TAU
    );
}
