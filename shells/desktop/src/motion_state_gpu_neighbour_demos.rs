//! The **neighbourhood** scenes (`PH2D_GPU_COOK_DEMO=7` and `=8`, ADR-0134) — a
//! sibling of `motion_state_gpu_demos.rs`, split for the same reason the panel
//! demo was: they answer a question none of the others can.
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
//! The two scenes are the two SHAPES that neighbourhood work comes in, which is
//! why they sit together — and why the second one matters as much as the first:
//!
//! - **`=7`, the murmuration** — a simulation STEP. `boids(1.048.576, spread √N)
//!   → scale → output` with the loop `output ──pre──> boids.state`; the tick IS
//!   the iteration, so it dispatches once per frame. **Spread ON is load-bearing**,
//!   not decoration: the node's default seed packs the flock into a fixed ~6×6 box,
//!   and a million agents in a handful of cells is back to `O(N²)` (measured:
//!   ~10 s/tick — the grid cannot help a crowd that dense). Spread grows the seed
//!   cloud with √N so the agents-per-cell stays a lively murmuration and the grid
//!   stays `O(N)` (~15,5 ms/tick).
//! - **`=8`, the breathing packing** — a relaxation SOLVER. `grid(512²) → collide
//!   → output` with an LFO on `spread`; it sweeps `iterations` times per cook, and
//!   the sequencer REBUILDS the grid between sweeps because each sweep moves the
//!   very column the grid indexes (6,4–9,5 ms at 262 k across the LFO's whole
//!   range, 38 ms at a million). ⚠️ That range used to STEP at the LFO's midpoint —
//!   see the cell cull in `motion.collide`'s kernel.
//!
//! Together they are the argument that the grid is a **service** (ADR-0134 D2) and
//! not a boids-shaped detour: one client keeps its state across ticks, the other
//! has no state at all and iterates within a single cook, and both get their
//! neighbours from the same counting sort.
//!
//! Under `PH2D_GPU_COOK=1` each chain is claimed with no boundary — grid build,
//! neighbour sweep, integrate and `scale` all on the device, zero readback. Both
//! auto-play on tool entry like every boot document; zoom out and watch.

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

/// **The breathing packing** (`PH2D_GPU_COOK_DEMO=8`, ADR-0134 Fase 5) — the
/// SECOND neighbourhood client, and the one that proves the grid is a reusable
/// service rather than a boids-shaped detour.
///
/// `grid(512×512) → collide → output`, with a `value.lfo` driving the collider's
/// `spread`. 262.144 discs start on a lattice whose pitch (0.25) is far tighter
/// than the contact distance (`2·radius` = 0.6), so essentially every disc begins
/// overlapping several neighbours and the whole field shoves itself apart into a
/// packing. The LFO then makes the discs grow and shrink, so the packing
/// **breathes** — without it a `Effect::Pure` relaxation over a static lattice
/// would re-cook the identical picture every frame and read as a still image.
///
/// **It is the first ITERATED kernel.** Boids dispatches once per tick (the tick
/// IS the iteration); this sweeps `iterations` times per cook, and because every
/// sweep moves the very column the grid indexes, the sequencer REBUILDS the grid
/// between sweeps (`GridSpec::sweeps_param`). Measured on the RTX at 8 sweeps:
/// 262.144 discs ≈ **6,8 ms/cook**, 1.048.576 ≈ 38 ms, 4.194.304 ≈ 288 ms — so
/// the artist can raise `rows`/`cols` into the millions; this scene is sized to
/// stay comfortably inside a 60 fps frame while it breathes.
///
/// ⚠️ **The breath used to cost a STEP, and that was a kernel bug, not a scene
/// that was too big** (Enio, 2026-07-19: *"profunda queda de FPS nos valores
/// positivos do LFO"*). The swept neighbourhood is `ceil(min_dist / cell)` cells
/// in each direction, and this LFO is centred on exactly 1.0 — the value where
/// that ceiling steps 2 → 3, i.e. 25 cells → 49. Measured: **7,58 ms at spread
/// 0.999, 13,08 ms at 1.001**. The kernel now culls a cell whose nearest point is
/// out of range before touching its discs, which is why the offset can stay at 1.0
/// (moving it would only have hidden the step at a different value of a slider the
/// artist can still reach).
///
/// The CPU reference is `O(N²·iterations)`: a million discs would be ~8·10¹² pair
/// tests, which is why this node had never left a few thousand. It only became
/// portable at all once the reference stopped being an in-place Gauss–Seidel sweep
/// (sequential by definition) and became averaged Jacobi — a change made for its
/// own reason, that the packing must not depend on the stream's index order.
pub(super) fn build_gpu_collide_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let src = g.add_node("motion.grid");
    // 512 × 512 = 262.144 discs on a pitch far tighter than the contact distance.
    g.set_param(src, "rows", 512.0);
    g.set_param(src, "cols", 512.0);
    g.set_param(src, "gap_x", 0.25);
    g.set_param(src, "gap_y", 0.25);

    let col = g.add_node("motion.collide");
    g.set_param(col, "radius", 0.3);
    g.set_param(col, "iterations", 8.0);
    g.set_param(col, "strength", 1.0);

    // The breath: `spread` multiplies the radius, so the discs swell and the
    // packing pushes wider, then relaxes back. Centred on 1 so the scene reads as
    // the collider's own default at the midpoint of the cycle.
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 6.0);
    g.set_param(lfo, "amplitude", 0.35);
    g.set_param(lfo, "offset", 1.0);

    let out = g.add_node("motion.output");
    for (i, n) in [src, col, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 200.0,
                y: 120.0,
            },
        );
    }
    g.set_pos(lfo, Pos { x: 80.0, y: 300.0 });

    for (from, to, port) in [(src, col, 0), (lfo, col, 1), (col, out, 0)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}
