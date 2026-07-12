//! Guards for `sim.step` (O4, doc 48). `super` is the crate root.

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    reg
}

/// One element at the origin with a velocity, and a clock already stamped on it.
fn moving(p: [f32; 2], v: [f32; 2], t: f32) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![p]))
        .with("vel", Column::Vec2(vec![v]))
        .with("sim_t", Column::Scalar(vec![t]))
}

fn pos(s: &Stream) -> [f32; 2] {
    match s.get("P") {
        Some(Column::Vec2(v)) => v[0],
        _ => panic!("no positions"),
    }
}

/// **An element with velocity moves by `v · dt`** — and `dt` comes from the clock the state
/// itself carries, not from a frame counter.
#[test]
fn a_moving_element_advances_by_v_dt() {
    let s = step(&moving([0.0, 0.0], [2.0, 0.0], 0.0), 0.02, 1.0);
    let p = pos(&s);
    assert!((p[0] - 0.04).abs() < 1e-6, "2 u/s for 20 ms = 0.04: {p:?}");
    assert!((p[1]).abs() < 1e-6);
}

/// **An element the sim has never seen does not move.** It has no clock on it, so its `dt` is
/// zero and it simply starts.
///
/// FALSIFIED by defaulting `sim_t` to 0 instead of to "no step": a zone dropped onto a graph at
/// playhead = 8 s would take a single EIGHT-SECOND step on the frame it was created and fling
/// the whole scene to infinity.
#[test]
fn a_brand_new_element_starts_it_does_not_leap() {
    let fresh = Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("vel", Column::Vec2(vec![[2.0, 0.0]])); // no `sim_t` yet
    let s = step(&fresh, 8.0, 1.0);
    assert_eq!(pos(&s), [0.0, 0.0], "it starts where it is, at t = 8 s");
    // …and now it carries the clock, so the NEXT step is a real one.
    let s2 = step(&s, 8.02, 1.0);
    assert!((pos(&s2)[0] - 0.04).abs() < 1e-6);
}

/// A huge `dt` (a pause, a scrub, a dropped frame) is CLAMPED — the sim slows down rather than
/// exploding. Same guarantee `motion.integrate` gives, for the same reason.
#[test]
fn a_scrub_sized_dt_is_clamped_not_taken() {
    let s = step(&moving([0.0, 0.0], [2.0, 0.0], 0.0), 5.0, 1.0);
    assert!(
        pos(&s)[0] <= 2.0 * MAX_DT + 1e-6,
        "a 5-second jump is taken as at most MAX_DT: {:?}",
        pos(&s)
    );
}

/// **Acceleration is consumed.** The `force.*` nodes accumulate into the transient `accel`
/// column; a step that left it on the stream would re-apply the same force forever, one tick out
/// of date — a wind that keeps blowing after you unplug it.
#[test]
fn the_step_consumes_the_acceleration_it_applied() {
    let falling =
        moving([0.0, 0.0], [0.0, 0.0], 0.0).with("accel", Column::Vec2(vec![[0.0, -10.0]]));
    let s = step(&falling, 0.02, 1.0);
    assert!(s.get("accel").is_none(), "the transient is spent");
    // Semi-implicit: velocity took the kick BEFORE the position used it.
    let p = pos(&s);
    assert!(p[1] < 0.0, "it fell: {p:?}");
    assert!(
        (p[1] - (-10.0 * 0.02 * 0.02)).abs() < 1e-6,
        "P += (v + a·dt)·dt — the NEW velocity: {p:?}"
    );
}

/// A pinned element (`inv_mass = 0`) does not move, however hard it is pushed.
#[test]
fn a_pinned_element_is_immovable() {
    let pinned = moving([1.0, 1.0], [5.0, 5.0], 0.0)
        .with("inv_mass", Column::Scalar(vec![0.0]))
        .with("accel", Column::Vec2(vec![[100.0, 100.0]]));
    assert_eq!(pos(&step(&pinned, 0.02, 1.0)), [1.0, 1.0]);
}

/// Damping bleeds velocity away; the default (1.0) is bit-identical to no damping at all, so an
/// undamped zone is not paying for a feature it did not ask for.
#[test]
fn damping_bleeds_velocity_and_its_default_is_exact() {
    let free = step(&moving([0.0, 0.0], [2.0, 0.0], 0.0), 0.02, 1.0);
    let thick = step(&moving([0.0, 0.0], [2.0, 0.0], 0.0), 0.02, 0.0);
    assert!(pos(&thick)[0] < pos(&free)[0], "damped travels less");
    assert_eq!(pos(&free), [0.04, 0.0], "…and undamped is the exact lerp");
}

/// The step cooks through the REAL registry inside a zone — the whole chain, not the function.
#[test]
fn the_step_runs_inside_a_zone_through_the_registry() {
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 2.0);
    g.set_param(grid, "cols", 2.0);
    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    let st = g.add_node("sim.step");
    for (from, fp, to, tp, delayed) in [
        (grid, 0, zone, 0, false),
        (zone, 0, wind, 0, true), // the engine-managed state entry
        (wind, 0, st, 0, false),
        (st, 0, zone, 1, false), // …back into `state`
    ] {
        g.connect(Edge {
            from: (NodeId(from.0), fp),
            to: (NodeId(to.0), tp),
            delayed,
        })
        .expect("wire");
    }
    assert!(g.validate(&reg).is_ok());

    let mut cook = Cook::new();
    let mut last = 0.0;
    for k in 0..8 {
        let out = cook.cook(&g, &reg, zone, k as f64 / 60.0).expect("cooks");
        last = match out[0].as_stream().get("P") {
            Some(Column::Vec2(v)) => v[0][1],
            _ => panic!("no P"),
        };
        cook.advance_tick(&g, &reg, k as f64 / 60.0).expect("tick");
    }
    assert!(
        last.is_finite() && last != 0.0,
        "the wind moved the state across ticks: y = {last}"
    );
}
