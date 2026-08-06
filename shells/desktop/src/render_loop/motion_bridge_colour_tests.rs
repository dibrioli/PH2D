//! **How a COLOUR is authored in the params panel** — the gates for the one subject
//! that ties `motion.tint` and `motion.color_array` together (split from
//! `motion_bridge_param_tests.rs` for the HR-18 LOC cap; `super` is
//! `render_loop::motion_bridge`).
//!
//! A colour reaches the artist as a SWATCH that opens the OKLCH picker, and the wire
//! carries linear-straight channels. Everything here guards that boundary: the swatch
//! is offered instead of raw channel sliders, the pick round-trips through it, and
//! merely OPENING the picker is not an edit.

use super::color::{apply_color_to_node, channel_values, linear_rgba_to_srgb8};
use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;

/// The reported-bug + colour-authoring seam, end to end and headless: a
/// selected `motion.tint` node resolves to a named Mode selector + colour
/// SWATCH rows (not raw channel sliders), the Start swatch's channels are the
/// RGBA params, and its display colour is **opaque white** — the identity
/// default that killed the red dominance. Proves the `Color`/`Enum` hints flow
/// all the way to paintable rows (registry -> snapshot builder).
#[test]
fn selected_tint_node_yields_mode_and_colour_swatch_rows() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let tint = motion.doc.graph.add_node("motion.tint");
    ph2d_panel_motion_graph::set_graph_selection(vec![tint.0]);

    let snap = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("tint node is resolvable");
    // A named Mode enum (Solid/Gradient), never a number slider.
    assert!(
        snap.rows
            .iter()
            .any(|r| matches!(r, ParamRow::Enum(e) if e.name == "mode")),
        "mode is a named Enum row"
    );
    // The Start colour is a swatch over r/g/b/a, opaque white by default.
    let start = snap
        .rows
        .iter()
        .find_map(|r| match r {
            ParamRow::Color(c) if c.channels == ["r", "g", "b", "a"] => Some(c),
            _ => None,
        })
        .expect("Start colour is a swatch, not four sliders");
    assert_eq!(start.srgb, [255, 255, 255, 255]);
    // The gradient End is its own swatch over r2/g2/b2/a2.
    assert!(
        snap.rows
            .iter()
            .any(|r| matches!(r, ParamRow::Color(c) if c.channels == ["r2", "g2", "b2", "a2"])),
        "End colour is its own swatch"
    );

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// The colour read-back is the inverse of the swatch display: writing a
/// picked sRGB colour lands linear-straight channel values on the node, and
/// re-reading them rebuilds the same sRGB swatch (round-trip stable). Guards
/// the sRGB↔linear boundary the bridge owns (the Motion wire is linear).
#[test]
fn color_pick_writes_linear_and_round_trips_to_srgb() {
    let mut motion = MotionState::new();
    let tint = motion.doc.graph.add_node("motion.tint");
    let picked = [40, 160, 220, 128]; // a saturated sRGB blue, half alpha

    apply_color_to_node(&mut motion, tint, ["r", "g", "b", "a"], picked);

    // The stored channels are linear-straight (RGB gamma-decoded, alpha /255).
    let lin = channel_values(&motion, tint, ["r", "g", "b", "a"]);
    assert!(lin[0] < lin[2], "blue channel dominates in linear too");
    assert!((lin[3] - 128.0 / 255.0).abs() < 1e-6, "alpha is straight");
    // Re-encoding the stored linear colour reproduces the pick (±1 LSB).
    let srgb = linear_rgba_to_srgb8(lin);
    for (got, want) in srgb.into_iter().zip(picked) {
        assert!(
            got.abs_diff(want) <= 1,
            "round-trip {srgb:?} ≈ {picked:?} within 1 LSB"
        );
    }
}

/// Merely OPENING a colour picker must not edit the document. The picker is seeded
/// with the swatch's 8-bit sRGB display colour and reports it straight back every
/// frame it is open; if the guard compared LINEAR values, a doc colour that is not
/// an exact 8-bit round-trip (here `0.5`) would be silently quantized — a doc edit
/// and an undo step the artist never asked for. The guard compares sRGB8, so an
/// unmoved picker is a no-op.
#[test]
fn opening_the_picker_does_not_quantize_an_unmoved_colour() {
    let mut motion = MotionState::new();
    let tint = motion.doc.graph.add_node("motion.tint");
    // A linear value that does NOT survive an 8-bit round-trip exactly.
    for name in ["r", "g", "b"] {
        motion.doc.graph.set_param(tint, name, 0.5);
    }
    let before = channel_values(&motion, tint, ["r", "g", "b", "a"]);

    // The picker reports back exactly what the swatch seeded it with.
    apply_color_to_node(
        &mut motion,
        tint,
        ["r", "g", "b", "a"],
        linear_rgba_to_srgb8(before),
    );

    assert_eq!(
        channel_values(&motion, tint, ["r", "g", "b", "a"]),
        before,
        "an unmoved picker must not rewrite the doc"
    );

    // A real pick still lands (the guard is not simply dead).
    apply_color_to_node(&mut motion, tint, ["r", "g", "b", "a"], [10, 20, 30, 255]);
    assert_ne!(channel_values(&motion, tint, ["r", "g", "b", "a"]), before);
}

/// **A palette is authored as COLOURS, and it shrinks with its own count.**
///
/// The Enio named this node as one of the two done badly: `motion.color_array` painted
/// THIRTEEN bare sliders (`C0 R` / `C0 G` / `C0 B` × 4 + the count), whose numbers are
/// LINEAR RGB — the wire's unit, not a colour anybody can picture. Now each slot is one
/// swatch that opens the OKLCH picker, exactly as `motion.tint` already did.
///
/// The second half is the one a screenshot would not show: a slot BEYOND the count is a
/// control the cook will never read (`cycle` clamps to `colors`), so it must not be
/// offered. The gate drives the count down and checks the swatches DISAPPEAR — presence
/// and absence, because a gate that only counts rows at full width passes with the
/// gating deleted.
#[test]
fn the_palette_is_four_swatches_that_shrink_with_the_colour_count() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let ca = motion.doc.graph.add_node("motion.color_array");
    ph2d_panel_motion_graph::set_graph_selection(vec![ca.0]);

    let swatches = |motion: &MotionState| -> Vec<[&'static str; 4]> {
        build_params_snapshot(motion, ProjectSettings::default())
            .expect("resolvable")
            .rows
            .into_iter()
            .filter_map(|r| match r {
                ParamRow::Color(c) => Some(c.channels),
                _ => None,
            })
            .collect()
    };
    // Full width: four swatches, each naming its own RGBA quad — and NOT one bare
    // scalar row for any colour channel (that is the defect, stated directly).
    assert_eq!(
        swatches(&motion),
        vec![
            ["c0_r", "c0_g", "c0_b", "c0_a"],
            ["c1_r", "c1_g", "c1_b", "c1_a"],
            ["c2_r", "c2_g", "c2_b", "c2_a"],
            ["c3_r", "c3_g", "c3_b", "c3_a"],
        ]
    );
    let scalars: Vec<&str> = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("resolvable")
        .rows
        .iter()
        .filter_map(|r| match r {
            ParamRow::Scalar(x) => Some(x.name),
            _ => None,
        })
        .collect();
    assert!(
        !scalars.iter().any(|n| n.starts_with("c0_")),
        "a colour channel is still a bare slider: {scalars:?}"
    );

    // The count is a whole number of slots, so it is a Seed-free INTEGER row.
    assert!(
        build_params_snapshot(&motion, ProjectSettings::default())
            .expect("resolvable")
            .rows
            .iter()
            .any(|r| matches!(r, ParamRow::Scalar(x) if x.name == "colors" && x.integer)),
        "`colors` counts slots — a fractional palette is not a thing"
    );

    // Shrink it: the slots the cycle stops reading stop being offered.
    for (count, want) in [(3.0, 3), (2.0, 2)] {
        motion.doc.graph.set_param(ca, "colors", count);
        assert_eq!(
            swatches(&motion).len(),
            want,
            "with colors={count} the panel must offer exactly {want} swatches"
        );
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}
