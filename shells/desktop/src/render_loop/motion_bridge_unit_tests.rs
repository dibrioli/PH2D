//! **The display FACE of a param row** (doc 88, Wave A) — the gates for the one
//! boundary where a stored quantity becomes the number the artist reads.
//!
//! Declared by the parent as a `#[path]` sibling, so `super` is
//! `render_loop::motion_bridge` and the builder is `super::params`.
//!
//! The class of bug these exist for is not "the suffix is wrong". It is the one
//! the params bridge's `contain()` already fights, arriving through a second
//! door: **a row whose range does not contain its value is a lying widget** —
//! the panel paints the clamped number instead of the real one, and the first
//! touch writes that clamped number back. Converting SOME of a row's numbers and
//! not the others produces exactly that, silently, on the first param a node
//! declares. So the gates below never check a suffix alone; every one of them
//! re-asserts the ordering of the whole tuple.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::{DisplayUnit, ProjectSettings};
use ph2d_panel_motion_params::{ParamRow, ScalarRow};

/// Select a fresh node of `ty` and return its scalar row named `param`.
fn scalar_row(ty: &str, param: &str, project: ProjectSettings) -> ScalarRow {
    let mut motion = MotionState::new();
    let node = motion.doc.graph.add_node(ty);
    ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
    build_params_snapshot(&motion, project)
        .expect("the node is resolvable")
        .rows
        .into_iter()
        .find_map(|r| match r {
            ParamRow::Scalar(s) if s.name == param => Some(s),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{ty} has no scalar row `{param}`"))
}

/// Compare two row numbers.
///
/// ⚠️ **Not `assert_eq!`, and the reason is the `f32` hints.** A `ParamUiHint`
/// stores `0.1f32`, which widened to `f64` is `0.100000001490116…`; scaled by
/// `100` it is `10.000000149…`. That is the arithmetic being CORRECT, not a
/// defect, and pinning the exact bits would make these gates fail every time a
/// hint's literal is retyped. The tolerance is `1e-6` relative — six orders of
/// magnitude tighter than any conversion mistake this file can make, because a
/// missed or doubled `pixels_per_meter` is a factor of **100**, never a ulp.
#[track_caller]
fn close(got: f64, want: f64, what: &str) {
    let tol = 1e-6 * want.abs().max(1.0);
    assert!(
        (got - want).abs() <= tol,
        "{what}: got {got}, want {want} (tolerance {tol})"
    );
}

/// The invariant EVERY row must satisfy, whatever face it wears. A conversion
/// that reaches some fields and not others breaks this ordering, which is what
/// makes it the honest assertion rather than a field census.
#[track_caller]
fn assert_row_is_orderly(row: &ScalarRow) {
    assert!(
        row.hard_min <= row.min,
        "{}: hard floor {} sits ABOVE the slider start {} — the box could not \
         type a value the slider can reach",
        row.name,
        row.hard_min,
        row.min
    );
    assert!(
        row.min <= row.value && row.value <= row.max,
        "{}: the range [{}, {}] does not contain the value {} — this is the \
         lying widget `contain()` exists to prevent",
        row.name,
        row.min,
        row.max,
        row.value
    );
    assert!(
        row.max <= row.hard_max,
        "{}: hard ceiling {} sits BELOW the slider end {}",
        row.name,
        row.hard_max,
        row.max
    );
    assert!(
        row.step > 0.0,
        "{}: a step of {} is not a step",
        row.name,
        row.step
    );
}

/// **A world LENGTH is shown in the project's unit, and the WHOLE tuple moves.**
///
/// `motion.emitter`'s `x` is the emitter origin: it becomes the `P` column, which
/// `lower_to_instances` reads into `RenderInstance::world_pos` — world metres.
/// With the shipped defaults (Pixels, 100 px/m) the artist reads pixels, so a
/// slider authored for `±10 m` has to read `±1000 px` and step by `10`, not by
/// `0.1`. A tuple half-converted is a slider that steps a hundredth of a pixel.
#[test]
fn a_length_row_is_shown_in_pixels_and_the_whole_tuple_moves_with_it() {
    let p = ProjectSettings::default();
    assert_eq!(p.display_unit, DisplayUnit::Pixels, "the shipped default");
    assert_eq!(p.pixels_per_meter, 100.0);

    let row = scalar_row("motion.emitter", "x", p);
    assert_row_is_orderly(&row);

    assert_eq!(row.display.suffix, "px");
    assert_eq!(row.display.scale, 100.0);
    // The hint is `-10.0 ..= 10.0`, step `0.1`, and the manifest default is 0.
    close(row.min, -1000.0, "min");
    close(row.max, 1000.0, "max");
    close(row.step, 10.0, "step");
    close(row.value, 0.0, "value");
    // No hard limits declared for `x` ⇒ they equal the soft ones, converted too.
    close(row.hard_min, -1000.0, "hard_min");
    close(row.hard_max, 1000.0, "hard_max");
}

/// The `size` row travels the same road — `RenderInstance::size` is documented as
/// *local meters*, so `0.15 m` reads as **15 px**. It is here because it is the
/// row an artist actually stares at, and because a second Length on the same node
/// proves the declaration table is being read per-param, not per-node.
#[test]
fn the_particle_size_reads_in_pixels_too() {
    let row = scalar_row("motion.emitter", "size", ProjectSettings::default());
    assert_row_is_orderly(&row);
    assert_eq!(row.display.suffix, "px");
    close(row.value, 15.0, "0.15 m is 15 px");
    close(row.min, 1.0, "0.01 m is 1 px");
}

/// **The face follows the PROJECT, never the node.**
///
/// This is the reason `ParamUnit` has no `Px`/`Meters` variants at all: a node
/// that could name its own face would be overriding a setting the artist owns.
/// Same node, same param, other project ⇒ metres, and the scale collapses to the
/// exact `1.0` that makes the whole conversion a no-op.
#[test]
fn the_display_face_comes_from_the_project_not_the_node() {
    let metres = ProjectSettings {
        display_unit: DisplayUnit::Meters,
        ..ProjectSettings::default()
    };
    let row = scalar_row("motion.emitter", "x", metres);
    assert_row_is_orderly(&row);
    assert_eq!(row.display.suffix, "m");
    assert_eq!(row.display.scale, 1.0);
    close(row.min, -10.0, "the hint's own number, untouched");
    close(row.max, 10.0, "max");
    close(row.step, 0.1, "step");
}

/// **A duration wears its suffix and is NOT scaled** — seconds are stored in the
/// unit they are shown in, so `pixels_per_meter` must never reach them. The same
/// row carries the FLOOR, which is the half of the soft/hard split that did not
/// exist before doc 88: the drag range starts at a fountain's `0.1 s` and the box
/// reaches a spark's milliseconds.
#[test]
fn a_seconds_row_wears_its_suffix_and_reaches_below_its_slider() {
    let row = scalar_row("motion.emitter", "life", ProjectSettings::default());
    assert_row_is_orderly(&row);
    assert_eq!(row.display.suffix, "s");
    assert_eq!(row.display.scale, 1.0, "a second is not a length");
    close(row.min, 0.1, "where the drag starts");
    close(row.hard_min, 0.001, "where typing reaches");
    assert!(
        row.hard_min < row.min,
        "a floor equal to the slider start is not a floor"
    );
}

/// **The control.** Every param that declared nothing is byte-identical to the
/// world before this wave: neutral scale, no suffix, the hint's own numbers. If
/// this fails, the wave moved something it had no business moving.
#[test]
fn an_undeclared_row_is_untouched_by_the_display_boundary() {
    let row = scalar_row("motion.emitter", "rate", ProjectSettings::default());
    assert_row_is_orderly(&row);
    assert_eq!(row.display.suffix, "");
    assert_eq!(row.display.scale, 1.0);
    assert_eq!(row.min, 0.0);
    assert_eq!(row.max, 12_000.0);
    assert_eq!(row.step, 1.0);
    assert_eq!(row.hard_max, 4_000_000.0, "the ceiling still reaches");
}

/// **`speed` is unitless ON PURPOSE, and that is a claim worth pinning.**
///
/// It is metres per second. Declaring it `Length` would scale it by
/// `pixels_per_meter` and print `px` on a rate — a number that is wrong AND
/// labelled, which teaches the artist something false. A bare number is honest.
/// The day a `Velocity` unit exists, this gate is the thing that says so out
/// loud instead of the omission being mistaken for an oversight.
#[test]
fn a_velocity_is_left_unitless_rather_than_mislabelled_a_length() {
    let row = scalar_row("motion.emitter", "speed", ProjectSettings::default());
    assert_eq!(row.display.suffix, "");
    assert_eq!(row.display.scale, 1.0);
}

/// **A declared unit names a param that EXISTS, and reaches a widget that can wear it.**
///
/// `ParamUnitDecl` is keyed by a `&'static str`, so a typo is not a compile error — it is
/// a declaration that matches nothing, is never consulted, and leaves the row bare while
/// the node's source reads as if the unit had been declared. That is the quietest possible
/// failure for this whole wave, and it is the one a sweep across forty crates produces.
///
/// The second half is the same fact from the other side: `unit_of` lets the WIDGET answer
/// first, so a declaration on an `Angle`, a `Seed`, a `Toggle`, an `Enum` or any non-numeric
/// widget is **overridden by construction**. It would be inert too — and, worse, it would
/// read as an intent that the panel silently refuses. A unit belongs on a plain `Slider`.
#[test]
fn every_declared_unit_names_a_real_param_on_a_widget_that_can_wear_it() {
    use ph2d_node_registry::{ParamUnit, ParamWidget, unit_of};
    use ph2d_nodegraph::cook::OpResolver;
    let mut motion = MotionState::new();
    let mut bad: Vec<String> = Vec::new();
    let mut declared = 0usize;

    let types: Vec<&'static str> = motion.registry.manifests().map(|m| m.name).collect();
    for ty in types {
        let node = motion.doc.graph.add_node(ty);
        let tid = motion.doc.graph.node(node).unwrap().type_id();
        let manifest = motion.registry.resolve(tid).unwrap().manifest();
        let hints = motion.registry.param_ui(tid);
        for spec in manifest.params {
            let Some(u) = motion.registry.param_unit_declared(tid, spec.name) else {
                continue;
            };
            declared += 1;
            let widget = hints
                .and_then(|hs| hs.iter().find(|h| h.param == spec.name))
                .map_or(ParamWidget::Slider, |h| h.widget);
            if unit_of(widget, Some(u)) != u {
                bad.push(format!(
                    "{ty}.{}: declared {u:?} but the {widget:?} widget answers \
                     {:?} — the declaration is inert",
                    spec.name,
                    unit_of(widget, Some(u))
                ));
            }
        }
        // The other direction: a declaration whose `param` matches no `ParamSpec` at all.
        // `param_unit_declared` can only be asked about names the manifest HAS, so the
        // orphan is invisible from there — it has to be counted against the table.
        let names: Vec<&str> = manifest.params.iter().map(|p| p.name).collect();
        let live = names
            .iter()
            .filter(|n| motion.registry.param_unit_declared(tid, n).is_some())
            .count();
        let total = motion.registry.param_units(tid).map_or(0, <[_]>::len);
        if live != total {
            bad.push(format!(
                "{ty}: {total} units declared but only {live} name a real param — \
                 the rest are typos that match nothing (params are {names:?})"
            ));
        }
    }
    assert!(
        declared > 0,
        "positive control: no node declares a unit at all, so this gate proved nothing"
    );
    assert!(bad.is_empty(), "{}", bad.join("\n"));
    // A cheap floor on the sweep itself: the emitter alone declares four, so a number
    // this size can only come from the per-node opt-in having actually happened.
    assert!(
        declared > 50,
        "only {declared} params carry a unit — the sweep across the node crates is gone"
    );
    let grid = motion.doc.graph.add_node("motion.grid");
    let grid_id = motion.doc.graph.node(grid).unwrap().type_id();
    assert_eq!(
        motion.registry.param_unit_declared(grid_id, "gap_x"),
        Some(ParamUnit::Length),
        "a world distance on a layout node is the case this wave exists for"
    );
}

/// **No param of a channel-driven node may be declared a fixed `Length`.**
///
/// A behaviour's magnitude means metres on Position, DEGREES on Rotation and a
/// bare scale factor on Size — the shell already carries `channel_range_override`
/// because that difference bit once, and the bug it left behind (`±90` shown as
/// `±10`) is the same shape as scaling degrees by `pixels_per_meter` would be,
/// only a hundred times larger.
///
/// A node that owns a `channel` param must therefore say `FromChannel`, which
/// resolves per-channel, or say nothing. This is a census over the LIVE registry,
/// so a node type added tomorrow is covered without anyone listing it.
#[test]
fn no_param_of_a_channel_driven_node_is_declared_a_fixed_length() {
    use ph2d_node_registry::ParamUnit;
    let motion = MotionState::new();
    let mut checked = 0usize;
    for m in motion.registry.manifests() {
        if !m.params.iter().any(|p| p.name == "channel") {
            continue;
        }
        checked += 1;
        for spec in m.params {
            let declared = motion.registry.param_unit_declared(m.id, spec.name);
            assert_ne!(
                declared,
                Some(ParamUnit::Length),
                "{}::{} is a fixed Length on a node whose magnitudes depend on \
                 `channel` — on Rotation that scales DEGREES by pixels_per_meter",
                m.name,
                spec.name
            );
        }
    }
    assert!(
        checked > 0,
        "positive control: the registry has no channel-driven node at all, so \
         this gate proved nothing"
    );
}
