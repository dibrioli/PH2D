//! Headless tests for the **param rows** the bridge builds (split for the HR-18
//! LOC cap). Declared by the parent as a `#[path]` sibling, so `super` is
//! `render_loop::motion_bridge` and the param-authoring helpers are in the
//! sibling `params` submodule.
//!
//! Their common theme is the class of bug the Enio caught: a widget that cannot
//! represent its value paints a clamped number and destroys the real one on the
//! first touch. `every_row_range_contains_its_value_for_every_node_and_param` is
//! the gate for that whole class.

use super::color::{apply_color_to_node, channel_values, linear_rgba_to_srgb8};
use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
// The display face a row is built in. `default()` is what the APP ships
// (Pixels, 100 px/m), so a fixture reads the same numbers the artist does.
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

/// A selected `motion.expression` resolves to a **Formula** text row that carries the
/// graph's text-param value and sits FIRST (the formula is the node's primary control).
/// Proves the `ParamWidget::Text` hint flows through the additive text channel to a
/// paintable row (docs/Motion Nodes/33). FALSIFIED if the text param were dropped (an
/// empty field) or never surfaced (no Text row).
#[test]
fn selected_expression_node_yields_a_formula_text_row() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let ex = motion.doc.graph.add_node("motion.expression");
    motion.doc.graph.set_text_param(ex, "expr", "sin(t) * a");
    ph2d_panel_motion_graph::set_graph_selection(vec![ex.0]);

    let snap = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("expression node is resolvable");
    match &snap.rows[0] {
        ParamRow::Text(t) => {
            assert_eq!(t.name, "expr");
            assert_eq!(
                t.value, "sin(t) * a",
                "the formula flows from the text channel"
            );
        }
        other => panic!("first row should be the Formula text field, got {other:?}"),
    }
    // The a..d coefficients remain scalar rows below the formula.
    assert!(
        snap.rows
            .iter()
            .any(|r| matches!(r, ParamRow::Scalar(s) if s.name == "a")),
        "the coefficient params remain scalar rows"
    );

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// A selected `field.remap` resolves to an interactive **Curve** row (A1-ui), FIRST, that
/// carries the graph's `curve` text-param — not a raw text field. Proves the
/// `ParamWidget::Curve` hint surfaces as the draggable editor; FALSIFIED if it fell back
/// to a Text row (the A1-core interim) or were dropped. The Contour selector + the scalar
/// knobs remain below it.
#[test]
fn selected_field_remap_yields_an_interactive_curve_row() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let rm = motion.doc.graph.add_node("field.remap");
    motion
        .doc
        .graph
        .set_text_param(rm, "curve", "c1 0:0:L 0.5:1:S 1:0:L");
    ph2d_panel_motion_graph::set_graph_selection(vec![rm.0]);

    let snap = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("field.remap node is resolvable");
    match &snap.rows[0] {
        ParamRow::Curve(c) => {
            assert_eq!(c.name, "curve");
            assert_eq!(
                c.value, "c1 0:0:L 0.5:1:S 1:0:L",
                "the curve flows from the text channel to the editor"
            );
        }
        other => panic!("first row should be the interactive Curve editor, got {other:?}"),
    }
    // The Contour enum + the scalar knobs remain below the curve.
    assert!(
        snap.rows
            .iter()
            .any(|r| matches!(r, ParamRow::Enum(e) if e.name == "contour")),
        "the Contour selector remains an enum row"
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

/// A behaviour's enum / boolean params resolve to NAMED widget rows, not
/// number sliders: the selected stagger node yields an `Enum` Channel row
/// (X/Y/Rotation/Size — one vocabulary across the whole family, audit
/// 2026-07-10), an `Enum` Easing row, and a `Toggle` Reverse row — the
/// exact fix the Enio asked for (no memorising slider steps).
#[test]
fn stagger_params_are_named_enums_and_a_checkbox() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let st = motion.doc.graph.add_node("motion.stagger");
    ph2d_panel_motion_graph::set_graph_selection(vec![st.0]);

    let snap =
        build_params_snapshot(&motion, ProjectSettings::default()).expect("stagger resolvable");
    let channel = snap
        .rows
        .iter()
        .find_map(|r| match r {
            ParamRow::Enum(e) if e.name == "channel" => Some(e),
            _ => None,
        })
        .expect("channel is a named Enum row, not a slider");
    assert_eq!(channel.labels, ["X", "Y", "Rotation", "Size"]);
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

    // EVERY registered node type — the real registry, not a stub list, and no prefix
    // filter. This used to keep only `motion.*`, which quietly excluded `sim.*`,
    // `value.*`, `force.*` and `pulse.*` from a gate whose name promised "every node
    // and param" — so the entire `sim.*` family shipped with no hints and a runaway
    // slider range, under a green test. A filter inside a gate is a hole in it.
    let types: Vec<&'static str> = motion.registry.manifests().map(|m| m.name).collect();
    assert!(
        types.len() >= 80,
        "the registry really has the node library"
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

            let snap = build_params_snapshot(&motion, ProjectSettings::default())
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

/// Angle params resolve to a `deg` number-box row, and the row is degrees end to
/// end — the param stores exactly what the box shows. `motion.rotate` (which adds
/// to the `rot` column) and `motion.orbit` (whose trig is cycle-based) both
/// author in the SAME unit; radians and turns exist nowhere on this surface.
#[test]
fn angle_params_resolve_to_degree_rows() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();

    let angle_row = |motion: &MotionState, who: &str| {
        build_params_snapshot(motion, ProjectSettings::default())
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

    let snap =
        build_params_snapshot(&motion, ProjectSettings::default()).expect("wiggle resolvable");
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

/// **The rename + re-tint UI exists** (F2): with a BACKDROP selected the params
/// panel stops showing node params and shows the backdrop's own — a Title text box
/// and the 8-tint picker, whose labels name the hue ramp the tokens actually walk.
/// A backdrop is not a node (no manifest, never cooks), so without this branch it
/// would be unnameable: created, and then stuck as "Group" forever.
/// FALSIFIED if the panel fell through to the node path (`None`, an empty panel).
#[test]
fn a_selected_backdrop_yields_its_title_and_colour_rows() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    super::backdrops::add(&mut motion, 0.0, 0.0, 300.0, 200.0);
    let id = motion.doc.backdrops[0].id;
    super::backdrops::set_title(&mut motion, id, "Force chain".to_string());
    super::backdrops::set_color(&mut motion, id, 4);
    ph2d_panel_motion_graph::set_graph_backdrop_selection(Some(id));

    let snap = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("the backdrop is the subject");
    assert_eq!(snap.node, id);
    match &snap.rows[0] {
        ParamRow::Text(t) => {
            assert_eq!(t.name, "title");
            assert_eq!(t.value, "Force chain", "the box opens on the current name");
        }
        other => panic!("the first row should be the Title box, got {other:?}"),
    }
    match &snap.rows[1] {
        ParamRow::Enum(e) => {
            assert_eq!(e.name, "color");
            assert_eq!(e.selected, 4, "the picker opens on the current tint");
            assert_eq!(e.labels.len(), 8, "one label per `graph-backdrop-*` token");
        }
        other => panic!("the second row should be the Color picker, got {other:?}"),
    }

    ph2d_panel_motion_graph::set_graph_backdrop_selection(None);
}

/// And the subjects are mutually exclusive: a selected NODE still shows node
/// params (the backdrop branch cannot hijack the panel once a node is picked).
#[test]
fn a_selected_node_still_wins_the_params_panel() {
    let mut motion = MotionState::new();
    super::backdrops::add(&mut motion, 0.0, 0.0, 300.0, 200.0);
    let grid = motion.doc.graph.add_node("motion.grid");
    ph2d_panel_motion_graph::set_graph_backdrop_selection(None);
    ph2d_panel_motion_graph::set_graph_selection(vec![grid.0]);

    let snap = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("the node is the subject");
    assert_eq!(snap.node, grid.0);
    assert_eq!(snap.title, "Grid");

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
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

/// **A look-at can be told WHERE to look, and the picker only shows where it means
/// something.**
///
/// The other node the Enio named: `motion.look_at` could aim anywhere and could not
/// aim at anything the artist can NAME or at the cursor — the two things the node is
/// for in every other tool. Wiring two `value.*` nodes to follow the mouse is not a
/// workaround, it is impossible, because the cursor is not in the graph.
///
/// The gate is presence AND absence: an object picker offered in Point or Cursor mode
/// is a control the cook will never read, which is the dead row this codebase keeps one
/// table per menu to prevent.
#[test]
fn the_look_at_picks_its_target_and_offers_the_picker_only_in_object_mode() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let la = motion.doc.graph.add_node("motion.look_at");
    ph2d_panel_motion_graph::set_graph_selection(vec![la.0]);

    let rows = |motion: &MotionState| {
        build_params_snapshot(motion, ProjectSettings::default())
            .expect("resolvable")
            .rows
    };
    // The mode is a NAMED choice, not an index the artist has to decode.
    let modes = rows(&motion)
        .into_iter()
        .find_map(|r| match r {
            ParamRow::Enum(e) if e.name == "mode" => Some(e.labels),
            _ => None,
        })
        .expect("`Aim At` is a named selector");
    assert_eq!(modes, ["Point", "Object", "Cursor"]);

    // The offset is DEGREES and says so — an `Angle` row, which no unit table can
    // contradict (doc 88).
    assert!(
        rows(&motion)
            .iter()
            .any(|r| matches!(r, ParamRow::Angle(a) if a.name == "offset")),
        "the offset is an angle, not a bare number"
    );

    let has_picker = |motion: &MotionState| {
        rows(motion)
            .iter()
            .any(|r| matches!(r, ParamRow::Source(s) if s.param == "target"))
    };
    // Default (Point): no object to name.
    assert!(!has_picker(&motion), "Point aims at the value inputs");
    motion.doc.graph.set_param(la, "mode", 1.0);
    assert!(has_picker(&motion), "Object mode offers the name picker");
    motion.doc.graph.set_param(la, "mode", 2.0);
    assert!(!has_picker(&motion), "the Cursor is not an object name");
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}
