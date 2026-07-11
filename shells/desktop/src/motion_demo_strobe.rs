//! The M4 simulation demo — the **default Motion document**: two small, side-by-side
//! sequential simulations. On the LEFT a **whip** swings and falls from a sliding
//! anchor (`motion.verlet_rope`); on the RIGHT a **flock** wheels after a moving
//! target (`motion.boids`). Two independent scenes (each its own `motion.output`
//! sink — the bridge composes several into one draw), kept deliberately small so
//! each new node reads on its own. A `#[path]` sibling of `motion_state`, kept out
//! of it for the LOC cap.
//!
//! ```text
//! LEFT  (whip):   rope  → tint(amber) → output       lfo_anchor → rope.anchor_x
//!                 rope --pre--> rope.state
//! RIGHT (flock):  boids → tint(cyan)  → output       lfo_target → boids.target_x
//!                 boids --pre--> boids.state
//! ```
//!
//! - **verlet-rope** (`motion.verlet_rope`, doc 21): a Verlet chain pinned at the
//!   head; gravity pulls it into a hanging curve and the `anchor_x` `value.lfo`
//!   slides the pin, so the strand **whips** with follow-through.
//! - **boids** (`motion.boids`, doc 21): Reynolds flocking (separation / alignment /
//!   cohesion) with a seek pull to a target; the `target_x` `value.lfo` slides the
//!   target, so the **whole flock wheels to chase it**.
//!
//! The payoff: two sequential simulations on the `pre` self-loop — one constrained
//! and deterministic, one emergent — each herded by the value domain, on one
//! legible canvas. See docs/Motion Nodes/21 (verlet-rope + boids). The whole
//! value/pulse vocabulary + the other M3 nodes stay registered (drop them in the
//! editor).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 220.0;
/// The two scenes' card rows in graph space (stacked, so the editor reads cleanly).
const ROPE_ROW: f32 = 0.0;
const BOIDS_ROW: f32 = 320.0;

/// Author both sim scenes into `g`; returns their Output nodes (the sinks), the
/// rope's first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let rope_out = build_rope_scene(g)?;
    let boids_out = build_boids_scene(g)?;
    Some(vec![rope_out, boids_out])
}

/// LEFT: a whip pinned at a sliding anchor. Returns its Output node.
fn build_rope_scene(g: &mut Graph) -> Option<NodeId> {
    let rope = g.add_node("motion.verlet_rope");
    let tint = g.add_node("motion.tint");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [(rope, 0.0), (tint, 1.0), (output, 2.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: ROPE_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: 0.0,
            y: ROPE_ROW + 160.0,
        },
    );

    // Visible trunk + the lfo into the anchor + the pre self-loop.
    g.connect(Edge {
        from: (rope, 0),
        to: (tint, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (tint, 0),
        to: (output, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (lfo, 0),
        to: (rope, 0),
        delayed: false,
    })
    .ok()?; // → anchor_x
    g.connect(Edge {
        from: (rope, 0),
        to: (rope, 2),
        delayed: true,
    })
    .ok()?; // pre state loop

    // A 24-point rope, 5 units long, hung on the left (x≈−6) and gently swinging.
    g.set_param(rope, "count", 24.0);
    g.set_param(rope, "length", 5.0);
    g.set_param(rope, "gravity", 9.0);
    g.set_param(rope, "pin_tail", 0.0); // free tail → a whip
    // A warm amber strand.
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.70);
    g.set_param(tint, "b", 0.20);
    // lfo → anchor_x: a slow (3 s) sine about x = −6, ±1.5 → the pin slides and the
    // strand whips. Unconnected `in` → a length-1 GLOBAL value (one anchor).
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 3.0);
    g.set_param(lfo, "amplitude", 1.5);
    g.set_param(lfo, "offset", -6.0);
    Some(output)
}

/// RIGHT: a flock seeking a sliding target. Returns its Output node.
fn build_boids_scene(g: &mut Graph) -> Option<NodeId> {
    let boids = g.add_node("motion.boids");
    let tint = g.add_node("motion.tint");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [(boids, 0.0), (tint, 1.0), (output, 2.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: BOIDS_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: 0.0,
            y: BOIDS_ROW + 160.0,
        },
    );

    g.connect(Edge {
        from: (boids, 0),
        to: (tint, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (tint, 0),
        to: (output, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (lfo, 0),
        to: (boids, 0),
        delayed: false,
    })
    .ok()?; // → target_x
    g.connect(Edge {
        from: (boids, 0),
        to: (boids, 2),
        delayed: true,
    })
    .ok()?; // pre state loop

    // A 48-agent flock, homed on the right (x≈+6).
    g.set_param(boids, "count", 48.0);
    g.set_param(boids, "separation", 1.6);
    g.set_param(boids, "alignment", 1.0);
    g.set_param(boids, "cohesion", 0.9);
    g.set_param(boids, "seek", 1.0);
    g.set_param(boids, "max_speed", 4.0);
    // A cool cyan swarm.
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.20);
    g.set_param(tint, "g", 0.80);
    g.set_param(tint, "b", 0.95);
    // lfo → target_x: a 4 s sine about x = +6, ±3 → the flock wheels to chase it.
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 4.0);
    g.set_param(lfo, "amplitude", 3.0);
    g.set_param(lfo, "offset", 6.0);
    Some(output)
}
