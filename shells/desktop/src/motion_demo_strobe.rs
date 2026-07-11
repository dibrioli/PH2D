//! The M3 deformer demo — the **default Motion document**: two small, side-by-side
//! deformers, each animated by the value domain. On the LEFT a grid billows into
//! **perspective** as its corners are pinned (`motion.four_point_warp`); on the RIGHT
//! a grid **bulges and pinches** like a lens (`motion.spherize`). Two independent
//! scenes (each its own `motion.output` sink — the bridge composes several into one
//! draw), kept deliberately small so each new node reads on its own. A `#[path]`
//! sibling of `motion_state`, kept out of it for the LOC cap.
//!
//! ```text
//! LEFT  (perspective): grid → four_point_warp → move(−6) → tint(amber) → output   lfo → warp
//! RIGHT (lens):        grid → spherize        → move(+6) → tint(cyan)  → output   lfo → amount
//! ```
//!
//! - **four_point_warp** (`motion.four_point_warp`, doc 24): the projective corner-pin;
//!   the top corners are pinned inward (a keystone) and the `warp` `value.lfo` billows
//!   the grid into perspective and flattens it back — straight lines stay straight.
//! - **spherize** (`motion.spherize`, doc 24): the radial lens; the `amount` `value.lfo`
//!   swings from pinch to bulge, so the grid's centre swells out and sucks back in.
//!
//! The payoff: two distinct deformer families (an affine/projective warp and a
//! nonlinear radial lens), each driven by the value domain, on one legible canvas. See
//! docs/Motion Nodes/24 (four-point-warp + spherize). The whole value/pulse vocabulary
//! + the other M3/M4 nodes stay registered (drop them in the editor).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 220.0;
/// The two scenes' card rows in graph space (stacked, so the editor reads cleanly).
const WARP_ROW: f32 = 0.0;
const LENS_ROW: f32 = 320.0;

/// Author both deformer scenes into `g`; returns their Output nodes (the sinks), the
/// perspective scene's first so the sink order is stable (id-ascending).
pub(crate) fn build(g: &mut Graph) -> Option<Vec<NodeId>> {
    let warp_out = build_warp_scene(g)?;
    let lens_out = build_lens_scene(g)?;
    Some(vec![warp_out, lens_out])
}

/// A `grid → deformer → move → tint → output` chain with an lfo into the deformer's
/// animation input. Returns the Output node. `deformer`/`anim_port` name the node type
/// and which input the lfo drives.
fn build_scene(
    g: &mut Graph,
    row: f32,
    deformer: &str,
    anim_port: u16,
    dx: f32,
    rgb: [f32; 3],
) -> Option<(NodeId, NodeId, NodeId)> {
    let grid = g.add_node("motion.grid");
    let def = g.add_node(deformer);
    let mv = g.add_node("motion.move");
    let tint = g.add_node("motion.tint");
    let output = g.add_node("motion.output");
    let lfo = g.add_node("value.lfo");

    for (n, col) in [
        (grid, 0.0),
        (def, 1.0),
        (mv, 2.0),
        (tint, 3.0),
        (output, 4.0),
    ] {
        g.set_pos(
            n,
            Pos {
                x: col * COL_W,
                y: row,
            },
        );
    }
    g.set_pos(
        lfo,
        Pos {
            x: COL_W,
            y: row + 160.0,
        },
    );

    g.connect(Edge {
        from: (grid, 0),
        to: (def, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (def, 0),
        to: (mv, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (mv, 0),
        to: (tint, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (tint, 0),
        to: (output, 0),
        delayed: false,
    })
    .ok()?;
    g.connect(Edge {
        from: (lfo, 0),
        to: (def, anim_port),
        delayed: false,
    })
    .ok()?;

    // A 5×5 grid, moved onto its half of the canvas.
    g.set_param(grid, "rows", 5.0);
    g.set_param(grid, "cols", 5.0);
    g.set_param(grid, "gap_x", 0.7);
    g.set_param(grid, "gap_y", 0.7);
    g.set_param(mv, "dx", dx);
    g.set_param(mv, "dy", 0.0);
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", rgb[0]);
    g.set_param(tint, "g", rgb[1]);
    g.set_param(tint, "b", rgb[2]);
    Some((def, lfo, output))
}

/// LEFT: a grid keystoned into perspective. Returns its Output node.
fn build_warp_scene(g: &mut Graph) -> Option<NodeId> {
    let (warp, lfo, output) = build_scene(
        g,
        WARP_ROW,
        "motion.four_point_warp",
        1,
        -6.0,
        [0.95, 0.70, 0.20],
    )?;
    // Pin the top corners inward (a keystone) — `warp` scales the offset 0→1.
    g.set_param(warp, "tl_dx", 1.2); // top-left → right
    g.set_param(warp, "tr_dx", -1.2); // top-right → left
    // lfo → warp: a slow (4 s) sine about 0.5, ±0.5 → warp ∈ [0, 1] (billow in/out).
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 4.0);
    g.set_param(lfo, "amplitude", 0.5);
    g.set_param(lfo, "offset", 0.5);
    Some(output)
}

/// RIGHT: a grid bulging and pinching like a lens. Returns its Output node.
fn build_lens_scene(g: &mut Graph) -> Option<NodeId> {
    let (spherize, lfo, output) =
        build_scene(g, LENS_ROW, "motion.spherize", 1, 6.0, [0.25, 0.80, 0.95])?;
    g.set_param(spherize, "radius", 2.5);
    // lfo → amount: a 3 s sine about 0, ±0.6 → amount ∈ [−0.6, 0.6] (pinch ↔ bulge).
    g.set_param(lfo, "wave", 0.0); // Sine
    g.set_param(lfo, "period", 3.0);
    g.set_param(lfo, "amplitude", 0.6);
    g.set_param(lfo, "offset", 0.0);
    Some(output)
}
