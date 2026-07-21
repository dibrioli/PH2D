//! The **simulation-zone** scene (`PH2D_GPU_COOK_DEMO=10`, ADR-0134 §1) — the
//! smoke for the `sim.zone` family running on the device.
//!
//! Every other GPU scene either has no state (`=1`/`=2`/`=8`/`=9`) or carries it
//! on a bare `motion.integrate` self-loop (`=3`/`=4`/`=5`/`=7`). This one runs a
//! `sim.zone` — the state-loop CONTAINER the artist actually reaches for — 100% on
//! the GPU, which is what unlocked the boot snow's integrator on the device:
//!
//! ```text
//!   grid → move ─> zone.init                zone.out ─> scale ─> output
//!                  zone.out ⊙──pre──> wind → buoyancy → sim.step → sim.collide ─> zone.state
//! ```
//!
//! It is the boot snow's physics (doc 52 — snow falling into a shallow sea,
//! punching the surface, tapping the bed, and bobbing on the swell) with ONE
//! thing removed: birth and death. The snow's `sim.spawn`/`lifetime`/`cull`/
//! `combine` are count-CHANGING and have no kernel yet, so the artist document
//! still cooks its interior on the pump (the `sim.zone` boundary the coverage
//! census reports); it recedes cleanly and the render suffix stays on the GPU
//! (the plan's RETREAT). A **fixed** population — a grid seeded once through
//! `zone.init` — has no count change, so the whole loop is claimable and the
//! sim itself lands on the device.
//!
//! What each node proves:
//! - **`sim.zone`** is a conditional passthrough: it forwards `init` (the lifted
//!   grid) on tick 0 and its `state` (the interior) thereafter — the device
//!   mirror of `ctx.started() ? state : init`. Frozen on `init` and the field
//!   never falls; frozen on `state` and it reads the empty interior on tick 0 and
//!   the population is zero forever. Both are gated (`gpu_cpu_parity_sim`).
//! - **`sim.step`** is the zone's own integrator, reading the per-element clock
//!   column `sim_t` — so a fresh element starts rather than leaping.
//! - **`sim.collide`** reflects velocity off the static bed (the bounce a collider
//!   OUTSIDE a zone cannot do, because it has no velocity to reflect).
//!
//! Under `PH2D_GPU_COOK=1` (the default) the chain is claimed with no boundary —
//! grid, lift, both forces, the step, the collide and the render `scale` all on
//! the device, zero readback. It auto-plays on tool entry; zoom out and watch a
//! the flakes fall into the sea and ride the swell.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// **The fixed-population snow globe** (`PH2D_GPU_COOK_DEMO=10`) — the ready-to-
/// smoke scene for the sim-zone family on the device (ADR-0134 §1). Returns the
/// sink.
pub(super) fn build_gpu_zone_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    // 64 × 1024 = 65.536 flakes — a wide, shallow band that reads as a fall of
    // snow. ⚠️ The count is a RENDER budget, not the cook's: the GPU cook of this
    // scene is **0,5 ms** even at 262.144 (measured, `the_zone_demo_scale_cook_cost`),
    // but a demo that fills the frame RENDERING a quarter-million packed quads has
    // nothing left when the artist zooms out (Enio, 2026-07-20: *"profunda queda de
    // fps"* — measured ~58 fps at 262 k before the drop). Sized for headroom like
    // the `=8` collide demo; the cook's ceiling is millions (the 4,19 M-in-3,6 ms
    // class), reached by raising `rows`/`cols`.
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 64.0);
    g.set_param(grid, "cols", 1024.0);
    g.set_param(grid, "gap_x", 0.03);
    g.set_param(grid, "gap_y", 0.05);

    // Lift the seed above the sea so the whole band falls INTO it (the grid is
    // centred on the origin; without this half of it starts underwater). This is
    // on the INIT side — it shapes the tick-0 population, not the running state.
    let lift = g.add_node("motion.move");
    g.set_param(lift, "dy", 9.0);

    let zone = g.add_node("sim.zone");

    // Gravity: 270 deg = straight down (y-up), with a little gust so the flakes do
    // not fall in lockstep columns.
    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", 270.0);
    g.set_param(wind, "strength", 4.0);
    g.set_param(wind, "gust", 0.35);

    // The shallow sea (the strobe's numbers, doc 52): density beats gravity, so a
    // flake settles a third of its draft under the surface and bobs; the swell
    // travels right, so the settled field is never still.
    let sea = g.add_node("force.buoyancy");
    g.set_param(sea, "level", -0.5);
    g.set_param(sea, "density", 14.0);
    g.set_param(sea, "depth", 0.3);
    g.set_param(sea, "drag", 5.0);
    g.set_param(sea, "wave_amplitude", 0.14);
    g.set_param(sea, "wave_length", 2.4);
    g.set_param(sea, "wave_speed", 0.5);

    let step = g.add_node("sim.step");

    // The bed the sea is shallow over: a flake punches the surface with a second
    // of gravity behind it, taps this, and the water lifts it back to float.
    let bed = g.add_node("sim.collide");
    g.set_param(bed, "shape", 0.0); // Floor
    g.set_param(bed, "height", -1.1);
    g.set_param(bed, "restitution", 0.25);
    g.set_param(bed, "friction", 0.35);

    // Grains, not blobs: unit quads would be a solid sheet.
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.06);

    let out = g.add_node("motion.output");

    for (i, n) in [grid, lift, zone, scale, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 190.0,
                y: 120.0,
            },
        );
    }
    // The interior sits one row below (a state loop reads right-to-left).
    for (i, n) in [wind, sea, step, bed].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 270.0 + i as f32 * 190.0,
                y: 300.0,
            },
        );
    }

    // The render chain: grid → lift → zone → scale → output.
    for (from, to, port, delayed) in [
        (grid, lift, 0, false),
        (lift, zone, 0, false), // → zone.init
        (zone, scale, 0, false),
        (scale, out, 0, false),
        // The state loop the editor's plumbing would draw as a portal badge.
        (zone, wind, 0, true), // zone.out --pre--> wind (the state entry)
        (wind, sea, 0, false),
        (sea, step, 0, false),
        (step, bed, 0, false),
        (bed, zone, 1, false), // interior tail → zone.state
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}
