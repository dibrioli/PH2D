//! Tests for [`super`] (the GPU cook routing / recusal) — sibling via `#[path]`
//! to keep `motion_bridge_gpu.rs` under the shell LOC cap; `use super::*` reaches
//! the private `gpu_route`/`graph_has_*`/`cook_publishes_live_geometry`.

use super::*;
use crate::motion_state::MotionState;

// A stand-in boundary node id (the routing never dereferences it).
fn node() -> NodeId {
    NodeId(7)
}

#[test]
fn only_the_emitters_renumbering_params_invalidate_the_gpu_sim() {
    // ADR-0130 D7. `rate` alone moves the id↔particle map (`birth(k) = k/rate`)
    // → restart; everything else keeps the running sim; and NO other node type
    // restarts. The mutation that drops the `motion.emitter` guard makes a
    // grid's or a force's "rate" restart too — the last loop below catches it.
    assert!(
        edit_renumbers_emitter("motion.emitter", "rate"),
        "rate re-numbers: `birth(k) = k/rate`"
    );
    // `life`/`max` are the ones this gate got WRONG at first. They move the
    // window's left edge, never `birth(k)` — survivors keep their rows, and
    // ids the window newly reveals are seeded by the per-element bounds check.
    // Listed by NAME, not folded into the loop below, because their absence
    // here is a decision and not an omission (the GPU gate
    // `shrinking_the_life_of_a_live_emitter_leaves_the_survivors_untouched`
    // is what proves the decision is safe).
    for p in ["life", "max"] {
        assert!(
            !edit_renumbers_emitter("motion.emitter", p),
            "{p} resizes the window; it does not re-number"
        );
    }
    for p in ["speed", "angle", "spread", "seed", "x", "y", "size"] {
        assert!(
            !edit_renumbers_emitter("motion.emitter", p),
            "{p} keeps the sim (the gather still pairs id-for-id)"
        );
    }
    for ty in [
        "motion.grid",
        "force.wind",
        "motion.integrate",
        "motion.spring",
    ] {
        assert!(
            !edit_renumbers_emitter(ty, "rate"),
            "{ty} is not the emitter"
        );
    }
}

#[test]
fn a_real_emitter_nodes_type_name_drives_the_policy() {
    // The seam reads `inst.type_name`; pin that a REAL emitter node resolves to
    // exactly the string the policy matches (and a grid does not), so the wire
    // from the param edit to the forget can never miss by a renamed type.
    let mut motion = MotionState::new();
    let em = motion.doc.graph.add_node("motion.emitter");
    let grid = motion.doc.graph.add_node("motion.grid");
    let ty = |m: &MotionState, n| m.doc.graph.node(n).expect("added").type_name.clone();
    assert!(edit_renumbers_emitter(&ty(&motion, em), "rate"));
    assert!(!edit_renumbers_emitter(&ty(&motion, em), "speed"));
    assert!(!edit_renumbers_emitter(&ty(&motion, grid), "rate"));
}

/// **The recusal catches the live vector but NOT the object** (ADR-0154 /
/// this wave). `source.shape` (a live vector, `geometry_id`) has no GPU
/// render route, so it recuses; `source.object` (an engine object,
/// `texture_id`) is now GPU-renderable, so it does NOT recuse via the
/// live-vector door — it is an OBJECT source, guarded only by the
/// count-changing cerca. Pinned through a REAL `MotionState` registry so the
/// flags the two source nodes declare are what drive it. FALSIFIED by
/// `source.object` re-registering the live-vector flag (it would recuse
/// again → lose the acceleration) or by dropping the shape's flag (white
/// rectangles for a live vector).
#[test]
fn the_recusal_catches_the_live_vector_but_not_the_object() {
    let build = |ty: &str| {
        let mut m = MotionState::new();
        let src = m.doc.graph.add_node(ty);
        let out = m.doc.graph.add_node("motion.output");
        m.doc
            .graph
            .connect(ph2d_nodegraph::graph::Edge {
                from: (src, 0),
                to: (out, 0),
                delayed: false,
            })
            .expect("connect");
        m
    };
    let live_vector = |ty: &str| {
        let m = build(ty);
        graph_has_live_vector_source(&m.doc.graph, &m.registry)
    };
    let object = |ty: &str| {
        let m = build(ty);
        graph_has_object_source(&m.doc.graph, &m.registry)
    };

    // A live vector SHAPE recuses; an OBJECT does NOT (it is GPU-renderable).
    assert!(
        live_vector("source.shape"),
        "a live vector shape recuses (geometry_id, no GPU route)"
    );
    assert!(
        !live_vector("source.object"),
        "an engine object does NOT recuse via the live-vector door (it is drawn)"
    );
    // The object IS an object source (for the count-changing cerca); the
    // shape is not, and neither is a plain point source.
    assert!(
        object("source.object"),
        "an engine object is an object source (texture_id)"
    );
    assert!(
        !object("source.shape"),
        "a live vector shape is not an object source"
    );
    // Controls: a point/value-domain document is neither. Without these the
    // test could pass by always returning the same answer.
    assert!(
        !live_vector("motion.grid") && !object("motion.grid"),
        "a plain point source is neither"
    );
    assert!(
        !live_vector("motion.rotate") && !object("motion.rotate"),
        "a modifier alone is neither"
    );
}

#[test]
fn the_object_recusal_is_content_aware_a_live_vector_recuses_but_a_sprite_stays() {
    use ph2d_nodegraph::attr::{Column, Stream};
    // Part 1 (Vetor Vivo): whether a `source.object` recuses depends on WHAT it
    // names — a live vector publishes `geometry_id` (no GPU route ⇒ recuse), a
    // sprite publishes `texture_id` (a GPU route ⇒ stay on the stamp). The node
    // TYPE is `source.object` either way, so the recusal must be CONTENT-aware:
    // scan the published externals. RED-FIRST: a node-type flag or an
    // unconditional recuse sends the sprite object to the CPU too, killing this
    // wave's GPU acceleration.
    let mut m = MotionState::new();
    assert!(
        !cook_publishes_live_geometry(&m.pump.cook),
        "no externals -> nothing live"
    );
    m.pump.cook.set_external(
        "Sprite",
        Stream::new(1).with("texture_id", Column::Scalar(vec![7.0])),
    );
    assert!(
        !cook_publishes_live_geometry(&m.pump.cook),
        "a sprite object (texture_id only) does NOT recuse — it stays on the GPU stamp"
    );
    m.pump.cook.set_external(
        "Star",
        Stream::new(1).with("geometry_id", Column::Scalar(vec![5.0])),
    );
    assert!(
        cook_publishes_live_geometry(&m.pump.cook),
        "a live vector object (geometry_id > 0) recuses — the GPU has no vector route"
    );
    // The atlas sentinel is not a live vector.
    let mut m0 = MotionState::new();
    m0.pump.cook.set_external(
        "Zero",
        Stream::new(1).with("geometry_id", Column::Scalar(vec![0.0])),
    );
    assert!(
        !cook_publishes_live_geometry(&m0.pump.cook),
        "geometry_id 0 is the sentinel, not a live vector"
    );
}

#[test]
fn disabled_or_multi_sink_or_scoped_is_always_cpu() {
    // Every gate is independent: flip one and the GPU is refused even when a
    // fully-claimed plan is on offer.
    assert_eq!(gpu_route(false, 1, true, &[], 3), GpuRoute::Cpu);
    assert_eq!(gpu_route(true, 2, true, &[], 3), GpuRoute::Cpu);
    assert_eq!(gpu_route(true, 0, true, &[], 3), GpuRoute::Cpu);
    assert_eq!(gpu_route(true, 1, false, &[], 3), GpuRoute::Cpu);
}

/// **Two seams take the GPU now** — the assertion that flipped when the pump
/// went plural, and the reason it was pinned before it could.
///
/// A multi-input kernel (`motion.look_at`) leaves a plan with two CPU
/// boundaries and real GPU work behind them (measured in `ph2d-gpu-cook`'s
/// `boundary_arity`, item (d)). This used to assert `Cpu`: the pump took ONE
/// boundary, so the route forfeited the GPU for the whole frame — never
/// wrong, and never fast either.
///
/// It was written as a gate rather than left implicit because
/// `_ => GpuRoute::Cpu` is a catch-all: it swallowed the two-seam case the
/// day it became reachable **without a single gate changing colour**. Being
/// pinned is what made the change visible when it came.
#[test]
fn two_seams_take_the_hybrid_route() {
    // Two DISTINCT nodes — `node()` is a single stand-in id, and reusing it
    // would model one node consumed on two ports (the duplicate entry the
    // pump dedupes), which is a different shape.
    let (a, b) = (NodeId(7), NodeId(8));
    assert_eq!(
        gpu_route(true, 1, true, &[(a, 0), (b, 0)], 3),
        GpuRoute::Hybrid
    );
    // Still no compute win, still the CPU — the dispatching-stage rule is
    // independent of how many seams there are.
    assert_eq!(
        gpu_route(true, 1, true, &[(a, 0), (b, 0)], 0),
        GpuRoute::Cpu
    );
}

#[test]
fn fully_claimed_plan_runs_fully_on_the_gpu() {
    assert_eq!(gpu_route(true, 1, true, &[], 3), GpuRoute::FullyGpu);
}

#[test]
fn a_boundary_with_gpu_work_is_hybrid() {
    assert_eq!(
        gpu_route(true, 1, true, &[(node(), 0)], 2),
        GpuRoute::Hybrid
    );
    // One dispatching stage is enough.
    assert_eq!(
        gpu_route(true, 1, true, &[(node(), 0)], 1),
        GpuRoute::Hybrid
    );
}

#[test]
fn a_boundary_with_no_dispatching_suffix_recuses_to_cpu() {
    // A lone pass-through `output` above the boundary is no compute win —
    // uploading the sink stream just to lower it — so it stays on the CPU.
    assert_eq!(gpu_route(true, 1, true, &[(node(), 0)], 0), GpuRoute::Cpu);
}
