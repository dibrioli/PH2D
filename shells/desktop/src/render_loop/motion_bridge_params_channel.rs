//! Channel-aware presets of the params bridge (a `#[path]` child of
//! `motion_bridge_params.rs`, split off for the 600-LOC shell file cap). "What a
//! behaviour's magnitude MEANS on the channel it drives" — the reset presets and the
//! Rotation option they hinge on. The widget-range WIDENING (`contain` /
//! `channel_range_override`) stays with `build_params_snapshot`, its only caller.

use crate::motion_state::MotionState;

/// Reset a behaviour node's magnitude params to a sensible default for the newly
/// selected channel (#10 consistency). Switching what a stagger/oscillator drives
/// — X/Y position (world units) vs Rotation (degrees) vs Size (scale delta) —
/// rewrites the range so a `±1` meant for position doesn't read as a barely-there
/// ±1° / ±huge-scale on the other channels. Editor UX (not node math): it
/// runs on the channel switch inside the same undo step, so Ctrl+Z restores the
/// artist's previous values. Non-behaviour node types are a no-op.
pub(in crate::render_loop::motion_bridge) fn apply_channel_presets(
    motion: &mut MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    type_name: &str,
    channel: f32,
) {
    let ch = channel.round() as i32;
    match type_name {
        "motion.stagger" => {
            let (min, max) = stagger_channel_range(ch);
            motion.doc.graph.set_param(nid, "min", min);
            motion.doc.graph.set_param(nid, "max", max);
        }
        // Both wave behaviours carry the same `channel` enum + `amplitude`.
        "motion.oscillator" | "motion.wiggle" => {
            motion
                .doc
                .graph
                .set_param(nid, "amplitude", wave_channel_amplitude(ch));
            // The oscillator's DC `offset` shares the amplitude's unit, so its
            // range is channel-dependent too (`channel_range_override`). Any param
            // whose RANGE moves with the channel must have its VALUE reset with it
            // — otherwise a 300° offset dialled on Rotation survives into a
            // ±10-world-unit position channel, outside the range it will be shown
            // in. Zero is the neutral offset and is legal on every channel.
            if type_name == "motion.oscillator" {
                motion.doc.graph.set_param(nid, "offset", 0.0);
            }
        }
        _ => {}
    }
}

/// The `channel` enum's Rotation option (see the behaviours' `channel` hint:
/// `0` X, `1` Y, `2` Rotation, `3` Size).
pub(super) const CHANNEL_ROTATION: i32 = 2;

/// Stagger `(min, max)` ramp endpoints per channel. The Rotation channel writes
/// the `rot` stream column, whose unit is **degrees** (the app's authored-angle
/// unit); Position is world units, Size a scale delta.
fn stagger_channel_range(channel: i32) -> (f32, f32) {
    match channel {
        CHANNEL_ROTATION => (-90.0, 90.0), // ±90 degrees
        3 => (-0.5, 0.5),                  // Size: ±0.5 scale
        _ => (-1.0, 1.0),                  // Position (X/Y): ±1 world unit
    }
}

/// Peak `amplitude` per channel for the wave behaviours (oscillator / wiggle) —
/// same unit logic as the stagger range.
fn wave_channel_amplitude(channel: i32) -> f32 {
    match channel {
        CHANNEL_ROTATION => 30.0, // ±30 degrees
        3 => 0.3,                 // Size: ±0.3 scale
        _ => 1.0,                 // Position: ±1 world unit
    }
}
