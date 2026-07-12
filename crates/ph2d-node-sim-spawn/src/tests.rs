//! Guards for `sim.spawn` (O4 birth, doc 49). `super` is the crate root.

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    reg
}

const FPS: f64 = 60.0;

/// Every id born over `secs` seconds of 60 fps ticks.
fn run(rate: f64, secs: f64) -> Vec<u32> {
    let ticks = (secs * FPS) as u64;
    let mut ids = Vec::new();
    for k in 1..=ticks {
        let t = k as f64 / FPS;
        ids.extend(born_in(rate, t, 1.0 / FPS));
    }
    ids
}

/// **A fractional rate is exact over time.** 7 births a second at 60 fps is 0.116 per tick —
/// and the obvious implementation (`round(rate · dt)` per tick) rounds that to ZERO on every
/// tick and emits **nothing, ever**. Births come from the difference of two floors, so the
/// leftovers accumulate instead of evaporating.
///
/// FALSIFIED by per-tick rounding (0 births) and by per-tick truncation (also 0).
#[test]
fn a_fractional_rate_is_exact_over_time_and_does_not_round_itself_away() {
    let ids = run(7.0, 1.0);
    assert_eq!(ids.len(), 7, "7 per second means 7, at 60 fps: {ids:?}");
    let ten = run(7.0, 10.0);
    assert_eq!(ten.len(), 70, "…and it stays exact over ten seconds");
}

/// **Identity is the birth ordinal**: ids are unique, monotone, and never reused — so a kill
/// node downstream can tell one tick's survivors from the next tick's newborns, and two
/// particles can never claim to be the same particle.
#[test]
fn ids_are_unique_and_monotone() {
    let ids = run(30.0, 3.0);
    assert_eq!(ids.len(), 90);
    assert!(ids.windows(2).all(|w| w[1] > w[0]), "monotone: {ids:?}");
    assert_eq!(ids[0], 0, "the first element ever born is number 0");
}

/// **A paused clock breeds nothing.** `dt = 0` is the first tick of a cook and every paused
/// frame; a sim that spawned on it would multiply while you stared at it.
#[test]
fn a_paused_clock_spawns_nothing() {
    assert!(born_in(50.0, 4.0, 0.0).is_empty(), "paused: no births");
    assert!(born_in(50.0, 0.0, 0.0).is_empty(), "the first tick: none");
    assert!(born_in(0.0, 4.0, 1.0 / FPS).is_empty(), "rate 0: none");
}

/// **A scrub reproduces the same particles.** The ids are a function of the CLOCK, not of a
/// counter the node keeps — so cooking t = 2 s after a rewind gives exactly what the first pass
/// gave.
///
/// FALSIFIED by an id counter carried in a state column: it would count the cook's HISTORY, so a
/// scrub would renumber the world and every element downstream would look brand new.
#[test]
fn a_rewind_reproduces_the_same_births() {
    let first = run(20.0, 2.0);
    let again = run(20.0, 2.0);
    assert_eq!(first, again);
    // …and a single tick in the middle is reproducible on its own, without any history at all.
    let mid: Vec<u32> = born_in(20.0, 1.0, 1.0 / FPS).collect();
    assert_eq!(mid, vec![19], "the 20th birth lands in that tick, always");
}

/// A newborn **inherits its template row, column for column** — position, velocity, size, tint,
/// whatever was authored upstream — and is stamped with its own id.
///
/// It is stamped with NO clock (`sim_t`): `sim.step` must see an element it has never stepped
/// and start it, not hurl it forward by the entire age of the simulation.
#[test]
fn a_newborn_inherits_its_template_row_and_carries_no_clock() {
    let template = Stream::new(2)
        .with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]))
        .with("vel", Column::Vec2(vec![[0.0, -1.0], [0.0, -2.0]]))
        .with("id", Column::Scalar(vec![99.0, 98.0])); // the template's own ids are NOT inherited

    let out = newborns(&template, &[5], false, 1); // round-robin: 5 % 2 = row 1
    assert_eq!(out.count(), 1);
    assert!(matches!(out.get("P"), Some(Column::Vec2(v)) if v[0] == [3.0, 4.0]));
    assert!(matches!(out.get("vel"), Some(Column::Vec2(v)) if v[0] == [0.0, -2.0]));
    assert!(
        matches!(out.get("id"), Some(Column::Scalar(v)) if v[0] == 5.0),
        "its identity is its BIRTH ORDINAL, not the template's id"
    );
    assert!(out.get("sim_t").is_none(), "a newborn carries no clock");
}

/// **Birth, life and death, through the real registry** — the whole triangle, in a zone.
///
/// The population is born (`sim.spawn` + `motion.combine`), lives (`sim.step`) and, without a
/// kill, only ever GROWS: the newborns persist in the state, tick after tick, which is precisely
/// what a stateless emitter cannot do and what makes this a simulation.
#[test]
fn a_zone_that_spawns_grows_a_population() {
    let reg = registry();
    let mut g = Graph::new();
    let seed = g.add_node("motion.grid"); // the birth template: a single point
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    let zone = g.add_node("sim.zone");
    let spawn = g.add_node("sim.spawn");
    let merge = g.add_node("motion.combine");
    let step = g.add_node("sim.step");
    g.set_param(spawn, "rate", 30.0);

    for (from, fp, to, tp, delayed) in [
        (seed, 0u16, spawn, 0u16, false),
        (zone, 0, merge, 0, true), // the engine-managed state entry: last tick's population
        (spawn, 0, merge, 1, false), // …and this tick's newborns
        (merge, 0, step, 0, false),
        (step, 0, zone, 1, false),
    ] {
        g.connect(Edge {
            from: (NodeId(from.0), fp),
            to: (NodeId(to.0), tp),
            delayed,
        })
        .expect("wire");
    }
    // The zone's `init` is left EMPTY: the population is entirely born, not seeded.
    assert!(g.validate(&reg).is_ok());

    let mut cook = Cook::new();
    let mut counts = Vec::new();
    for k in 0..=120u64 {
        let t = k as f64 / FPS;
        let out = cook.cook(&g, &reg, zone, t).expect("cooks");
        counts.push(out[0].as_stream().count());
        cook.advance_tick(&g, &reg, t).expect("tick closes");
    }
    assert_eq!(counts[0], 0, "nothing is born on the first tick (dt = 0)");
    assert!(
        counts.windows(2).all(|w| w[1] >= w[0]),
        "a population with no kill never shrinks: {counts:?}"
    );
    // 30/s for 2 s ≈ 60 (the first tick spawns none).
    let last = *counts.last().unwrap();
    assert!(
        (58..=60).contains(&last),
        "30 a second for two seconds: {last}"
    );
}
