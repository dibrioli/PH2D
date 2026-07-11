//! The M3 deformer demo — the **sole scene of the default Motion document**: a
//! grid that **curls like a wave while every square turns to track a moving
//! target**. A small scene (~7 nodes) showing two M3 deformers. A `#[path]`
//! sibling of `motion_state`, kept out of it for the LOC cap.
//!
//! ```text
//! grid → bend → look_at → tint → output
//!        │(amount)  │(target_x)
//!        lfo_bend   lfo_target
//! ```
//!
//! - **bend** (`motion.bend`, doc 20): the arc deformer — wraps the grid's X extent
//!   onto a circular arc, so the rows curl up/down while the centre column holds;
//!   its `amount` is a `value.lfo` (±1), so the grid **curls up and uncurls** in
//!   time.
//! - **look_at** (`motion.look_at`, doc 20): orients each square's `rot` at a
//!   target point; the `target_x` is a `value.lfo` that slides the target left↔right,
//!   so the whole field **turns to follow it** (arrows tracking a cursor).
//!
//! The payoff: a bending sheet of squares that all **swivel to face a passing
//! point** — two M3 deformers (an arc-wrap and an orient-toward), each animated by
//! the value domain, on one legible grid. See docs/Motion Nodes/20 (bend+look_at).
//! The whole value/pulse vocabulary + the other M3 nodes stay registered (drop them
//! in the editor). Pure function of the playhead (the lfos are Temporal; no `pre`
//! state).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Graph-space origin of this scene's card row (the sole scene → at the origin).
const ROW_Y: f32 = 0.0;
const COL_W: f32 = 220.0;

/// Author the bend + look-at scene into `g`; returns its Output node (the sink).
pub(crate) fn build(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let bend = g.add_node("motion.bend");
    let look_at = g.add_node("motion.look_at");
    let tint = g.add_node("motion.tint");
    let output = g.add_node("motion.output");
    let lfo_bend = g.add_node("value.lfo");
    let lfo_target = g.add_node("value.lfo");

    // Visible trunk: grid → bend → look_at → tint → output.
    for (n, col) in [
        (grid, 0.0),
        (bend, 1.0),
        (look_at, 2.0),
        (tint, 3.0),
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
        (grid, bend),
        (bend, look_at),
        (look_at, tint),
        (tint, output),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .ok()?;
    }

    // The two lfos animate the deformers: one curls the bend, the other slides the
    // look-at target.
    for (from, to) in [
        ((lfo_bend, 0), (bend, 1)),      // lfo → bend.amount (curl up/down)
        ((lfo_target, 0), (look_at, 1)), // lfo → look_at.target_x (slide the aim)
    ] {
        g.connect(Edge {
            from,
            to,
            delayed: false,
        })
        .ok()?;
    }
    for (n, col, dy) in [(lfo_bend, 1.0, 220.0), (lfo_target, 2.0, 220.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: ROW_Y + dy,
            },
        );
    }

    // A 4×5 grid of squares (default size), well spaced so a bend/turn reads.
    g.set_param(grid, "rows", 4.0);
    g.set_param(grid, "cols", 5.0);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);
    // bend: up to 70° over the grid's X extent; `amount` (±1 lfo) curls it either way.
    g.set_param(bend, "angle", 70.0);
    g.set_param(bend, "pivot_x", 0.0);
    g.set_param(bend, "pivot_y", 0.0);
    // look_at: target_y stays 0 (unconnected); target_x slides with the lfo.
    g.set_param(look_at, "offset", 0.0);
    // A warm amber base.
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.70);
    g.set_param(tint, "b", 0.20);
    // lfo_bend → amount: a slow (5 s) sine, amplitude 1, offset 0 → ±1 → the grid
    // curls up and down. Unconnected `in` → a length-1 GLOBAL amount (one curvature).
    g.set_param(lfo_bend, "wave", 0.0); // Sine
    g.set_param(lfo_bend, "period", 5.0);
    g.set_param(lfo_bend, "amplitude", 1.0);
    g.set_param(lfo_bend, "offset", 0.0);
    // lfo_target → target_x: a faster (3 s) sine sliding the target across ±2.5, so
    // the squares swivel to track it.
    g.set_param(lfo_target, "wave", 0.0); // Sine
    g.set_param(lfo_target, "period", 3.0);
    g.set_param(lfo_target, "amplitude", 2.5);
    g.set_param(lfo_target, "offset", 0.0);
    Some(output)
}
