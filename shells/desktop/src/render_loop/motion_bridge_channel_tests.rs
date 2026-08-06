//! **What a behaviour's magnitude MEANS on the channel it drives** — the gates for the
//! one question that ties `motion.stagger` / `motion.oscillator` / `motion.wiggle`
//! together (split from `motion_bridge_param_tests.rs` for the HR-18 LOC cap; `super` is
//! `render_loop::motion_bridge`).
//!
//! The subject is a single fact with three faces: the same param means world metres on
//! Position, DEGREES on Rotation and a bare scale factor on Size. The shell answers it in
//! three places — the reset PRESET, the widget RANGE, and (since doc 88) the display
//! UNIT — and the three live in one file so they cannot drift apart.

use super::params::{apply_channel_presets, build_params_snapshot, param_value};
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;

/// #10 consistency: switching a behaviour's channel resets its magnitude to a
/// channel-sensible default. The Rotation channel writes the `rot` stream column,
/// whose unit is **degrees** — so a stagger driving Rotation gets a ±90 ramp, not
/// the ±1 world-unit range meant for position. Non-behaviour types are untouched.
#[test]
fn channel_switch_resets_behaviour_magnitude_to_channel_defaults() {
    let mut motion = MotionState::new();
    let st = motion.doc.graph.add_node("motion.stagger");

    // -> Rotation (channel 2): a ±90 degree ramp.
    apply_channel_presets(&mut motion, st, "motion.stagger", 2.0);
    assert_eq!(param_value(&motion, st, "min"), -90.0);
    assert_eq!(param_value(&motion, st, "max"), 90.0);
    // -> Size (channel 3): ±½ scale.
    apply_channel_presets(&mut motion, st, "motion.stagger", 3.0);
    assert_eq!(param_value(&motion, st, "min"), -0.5);
    assert_eq!(param_value(&motion, st, "max"), 0.5);
    // -> back to Y (channel 1): the world-unit range returns.
    apply_channel_presets(&mut motion, st, "motion.stagger", 1.0);
    assert_eq!(param_value(&motion, st, "min"), -1.0);
    assert_eq!(param_value(&motion, st, "max"), 1.0);

    // Oscillator amplitude scales the same way (Rotation peaks at 30 degrees).
    let osc = motion.doc.graph.add_node("motion.oscillator");
    apply_channel_presets(&mut motion, osc, "motion.oscillator", 2.0);
    assert_eq!(param_value(&motion, osc, "amplitude"), 30.0);
    apply_channel_presets(&mut motion, osc, "motion.oscillator", 1.0);
    assert_eq!(param_value(&motion, osc, "amplitude"), 1.0);

    // A non-behaviour node (transform) is left alone.
    let xf = motion.doc.graph.add_node("motion.transform");
    apply_channel_presets(&mut motion, xf, "motion.transform", 2.0);
    assert_eq!(
        param_value(&motion, xf, "scale"),
        1.0,
        "transform untouched"
    );
}

/// Any param whose widget RANGE moves with the channel must have its VALUE reset
/// when the channel switches — otherwise it survives into a channel whose range
/// cannot show it. The oscillator's `offset` was exactly that hole: widened to
/// ±360 on Rotation, never reset, so a 300° offset landed in a ±10 world-unit
/// position channel. This pins preset-domain == override-domain.
#[test]
fn every_channel_ranged_param_is_reset_on_a_channel_switch() {
    let mut motion = MotionState::new();
    let osc = motion.doc.graph.add_node("motion.oscillator");

    // Dial an offset that is only legal on the Rotation channel.
    motion.doc.graph.set_param(osc, "channel", 2.0);
    motion.doc.graph.set_param(osc, "offset", 300.0);

    // Switch to a position channel: the preset must bring `offset` back in range.
    apply_channel_presets(&mut motion, osc, "motion.oscillator", 1.0);
    assert_eq!(
        param_value(&motion, osc, "offset"),
        0.0,
        "offset must reset with the channel whose range it borrowed"
    );
    assert_eq!(param_value(&motion, osc, "amplitude"), 1.0);
}

/// A behaviour's magnitude WIDGET RANGE follows the channel, not just its value.
/// The Enio caught this: with Channel=Rot the Stagger showed `Min -10 / Max 10`
/// even though the preset had written ±90 into the doc. The static hint range
/// (±10, authored for world units) could not represent ±90, so the slider
/// saturated, DISPLAYED -10, and would have overwritten the doc with -10 on the
/// first touch. On Rotation the range must be degrees-scaled and contain the
/// preset; on a position channel it stays the world-unit hint.
///
/// ⚠️ **The range assertions are made in STORE units, and that is not pedantry.**
/// Since doc 88 these behaviours declare `ParamUnit::FromChannel`, so on a position
/// channel the row is a `Length` and its numbers arrive in the artist's display unit
/// — `±1000 px` for the same `±10 m` hint. Asserting the painted numbers would make
/// this gate a hostage of `ProjectSettings::display_unit`: it would go red the day
/// somebody changed a default, while saying nothing about the channel logic it
/// exists to guard. `to_stored` is the same door `events.rs` uses on the way back.
#[test]
fn rotation_channel_widens_the_magnitude_range_to_contain_its_preset() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();

    let scalar = |motion: &MotionState, name: &str| {
        build_params_snapshot(motion, ProjectSettings::default())
            .expect("resolvable")
            .rows
            .into_iter()
            .find_map(|r| match r {
                ParamRow::Scalar(s) if s.name == name => Some(s),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no scalar row {name}"))
    };

    // Stagger on Rot: the preset writes ±90 — the range must hold it.
    let st = motion.doc.graph.add_node("motion.stagger");
    ph2d_panel_motion_graph::set_graph_selection(vec![st.0]);
    motion.doc.graph.set_param(st, "channel", 2.0);
    apply_channel_presets(&mut motion, st, "motion.stagger", 2.0);
    for name in ["min", "max"] {
        let row = scalar(&motion, name);
        assert!(
            row.min <= row.value && row.value <= row.max,
            "{name}: preset {} escapes the widget range [{}, {}]",
            row.value,
            row.min,
            row.max
        );
        assert_eq!(
            (
                row.display.to_stored(row.min),
                row.display.to_stored(row.max)
            ),
            (-360.0, 360.0),
            "{name} is degree-scaled"
        );
    }
    // Back on a position channel the world-unit hint range returns — but ONLY
    // because the preset also brings the value home. (Switch the channel without
    // the preset and `contain` correctly keeps the range wide enough to show the
    // stale ±90 rather than lie about it — that is the other half of the fix.)
    motion.doc.graph.set_param(st, "channel", 1.0);
    apply_channel_presets(&mut motion, st, "motion.stagger", 1.0);
    let back = scalar(&motion, "min");
    assert_eq!(
        (
            back.display.to_stored(back.min),
            back.display.to_stored(back.max)
        ),
        (-10.0, 10.0)
    );

    // The wave behaviours' amplitude, same story (preset 30 vs a 0..10 hint).
    for (ty, node) in [
        (
            "motion.oscillator",
            motion.doc.graph.add_node("motion.oscillator"),
        ),
        ("motion.wiggle", motion.doc.graph.add_node("motion.wiggle")),
    ] {
        ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
        motion.doc.graph.set_param(node, "channel", 2.0);
        apply_channel_presets(&mut motion, node, ty, 2.0);
        let row = scalar(&motion, "amplitude");
        assert!(
            row.min <= row.value && row.value <= row.max,
            "{ty}: amplitude preset {} escapes [{}, {}]",
            row.value,
            row.min,
            row.max
        );
        assert_eq!(row.max, 360.0, "{ty} amplitude is degree-scaled on Rot");
    }

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}
