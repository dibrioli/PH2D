//! **The widget RANGE invariants** — the family of gates that keep a slider honest
//! (split from `motion_bridge_param_tests.rs` for the 700-LOC file cap). `super` is
//! `render_loop::motion_bridge`.
//!
//! A slider is two things: a value, and the SCALE the value is read against. Every bug
//! gated here is the scale misbehaving — deriving itself from the value it measures
//! (a feedback loop), being guessed because nobody declared it, or being too small to
//! hold the value it must show. Enio's smoke of 2026-07-12 hit the first of those, and
//! the sliders ran to billions.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;

/// Every node type in the registry, with its params — the sweep the older guards
/// missed by filtering on `motion.`, which is precisely why the `sim.*` nodes shipped
/// with no hints at all and every slider in the rain graph ran away.
fn every_type_and_its_params(motion: &MotionState) -> Vec<(&'static str, Vec<&'static str>)> {
    motion
        .registry
        .manifests()
        .map(|m| (m.name, m.params.iter().map(|p| p.name).collect()))
        .collect()
}

/// **A range is not allowed to be a function of the value it ranges over.**
///
/// The bug this pins (Enio, smoke 2026-07-12 — *"os sliders chegam a bilhões e não
/// arrastam linearmente"*): the hintless fallback set `max = value * 4`, so the slider's
/// SCALE grew with the value it was measuring. That is positive feedback with a fixed
/// point at a quarter of the track — drag above it and the value multiplies every frame
/// until it is astronomical; drag below it and it collapses to zero. The knob never maps
/// to a stable number, which is what "não arrasta linearmente" means from the outside.
///
/// So: put an in-range value on the param — exactly what a drag does — and the range must
/// come back IDENTICAL. (Widening to hold an out-of-range value is still allowed; it is
/// idempotent, and a drag inside the range never triggers it.)
#[test]
fn a_drag_inside_the_range_never_moves_the_range() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();

    // The range AND the face it is expressed in. ⚠️ The face is not decoration
    // here: a row's numbers are what the ARTIST reads, and `set_param` writes what
    // the DOCUMENT stores. Feeding a displayed number straight back would put
    // pixels into a metres param — the very defect the display boundary exists to
    // prevent — and this gate would then report the resulting widened range as a
    // feedback loop in the product. (It did, the first time this wave ran: the
    // gate caught its own fixture skipping the conversion, which is the gate
    // working.) Angle/Seed rows never convert, so they carry the neutral face.
    let range_of = |motion: &MotionState,
                    param: &str|
     -> Option<(f64, f64, ph2d_panel_motion_params::RowDisplay)> {
        build_params_snapshot(motion, ProjectSettings::default())?
            .rows
            .into_iter()
            .find_map(|r| match r {
                ParamRow::Scalar(s) if s.name == param => Some((s.min, s.max, s.display)),
                ParamRow::Angle(a) if a.name == param => {
                    Some((a.min_deg, a.max_deg, Default::default()))
                }
                ParamRow::Seed(s) if s.name == param => Some((s.min, s.max, Default::default())),
                _ => None,
            })
    };

    for (ty, params) in every_type_and_its_params(&motion) {
        for param in params {
            let node = motion.doc.graph.add_node(ty);
            ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
            let Some(before) = range_of(&motion, param) else {
                continue; // not a continuous row (colour / toggle / enum / text)
            };
            let (min, max, face) = before;
            // Every place the knob can land, including the ends.
            for track in [0.0f64, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
                // What the panel would emit: land the affine in the DISPLAY face,
                // then undo it — exactly the one door `events.rs` uses.
                let stored = face.to_stored(min + track * (max - min));
                motion.doc.graph.set_param(node, param, stored as f32);
                let after = range_of(&motion, param).expect("the row survives its own edit");
                assert_eq!(
                    after, before,
                    "{ty}.{param}: dragging to {track} of the track moved the range from \
                     {before:?} to {after:?} — the scale is a function of the value it \
                     measures, so the drag is a feedback loop, not a drag"
                );
            }
        }
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **No param reaches the panel without a declared range.** A node crate that registers its
/// op but forgets `register_param_ui` used to fall through to a guessed range — which is how
/// `sim.collide.shape` (Floor / Disc / Bowl) came to be painted as a float slider the artist
/// had to decode, and how the whole `sim.*` family got the runaway fallback above. The hint
/// is where a param says what it MEANS (a count, a seed, a toggle, a named choice), so a
/// missing hint is a missing decision, not a cosmetic gap.
#[test]
fn every_scalar_row_comes_from_a_declared_hint() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let mut missing: Vec<String> = Vec::new();

    for (ty, _) in every_type_and_its_params(&motion) {
        let node = motion.doc.graph.add_node(ty);
        let tid = motion.doc.graph.node(node).unwrap().type_id();
        let hints = motion.registry.param_ui(tid);
        ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
        let Some(snap) = build_params_snapshot(&motion, ProjectSettings::default()) else {
            continue;
        };
        for row in &snap.rows {
            let param = match row {
                ParamRow::Scalar(r) => r.name,
                ParamRow::Angle(r) => r.name,
                ParamRow::Seed(r) => r.name,
                _ => continue,
            };
            if hints
                .and_then(|hs| hs.iter().find(|h| h.param == param))
                .is_none()
            {
                missing.push(format!("{ty}.{param}"));
            }
        }
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    assert!(
        missing.is_empty(),
        "these params are painted with a GUESSED range — the node crate never called \
         `register_param_ui`, so nobody ever decided what they mean:\n{}",
        missing.join("\n")
    );
}

/// A param's default must be inside the range the panel paints it in. Otherwise the very
/// first frame widens the range to hold it (`contain`), and the widened bound then shrinks
/// back as the artist drags the value down — a range that chases its value, which is the
/// same class of bug as the one above, just quieter.
#[test]
fn every_param_default_is_inside_its_declared_range() {
    use ph2d_nodegraph::cook::OpResolver;
    let mut motion = MotionState::new();
    for (ty, _) in every_type_and_its_params(&motion) {
        let node = motion.doc.graph.add_node(ty);
        let tid = motion.doc.graph.node(node).unwrap().type_id();
        let manifest = motion.registry.resolve(tid).unwrap().manifest();
        let Some(hints) = motion.registry.param_ui(tid) else {
            continue;
        };
        for spec in manifest.params {
            let Some(h) = hints.iter().find(|h| h.param == spec.name) else {
                continue;
            };
            // Colour channels + free text are not painted on a numeric range.
            if !matches!(
                h.widget,
                ph2d_node_registry::ParamWidget::Slider
                    | ph2d_node_registry::ParamWidget::IntSlider
                    | ph2d_node_registry::ParamWidget::Angle
                    | ph2d_node_registry::ParamWidget::Seed
            ) {
                continue;
            }
            assert!(
                h.min <= spec.default && spec.default <= h.max,
                "{ty}.{}: default {} is outside its declared range [{}, {}]",
                spec.name,
                spec.default,
                h.min,
                h.max
            );
        }
    }
}
