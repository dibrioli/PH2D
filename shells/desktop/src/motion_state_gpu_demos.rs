//! The **ready-to-smoke GPU documents** (`PH2D_GPU_COOK_DEMO`), split out of
//! `motion_state.rs` at the HR-18 cap.
//!
//! Each one is a graph the artist never has to build by hand: set the env var,
//! run, and the path under test is already on screen. They live together because
//! they are one thing — the demonstration surface of GPU/M5 — and because the
//! gates that prove each one still plans the way it claims (`motion_state_gpu_tests.rs`)
//! are its sibling.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// The GPU/M5 Fase 1 **ready-to-smoke** document (`PH2D_GPU_COOK_DEMO=1`):
/// `grid(1250×1600) → oscillator(Y wave) → move → output` — **2.000.000
/// instances**, every node kernel-covered, so `PH2D_GPU_COOK=1` runs it 100%
/// GPU-resident (`ph2d_gpu_cook::plan` claims the whole chain; the renderer binds
/// the lowering's buffer with zero readback). Measured on the RTX at ~4 ms/frame
/// for the cook (probe `gpu_cook_millions_timing`), i.e. the roadmap's "millions
/// at 60fps" with headroom. The same chain, at 25.6k, is the parity gate's
/// fixture. Auto-plays on tool entry like every boot document.
pub(super) fn build_gpu_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    // 1250 × 1600 = exactly 2.000.000 cells (well under the grid's 16.7M cap).
    g.set_param(grid, "rows", 1250.0);
    g.set_param(grid, "cols", 1600.0);
    // A dense lattice: the quads are unit-sized (the shell's `default_size`
    // identity), so gap 1.0 tiles them edge-to-edge — zoom out and the whole
    // 1600×1250 field reads as a shimmering cloth of two million quads.
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);
    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "channel", 1.0); // Y — the kernel-covered channels are X/Y
    g.set_param(osc, "amplitude", 6.0);
    g.set_param(osc, "frequency", 0.5);
    // A travelling wave across the field. The phase advances per row-major index,
    // so the stagger is scaled down for 2M cells (~500 cycles across the cloth,
    // the same band density the 262k version read at 0.002).
    g.set_param(osc, "phase_stagger", 0.00025);
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    for (i, n) in [grid, osc, mv, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 120.0,
            },
        );
    }
    g.connect(Edge {
        from: (grid, 0),
        to: (osc, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (osc, 0),
        to: (mv, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (mv, 0),
        to: (out, 0),
        delayed: false,
    })
    .ok()?;
    g.validate(reg).ok()?;
    Some(vec![out])
}

/// The GPU/M5 **Fase 3 simulation** ready-to-smoke document
/// (`PH2D_GPU_COOK_DEMO=3`, ADR-0127): `grid(700×700) → color_ramp → integrate →
/// output` with the force loop `integrate --pre--> vortex → attractor → drag →
/// curl --> integrate.forces` — **490.000 particles, simulated 100% on the GPU**.
///
/// The forces existed since M2 but had never run a frame on the GPU: they are
/// only ever evaluated INSIDE the `pre` loop, and the loop was what the plan
/// refused. Under `PH2D_GPU_COOK=1` the whole thing is claimed (no boundary),
/// the state ping-pongs as held `Arc`s across ticks, and scrubbing the playhead
/// backwards restores a device-side checkpoint instead of showing the future.
///
/// The mix is the classic stable orbit (vortex + attractor at one centre + drag
/// — without drag a pure vortex spirals outward by centrifugal drift), plus curl
/// noise so the cloud breathes instead of settling into a clean ring.
///
/// **The colour waves are dye advection.** The ramp colours the SOURCE — a full
/// hue circle laid across the grid's row-major index, so the field starts as
/// horizontal rainbow bands — and then the sim carries each particle's tint with
/// it. The vortex winds those bands into spirals that keep tightening, the curl
/// noise frays them, and the drag lets the inner ones lap the outer: the waves
/// are the FLOW made visible, not a colour animation played over it. It is what
/// a dye tracer does in a real vortex, and it costs one node.
///
/// The ramp sits in the `rest` chain rather than after the integrator on
/// purpose: the tint belongs to the particle, so colour the source and let the
/// simulation carry it. (Downstream would look identical — the integrator pairs
/// positionally, so element `i` keeps index `i` — but it would say the wrong
/// thing about where colour comes from.)
pub(super) fn build_gpu_sim_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 700.0);
    g.set_param(grid, "cols", 700.0);
    g.set_param(grid, "gap_x", 0.12);
    g.set_param(grid, "gap_y", 0.12);
    // Rainbow: 7 stops closing the hue circle, so the field reads as bands
    // rather than one gradient. `t` stays unconnected — the ramp keys on the
    // normalised index, which is the only shape the kernel claims (a connected
    // `t` would put the node back on the CPU, and the CPU would then own the
    // whole document: this graph drives a `pre` loop).
    let ramp = g.add_node("motion.color_ramp");
    g.set_param(ramp, "preset", 0.0);
    g.set_param(ramp, "interp", 1.0); // Ease — the bands blend instead of banding hard
    let ig = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");

    // The force chain, source→sink inside the loop.
    let vortex = g.add_node("force.vortex");
    g.set_param(vortex, "strength", 14.0);
    g.set_param(vortex, "radius", 46.0);
    g.set_param(vortex, "clockwise", 1.0);
    let attractor = g.add_node("force.attractor");
    g.set_param(attractor, "strength", 9.0);
    g.set_param(attractor, "radius", 46.0);
    g.set_param(attractor, "curve", 0.0); // Linear: a steady inward pull
    let drag = g.add_node("force.drag");
    g.set_param(drag, "coefficient", 0.35);
    let curl = g.add_node("force.curl");
    g.set_param(curl, "strength", 6.0);
    g.set_param(curl, "scale", 0.06);
    g.set_param(curl, "speed", 0.4);
    g.set_param(curl, "octaves", 2.0);

    for (i, n) in [grid, ramp, ig, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 220.0,
                y: 120.0,
            },
        );
    }
    for (i, n) in [vortex, attractor, drag, curl].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 300.0,
            },
        );
    }
    g.connect(Edge {
        from: (grid, 0),
        to: (ramp, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (ramp, 0),
        to: (ig, 0),
        delayed: false,
    })
    .ok()?;
    // The feedback the user never draws: last tick's state into the chain head.
    g.connect(Edge {
        from: (ig, 0),
        to: (vortex, 0),
        delayed: true,
    })
    .ok()?;
    g.connect(Edge {
        from: (vortex, 0),
        to: (attractor, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (attractor, 0),
        to: (drag, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (drag, 0),
        to: (curl, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (curl, 0),
        to: (ig, 1),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (ig, 0),
        to: (out, 0),
        delayed: false,
    })
    .ok()?;
    g.validate(reg).ok()?;
    Some(vec![out])
}

/// The **sea** (`PH2D_GPU_COOK_DEMO=4`): `grid(700×700) → color_ramp →
/// integrate → output`, with the loop `integrate --pre--> wind → buoyancy -->
/// integrate.forces` — **490.000 particles raining onto a travelling wave, 100%
/// on the GPU**.
///
/// `force.wind` pointing down (`angle = 270`, `gust = 0`) IS the gravity: the
/// buoyancy node deliberately has no gravity param, by the same argument that
/// there is no gravity node — a steady directional force already exists. That is
/// also what makes the scene read: buoyancy alone would launch everything
/// upward, and it is the *contest* between `density = 12` and `strength = 4`
/// that settles the field at `submersion = 4/12` and lets it bob.
///
/// What you should see: the dry half falls, the submerged half is pushed up,
/// and 490k particles collapse onto the surface — then keep bobbing, because
/// each one overshoots and `drag` only takes the energy out slowly. They also
/// drift SIDEWAYS into the troughs: the buoyant push is normal to the surface,
/// not straight up, so a flank pushes downhill. The rainbow the ramp lays on the
/// source rides along, so the wave arrives already coloured.
pub(super) fn build_gpu_sea_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 700.0);
    g.set_param(grid, "cols", 700.0);
    g.set_param(grid, "gap_x", 0.12);
    g.set_param(grid, "gap_y", 0.12);
    // The field spans ±42, so the sea at 0 cuts it in half: half of it starts
    // dry and falls, half starts under and rises. Both regimes on screen at once.
    let ramp = g.add_node("motion.color_ramp");
    g.set_param(ramp, "preset", 0.0);
    g.set_param(ramp, "interp", 1.0);
    let ig = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");

    // Gravity: steady (`gust = 0`), straight down (270° → the `(cos, sin)` pair
    // reads (0, −1)).
    let gravity = g.add_node("force.wind");
    g.set_param(gravity, "angle", 270.0);
    g.set_param(gravity, "strength", 4.0);
    g.set_param(gravity, "gust", 0.0);
    let sea = g.add_node("force.buoyancy");
    g.set_param(sea, "level", 0.0);
    // Beats gravity 12-to-4 → it floats rather than sinks, settling at a third
    // of its draft under.
    g.set_param(sea, "density", 12.0);
    // A deep draft on purpose: the submersion ramp is what spreads the field
    // into a band instead of pinning every particle to one hairline.
    g.set_param(sea, "depth", 4.0);
    g.set_param(sea, "drag", 2.0);
    // A swell on the field's own scale (the grid spans ±42).
    g.set_param(sea, "wave_amplitude", 2.0);
    g.set_param(sea, "wave_length", 20.0);
    g.set_param(sea, "wave_speed", 3.0);

    for (i, n) in [grid, ramp, ig, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 220.0,
                y: 120.0,
            },
        );
    }
    for (i, n) in [gravity, sea].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 300.0,
            },
        );
    }
    for (from, to, port, delayed) in [
        (grid, ramp, 0, false),
        (ramp, ig, 0, false),
        // The feedback the user never draws.
        (ig, gravity, 0, true),
        (gravity, sea, 0, false),
        (sea, ig, 1, false),
        (ig, out, 0, false),
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

/// The **emitter fountain** (`PH2D_GPU_COOK_DEMO=5`, ADR-0130) — the ready-to-smoke
/// document for the **id-gather**: `emitter → integrate → output`, with the loop
/// `integrate --pre--> gravity → curl --> integrate.forces`. Up to **3.000
/// particles born, launched, arced and killed 100% on the GPU**, their per-particle
/// state paired across a sliding id window by ARITHMETIC (`current_id −
/// prev_first`), never a hash/sort.
///
/// This is the scene the earlier demos could NOT be: `=3`/`=4` are a fixed GRID
/// (positional pairing — element `i` is always element `i`), so the gather never
/// ran. Here the alive set CHURNS every tick — the emitter is stateless, so at
/// playhead `t` it is the closed id window `[first, first+n)`, and `n`/`first`
/// slide as particles are born and die. Each particle launches with its OWN muzzle
/// velocity (`hash(seed, id)` over the `spread` cone), so WHICH prior row a
/// survivor pairs to is exactly what its trajectory is: a positional mispair would
/// give it a stranger's velocity and the fountain would boil. It doesn't — the arcs
/// are clean, which is the gather working.
///
/// `gravity` is `force.wind` pointing down (there is no gravity node — a steady
/// directional force already is one); `curl` frays the stream so it sparkles
/// instead of falling as a sheet. The emitter is **capped** (`max` < `rate·life`)
/// so `first` advances every tick — the sliding-window regime the gather is for.
///
/// **Fatia 5 lives here too:** with the sim running, drag the emitter's `rate` /
/// `life` / `max` in the params panel — the fountain re-bakes from the seed under
/// the new numbering (a clean restart, not a partial mispair). Drag `speed` /
/// `size` and the running sim keeps (only new particles change).
pub(super) fn build_gpu_emitter_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let em = g.add_node("motion.emitter");
    // Capped: rate·life = 4200 > max, so the window binds at `max` and `first`
    // advances every tick — the gather's whole point. A wide cone gives each id a
    // distinct muzzle velocity, so a mispair would be visible as a boiling arc.
    // 400.000/s × 3 s = 1,2 MILLION alive. Measured on the RTX: a sim this size
    // costs ~1 ms/tick — 6% of a 60 fps frame — and the ceiling above it is 4M.
    // This demo has been 3.000 particles, then 4.200 (silently bounded by
    // `rate × life`), then 12.000; each of those was a number some CEILING chose,
    // never the device. This one is what the hardware does.
    g.set_param(em, "rate", 400_000.0);
    g.set_param(em, "life", 3.0);
    g.set_param(em, "max", 1_200_000.0);
    // Grains, not blobs: a million quads at 0,18 world units would be one solid
    // sheet of colour.
    g.set_param(em, "size", 0.012);
    g.set_param(em, "speed", 22.0);
    g.set_param(em, "angle", 90.0); // up (Y-up world)
    g.set_param(em, "spread", 60.0);
    g.set_param(em, "x", 0.0);
    g.set_param(em, "y", -8.0); // launch from the low centre, up into view
    g.set_param(em, "seed", 7.0);
    let ig = g.add_node("motion.integrate");
    // Colour by AGE. The emitter's ids ascend oldest-first, so `Index` IS age
    // order and a Gradient Tint paints the jet hot at the muzzle and cold at the
    // tips — the affordance the emitter's own doc promises. It is here to be
    // SEEN: until Gradient got its kernel it would have split this chain at a CPU
    // boundary, so a coloured fountain that still plans fully-GPU is the visible
    // form of that fix.
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 1.0); // Gradient
    g.set_param(tint, "r", 1.0); // oldest → warm white
    g.set_param(tint, "g", 0.86);
    g.set_param(tint, "b", 0.45);
    g.set_param(tint, "a", 1.0);
    g.set_param(tint, "r2", 0.16); // newest → deep blue
    g.set_param(tint, "g2", 0.32);
    g.set_param(tint, "b2", 0.95);
    g.set_param(tint, "a2", 1.0);
    let out = g.add_node("motion.output");

    // Gravity: steady (`gust = 0`), straight down (270° → the `(cos, sin)` pair
    // reads (0, −1)) — arcs the fountain back down.
    let gravity = g.add_node("force.wind");
    g.set_param(gravity, "angle", 270.0);
    g.set_param(gravity, "strength", 26.0);
    g.set_param(gravity, "gust", 0.0);
    let curl = g.add_node("force.curl");
    g.set_param(curl, "strength", 5.0);
    g.set_param(curl, "scale", 0.08);
    g.set_param(curl, "speed", 0.5);
    g.set_param(curl, "octaves", 2.0);

    for (i, n) in [em, ig, tint, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 220.0,
                y: 120.0,
            },
        );
    }
    for (i, n) in [gravity, curl].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 300.0,
            },
        );
    }
    for (from, to, port, delayed) in [
        (em, ig, 0, false),
        // The feedback the user never draws: last tick's state into the loop head.
        (ig, gravity, 0, true),
        (gravity, curl, 0, false),
        (curl, ig, 1, false),
        (ig, tint, 0, false),
        (tint, out, 0, false),
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

/// The GPU/M5 **F1.2 hybrid** ready-to-smoke document (`PH2D_GPU_COOK_DEMO=2`):
/// `grid(360×360) → oscillator(Rotation) → oscillator(Y) → scale → output`.
///
/// The FIRST oscillator targets the **Rotation** channel, which the oscillator
/// kernel does not cover (`applicable` = X/Y only) — so the plan puts the CPU
/// boundary there: the CPU pump cooks `grid → oscillator(Rotation)` (a travelling
/// spin wave in the `rot` column), its stream crosses to the GPU ONCE, and the
/// GPU runs `oscillator(Y) → scale → output` (a travelling height wave + a size
/// pulse) plus the lowering — zero readback. So the field waves in Y AND the
/// quads spin, the two halves computed on opposite sides of the CPU↔GPU seam.
/// 129.600 instances. Auto-plays on tool entry like every boot document.
pub(super) fn build_gpu_hybrid_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 360.0);
    g.set_param(grid, "cols", 360.0);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);
    // CPU boundary: `motion.sort` REORDERS the stream — a global permutation, and
    // this engine's kernel contract is strictly per-element (`i` → element `i`),
    // so it is uncoverable by STRUCTURE and the seam it creates is permanent.
    //
    // ⚠️ It used to be an oscillator on Rotation, "outside the kernel's X/Y
    // coverage" — and `GpuKernel::variant_by_param` gave the oscillator every
    // channel, so the demo silently became fully-GPU: a demo built to SHOW the
    // hybrid seam, exercising the wrong path. Anchoring a seam on "not covered
    // yet" makes the demo decay every time coverage advances
    // ([[feedback_a_seam_fixture_must_rest_on_something_uncoverable]]).
    let spin = g.add_node("motion.sort");
    g.set_param(spin, "key", 1.0);
    // GPU suffix: a Y wave, then a size pulse.
    let wave = g.add_node("motion.oscillator");
    g.set_param(wave, "channel", 1.0); // Y — kernel-covered
    g.set_param(wave, "amplitude", 5.0);
    g.set_param(wave, "frequency", 0.5);
    g.set_param(wave, "phase_stagger", 0.003);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 1.6);
    let out = g.add_node("motion.output");
    for (i, n) in [grid, spin, wave, scale, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 120.0,
            },
        );
    }
    g.connect(Edge {
        from: (grid, 0),
        to: (spin, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (spin, 0),
        to: (wave, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (wave, 0),
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

/// The **FIELD family** smoke (`PH2D_GPU_COOK_DEMO=17`): `grid(512×512) →
/// field.index_range → tint(Solid) → output` — **262.144 instances**, every node
/// kernel-covered, so `PH2D_GPU_COOK=1` runs it 100 % GPU-resident (parity-proven
/// bit-exact at 25.6k, `field_index_range_kernel_matches_the_cpu_within_epsilon`).
///
/// `field.index_range` writes the multiplicative `falloff` mask keyed by ORDINAL
/// `i/(count−1)` — row-major here — and the Solid tint lerps the sprites' white
/// toward a saturated red BY that mask. The result is a **horizontal band of the
/// middle rows** glowing red, the rest white, the seam rows fading through the
/// soft edge. It is the one mask a spatial `motion.falloff` cannot draw: it
/// selects by RANK, not position — "the middle third of the clones", the stagger
/// a grid + circle can never reach. Auto-plays on tool entry like every boot doc.
pub(super) fn build_gpu_field_index_range_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;
    let grid = g.add_node("motion.grid");
    // 512 × 512 = 262.144 unit quads, gap 1.0 tiling them edge-to-edge into a
    // dense field (the panel demo's scale — legibly a grid, substantial on GPU).
    g.set_param(grid, "rows", 512.0);
    g.set_param(grid, "cols", 512.0);
    // gap 0.024 makes the 512-wide field ~12 units (centred on origin), so it
    // FRAMES at the default zoom instead of being a 512-unit wall you must zoom
    // out to see. The 512-row count keeps the ordinal tilt at ~1/512 (a fine band
    // edge, not a staircase).
    g.set_param(grid, "gap_x", 0.024);
    g.set_param(grid, "gap_y", 0.024);
    // Shrink the unit quads to dots BEFORE the field writes `falloff` (absent =>
    // reads its 1.0 identity => the scale is UNIFORM): 0.018 < the 0.024 gap, so
    // the dots stay distinct instead of tiling into the sheet that hid the band.
    // (motion.scale eased by falloff = 1 is the full amount — the STRENGTH cluster.)
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.018);
    let field = g.add_node("field.index_range");
    // The middle ~half of the ordinal range, soft-edged — a horizontal band of
    // rows (index is row-major), unmistakably index-keyed rather than spatial.
    g.set_param(field, "start", 0.25);
    g.set_param(field, "end", 0.75);
    g.set_param(field, "soft", 0.08);
    g.set_param(field, "curve", 2.0); // Smooth edges
    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid — the GPU-covered mode
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.25);
    g.set_param(tint, "b", 0.15);
    g.set_param(tint, "a", 1.0);
    let out = g.add_node("motion.output");
    for (i, n) in [grid, scale, field, tint, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 180.0,
                y: 120.0,
            },
        );
    }
    for (a, b) in [(grid, scale), (scale, field), (field, tint), (tint, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}
