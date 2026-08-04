//! ADR-0155 gates for the setup diagnoser. Every gate drives the REAL registry
//! (`register_all_nodes`) so the couplings under test are the ones the app ships,
//! not a fixture's private copy.

use ph2d_motion_diagnose::{canonical_consumer, diagnose, Deficit, Fix};
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
        &["motion.grid", "motion.integrate", "force.wind", "motion.output"],
    );

    let ds = diagnose(&g, &reg);
    assert_eq!(ds.len(), 1, "only the misplaced force");
    assert_eq!(ds[0].node, ids[2], "the force.wind, downstream of integrate");
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
    assert_eq!(ds[0].fix, Fix::Offer, "no canonical solver -> offer, don't guess");
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
            cs.iter().any(|c| matches!(c, Coupling::Produces(x) if *x == col)),
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
            cs.iter().any(|c| matches!(c, Coupling::Consumes(x) if *x == col)),
            "{name} must Consume {col}"
        );
    }
}
