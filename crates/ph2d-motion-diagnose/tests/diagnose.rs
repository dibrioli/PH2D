//! ADR-0155 gates for the setup diagnoser. Every gate drives the REAL registry
//! (`register_all_nodes`) so the couplings under test are the ones the app ships,
//! not a fixture's private copy.

use ph2d_motion_diagnose::{Deficit, Fix, canonical_consumer, diagnose};
use ph2d_node_registry::{Coupling, NodeRegistry};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;

/// The full runtime registry — the real couplings, not a fixture's.
fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("register all nodes");
    reg
}

/// Wire a straight chain `a -> b -> c -> ...` on output/input port 0, returning
/// the node ids in order.
fn chain(g: &mut Graph, types: &[&str]) -> Vec<NodeId> {
    let ids: Vec<NodeId> = types.iter().map(|t| g.add_node(*t)).collect();
    for w in ids.windows(2) {
        g.connect(Edge {
            from: (w[0], 0),
            to: (w[1], 0),
            delayed: false,
        })
        .expect("connect");
    }
    ids
}

/// **The star: a force wired toward the sink with no integrator is diagnosed
/// inert, and the cure is to insert `motion.integrate`.** This is the Enio's
/// example. FALSIFIED by a `diagnose` that ignores `Produces("accel")` (no
/// diagnostic) or by dropping `force.wind`'s coupling (nothing to detect).
#[test]
fn a_force_with_no_integrator_is_diagnosed_inert() {
    let reg = registry();
    let mut g = Graph::new();
    let ids = chain(&mut g, &["motion.grid", "force.wind", "motion.output"]);

    let ds = diagnose(&g, &reg);
    assert_eq!(ds.len(), 1, "exactly the force is inert");
    assert_eq!(ds[0].node, ids[1], "the force.wind node");
    assert_eq!(ds[0].deficit, Deficit::InertProducer("accel"));
    assert_eq!(
        ds[0].fix,
        Fix::Insert("motion.integrate"),
        "auto-heal: insert the integrator the artist forgot"
    );
}

/// **The negative control — the same chain WITH an integrator diagnoses clean.**
/// Proves the gate above is not vacuously green. FALSIFIED by a `consumer_reachable`
/// that never returns true (every healthy graph would be flagged).
#[test]
fn a_healthy_chain_is_diagnosed_clean() {
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let force = g.add_node("force.wind");
    let integ = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");
    // grid feeds the integrator's rest (port 0) and the force (port 0); the force
    // feeds the integrator's forces port (1); the integrator feeds the output.
    for (a, ap, b, bp) in [
        (grid, 0, force, 0),
        (grid, 0, integ, 0),
        (force, 0, integ, 1),
        (integ, 0, out, 0),
    ] {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed: false,
        })
        .expect("connect");
    }
    assert!(
        diagnose(&g, &reg).is_empty(),
        "a force with an integrator downstream is healthy"
    );
}

/// **A force placed DOWNSTREAM of the integrator is a REORDER, not a second
/// insert.** The integrator already consumed `accel` before the force wrote it, so
/// the force is inert — but the cure is to move it upstream, never to add a second
/// integrator (one integrator applies). FALSIFIED by a `diagnose` that returns
/// `Fix::Insert` whenever a producer is inert (it would suggest a second one).
#[test]
fn a_force_downstream_of_the_integrator_is_a_reorder_not_a_second_insert() {
    let reg = registry();
    let mut g = Graph::new();
    let ids = chain(
        &mut g,
        &[
            "motion.grid",
            "motion.integrate",
            "force.wind",
            "motion.output",
        ],
    );

    let ds = diagnose(&g, &reg);
    assert_eq!(ds.len(), 1, "only the misplaced force");
    assert_eq!(
        ds[0].node, ids[2],
        "the force.wind, downstream of integrate"
    );
    assert_eq!(ds[0].deficit, Deficit::InertProducer("accel"));
    assert_eq!(
        ds[0].fix,
        Fix::Reorder,
        "an integrator exists off-path -> reorder, never a 2nd insert"
    );
}

/// **A `sim.spawn` chain heals with `sim.step`, not `motion.integrate`.** The
/// disambiguator picks the particle integrator when a spawn feeds the force.
/// FALSIFIED by a `canonical_consumer` that ignores the particle context.
#[test]
fn a_particle_chain_heals_with_sim_step() {
    let reg = registry();
    let mut g = Graph::new();
    let ids = chain(&mut g, &["sim.spawn", "force.wind", "motion.output"]);

    let ds = diagnose(&g, &reg);
    assert_eq!(ds.len(), 1, "the force in the particle chain");
    assert_eq!(ds[0].node, ids[1]);
    assert_eq!(ds[0].fix, Fix::Insert("sim.step"));
    // and the door itself:
    assert_eq!(canonical_consumer("accel", true), Some("sim.step"));
    assert_eq!(canonical_consumer("accel", false), Some("motion.integrate"));
}

/// **A pin with no solver is an OFFER, never an auto-insert.** `inv_mass` has more
/// than one reasonable consumer (integrate / sim.step / spring / collide), so the
/// diagnoser refuses to guess. FALSIFIED by a `canonical_consumer` that returns a
/// solver for `inv_mass` (it would auto-heal an ambiguous case).
#[test]
fn a_pin_with_no_solver_is_an_offer_not_a_guess() {
    let reg = registry();
    let mut g = Graph::new();
    let ids = chain(
        &mut g,
        &["motion.grid", "motion.pin_constraint", "motion.output"],
    );

    let ds = diagnose(&g, &reg);
    assert_eq!(ds.len(), 1);
    assert_eq!(ds[0].node, ids[1], "the pin");
    assert_eq!(ds[0].deficit, Deficit::InertProducer("inv_mass"));
    assert_eq!(
        ds[0].fix,
        Fix::Offer,
        "no canonical solver -> offer, don't guess"
    );
    assert_eq!(canonical_consumer("inv_mass", false), None);
}

/// **Every producer this wave annotates declares its coupling, and every solver
/// declares its consume.** A force that ships without `Produces("accel")` would be
/// undiagnosable — this gate is what keeps a new force from being born inert-blind.
/// FALSIFIED by dropping any of the annotations added in W1.
#[test]
fn every_producer_and_consumer_declares_its_coupling() {
    let reg = registry();

    let produces: &[(&str, &str)] = &[
        ("force.attractor", "accel"),
        ("force.vortex", "accel"),
        ("force.wind", "accel"),
        ("force.curl", "accel"),
        ("force.drag", "accel"),
        ("force.buoyancy", "accel"),
        ("motion.pin_constraint", "inv_mass"),
    ];
    for &(name, col) in produces {
        let cs = reg
            .couplings(NodeTypeId::of(name))
            .unwrap_or_else(|| panic!("{name} has no couplings"));
        assert!(
            cs.iter()
                .any(|c| matches!(c, Coupling::Produces(x) if *x == col)),
            "{name} must Produce {col}"
        );
    }

    let consumes: &[(&str, &str)] = &[
        ("motion.integrate", "accel"),
        ("sim.step", "accel"),
        ("motion.integrate", "inv_mass"),
        ("sim.step", "inv_mass"),
        ("motion.spring", "inv_mass"),
        ("motion.collide", "inv_mass"),
    ];
    for &(name, col) in consumes {
        let cs = reg
            .couplings(NodeTypeId::of(name))
            .unwrap_or_else(|| panic!("{name} has no couplings"));
        assert!(
            cs.iter()
                .any(|c| matches!(c, Coupling::Consumes(x) if *x == col)),
            "{name} must Consume {col}"
        );
    }
}

/// **A field wired to nothing is an inert `falloff` OFFER — derived, with ZERO
/// annotation on the field.** `field.box` declares a `falloff` `Write` binding for
/// the GPU cook; the diagnoser reads that same declaration, so the whole falloff
/// family is covered without a hand-written `Coupling`. The cure is an offer (a
/// field needs *some* force/deformer — there is no canonical one to insert).
/// FALSIFIED by a `diagnose` that only reads the `Coupling` side-channel (field.box
/// has none → no diagnostic) — i.e. by dropping the derived GPU-binding half.
#[test]
fn a_field_wired_to_nothing_is_an_inert_falloff_offer() {
    let reg = registry();
    let mut g = Graph::new();
    let ids = chain(&mut g, &["motion.grid", "field.box", "motion.output"]);

    let ds = diagnose(&g, &reg);
    assert_eq!(ds.len(), 1, "exactly the field is inert");
    assert_eq!(ds[0].node, ids[1], "the field.box node");
    assert_eq!(ds[0].deficit, Deficit::InertProducer("falloff"));
    assert_eq!(
        ds[0].fix,
        Fix::Offer,
        "a field needs *a* force/deformer — no canonical inserter, so offer"
    );
}

/// **A field READ by a force is healthy — the derived consume half.** `field.box`
/// writes `falloff`; `force.wind` declares a `falloff` `Read` binding, so the
/// diagnoser sees the field as consumed and leaves it alone. The force is still
/// separately inert (its `accel` has no integrator), which is the ONLY diagnostic:
/// the field is healthy purely because the force reads its falloff, with no
/// annotation on either node. FALSIFIED by a diagnoser that cannot see the force's
/// derived `falloff` consume (the field would be wrongly flagged).
#[test]
fn a_field_read_by_a_force_is_a_healthy_falloff() {
    let reg = registry();
    let mut g = Graph::new();
    let ids = chain(
        &mut g,
        &["motion.grid", "field.box", "force.wind", "motion.output"],
    );

    let ds = diagnose(&g, &reg);
    // The field is NOT among the inert — its falloff is read by the force.
    assert!(
        !ds.iter().any(|d| d.node == ids[1]),
        "field.box is healthy: force.wind reads its falloff"
    );
    // The only diagnostic is the force's own accel (no integrator) — the control.
    assert_eq!(ds.len(), 1, "only the force's accel is inert");
    assert_eq!(ds[0].node, ids[2], "the force.wind node");
    assert_eq!(ds[0].deficit, Deficit::InertProducer("accel"));
}

/// **Two forces with no integrator are BOTH inert — the re-producer rule.** A force
/// declares `accel` as `ReadWrite`: it reads the running sum AND re-writes it. That
/// makes it a producer, NOT a consumer, so the second force does not "save" the
/// first — the pair is dead without an integrator. This is the load-bearing
/// subtlety of the derived rule. FALSIFIED by treating any `reads()` as a consume
/// (the second force would then rescue the first: one diagnostic, not two).
#[test]
fn two_forces_with_no_integrator_are_both_inert() {
    let reg = registry();
    let mut g = Graph::new();
    let ids = chain(
        &mut g,
        &["motion.grid", "force.wind", "force.wind", "motion.output"],
    );

    let ds = diagnose(&g, &reg);
    assert_eq!(
        ds.len(),
        2,
        "a force reading+rewriting accel is not a consumer"
    );
    let inert: std::collections::BTreeSet<NodeId> = ds.iter().map(|d| d.node).collect();
    assert!(
        inert.contains(&ids[1]) && inert.contains(&ids[2]),
        "both forces"
    );
    assert!(
        ds.iter()
            .all(|d| d.deficit == Deficit::InertProducer("accel")),
        "both are inert accel"
    );
}

/// **A CPU-only falloff consumer makes the field healthy — the 6-crate annotation.**
/// `fx.drop_shadow` reads `falloff` only at eval runtime (no GPU kernel), so its role
/// cannot be derived; this wave declares it with a `Coupling::Consumes("falloff")`.
/// With that, `field.box → fx.drop_shadow` is a healthy pair. FALSIFIED by dropping
/// the annotation added to `ph2d-node-fx-drop-shadow` (the field is flagged inert).
#[test]
fn a_cpu_only_falloff_consumer_makes_the_field_healthy() {
    let reg = registry();
    let mut g = Graph::new();
    let ids = chain(
        &mut g,
        &[
            "motion.grid",
            "field.box",
            "fx.drop_shadow",
            "motion.output",
        ],
    );

    let ds = diagnose(&g, &reg);
    assert!(
        !ds.iter().any(|d| d.node == ids[1]),
        "field.box is healthy: fx.drop_shadow (CPU-only) declares it consumes falloff"
    );
    assert!(
        ds.is_empty(),
        "nothing inert: the field is read, drop_shadow adds no transient"
    );
}

/// **A LOWERED column wired to the output is never inert — the transient filter.**
/// `motion.move` writes `P` (the position), which the render lowering consumes, so
/// wiring it straight to the output is the normal case, not a defect. The diagnoser
/// only subjects [transient columns] to the analysis, so `P` is never flagged.
/// FALSIFIED by removing the transient filter (or adding `P` to it): `move → output`
/// would then be reported inert, the false positive this guard exists to prevent.
#[test]
fn a_lowered_column_wired_to_output_is_never_inert() {
    let reg = registry();
    let mut g = Graph::new();
    chain(&mut g, &["motion.grid", "motion.move", "motion.output"]);

    assert!(
        diagnose(&g, &reg).is_empty(),
        "P is lowered to instances, so a node writing P to the output is not inert"
    );
}

// ── The required-upstream family (2a): a P-reader with nothing wired in has no points ──

/// **A deformer with nothing wired to it needs points — DERIVED, zero annotation.**
/// `motion.bend` reads `P` (a `ReadWrite` binding for the GPU cook); with no input edge
/// it has no stream to bend, so it is silently a no-op. The diagnoser reports it a
/// `MissingSource("P")` OFFER (WHICH source — grid / emitter / object — is the artist's
/// choice, never guessed), purely from the same binding the cook reads. FALSIFIED by a
/// `reads_column` that ignores the GPU binding (nothing detected) or a `has_input` that is
/// always true (a fed reader is never source-less, so this could never fire).
#[test]
fn a_deformer_with_nothing_wired_needs_points() {
    let reg = registry();
    let mut g = Graph::new();
    let bend = g.add_node("motion.bend");
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (bend, 0),
        to: (out, 0),
        delayed: false,
    })
    .expect("bend -> output");

    let ds = diagnose(&g, &reg);
    assert_eq!(ds.len(), 1, "exactly the floating deformer");
    assert_eq!(ds[0].node, bend, "the motion.bend node");
    assert_eq!(ds[0].deficit, Deficit::MissingSource("P"));
    assert_eq!(
        ds[0].fix,
        Fix::Offer,
        "WHICH source is a creative choice — offer, don't guess"
    );
}

/// **A deformer FED by a source needs nothing — the negative control.** `grid → bend →
/// output`: `motion.bend` has an input, so it is not source-less; it reads+writes `P`
/// (lowered, consumed by the output), so the graph is clean. FALSIFIED by a `has_input`
/// that is always false (the fed bend would be wrongly flagged source-less).
#[test]
fn a_deformer_fed_by_a_source_is_clean() {
    let reg = registry();
    let mut g = Graph::new();
    chain(&mut g, &["motion.grid", "motion.bend", "motion.output"]);
    assert!(
        diagnose(&g, &reg).is_empty(),
        "a fed deformer that writes lowered P is healthy"
    );
}

/// **A floating force needs POINTS, not an integrator — the root cause wins.** A
/// `force.attractor` with nothing wired reads `P` (source-less) AND produces `accel` (no
/// integrator) — two defects, but "no points" is the ROOT (a force with no stream produces
/// no accel either). The diagnoser reports ONLY the `MissingSource("P")`, so the artist is
/// pointed at the real problem. FALSIFIED by dropping the `continue` after a
/// `MissingSource` (the misleading `accel` diagnostic would ride along — two, not one).
#[test]
fn a_floating_force_needs_points_not_an_integrator() {
    let reg = registry();
    let mut g = Graph::new();
    let force = g.add_node("force.attractor");
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (force, 0),
        to: (out, 0),
        delayed: false,
    })
    .expect("force -> output");

    let ds = diagnose(&g, &reg);
    assert_eq!(
        ds.len(),
        1,
        "only the root cause — no points, not the accel"
    );
    assert_eq!(ds[0].node, force, "the force.attractor node");
    assert_eq!(
        ds[0].deficit,
        Deficit::MissingSource("P"),
        "the floating force needs points, not an integrator"
    );
}

/// **A source is never told it needs points — the negative control for `reads()`.**
/// `motion.grid` WRITES `P` (`ColumnAccess::Write`, which does not `reads()`), so a bare
/// `grid → output` is never a `MissingSource` — grid IS the source. FALSIFIED by a
/// `reads_column` that treats a `Write` as a read (grid, with nothing feeding it, would
/// be wrongly told it needs points).
#[test]
fn a_source_is_never_told_it_needs_points() {
    let reg = registry();
    let mut g = Graph::new();
    chain(&mut g, &["motion.grid", "motion.output"]);
    assert!(
        diagnose(&g, &reg).is_empty(),
        "grid writes P without reading it — it IS the source, not a needer of one"
    );
}
