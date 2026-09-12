//! The **neighbourhood** scenes (`PH2D_GPU_COOK_DEMO=7` and `=8`, ADR-0140) — a
//! sibling of `motion_state_gpu_demos.rs`, split for the same reason the panel
//! demo was: they answer a question none of the others can.
//!
//! Every scene next door is **embarrassingly parallel** — a grid element, a
//! particle, an emitter's id, each evolves reading only ITSELF and its uniforms.
//! The GPU has always been good at that. This one is the opposite: a boid reads
//! its NEIGHBOURS, an all-pairs `O(N²)` that is exactly why boids has been a
//! few-hundred-agent toy for forty years. The breakthrough is the spatial grid
//! (ADR-0140): a counting sort into perception-radius cells, so each agent sweeps
//! a 3×3 neighbourhood instead of the whole flock, and the interacting sim joins
//! the throughput ones at a **million agents**.
//!
//! The two scenes are the two SHAPES that neighbourhood work comes in, which is
//! why they sit together — and why the second one matters as much as the first:
//!
//! - **`=7`, the murmuration** — a simulation STEP. `boids(1.048.576, spread √N,
//!   seek 0) → scale → output` with the loop `output ──pre──> boids.state`; the
//!   tick IS the iteration, so it dispatches once per frame. **Spread ON is
//!   load-bearing** (the seed must not pack into a box) and **seek 0 is what lets
//!   a MILLION hold 60 fps**: any global attractor turns superlinear at this
//!   count (its pull compresses the core with the whole swarm's weight — even
//!   seek 0.005 measured 26–30 ms/tick settled), while a pure local murmuration
//!   can only loosen (9,9 → 5,4 ms over 160 s, monotonically down). The measured
//!   saga is in `build_gpu_boids_demo_document`.
//! - **`=8`, the breathing packing** — a relaxation SOLVER. `grid(360²) → collide
//!   → output` with an LFO on `spread`; it sweeps `iterations` times per cook, and
//!   the sequencer REBUILDS the grid between sweeps because each sweep moves the
//!   very column the grid indexes (2,5–4,7 ms across the LFO's range; the ceiling
//!   is millions — 38 ms at 1 M discs). ⚠️ Two things about this cost were reported
//!   and fixed: it used to STEP at the LFO's midpoint (the cell cull, in
//!   `motion.collide`'s kernel), and the scene used to be sized with no headroom
//!   for the Amplitude knob (see `build_gpu_collide_demo_document`).
//!
//! Together they are the argument that the grid is a **service** (ADR-0140 D2) and
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
/// scene for the neighbourhood sim on the device (ADR-0140). Returns the sink.
/// (The million holds 60 fps because there is NO global attractor — see `count`
/// below for the three measured rounds that led here.)
pub(super) fn build_gpu_boids_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let boids = g.add_node("motion.boids");
    // 2²⁰ = 1.048.576 agents, and the key that makes a MILLION hold 60 fps is
    // `seek = 0`. Three rounds of Enio's smoke, all measured to equilibrium
    // (`gpu_boids_scale.rs::where_does_the_flock_settle`), built this table:
    //
    // 1. "queda de FPS quando boids se aproximam" — resized 1 M → 524 k off a
    //    600-tick window. ⚠️ The curve was STILL CLIMBING; the fixture did not
    //    contain the settled flock.
    // 2. "até metade se juntar, depois queda grave" — measured to 80 s at 262 k:
    //    seek 0.35 plateaus at 28,5 ms (a dense ball parked on the target, at ANY
    //    count); an ORBITING target was refuted (the flock rides it as a dense
    //    comet); seek 0.02 + sep 3.0 settles at 5,3–6,3 ms. Shipped that, at 262 k.
    // 3. "tente 1 milhão" — and at 1 M the attractor law turns SUPERLINEAR: the
    //    pull compresses the core with the whole swarm's weight, so the same
    //    seek 0.02 tuning plateaus at 74–80 ms, sep 4.0 at ~50, and even
    //    seek 0.005 at 26–30. **No attractor fits a million.** With `seek = 0`
    //    (a pure local murmuration — real starlings have no global target) the
    //    density can only FALL: measured over 160 s, r 1.5 runs 9,9 → 5,4 ms and
    //    r 2.0 runs 13,9 → 5,8, both monotonically DOWN — the worst frame is the
    //    opening one, and it improves from there.
    //
    // Shipped: 1 M, seek 0, radius 1.5 — opening ≈10 ms (60 % of a frame, the
    // safe side of r 2.0's 84 % first window), falling for as long as you watch.
    // The swarm slowly disperses instead of balling up; that IS the murmuration.
    g.set_param(boids, "count", 1_048_576.0);
    // Load-bearing: without it the agents SEED into a fixed box and the grid
    // cannot help (O(N²)). √N holds the seed density → the grid starts O(N);
    // `seek = 0` is what keeps it O(N) forever after.
    g.set_param(boids, "spread", 1.0);
    g.set_param(boids, "radius", 1.5);
    g.set_param(boids, "separation", 2.4);
    g.set_param(boids, "alignment", 1.4);
    g.set_param(boids, "cohesion", 0.6);
    // ⚠️ ZERO is load-bearing at this count — see round 3 above. Even 0.005
    // measured 26–30 ms; the gate pins it.
    g.set_param(boids, "seek", 0.0);
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

/// **The breathing packing** (`PH2D_GPU_COOK_DEMO=8`, ADR-0140 Fase 5) — the
/// SECOND neighbourhood client, and the one that proves the grid is a reusable
/// service rather than a boids-shaped detour.
///
/// `grid(360×360) → collide → output`, with a `value.lfo` driving the collider's
/// `spread`. 129.600 discs start on a lattice whose pitch (0.25) is far tighter
/// than the contact distance (`2·radius` = 0.6), so essentially every disc begins
/// overlapping several neighbours and the whole field shoves itself apart into a
/// packing. The LFO then makes the discs grow and shrink, so the packing
/// **breathes** — without it a `Effect::Pure` relaxation over a static lattice
/// would re-cook the identical picture every frame and read as a still image.
///
/// **It is the first ITERATED kernel.** Boids dispatches once per tick (the tick
/// IS the iteration); this sweeps `iterations` times per cook, and because every
/// sweep moves the very column the grid indexes, the sequencer REBUILDS the grid
/// between sweeps (`GridSpec::sweeps_param`). Measured on the RTX at 8 sweeps and
/// `spread` 1: 262.144 discs ≈ **6,2 ms/cook**, 1.048.576 ≈ 37 ms, 4.194.304 ≈
/// 274 ms — so the artist can raise `rows`/`cols` into the millions. This scene is
/// sized well under that, for the reason in the `rows` comment: the ceiling of the
/// MACHINE and the size of a DEMO are different questions, and a demo that spends
/// the whole frame at rest has nothing left for the knobs.
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
    // 360 × 360 = 129.600 discs on a pitch far tighter than the contact distance.
    // ⚠️ SIZED FOR HEADROOM, not for the ceiling (which is millions — see below).
    // What this node costs is contacts-per-disc, and `spread` grows that with the
    // AREA of the interaction disc, so a scene tuned to just fit AT REST has none
    // left the moment the artist touches the LFO's Amplitude — which is exactly
    // what happened (Enio, 2026-07-19). Measured, 8 sweeps, at the breath's peak
    // and beyond it:
    //
    //     discs    spread 1.35   spread 2.0   spread 3.0
    //   262 144       9.06 ms     15.94 ms     30.12 ms
    //   129 600       4.71 ms      7.93 ms     14.58 ms
    //
    // At 262 144 the default breath already spent 54 % of a 60 fps frame and
    // Amplitude 1.0 spent 95 %. At 129 600 the artist can DOUBLE the Amplitude and
    // still hold 60 fps — and 129 600 is still ~100× what this node could reach
    // before the grid existed.
    g.set_param(src, "rows", 360.0);
    g.set_param(src, "cols", 360.0);
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

/// **The spread SWEEP** (`PH2D_GPU_COOK_DEMO=9`, ADR-0140 Fase 5) — the DIAGNOSTIC
/// scene, built because `=8` is a poor instrument for judging a PERFORMANCE fix.
///
/// A breathing blob hides frame time: the packing looks the same whether the cook
/// costs 4 ms or 14, so all you can feel is gross jank. This scene turns the cost
/// into something you can WATCH — on the very GPU-utilisation meter that surfaced
/// the report. Same `grid → collide → output`, but the `value.lfo` is a **slow,
/// LINEAR triangle** (not the fast sine of `=8`) that sweeps `spread` from ~0.3 to
/// ~2.5 and back over 12 s.
///
/// Why linear-and-wide makes the two fixes legible:
/// - **The reach-boundary step (the kernel fix).** The swept neighbourhood is
///   `ceil(2·spread)` cells across, so its size steps at `spread` = 0.5, 1.0, 1.5,
///   2.0 — FOUR boundaries inside this sweep, at EVEN spacing in time because the
///   ramp is linear. Before the fix, the GPU meter would climb the sweep in four
///   distinct STAIRSTEPS (and back down); after it, the meter is a smooth mountain.
///   A staircase vs. a mountain is a signature you can read at a glance, and its
///   absence is the proof. (A fast sine centred on 1.0 crosses one boundary four
///   times per cycle at high speed — a buzz you cannot localise.)
/// - **The honest area-cost (the sizing question).** As the ramp climbs toward
///   spread 2.5 the discs' contact radius grows, so the meter rises — smoothly, a
///   slope not a cliff. That is the real cost of the knob, made visible as a
///   gradual rise rather than a mysterious drop.
///
/// 129.600 discs (same as `=8`) keeps the whole sweep inside a 60 fps frame
/// (measured: ~2,5 ms at the valley, ~10 ms at the peak of this range), so the
/// meter swings through a wide, readable band without pinning at either rail.
pub(super) fn build_gpu_sweep_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let src = g.add_node("motion.grid");
    g.set_param(src, "rows", 360.0);
    g.set_param(src, "cols", 360.0);
    g.set_param(src, "gap_x", 0.25);
    g.set_param(src, "gap_y", 0.25);

    let col = g.add_node("motion.collide");
    g.set_param(col, "radius", 0.3);
    g.set_param(col, "iterations", 8.0);
    g.set_param(col, "strength", 1.0);

    // The sweep: a SLOW, LINEAR triangle. `wave 1` = Triangle (no jump, unlike Saw),
    // so the ramp rate is constant and any step in cost reads as a hitch at a fixed
    // phase — not confounded by a sine's changing speed. offset 1.4 ± amplitude 1.1
    // ⇒ spread ∈ [0.3, 2.5], crossing the reach boundaries at 0.5/1.0/1.5/2.0. A
    // long period so the eye (and the meter) can follow it.
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "wave", 1.0);
    g.set_param(lfo, "period", 12.0);
    g.set_param(lfo, "amplitude", 1.1);
    g.set_param(lfo, "offset", 1.4);

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
