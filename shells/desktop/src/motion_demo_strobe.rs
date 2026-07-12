//! The M3 simulation demo — the **default Motion document**, showing the slice's two
//! new nodes: on the LEFT a curtain whose top row is **pinned** while the rest is sucked
//! into an attractor and packs around it; on the RIGHT a strip that bobs in LOCKSTEP and
//! is sheared into a travelling wave by the **slit scan**. Two independent scenes (each
//! its own `motion.output` sink), kept small so each new node reads on its own. A
//! `#[path]` sibling of `motion_state`, kept out for the LOC cap.
//!
//! ```text
//! LEFT  (pin):       grid ─> pin_constraint ─> integrate ─> collide ─> move(−7) ─> output
//!                                                  ^                (the pinned row is an
//!                                    pre└─ attractor ─ drag ─┘       obstacle, not cargo)
//! RIGHT (slit scan): grid ─> oscillator ─> slit_scan ─> move(+7) ─> output
//!                                            ^   └─pre─┘
//! ```
//!
//! - **pin_constraint** (`motion.pin_constraint`, doc 34): writes the PBD inverse-mass
//!   column `inv_mass` (`0` = pinned). `motion.integrate` gives a pinned element no force
//!   and no displacement — it rides its rest animation — and `motion.collide` refuses to
//!   push it, so the free elements pack AROUND the pinned row. Drag the `count` slider to
//!   0 in the params panel and the whole curtain falls in: that is the falsifiable read.
//! - **slit_scan** (`motion.slit_scan`, doc 34): the oscillator's `phase_stagger` is **0**
//!   here, so every element bobs at the same phase; the wave you see is the scan sampling
//!   each element at its own delay (`lag` ticks across the set). Set `lag` to 0 and the
//!   wave collapses back into a rigid, lockstep bob.
//!
//! See docs/Motion Nodes/34 (pinning + the slit scan). The whole value/pulse vocabulary +
//! the other M3/M4 nodes stay registered (drop them in the editor).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 190.0;
const PIN_ROW: f32 = 0.0;
const SCAN_ROW: f32 = 360.0;
/// The pinned curtain's shape. `cols` is also the pin's `count` — one row's worth.
const CURTAIN_COLS: f32 = 8.0;
const CURTAIN_ROWS: f32 = 8.0;
/// The first element of the curtain's **top** row. `motion.grid` is row-major from
/// `y = −cy` upward (row 0 is the LOWEST y, i.e. the bottom of the screen in a y-up
/// world), so the top row is the LAST one — the curtain hangs from it.
const CURTAIN_TOP_ROW: f32 = (CURTAIN_ROWS - 1.0) * CURTAIN_COLS;

/// Author both scenes into `g`; returns their Output nodes (the sinks), the pin scene's
/// first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let pin = build_pin_scene(g)?;
    let scan = build_slit_scan_scene(g)?;
    Some(vec![pin, scan])
}

/// Connect `from` → `to` on the given ports, an immediate (non-delayed) edge.
fn wire(g: &mut Graph, from: (NodeId, u16), to: (NodeId, u16)) -> Option<()> {
    g.connect(Edge {
        from,
        to,
        delayed: false,
    })
    .ok()
}

/// The `pre` edge: last tick's value. The editor plumbs these itself when a sequential
/// node is dropped; a document authored in code wires them by hand.
fn wire_pre(g: &mut Graph, from: (NodeId, u16), to: (NodeId, u16)) -> Option<()> {
    g.connect(Edge {
        from,
        to,
        delayed: true,
    })
    .ok()
}

/// LEFT: a curtain nailed by its top row, the rest pulled into an attractor. Returns its
/// Output.
fn build_pin_scene(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let pin = g.add_node("motion.pin_constraint");
    let integrate = g.add_node("motion.integrate");
    let attractor = g.add_node("force.attractor");
    let drag = g.add_node("force.drag");
    let collide = g.add_node("motion.collide");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");

    for (n, col) in [
        (grid, 0.0),
        (pin, 1.0),
        (integrate, 2.0),
        (collide, 3.0),
        (mv, 4.0),
        (output, 5.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: PIN_ROW,
            },
        );
    }
    // The force chain hangs below the integrator it feeds back into.
    for (n, col) in [(attractor, 2.0), (drag, 3.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: PIN_ROW + 150.0,
            },
        );
    }

    wire(g, (grid, 0), (pin, 0))?;
    wire(g, (pin, 0), (integrate, 0))?; // the rest chain carries `inv_mass` in
    wire_pre(g, (integrate, 0), (attractor, 0))?; // the simulation loop
    wire(g, (attractor, 0), (drag, 0))?;
    wire(g, (drag, 0), (integrate, 1))?;
    wire(g, (integrate, 0), (collide, 0))?;
    wire(g, (collide, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;

    g.set_param(grid, "rows", CURTAIN_ROWS);
    g.set_param(grid, "cols", CURTAIN_COLS);
    g.set_param(grid, "gap_x", 0.6);
    g.set_param(grid, "gap_y", 0.6);
    // Nail the top row — the curtain hangs from it (see CURTAIN_TOP_ROW: the grid's
    // first row is the BOTTOM one, so the top row is the last `cols` elements).
    g.set_param(pin, "first", CURTAIN_TOP_ROW);
    g.set_param(pin, "count", CURTAIN_COLS);
    g.set_param(pin, "strength", 1.0);
    // Suck everything else toward the scene's origin; drag keeps it from orbiting away.
    g.set_param(attractor, "strength", 6.0);
    g.set_param(attractor, "radius", 6.0);
    g.set_param(drag, "coefficient", 1.4);
    // Discs, so the falling elements pile up around the pinned row instead of collapsing
    // into one point — and the pinned row does not budge when they land on it.
    g.set_param(collide, "radius", 0.28);
    g.set_param(collide, "iterations", 8.0);
    g.set_param(mv, "dx", -7.0);
    g.set_param(mv, "dy", 0.0);
    Some(output)
}

/// RIGHT: a lockstep bob sheared into a travelling wave. Returns its Output.
fn build_slit_scan_scene(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let osc = g.add_node("motion.oscillator");
    let scan = g.add_node("motion.slit_scan");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");

    for (n, col) in [
        (grid, 0.0),
        (osc, 1.0),
        (scan, 2.0),
        (mv, 3.0),
        (output, 4.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: SCAN_ROW,
            },
        );
    }

    wire(g, (grid, 0), (osc, 0))?;
    wire(g, (osc, 0), (scan, 0))?;
    wire_pre(g, (scan, 0), (scan, 1))?; // the delay line rides its own output
    wire(g, (scan, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;

    g.set_param(grid, "rows", 6.0);
    g.set_param(grid, "cols", 12.0);
    g.set_param(grid, "gap_x", 0.55);
    g.set_param(grid, "gap_y", 0.55);
    // Bob in Y, every element at the SAME phase (stagger 0) — so the wave on screen can
    // only come from the scan.
    g.set_param(osc, "channel", 1.0);
    g.set_param(osc, "amplitude", 1.2);
    g.set_param(osc, "frequency", 0.8);
    g.set_param(osc, "phase_stagger", 0.0);
    // Two thirds of a tick per element (72 elements, 24 ticks across the set).
    g.set_param(scan, "lag", 24.0);
    g.set_param(mv, "dx", 7.0);
    g.set_param(mv, "dy", 0.0);
    Some(output)
}
