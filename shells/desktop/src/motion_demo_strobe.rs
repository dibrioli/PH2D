//! The value-domain demo — the **sole scene of the default Motion document**. A
//! deliberately SMALL scene (one grid, ~10 nodes) so each new node reads on its
//! own instead of drowning in a four-channel pile-up. A `#[path]` sibling of
//! `motion_state`, kept out of it for the LOC cap.
//!
//! It isolates the two newest value-domain nodes (doc 16) on ONE grid, both
//! driven by the SAME travelling `value.lfo` so the continuous↔discrete duality
//! is legible — one wave sweeps across the grid, and you see it two ways at once:
//!
//! ```text
//! grid → tint → drive_size → strobe → output
//!        grid → instance_field ─┐
//!        grid → lfo ────────────┴→ math → size_range → drive_size.value   (SIZE: continuous)
//!               lfo → compare ⟲ → strobe.pulse                            (FLASH: discrete)
//! ```
//!
//! - **lfo** (`value.lfo`): reads the grid for its count and emits a length-N
//!   **travelling wave** (a per-instance `phase_stagger`) — one wave that ripples
//!   across the dots. It feeds BOTH new nodes.
//! - **math** (`value.math`, doc 16): the first combiner of TWO value fields —
//!   MULTIPLIES the static `instance_field` Ramp (a small→big spatial gradient)
//!   by the travelling wave. So each dot's SIZE swells and shrinks as the wave
//!   passes, by an amount graded by its position — a **spatial gradient modulated
//!   in time**, the *continuous* read of the wave.
//! - **compare** (`pulse.compare`, doc 16): the value→pulse bridge (the dual of
//!   `sample_hold`) — fires a PULSE the moment a dot's wave rises past the
//!   threshold, with Schmitt hysteresis. The `motion.strobe` turns each pulse into
//!   a white **flash**, so the dots light up in a ripple as the wave sweeps — the
//!   *discrete* read of the very same wave. (The strobe flashes COLOUR only —
//!   `size_boost = 0` — so Size stays the pure `math` signal.)
//!
//! The payoff: one travelling wave, shown two ways — the dots **swell smoothly**
//! (math, a gradient breathing in time) AND **flash on the crossing** (compare, a
//! value turned into a pulse). Two nodes, one legible grid; the continuous and the
//! discrete face of the value domain, side by side. See docs/Motion Nodes/12
//! (value), 13 (lfo/map_range), 16 (math+compare). The pulse metronome, counter,
//! sample-hold and per-channel drives of the earlier scenes stay registered (drop
//! them in the editor); the boot scene now shows the newest pair in isolation.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Graph-space origin of this scene's card row (the sole scene → at the origin).
const ROW_Y: f32 = 0.0;
const COL_W: f32 = 220.0;

/// Author the value-domain scene into `g`; returns its Output node (the sink).
pub(crate) fn build(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let tint = g.add_node("motion.tint");
    let drive_size = g.add_node("motion.drive");
    let strobe = g.add_node("motion.strobe");
    let output = g.add_node("motion.output");
    let instance_field = g.add_node("value.instance_field");
    let lfo = g.add_node("value.lfo");
    let math = g.add_node("value.math");
    let size_range = g.add_node("value.map_range");
    let compare = g.add_node("pulse.compare");

    // Visible trunk: grid → tint → drive_size → strobe → output. `drive_size`
    // writes the Size channel from `math`; `strobe` flashes colour from `compare`.
    for (n, col) in [
        (grid, 0.0),
        (tint, 1.0),
        (drive_size, 2.0),
        (strobe, 3.0),
        (output, 4.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: ROW_Y,
            },
        );
    }
    for (from, to) in [
        (grid, tint),
        (tint, drive_size),
        (drive_size, strobe),
        (strobe, output),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
    }

    // Value branches. `instance_field` mints a per-dot Ramp (a spatial gradient);
    // `lfo` reads the grid for its count and emits a travelling wave. `math`
    // MULTIPLIES them → a graded, time-modulated field that `size_range` reshapes
    // and drives onto Size. The SAME `lfo` feeds `compare`, which fires a pulse on
    // each dot's rising crossing to flash the strobe. One wave, continuous AND
    // discrete.
    for (from, to) in [
        ((grid, 0), (instance_field, 0)),   // grid → instance_field (count)
        ((grid, 0), (lfo, 0)),              // grid → lfo (count → travelling wave)
        ((instance_field, 0), (math, 0)),   // ramp → math.a
        ((lfo, 0), (math, 1)),              // travelling wave → math.b
        ((math, 0), (size_range, 0)),       // graded field → size_range.in
        ((size_range, 0), (drive_size, 1)), // degrees/size → drive_size.value
        ((lfo, 0), (compare, 0)),           // wave → compare.value
        ((compare, 0), (strobe, 1)),        // per-dot crossings → strobe.pulse
    ] {
        g.connect(Edge {
            from,
            to,
            delayed: false,
        })
        .ok()?;
    }
    // The `pre` self-loops (what the editor auto-plumbs on drop): the compare's
    // armed state and the strobe's glow. The lfo, math, size_range and
    // instance_field are stateless.
    for (n, port) in [(compare, 1), (strobe, 2)] {
        g.connect(Edge {
            from: (n, 0),
            to: (n, port),
            delayed: true,
        })
        .ok()?;
    }
    for (n, col, dy) in [
        (instance_field, 1.0, 220.0),
        (lfo, 2.0, 220.0),
        (math, 3.0, 220.0),
        (size_range, 4.0, 220.0),
        (compare, 2.0, 440.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: ROW_Y + dy,
            },
        );
    }

    // A 3×4 grid of well-spaced dots, centred on the origin.
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 4.0);
    g.set_param(grid, "gap_x", 1.4);
    g.set_param(grid, "gap_y", 1.1);
    // A calm blue base so the white flash reads as a brighten, not a hue jump.
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.25);
    g.set_param(tint, "g", 0.35);
    g.set_param(tint, "b", 0.85);
    // instance_field: a per-dot Ramp (0..1 across the 12 dots by index) — the
    // spatial gradient that `math` modulates.
    g.set_param(instance_field, "mode", 1.0); // Ramp
    // lfo: a slow (2 s) sine, staggered 0.18 cycle per instance → a travelling
    // wave rippling across the grid. Raw [-1,1] amplitude; feeds math AND compare.
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 2.0);
    g.set_param(lfo, "amplitude", 1.0);
    g.set_param(lfo, "phase_stagger", 0.18);
    // math: MULTIPLY the Ramp (0..1) by the travelling wave (±1) → a field whose
    // amplitude is graded by index (dot 0 barely moves, the top dot swings full).
    g.set_param(math, "op", 2.0); // Multiply
    // size_range: map the graded [-1,1] field to a visible size span. Clamp on
    // (the default) keeps sizes in [0.25, 0.6] even past the wave's extremes.
    g.set_param(size_range, "in_lo", -1.0);
    g.set_param(size_range, "in_hi", 1.0);
    g.set_param(size_range, "out_lo", 0.25);
    g.set_param(size_range, "out_hi", 0.6);
    // drive_size: SET the size to the gradient (channel 3, mode Set), so each dot
    // has its own time-varying size.
    g.set_param(drive_size, "channel", 3.0); // Size
    g.set_param(drive_size, "scale", 1.0);
    g.set_param(drive_size, "mode", 1.0); // Set
    // compare: fire on the rising crossing of 0.5, with hysteresis down to 0.1
    // (no chatter as the wave peaks). Every dot crosses each cycle → the flashes
    // ripple across the grid with the travelling wave.
    g.set_param(compare, "rise", 0.5);
    g.set_param(compare, "fall", 0.1);
    g.set_param(compare, "edge", 0.0); // Rise
    // strobe: a punchy COLOUR-ONLY flash (size_boost 0 → Size stays the pure math
    // signal) fading over ~0.3 s, lit white so it reads on the blue base.
    g.set_param(strobe, "decay", 0.88);
    g.set_param(strobe, "size_boost", 0.0);
    g.set_param(strobe, "flash_r", 1.0);
    g.set_param(strobe, "flash_g", 1.0);
    g.set_param(strobe, "flash_b", 1.0);
    g.set_param(strobe, "flash_amount", 0.9);
    Some(output)
}
