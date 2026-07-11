//! The M3 curve demo — the **default Motion document**: on the LEFT a marquee of dots
//! **flowing along a Bézier path** (`motion.distribute_curve`, its `offset` ramped by a
//! saw `value.lfo`); on the RIGHT a grid ribbon **wrapped onto an S-curve**
//! (`motion.spline_wrap`, its `amount` swept by a sine `value.lfo` so it flattens and
//! re-wraps). Both curves are authored in the nodes' own params — self-contained, no
//! vector document. Two independent scenes (each its own `motion.output` sink — the
//! bridge composes several into one draw), kept small so each new node reads on its own.
//! A `#[path]` sibling of `motion_state`, kept out for the LOC cap.
//!
//! ```text
//! LEFT  (marquee): distribute_curve → tint(amber) → move(−6) → output   lfo(saw)  → offset
//! RIGHT (ribbon):  grid → spline_wrap → tint(cyan) → move(+6) → output   lfo(sine) → amount
//! ```
//!
//! - **distribute_curve** (`motion.distribute_curve`, doc 28): dots spaced evenly by arc
//!   length along a Bézier; the saw `offset` slides them so they flow down the path.
//! - **spline_wrap** (`motion.spline_wrap`, doc 28): a 3×12 grid mapped onto the S-curve;
//!   the sine `amount` blends flat → wrapped, so the ribbon bends onto the curve and back.
//!
//! See docs/Motion Nodes/28 (distribute_curve + spline_wrap). The whole value/pulse
//! vocabulary + the other M3/M4 nodes stay registered (drop them in the editor).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 200.0;
const MARQUEE_ROW: f32 = 0.0;
const RIBBON_ROW: f32 = 320.0;

/// Author both scenes into `g`; returns their Output nodes (the sinks), the marquee
/// scene's first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let marquee = build_marquee_scene(g)?;
    let ribbon = build_ribbon_scene(g)?;
    Some(vec![marquee, ribbon])
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

/// LEFT: dots flowing along a Bézier path. Returns its Output node.
fn build_marquee_scene(g: &mut Graph) -> Option<NodeId> {
    let curve = g.add_node("motion.distribute_curve");
    let tint = g.add_node("motion.tint");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [(curve, 0.0), (tint, 1.0), (mv, 2.0), (output, 3.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: MARQUEE_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: 0.0,
            y: MARQUEE_ROW + 160.0,
        },
    );

    wire(g, (curve, 0), (tint, 0))?;
    wire(g, (tint, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;
    wire(g, (lfo, 0), (curve, 0))?; // → offset

    // 24 dots on the default S-curve, flowing left.
    g.set_param(curve, "count", 24.0);
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.70);
    g.set_param(tint, "b", 0.20);
    g.set_param(mv, "dx", -6.0);
    g.set_param(mv, "dy", 0.0);
    // lfo → offset: a saw ramping 0→1 (amp 0.5 about 0.5 → waveform·0.5+0.5 = frac) so the
    // marquee flows one way and loops.
    g.set_param(lfo, "wave", 3.0); // Saw
    g.set_param(lfo, "period", 4.0);
    g.set_param(lfo, "amplitude", 0.5);
    g.set_param(lfo, "offset", 0.5);
    Some(output)
}

/// RIGHT: a grid ribbon wrapped onto an S-curve. Returns its Output node.
fn build_ribbon_scene(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let wrap = g.add_node("motion.spline_wrap");
    let tint = g.add_node("motion.tint");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [
        (grid, 0.0),
        (wrap, 1.0),
        (tint, 2.0),
        (mv, 3.0),
        (output, 4.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: RIBBON_ROW,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: COL_W,
            y: RIBBON_ROW + 160.0,
        },
    );

    wire(g, (grid, 0), (wrap, 0))?;
    wire(g, (wrap, 0), (tint, 0))?;
    wire(g, (tint, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;
    wire(g, (lfo, 0), (wrap, 1))?; // → amount

    // A 3×12 grid ribbon (36 dots) wrapped onto the default S-curve, on the right.
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 12.0);
    g.set_param(grid, "gap_x", 0.4);
    g.set_param(grid, "gap_y", 0.4);
    g.set_param(mv, "dx", 6.0);
    g.set_param(mv, "dy", 0.0);
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.25);
    g.set_param(tint, "g", 0.80);
    g.set_param(tint, "b", 0.95);
    // lfo → amount: a sine about 0.5, ±0.5 → amount ∈ [0, 1] (flat ↔ wrapped).
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 5.0);
    g.set_param(lfo, "amplitude", 0.5);
    g.set_param(lfo, "offset", 0.5);
    Some(output)
}
