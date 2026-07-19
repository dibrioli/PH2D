//! The **million-boid murmuration** (`PH2D_GPU_COOK_DEMO=7`, ADR-0134) — a
//! sibling of `motion_state_gpu_demos.rs`, split for the same reason the panel
//! demo was: it answers a question none of the others can.
//!
//! Every scene next door is **embarrassingly parallel** — a grid element, a
//! particle, an emitter's id, each evolves reading only ITSELF and its uniforms.
//! The GPU has always been good at that. This one is the opposite: a boid reads
//! its NEIGHBOURS, an all-pairs `O(N²)` that is exactly why boids has been a
//! few-hundred-agent toy for forty years. The breakthrough is the spatial grid
//! (ADR-0134): a counting sort into perception-radius cells, so each agent sweeps
//! a 3×3 neighbourhood instead of the whole flock, and the interacting sim joins
//! the throughput ones at a **million agents**.
//!
//! `boids(1.048.576, spread √N) → scale → output`, with the loop
//! `output ──pre──> boids.state`. **Spread ON is load-bearing**, not decoration:
//! the node's default seed packs the flock into a fixed ~6×6 box, and a million
//! agents in a handful of cells is back to `O(N²)` (measured: ~10 s/tick — the
//! grid cannot help a crowd that dense). Spread grows the seed cloud with √N so
//! the density — agents per perception cell — stays a lively murmuration, and the
//! grid stays `O(N)`. Measured on the RTX at ~15.5 ms/tick for the cook (probe
//! `gpu_boids_scale::how_far_does_the_flock_scale`); the ceiling above it is the
//! 2 GiB instance-buffer binding (~11.67 M agents), not the sim.
//!
//! Under `PH2D_GPU_COOK=1` the whole chain is claimed with no boundary — the grid
//! build, the neighbour sweep, the integrate and the `scale` all run on the
//! device, zero readback. Auto-plays on tool entry like every boot document; zoom
//! out and the cloud gathers, splits and re-forms — the flock is what emerges.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// **The million-boid murmuration** (`PH2D_GPU_COOK_DEMO=7`) — the ready-to-smoke
/// scene for the neighbourhood sim on the device (ADR-0134). Returns the sink.
pub(super) fn build_gpu_boids_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let boids = g.add_node("motion.boids");
    // 2²⁰ = 1.048.576 agents. The grid keeps this O(N) at ~15.5 ms/tick on the
    // RTX; the ceiling above is the instance-buffer binding (~11.67 M), never the
    // count the CPU reference could bear — that path only computes the same answer.
    g.set_param(boids, "count", 1_048_576.0);
    // Load-bearing: without it a million agents pack into a fixed box and the grid
    // cannot help (O(N²), ~10 s/tick). √N holds the density → the grid stays O(N).
    g.set_param(boids, "spread", 1.0);
    // Reynolds weights for a coherent, spread murmuration — strong alignment so
    // the flock flies as one, gentle seek so it gathers WITHOUT collapsing to a
    // point (a collapse would re-densify and drag the grid back toward O(N²)).
    g.set_param(boids, "radius", 2.0);
    g.set_param(boids, "separation", 1.6);
    g.set_param(boids, "alignment", 1.4);
    g.set_param(boids, "cohesion", 0.6);
    g.set_param(boids, "seek", 0.35);
    g.set_param(boids, "max_speed", 5.0);
    g.set_param(boids, "seed", 7.0);

    // Grains, not blobs: a million unit quads over the √N field would be a solid
    // sheet. Shrink them so the cloud reads as a swarm of specks.
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.3);

    let out = g.add_node("motion.output");
    for (i, n) in [boids, scale, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 200.0,
                y: 120.0,
            },
        );
    }

    // The feedback the artist never draws: last tick's flock state into the head.
    g.connect(Edge {
        from: (boids, 0),
        to: (boids, 2),
        delayed: true,
    })
    .ok()?;
    g.connect(Edge {
        from: (boids, 0),
        to: (scale, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (scale, 0),
        to: (out, 0),
        delayed: false,
    })
    .ok()?;
    g.validate(reg).ok()?;
    Some(vec![out])
}
