//! Headless bridge tests (split out of `motion_bridge.rs` for the HR-18 LOC
//! cap). Declared by the parent as a `#[path]` sibling, so `super` is
//! `render_loop::motion_bridge`; the param-authoring helpers it exercises live
//! in the sibling `params` submodule.

use super::params::{
    apply_channel_presets, apply_color_to_node, build_params_snapshot, channel_values,
    linear_rgba_to_srgb8, param_value,
};
use super::*;
use crate::motion_state::MotionState;

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

    let snap = build_params_snapshot(&motion).expect("tint node is resolvable");
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

/// The Behaviours seam, cooked through the REAL registry (not a unit-test
/// stub): `grid -> stagger -> oscillator` is well-typed (validate passes — the
/// nodes are registered and their ports match) and cooks end to end, and the
/// behaviours actually displace the grid. This is the "isolamento órfão"
/// antidote — a node can be unit-green yet unregistered / mistyped in the
/// live pipeline; this proves it is wired.
#[test]
fn grid_stagger_oscillator_cook_through_the_real_registry() {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::{Edge, Graph};

    let motion = MotionState::new(); // registry = register_all_nodes
    let cook_p = |g: &Graph, target| {
        let mut cook = Cook::new();
        let out = cook.cook(g, &motion.registry, target, 0.25).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    };

    // Bare grid (baseline) vs grid -> stagger(Y) -> oscillator(Y).
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let stagger = g.add_node("motion.stagger");
    let osc = g.add_node("motion.oscillator");
    g.connect(Edge {
        from: (grid, 0),
        to: (stagger, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (stagger, 0),
        to: (osc, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(stagger, "channel", 1.0);
    g.set_param(stagger, "min", 0.0);
    g.set_param(stagger, "max", 2.0);
    g.set_param(osc, "channel", 1.0);
    g.set_param(osc, "amplitude", 1.0);
    g.set_param(osc, "phase_stagger", 0.0); // uniform bob -> +amplitude at t=¼

    // The whole chain type-checks against the real registry.
    g.validate(&motion.registry)
        .expect("grid -> stagger -> oscillator is well-typed");

    let base = cook_p(&g, grid);
    let out = cook_p(&g, osc);
    assert_eq!(out.len(), base.len(), "count preserved through behaviours");
    assert!(base.len() >= 4, "grid emits its default cells");
    let n = base.len();
    for (i, (b, o)) in base.iter().zip(&out).enumerate() {
        // X untouched; Y = base + stagger ramp (i/(n-1)·2) + oscillator (+1).
        let ramp = 2.0 * i as f32 / (n as f32 - 1.0);
        assert!((o[0] - b[0]).abs() < 1e-4, "X untouched at {i}");
        assert!(
            (o[1] - (b[1] + ramp + 1.0)).abs() < 1e-4,
            "Y = base + ramp + osc at {i}"
        );
    }
}

/// A behaviour's enum / boolean params resolve to NAMED widget rows, not
/// number sliders: the selected stagger node yields an `Enum` Channel row
/// (X/Y/Rot/Size), an `Enum` Easing row, and a `Toggle` Reverse row — the
/// exact fix the Enio asked for (no memorising slider steps).
#[test]
fn stagger_params_are_named_enums_and_a_checkbox() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let st = motion.doc.graph.add_node("motion.stagger");
    ph2d_panel_motion_graph::set_graph_selection(vec![st.0]);

    let snap = build_params_snapshot(&motion).expect("stagger resolvable");
    let channel = snap
        .rows
        .iter()
        .find_map(|r| match r {
            ParamRow::Enum(e) if e.name == "channel" => Some(e),
            _ => None,
        })
        .expect("channel is a named Enum row, not a slider");
    assert_eq!(channel.labels, ["X", "Y", "Rot", "Size"]);
    let ease = snap
        .rows
        .iter()
        .find_map(|r| match r {
            ParamRow::Enum(e) if e.name == "ease_curve" => Some(e),
            _ => None,
        })
        .expect("ease_curve is a named Enum row");
    // The rich curve family set (Penner minus the transcendental ones).
    assert!(ease.labels.contains(&"Bounce") && ease.labels.contains(&"Back"));
    assert!(
        snap.rows
            .iter()
            .any(|r| matches!(r, ParamRow::Enum(e) if e.name == "ease_dir")),
        "ease_dir (In/Out/In-Out) is its own named Enum row"
    );
    assert!(
        snap.rows
            .iter()
            .any(|r| matches!(r, ParamRow::Toggle(t) if t.name == "reverse")),
        "reverse is a checkbox (Toggle) row, not a 0/1 slider"
    );
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

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

/// **The invariant, over every registered Motion node and every param:** a row's
/// widget range CONTAINS its value. A row that violates it is a lying widget —
/// the track clamps, the panel paints the clamped number, and the first touch
/// writes it back, destroying the authored value.
///
/// `Graph::set_param` never clamps to the hint, so a preset, an undo, or a loaded
/// document can put any value on any param. This drives every node type with a
/// value far outside its hint (both signs) and asserts the row still contains it.
/// It is the gate for the whole bug class, not for one node.
#[test]
fn every_row_range_contains_its_value_for_every_node_and_param() {
    use ph2d_nodegraph::cook::OpResolver;
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();

    // Every registered motion node type (the real registry, not a stub list).
    let types: Vec<&'static str> = motion
        .registry
        .manifests()
        .map(|m| m.name)
        .filter(|n| n.starts_with("motion."))
        .collect();
    assert!(
        types.len() >= 10,
        "the registry really has the motion nodes"
    );

    for ty in types {
        for extreme in [-9999.0f32, 9999.0] {
            let node = motion.doc.graph.add_node(ty);
            // Shove the extreme onto EVERY declared param of this node.
            let params: Vec<&'static str> = motion
                .registry
                .resolve(motion.doc.graph.node(node).unwrap().type_id())
                .unwrap()
                .manifest()
                .params
                .iter()
                .map(|p| p.name)
                .collect();
            for p in params {
                motion.doc.graph.set_param(node, p, extreme);
            }
            ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);

            let snap = build_params_snapshot(&motion)
                .unwrap_or_else(|| panic!("{ty} must resolve a snapshot"));
            for row in &snap.rows {
                let (name, value, min, max) = match row {
                    ParamRow::Scalar(r) => (r.name, r.value, r.min, r.max),
                    ParamRow::Angle(r) => (r.name, r.deg, r.min_deg, r.max_deg),
                    ParamRow::Seed(r) => (r.name, r.value, r.min, r.max),
                    // Color / Toggle / Enum carry no continuous range.
                    _ => continue,
                };
                assert!(
                    min <= value && value <= max,
                    "{ty}.{name}: value {value} escapes the widget range [{min}, {max}] \
                     -> the panel would paint a clamped number and destroy it on touch"
                );
            }
        }
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
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
#[test]
fn rotation_channel_widens_the_magnitude_range_to_contain_its_preset() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();

    let scalar = |motion: &MotionState, name: &str| {
        build_params_snapshot(motion)
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
            (row.min, row.max),
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
    assert_eq!(
        (scalar(&motion, "min").min, scalar(&motion, "min").max),
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

/// Angle params resolve to a `deg` number-box row, and the row is degrees end to
/// end — the param stores exactly what the box shows. `motion.rotate` (which adds
/// to the `rot` column) and `motion.orbit` (whose trig is cycle-based) both
/// author in the SAME unit; radians and turns exist nowhere on this surface.
#[test]
fn angle_params_resolve_to_degree_rows() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();

    let angle_row = |motion: &MotionState, who: &str| {
        build_params_snapshot(motion)
            .expect("node resolvable")
            .rows
            .into_iter()
            .find_map(|r| match r {
                ParamRow::Angle(a) if a.name == "angle" => Some(a),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{who} has no Angle row"))
    };

    // motion.rotate feeds the `rot` column: a full-circle range in degrees.
    let rot = motion.doc.graph.add_node("motion.rotate");
    ph2d_panel_motion_graph::set_graph_selection(vec![rot.0]);
    let a = angle_row(&motion, "rotate");
    assert_eq!(
        (a.min_deg, a.max_deg),
        (-180.0, 180.0),
        "degrees, not radians"
    );
    assert_eq!(a.deg, 0.0, "default 0 deg");

    // motion.orbit's polar angle: the same unit, a wider range.
    let orbit = motion.doc.graph.add_node("motion.orbit");
    ph2d_panel_motion_graph::set_graph_selection(vec![orbit.0]);
    let a = angle_row(&motion, "orbit");
    assert_eq!(
        (a.min_deg, a.max_deg),
        (-360.0, 360.0),
        "degrees, not turns"
    );

    // Setting 90 deg on the doc reads back as 90 in the row — no conversion.
    motion.doc.graph.set_param(orbit, "angle", 90.0);
    assert_eq!(angle_row(&motion, "orbit").deg, 90.0);

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// A Seed param resolves to a Seed row (whole-number box + re-roll button), never
/// a slider the artist must drag through a range that means nothing.
#[test]
fn seed_param_resolves_to_a_seed_row_not_a_slider() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let wig = motion.doc.graph.add_node("motion.wiggle");
    ph2d_panel_motion_graph::set_graph_selection(vec![wig.0]);

    let snap = build_params_snapshot(&motion).expect("wiggle resolvable");
    let seed = snap
        .rows
        .iter()
        .find_map(|r| match r {
            ParamRow::Seed(s) => Some(s),
            _ => None,
        })
        .expect("seed is a Seed row");
    assert_eq!(seed.name, "seed");
    assert!(seed.min < seed.max, "the seed box has a usable range");
    assert!(
        !snap
            .rows
            .iter()
            .any(|r| matches!(r, ParamRow::Scalar(s) if s.name == "seed")),
        "seed must not ALSO appear as a scalar slider"
    );

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// The animation enabler (ask "when do we see animation?"): playing advances
/// the playhead — so any `Temporal` behaviour moves — and pausing freezes it.
/// The default transport is paused, which is why nothing moved before.
#[test]
fn transport_play_advances_time_and_pause_freezes_it() {
    let mut motion = MotionState::new();
    let dt = 1.0 / 60.0;
    assert_eq!(motion.playhead(dt), 0.0, "starts paused at t=0");
    motion.transport.play();
    motion.transport.advance(30);
    let t = motion.playhead(dt);
    assert!(
        t > 0.0,
        "playing advances the playhead -> behaviours animate"
    );
    motion.transport.toggle(); // -> paused
    motion.transport.advance(30);
    assert_eq!(motion.playhead(dt), t, "paused freezes the playhead");
}

/// The #1->#2 producer/consumer seam through the REAL registry: the grid's
/// `Index`/`Count` identity columns drive the tint's Gradient mode, so a grid
/// reads as a colour ramp. A 1×3 grid + gradient Start=white/End=black ->
/// tints white->grey->black across the row.
#[test]
fn grid_index_drives_the_tint_gradient() {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::{Edge, Graph};

    let motion = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let tint = g.add_node("motion.tint");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 3.0); // 3 cells -> Index 0,1,2 / Count 3
    g.set_param(tint, "mode", 1.0); // Gradient (white->black defaults)
    g.connect(Edge {
        from: (grid, 0),
        to: (tint, 0),
        delayed: false,
    })
    .unwrap();
    g.validate(&motion.registry)
        .expect("grid -> tint is well-typed");
    let mut cook = Cook::new();
    let out = cook.cook(&g, &motion.registry, tint, 0.0).unwrap();
    match out[0].as_stream().get("tint").unwrap() {
        Column::Vec4(v) => {
            assert_eq!(v.len(), 3);
            assert_eq!(v[0], [1.0, 1.0, 1.0, 1.0], "index 0 -> Start (white)");
            assert_eq!(v[1], [0.5, 0.5, 0.5, 1.0], "index 1 -> mid grey");
            assert_eq!(v[2], [0.0, 0.0, 0.0, 1.0], "index 2 -> End (black)");
        }
        _ => panic!("tint"),
    }
}

/// The Output node IS the render sink: the bridge auto-selects it, cooking it
/// draws whatever feeds it, and deleting it stops the render (no Output -> no
/// sink -> empty). The output follows the graph, not a hidden toggle.
#[test]
fn output_node_is_the_render_sink() {
    use ph2d_nodegraph::graph::{Edge, Graph};
    let mut motion = MotionState::new();
    let (uv, size) = (motion.default_uv_rect, motion.default_size);

    // Fresh graph: grid -> Output.
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (grid, 0),
        to: (out, 0),
        delayed: false,
    })
    .unwrap();
    motion.doc.graph = g;

    // The bridge resolves the sink to the Output node…
    let sink = output_node(&motion.doc.graph);
    assert_eq!(sink, Some(out), "the Output node is the render sink");
    // …and cooking it draws the grid cells feeding it.
    motion.pump.mark_dirty();
    motion
        .pump
        .pump(&motion.doc.graph, &motion.registry, sink, 0, 0.0, uv, size);
    assert!(
        motion.pump.instances.len() >= 4,
        "Output renders whatever feeds it"
    );

    // Delete the Output node -> no sink -> nothing renders.
    assert!(motion.doc.graph.remove_node(out));
    let sink = output_node(&motion.doc.graph);
    assert_eq!(sink, None, "no Output node -> no sink");
    motion.pump.mark_dirty();
    motion
        .pump
        .pump(&motion.doc.graph, &motion.registry, sink, 0, 0.0, uv, size);
    assert_eq!(motion.pump.instances.len(), 0, "no Output -> empty render");
}
