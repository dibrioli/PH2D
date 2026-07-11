//! The M3 distribution demo — the **default Motion document**: two small,
//! side-by-side point distributions, each animated by the value domain. On the LEFT
//! a **hexagonal lattice** melts toward noise and reforms (`motion.lattice`); on the
//! RIGHT a random cloud **relaxes into a honeycomb** and dissolves via Lloyd's
//! algorithm (`motion.voronoi`). Two independent scenes (each its own `motion.output`
//! sink — the bridge composes several into one draw), kept deliberately small so each
//! new node reads on its own. A `#[path]` sibling of `motion_state`, kept out of it
//! for the LOC cap.
//!
//! ```text
//! LEFT  (lattice): lattice → move(−6) → tint(amber) → output   lfo_jitter → lattice.jitter
//! RIGHT (voronoi): voronoi → move(+6) → tint(cyan)  → output   lfo_relax  → voronoi.relax
//! ```
//!
//! - **lattice** (`motion.lattice`, doc 23): the hexagonal (triangular) packing; the
//!   `jitter` `value.lfo` displaces each point by a hashed offset, so the honeycomb
//!   **shimmers and reforms**.
//! - **voronoi** (`motion.voronoi`, doc 23): Lloyd's relaxation toward a centroidal
//!   Voronoi tessellation; the `relax` `value.lfo` plays that relaxation forward, so
//!   the cloud **organises into an even honeycomb and dissolves back**.
//!
//! The payoff: the two ordered/organic distributions that round out the family
//! (`grid`, `fibonacci`, `scatter` are the others), each driven by the value domain,
//! on one legible canvas. See docs/Motion Nodes/23 (lattice + voronoi). The whole
//! value/pulse vocabulary + the other M3/M4 nodes stay registered (drop them in).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 220.0;
/// The two scenes' card rows in graph space (stacked, so the editor reads cleanly).
const LATTICE_ROW: f32 = 0.0;
const VORONOI_ROW: f32 = 320.0;

/// Author both distribution scenes into `g`; returns their Output nodes (the sinks),
/// the lattice's first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let lattice_out = build_lattice_scene(g)?;
    let voronoi_out = build_voronoi_scene(g)?;
    Some(vec![lattice_out, voronoi_out])
}

/// LEFT: a hexagonal lattice shimmering under jitter. Returns its Output node.
fn build_lattice_scene(g: &mut Graph) -> Option<NodeId> {
    let lattice = g.add_node("motion.lattice");
    let mv = g.add_node("motion.move");
    let tint = g.add_node("motion.tint");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [(lattice, 0.0), (mv, 1.0), (tint, 2.0), (output, 3.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: LATTICE_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: 0.0,
            y: LATTICE_ROW + 160.0,
        },
    );

    g.connect(Edge {
        from: (lattice, 0),
        to: (mv, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (mv, 0),
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
        to: (lattice, 0),
        delayed: false,
    })
    .ok()?; // → jitter

    // A 6×7 hex lattice, shifted onto the left half.
    g.set_param(lattice, "rows", 6.0);
    g.set_param(lattice, "cols", 7.0);
    g.set_param(lattice, "spacing", 0.7);
    g.set_param(mv, "dx", -6.0);
    g.set_param(mv, "dy", 0.0);
    // A warm amber honeycomb.
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.70);
    g.set_param(tint, "b", 0.20);
    // lfo → jitter: a slow (4 s) sine about 0.25, ±0.25 → jitter ∈ [0, 0.5] (world
    // units), so the packing melts toward noise and snaps back.
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 4.0);
    g.set_param(lfo, "amplitude", 0.25);
    g.set_param(lfo, "offset", 0.25);
    Some(output)
}

/// RIGHT: a cloud relaxing into a CVT via Lloyd. Returns its Output node.
fn build_voronoi_scene(g: &mut Graph) -> Option<NodeId> {
    let voronoi = g.add_node("motion.voronoi");
    let mv = g.add_node("motion.move");
    let tint = g.add_node("motion.tint");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [(voronoi, 0.0), (mv, 1.0), (tint, 2.0), (output, 3.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: VORONOI_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: 0.0,
            y: VORONOI_ROW + 160.0,
        },
    );

    g.connect(Edge {
        from: (voronoi, 0),
        to: (mv, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (mv, 0),
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
        to: (voronoi, 0),
        delayed: false,
    })
    .ok()?; // → relax

    // A 64-point cloud in a 5×5 domain, shifted onto the right half.
    g.set_param(voronoi, "count", 64.0);
    g.set_param(voronoi, "width", 5.0);
    g.set_param(voronoi, "height", 5.0);
    g.set_param(voronoi, "iterations", 10.0);
    g.set_param(mv, "dx", 6.0);
    g.set_param(mv, "dy", 0.0);
    // A cool cyan cloud.
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.25);
    g.set_param(tint, "g", 0.80);
    g.set_param(tint, "b", 0.95);
    // lfo → relax: a slow (5 s) sine about 0.5, ±0.5 → relax ∈ [0, 1], so the cloud
    // organises into a honeycomb and dissolves back to noise.
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 5.0);
    g.set_param(lfo, "amplitude", 0.5);
    g.set_param(lfo, "offset", 0.5);
    Some(output)
}
