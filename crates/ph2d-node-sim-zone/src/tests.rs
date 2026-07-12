//! Guards for the Simulation Zone (O4, doc 48). Cooked through the REAL registry, with the
//! `pre` edge wired exactly as the editor's plumbing wires it (`out --pre--> state`).

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("wire");
}

/// Cook the zone at `t`, then close the tick (which is what publishes this tick's outputs as
/// next tick's `pre` values) — the two calls the pump makes, in the pump's order.
fn tick(cook: &mut Cook, g: &Graph, reg: &NodeRegistry, sink: NodeId, t: f64) -> usize {
    let out = cook.cook(g, reg, sink, t).expect("cooks");
    let n = out[0].as_stream().count();
    cook.advance_tick(g, reg, t).expect("tick closes");
    n
}

/// A 3x4 grid into the zone's `init`, and whatever `interior` builds as the chain from the
/// zone's output back into its `state` port (the editor's plumbing does this wiring).
fn scene(interior: impl Fn(&mut Graph, NodeId) -> NodeId) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 4.0);
    let zone = g.add_node("sim.zone");
    wire(&mut g, grid, 0, zone, 0, false); // grid -> init

    let tail = interior(&mut g, zone);
    if tail != zone {
        wire(&mut g, tail, 0, zone, 1, false); // the interior's END lands on `state`…
    }
    (g, zone)
}

/// **Tick 1 emits `init`; every tick after emits what the INTERIOR handed back.** That is the
/// whole zone: it holds live state across ticks, and the interior may do anything at all to it.
#[test]
fn the_first_tick_seeds_from_init_and_the_rest_run_the_interior() {
    let reg = registry();
    // Interior: a single `sim.step`, fed last tick's state by the `pre` edge (the plumbing's
    // job: zone.out --pre--> head.in).
    let (g, zone) = scene(|g, zone| {
        let step = g.add_node("sim.step");
        wire(g, zone, 0, step, 0, true); // <- the engine-managed state entry
        step
    });
    assert!(g.validate(&reg).is_ok(), "the zone is well-typed");

    let mut cook = Cook::new();
    assert_eq!(tick(&mut cook, &g, &reg, zone, 0.0), 12, "tick 1: the seed");
    assert_eq!(
        tick(&mut cook, &g, &reg, zone, 1.0 / 60.0),
        12,
        "tick 2: the state, carried by the interior"
    );
}

/// **A zone that killed everything STAYS EMPTY.** This is the guard the whole `input_present`
/// accessor exists for.
///
/// FALSIFIED by asking `state.count() == 0` instead of "did a value arrive at all": an empty
/// stream is a real answer (the sim killed its last element), and a zone that reads it as "I
/// have not started" re-seeds from `init` on the next tick — so the scene **resurrects**, one
/// frame after every kill, forever.
#[test]
fn a_zone_that_killed_everything_stays_dead() {
    let reg = registry();
    // Interior: cull EVERYTHING (Fraction mode, amount 0 → keep zero elements).
    let (g, zone) = scene(|g, zone| {
        let cull = g.add_node("motion.cull");
        g.set_param(cull, "mode", 0.0); // Fraction
        g.set_param(cull, "amount", 0.0); // …keep none
        wire(g, zone, 0, cull, 0, true);
        cull
    });

    let mut cook = Cook::new();
    assert_eq!(tick(&mut cook, &g, &reg, zone, 0.0), 12, "the seed arrives");
    assert_eq!(
        tick(&mut cook, &g, &reg, zone, 1.0 / 60.0),
        0,
        "the interior killed them all"
    );
    for k in 2..6 {
        assert_eq!(
            tick(&mut cook, &g, &reg, zone, k as f64 / 60.0),
            0,
            "…and they STAY dead (tick {k}): an empty sim is not an unstarted one"
        );
    }
}

/// Culling inside a zone is a **KILL**: the survivors are carried forward by the state, so what
/// dies does not come back next frame. Outside a zone the very same `motion.cull` is a filter on
/// a stream the next frame rebuilds from scratch — the node did not change; the zone did.
#[test]
fn a_cull_inside_a_zone_is_a_permanent_kill() {
    let reg = registry();
    // Keep half the elements each tick: 12 -> 6 -> 3 -> 2 (round), never back up.
    let (g, zone) = scene(|g, zone| {
        let cull = g.add_node("motion.cull");
        g.set_param(cull, "mode", 0.0);
        g.set_param(cull, "amount", 0.5);
        wire(g, zone, 0, cull, 0, true);
        cull
    });

    let mut cook = Cook::new();
    let mut counts = vec![tick(&mut cook, &g, &reg, zone, 0.0)];
    for k in 1..4 {
        counts.push(tick(&mut cook, &g, &reg, zone, k as f64 / 60.0));
    }
    assert_eq!(counts, vec![12, 6, 3, 2], "the population only ever falls");
}

/// **A bare zone FREEZES its seed** — nothing is wired into `state`, so the plumbing self-loops
/// it and the zone re-emits what it emitted before, forever. This is Blender's documented
/// surprise (*"even just a simulation zone that does nothing freezes the mesh"*), and it is why
/// the port is named `init` and not `in`: it is read ONCE.
///
/// The proof that it is a freeze and not a pass-through: change the seed's parameters mid-run
/// and the zone does not notice.
#[test]
fn a_bare_zone_freezes_its_seed() {
    let reg = registry();
    let (mut g, zone) = scene(|_, zone| zone); // nothing in the interior
    let grid = NodeId(0);
    wire(&mut g, zone, 0, zone, 1, true); // the plumbing's self-loop

    let mut cook = Cook::new();
    assert_eq!(tick(&mut cook, &g, &reg, zone, 0.0), 12);

    // Grow the seed to 5x4 = 20. A pass-through would follow it; a FROZEN zone does not.
    g.set_param(grid, "rows", 5.0);
    assert_eq!(
        tick(&mut cook, &g, &reg, zone, 1.0 / 60.0),
        12,
        "the seed is read once - the zone holds what it held"
    );
}

/// **Every column the seed carried survives the loop.** The state goes out of the zone, through
/// the interior and back in; whatever the interior does not own (`id`, `size`, `tint`…) has to
/// come out the other side, or the elements lose the very attributes that make them themselves —
/// and a kill node downstream could never tell one tick's survivors from the next tick's.
#[test]
fn the_zone_carries_every_column_through() {
    let reg = registry();
    let (g, zone) = scene(|g, zone| {
        let step = g.add_node("sim.step");
        wire(g, zone, 0, step, 0, true);
        step
    });

    let mut cook = Cook::new();
    let seeded: Vec<String> = {
        let out = cook.cook(&g, &reg, zone, 0.0).expect("cooks");
        out[0]
            .as_stream()
            .columns()
            .map(|(name, _)| name.clone())
            .collect()
    };
    cook.advance_tick(&g, &reg, 0.0).expect("tick closes");
    assert!(!seeded.is_empty(), "the seed carries columns to begin with");

    let out = cook.cook(&g, &reg, zone, 1.0 / 60.0).expect("cooks");
    let s = out[0].as_stream();
    assert!(matches!(s.get("P"), Some(Column::Vec2(v)) if v.len() == 12));
    for name in &seeded {
        assert!(s.get(name).is_some(), "`{name}` survived the state loop");
    }
}
