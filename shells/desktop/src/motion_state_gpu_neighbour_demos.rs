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
//! - **`=7`, the murmuration** — a simulation STEP. `boids(262.144, spread √N)
//!   → scale → output` with the loop `output ──pre──> boids.state`; the tick IS
//!   the iteration, so it dispatches once per frame. **Spread ON is load-bearing**,
//!   not decoration: the node's default seed packs the flock into a fixed ~6×6 box,
//!   and a quarter million agents in a handful of cells is back to `O(N²)` (the
//!   grid cannot help a crowd that dense). And the FORCES are part of the sizing:
//!   the cost that matters is the EQUILIBRIUM (the settled flock), set by
//!   seek-vs-separation — see the measured table in
//!   `build_gpu_boids_demo_document` (~2,5 ms at rest, 5,3–6,3 ms settled).
//! - **`=8`, the breathing packing** — a relaxation SOLVER. `grid(360²) → collide
//!   → output` with an LFO on `spread`; it sweeps `iterations` times per cook, and
//!   the sequencer REBUILDS the grid between sweeps because each sweep moves the
//!   very column the grid indexes (2,5–4,7 ms across the LFO's range; the ceiling
//!   is millions — 38 ms at 1 M discs). ⚠️ Two things about this cost were reported
//!   and fixed: it used to STEP at the LFO's midpoint (the cell cull, in
//!   `motion.collide`'s kernel), and the scene used to be sized with no headroom
//!   for the Amplitude knob (see `build_gpu_collide_demo_document`).
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

/// **The murmuration** (`PH2D_GPU_COOK_DEMO=7`) — the ready-to-smoke scene for the
/// neighbourhood sim on the device (ADR-0134). Returns the sink. (A quarter
/// million agents with forces tuned for a bounded EQUILIBRIUM; the ceiling is
/// millions — see `count` below.)
pub(super) fn build_gpu_boids_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    let boids = g.add_node("motion.boids");
    // 262.144 agents, and the FORCES are part of the sizing — the cost of a flock
    // is its EQUILIBRIUM density, and that is set by seek-vs-separation, not by
    // the count alone. Two rounds of Enio's smoke taught this the hard way:
    //
    // 1. "queda de FPS quando boids se aproximam" — measured 600 ticks in, resized
    //    1 M → 524 k. ⚠️ That window was read off a curve STILL CLIMBING — the
    //    fixture did not contain the phenomenon (the settled flock).
    // 2. "até metade se juntar rodou bem, depois queda grave" — the missing half.
    //    Measured to EQUILIBRIUM (4800 ticks = 80 s), 262 144 agents, ms/tick:
    //
    //      seek 0.35 (old)        climbs to 28,5 and PLATEAUS — a dense ball
    //                             parked on the static target (124+ % of a frame)
    //      orbiting target        26,5 — REFUTED: the flock converges onto the
    //                             moving target and rides it as a dense comet
    //      seek 0.05, sep 2.4     9,4 — bounded
    //      seek 0.02, sep 3.0     5,3–6,3 — bounded, breathing gently  ← shipped
    //
    // With the shipped tuning the equilibrium costs ≤38 % of a 60 fps frame and
    // holds there indefinitely (verified through 80 s). At 524 k even the best
    // tuning plateaus ~12 ms + render on top — no headroom, so the count came
    // down too. The ceiling stays MILLIONS (raise `count` — 4 M sims at 3,6 ms
    // when density is bounded); the DEMO is sized to never stutter doing the
    // thing it exists to show.
    g.set_param(boids, "count", 262_144.0);
    // Load-bearing: without it the agents SEED into a fixed box and the grid
    // cannot help (O(N²)). √N holds the seed density → the grid starts O(N);
    // the forces below are what keep it O(N) once settled.
    g.set_param(boids, "spread", 1.0);
    g.set_param(boids, "radius", 2.0);
    // The equilibrium knobs (see the table above): separation pushes the settled
    // spacing OPEN, seek is a global attractor whose strength sets how dense the
    // flock parks. 0.35 collapsed at ANY count; 0.02 is just enough to keep the
    // murmuration on screen without ever packing it.
    g.set_param(boids, "separation", 3.0);
    g.set_param(boids, "alignment", 1.4);
    g.set_param(boids, "cohesion", 0.4);
    g.set_param(boids, "seek", 0.02);
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

/// **The spread SWEEP** (`PH2D_GPU_COOK_DEMO=9`, ADR-0134 Fase 5) — the DIAGNOSTIC
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
