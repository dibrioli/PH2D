//! The M1 expression demo — the **default Motion document**, driven by typed formulas:
//! on the LEFT a **spiral** whose x/y are `cos`/`sin` expressions plotted through
//! `motion.make_point`; on the RIGHT a grid whose **colour wave** is an expression fed to
//! a ramp. The `motion.expression` formula lives in the graph's text channel (set here via
//! `set_text_param`) — the frozen node contract stays f32-only (doc 32). Two independent
//! scenes (each its own `motion.output` sink), kept small so each new node reads on its
//! own. A `#[path]` sibling of `motion_state`, kept out for the LOC cap.
//!
//! ```text
//! LEFT  (spiral): grid → expr("cos(f*a+t)*f*4") ┐
//!                 grid → expr("sin(f*a+t)*f*4") ┴→ make_point → tint → move(−6) → output
//! RIGHT (wave):   grid → expr("sin(t*2+f*a)*.5+.5") → color_ramp(t) → move(+6) → output
//! ```
//!
//! - **expression** (`motion.expression`, doc 32): a VEX-lite formula over `i`/`n`/`t`/`f`
//!   (normalised index) + columns + params `a`..`d`. The spiral rotates and the wave
//!   scrolls because both formulas read `t`.
//!
//! See docs/Motion Nodes/32 (expression + the text-param channel). The whole value/pulse
//! vocabulary + the other M3/M4 nodes stay registered (drop them in the editor).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 190.0;
const SPIRAL_ROW: f32 = 0.0;
const WAVE_ROW: f32 = 340.0;

/// Author both scenes into `g`; returns their Output nodes (the sinks), the spiral
/// scene's first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let spiral = build_spiral_scene(g)?;
    let wave = build_wave_scene(g)?;
    Some(vec![spiral, wave])
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

/// LEFT: a rotating spiral, x/y from cos/sin expressions. Returns its Output.
fn build_spiral_scene(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let expr_x = g.add_node("motion.expression");
    let expr_y = g.add_node("motion.expression");
    let point = g.add_node("motion.make_point");
    let tint = g.add_node("motion.tint");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");

    g.set_pos(
        grid,
        Pos {
            x: 0.0,
            y: SPIRAL_ROW,
        },
    );
    g.set_pos(
        expr_x,
        Pos {
            x: COL_W,
            y: SPIRAL_ROW - 120.0,
        },
    );
    g.set_pos(
        expr_y,
        Pos {
            x: COL_W,
            y: SPIRAL_ROW + 120.0,
        },
    );
    for (n, col) in [(point, 2.0), (tint, 3.0), (mv, 4.0), (output, 5.0)] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: SPIRAL_ROW,
            },
        );
    }

    wire(g, (grid, 0), (expr_x, 0))?; // grid fixes the count for the expressions
    wire(g, (grid, 0), (expr_y, 0))?;
    wire(g, (grid, 0), (point, 0))?;
    wire(g, (expr_x, 0), (point, 1))?; // → x
    wire(g, (expr_y, 0), (point, 2))?; // → y
    wire(g, (point, 0), (tint, 0))?;
    wire(g, (tint, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;

    // A 12×12 grid (144 pts) plotted as a spiral: angle = f·a + t (a ≈ 7 turns), radius
    // = f·4. Both read `t`, so the spiral rotates. On the left.
    g.set_param(grid, "rows", 12.0);
    g.set_param(grid, "cols", 12.0);
    g.set_param(expr_x, "a", 45.0);
    g.set_text_param(expr_x, "expr", "cos(f * a + t) * f * 4");
    g.set_param(expr_y, "a", 45.0);
    g.set_text_param(expr_y, "expr", "sin(f * a + t) * f * 4");
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", 0.95);
    g.set_param(tint, "g", 0.70);
    g.set_param(tint, "b", 0.20);
    g.set_param(mv, "dx", -6.0);
    g.set_param(mv, "dy", 0.0);
    Some(output)
}

/// RIGHT: a grid whose colour is a scrolling expression wave. Returns its Output.
fn build_wave_scene(g: &mut Graph) -> Option<NodeId> {
    let grid = g.add_node("motion.grid");
    let expr = g.add_node("motion.expression");
    let ramp = g.add_node("motion.color_ramp");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");

    for (n, col) in [
        (grid, 0.0),
        (expr, 1.0),
        (ramp, 2.0),
        (mv, 3.0),
        (output, 4.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: WAVE_ROW,
            },
        );
    }

    wire(g, (grid, 0), (expr, 0))?;
    wire(g, (grid, 0), (ramp, 0))?; // the grid stream carries the geometry to the ramp
    wire(g, (expr, 0), (ramp, 1))?; // the expression value drives the ramp's `t`
    wire(g, (ramp, 0), (mv, 0))?;
    wire(g, (mv, 0), (output, 0))?;

    // A 10×10 grid coloured by a wave: v = sin(t·2 + f·a)·0.5 + 0.5 ∈ [0,1] (a = spatial
    // frequency), fed as the ramp's `t`. The `t` term scrolls it. On the right.
    g.set_param(grid, "rows", 10.0);
    g.set_param(grid, "cols", 10.0);
    g.set_param(grid, "gap_x", 0.5);
    g.set_param(grid, "gap_y", 0.5);
    g.set_param(expr, "a", 12.0);
    g.set_text_param(expr, "expr", "sin(t * 2 + f * a) * 0.5 + 0.5");
    g.set_param(ramp, "preset", 2.0); // Ice
    g.set_param(mv, "dx", 6.0);
    g.set_param(mv, "dy", 0.0);
    Some(output)
}
