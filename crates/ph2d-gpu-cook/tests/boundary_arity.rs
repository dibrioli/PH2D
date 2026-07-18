//! **How many seams can a plan actually have, and what does one uncovered node
//! cost?** — the measured answer to the question slice B was going to assume.
//!
//! The continuation handoff recommended building an N-boundary shell seam first,
//! on the reading that *"com 52 nós descobertos, qualquer grafo real tem várias
//! [fronteiras]"*. That conflates **many uncovered nodes** with **many seams**,
//! and the two are not the same thing: the claimed region grows UPWARD from the
//! sink, so the boundaries are its **frontier**, and a *chain* of uncovered nodes
//! presents exactly one frontier node — the walk stops at the first and never
//! sees the rest.
//!
//! For the frontier to branch, a **staged** node needs ≥2 inputs whose sources
//! are both uncovered. Today exactly three kernel-having nodes have 2 inputs, and
//! all three decline that shape:
//!
//! | node | 2nd port | why it cannot branch the frontier |
//! |---|---|---|
//! | `motion.integrate` | `forces` | the `pre` feedback → `GpuSource::Prev`, not a boundary; wired plain + uncovered, the node itself refuses (D3 shape) |
//! | `motion.spring` | `state` | same — the auto-wired `out --pre--> state` loop |
//! | `motion.color_ramp` | `t` (VALUE) | connecting `t` refuses the node whole (the documented `t` engine block) |
//!
//! ⇒ **`plan.boundaries.len() > 1` is unreachable today**, and the shell's
//! `_ => GpuRoute::Cpu` arm is dead code. Building the N-seam pump would be
//! machinery for a state that cannot occur.
//!
//! These gates pin that, and the FIRST one is a deliberate **tripwire**: the day
//! someone lands a multi-input kernel (`motion.look_at` / `motion.combine`, the
//! natural next ports) it goes red, and red means *"now the N-seam slice is real
//! — go build it"*. A comment saying so would rot; a failing gate cannot
//! ([[feedback_a_condition_that_enumerates_its_readers_rots]]).
//!
//! The second pair measures the thing that DOES bite, and it is much harsher than
//! N seams: the seam only ever hands the GPU the **suffix**, so an uncovered node
//! anywhere in a particle graph's stream path collapses the whole 4.19 M-particle
//! sim onto the CPU — not because of extra seams, but because the claimed region
//! shrinks to the pass-through `output` and dispatches nothing.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_motion_move::register(&mut reg).unwrap();
    ph2d_node_motion_color_ramp::register(&mut reg).unwrap();
    ph2d_node_motion_integrate::register(&mut reg).unwrap();
    ph2d_node_motion_emitter::register(&mut reg).unwrap();
    ph2d_node_force_wind::register(&mut reg).unwrap();
    // The stand-in for "any of the 52 nodes with no kernel".
    ph2d_node_motion_sort::register(&mut reg).unwrap();
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    reg
}

fn edge(g: &mut Graph, a: NodeId, pa: u16, b: NodeId, pb: u16) {
    g.connect(Edge {
        from: (a, pa),
        to: (b, pb),
        delayed: false,
    })
    .unwrap();
}

/// Every shape that could plausibly branch the frontier, measured. If ANY of
/// these ever yields 2+ boundaries, the N-seam shell slice has become real.
#[test]
fn no_plan_can_leave_more_than_one_seam_today() {
    let reg = registry();

    // (a) A CHAIN of two uncovered nodes: the walk stops at the FIRST one, so
    //     "two uncovered nodes" is still one seam. This is the shape the handoff
    //     mistook for a multi-seam graph.
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let s1 = g.add_node("motion.sort");
    let s2 = g.add_node("motion.sort");
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    edge(&mut g, grid, 0, s1, 0);
    edge(&mut g, s1, 0, s2, 0);
    edge(&mut g, s2, 0, mv, 0);
    edge(&mut g, mv, 0, out, 0);
    let chain = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert_eq!(
        chain.boundaries,
        vec![(s2, 0)],
        "the frontier is the FIRST uncovered node, not every uncovered node"
    );

    // (b) `motion.color_ramp` — the only kernel node with a second non-feedback
    //     port. Connecting `t` refuses the node, so it recedes instead of
    //     branching.
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let srt = g.add_node("motion.sort");
    let cr = g.add_node("motion.color_ramp");
    let lfo = g.add_node("value.lfo");
    let out = g.add_node("motion.output");
    edge(&mut g, grid, 0, srt, 0);
    edge(&mut g, srt, 0, cr, 0);
    edge(&mut g, lfo, 0, cr, 1);
    edge(&mut g, cr, 0, out, 0);
    let ramp = ph2d_gpu_cook::plan(&g, &reg, &reg, out);

    // (c) `motion.integrate` with BOTH ports plain and uncovered — the last
    //     theoretical route. The node refuses (it cannot derive its input shape).
    let mut g = Graph::new();
    let s1 = g.add_node("motion.sort");
    let s2 = g.add_node("motion.sort");
    let it = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");
    edge(&mut g, s1, 0, it, 0);
    edge(&mut g, s2, 0, it, 1);
    edge(&mut g, it, 0, out, 0);
    let integ = ph2d_gpu_cook::plan(&g, &reg, &reg, out);

    for (name, plan) in [
        ("chain", &chain),
        ("color_ramp.t", &ramp),
        ("integrate", &integ),
    ] {
        assert!(
            plan.boundaries.len() <= 1,
            "{name}: a plan left {} seams — a multi-input kernel has landed, so \
             the N-seam shell slice (handoff §2 B) is now REAL and unbuilt. \
             Build it; then this gate becomes the parity gate for it.",
            plan.boundaries.len()
        );
    }
}

/// The cost that DOES bite: one uncovered node downstream of the sim forfeits
/// the entire GPU simulation, because the seam only hands over the SUFFIX.
#[test]
fn one_uncovered_node_downstream_of_the_sim_forfeits_the_whole_gpu_path() {
    let reg = registry();
    // emitter → integrate → output, loop closed `integrate.out --pre--> wind`.
    let build = |uncovered: bool| {
        let mut g = Graph::new();
        let em = g.add_node("motion.emitter");
        let wind = g.add_node("force.wind");
        let it = g.add_node("motion.integrate");
        let out = g.add_node("motion.output");
        edge(&mut g, em, 0, it, 0);
        edge(&mut g, wind, 0, it, 1);
        g.connect(Edge {
            from: (it, 0),
            to: (wind, 0),
            delayed: true,
        })
        .unwrap();
        if uncovered {
            let srt = g.add_node("motion.sort");
            edge(&mut g, it, 0, srt, 0);
            edge(&mut g, srt, 0, out, 0);
        } else {
            edge(&mut g, it, 0, out, 0);
        }
        (g, out)
    };

    // The control: fully covered → the whole chain is claimed and dispatches.
    let (g, out) = build(false);
    let covered = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert!(
        covered.is_fully_gpu(),
        "the covered particle chain is claimed whole"
    );
    assert!(
        covered.dispatching_stages(&reg) >= 3,
        "emitter + wind + integrate all dispatch"
    );

    // The measurement: ONE uncovered node late in the chain, and the claimed
    // region collapses to the pass-through `output` — zero dispatch, so the
    // shell recuses whole and the CPU runs 4.19 M particles at 227 ms/tick.
    let (g, out) = build(true);
    let cut = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
    assert_eq!(
        cut.boundaries.len(),
        1,
        "still ONE seam — not an N-seam problem"
    );
    assert_eq!(
        cut.dispatching_stages(&reg),
        0,
        "the sim is upstream of the seam, so the GPU is left with only the \
         pass-through output: nothing to dispatch, and the shell recuses"
    );
}
